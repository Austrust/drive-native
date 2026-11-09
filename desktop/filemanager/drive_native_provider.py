"""Nemo/Nautilus: local asynchronous D-Bus only, never opens file contents."""

import json
import os
import re
import time
from collections import OrderedDict
import gi

from gi.repository import GObject, Gio, GLib

NAME = "org.drivenative.Client1"
PATH = "/org/drivenative/Client1"
STATES = {
    "initializing": ("初始化预览 · 正文尚未可用", "drive-native-pending"),
    "not_summarized": ("状态尚未汇总", "drive-native-pending"),
    "download_retry": ("下载失败 · 将重试", "drive-native-pending"),
    "update_retry": ("更新失败 · 将重试", "drive-native-pending"),
    "download_failed": ("下载失败", "drive-native-error"),
    "update_failed": ("更新失败 · 旧版可用", "drive-native-error"),
    "online_only": ("仅在线", "drive-native-online"),
    "local_complete": ("本地可用", "drive-native-local"),
    "pinned_complete": ("已固定", "drive-native-pinned"),
    "pinning": ("正在固定", "drive-native-sync"),
    "downloading": ("正在下载", "drive-native-sync"),
    "updating": ("正在更新 · 保留已有完整版本", "drive-native-sync"),
    "pending_upload": ("等待同步", "drive-native-pending"),
    "conflict": ("需要处理冲突", "drive-native-error"),
    "mixed": ("部分本地可用", "drive-native-online"),
}
STATUS_SIGNAL_INTERVAL = 1.0
STATUS_KINDS = frozenset(("status", "listing", "subtree"))


def _beneath(path, root):
    root = root.rstrip("/") or "/"
    if root == "/":
        return path != "/" and path.startswith("/")
    return path != root and path.startswith(root + "/")


def parse_status_changes(changes_json):
    """Parse the change list carried by StatusChanged2 (issue #31).

    Returns ``(changes, truncated)``; ``changes`` is None for an invalid or
    unknown payload, in which case the consumer must fall back to a full
    refresh instead of trusting a partial change set.
    """
    if not isinstance(changes_json, str) or not changes_json:
        return None, False
    try:
        payload = json.loads(changes_json)
    except (TypeError, ValueError):
        return None, False
    if not isinstance(payload, dict) or payload.get("schema") != 1:
        return None, False
    raw = payload.get("changes") or ()
    if not isinstance(raw, (list, tuple)):
        return None, False
    changes = []
    for change in raw:
        if not isinstance(change, dict):
            return None, False
        path = change.get("path")
        kind = change.get("kind")
        if (not isinstance(path, str) or not path.startswith("/")
                or kind not in STATUS_KINDS):
            return None, False
        changes.append({
            "path": os.path.normpath(path),
            "directory": bool(change.get("directory")),
            "kind": kind,
        })
    return changes, bool(payload.get("truncated"))


def affected_tracked(changes, tracked_paths):
    """Tracked paths equal to or below a changed path (issue #31)."""
    affected = set()
    for tracked in tracked_paths:
        for change in changes:
            path = change["path"]
            if tracked == path or _beneath(tracked, path):
                affected.add(tracked)
                break
    return affected


def build_provider(FileManager):
    class _TaskExtension(GObject.GObject):
        def __init__(self, service=NAME):
            super().__init__()
            self.proxy = None
            self.root = None
            self.connection = "locked"
            self.preview_only = False
            self.cache = {}
            self.files = OrderedDict()
            self.pending = set()
            self.timer = None
            self.busy = False
            self.generation = 0
            # StatusChanged storms used to run a full reorder + GetOverview per
            # signal (issue #29). At most one window per second fires; signals
            # arriving inside the window collapse into one trailing refresh.
            self.status_signal_at = 0.0
            self.status_signal_timer = None
            self.status_signal_seen = 0
            self.overview_queries = 0
            self.coalesced_signals = 0
            # #31 revisioned change sets: a v2 event with a contiguous revision
            # only re-queries the tracked paths it names; reset/gap/invalid
            # payloads still run the full GetOverview + requery fallback.
            self.status2_seen = False
            self.status_epoch = None
            self.status_revision = None
            self.status2_events = 0
            Gio.DBusProxy.new_for_bus(
                Gio.BusType.SESSION,
                Gio.DBusProxyFlags.DO_NOT_AUTO_START | Gio.DBusProxyFlags.DO_NOT_LOAD_PROPERTIES,
                None,
                service,
                PATH,
                NAME,
                None,
                self._connected,
            )

        def _connected(self, _source, result):
            try:
                self.proxy = Gio.DBusProxy.new_for_bus_finish(result)
                self.proxy.connect("g-signal", self._signal)
                self.proxy.connect("notify::g-name-owner", self._owner_changed)
                self._changed()
            except GLib.Error:
                self.proxy = None

        def _call(self, method, arguments, callback):
            if not self.proxy or not self.proxy.get_name_owner():
                callback(None, True)
                return

            def done(proxy, result):
                try:
                    reply = proxy.call_finish(result).unpack()
                    data = json.loads(reply[0]) if reply else None
                    if data is not None and (not isinstance(data, dict) or data.get("schema") != 1):
                        raise ValueError()
                    callback(data, False)
                except (GLib.Error, ValueError, TypeError):
                    callback(None, True)

            self.proxy.call(method, arguments, Gio.DBusCallFlags.NO_AUTO_START, 15000, None, done)

        def _signal(self, _proxy, _sender, signal, _parameters):
            if signal == "StatusChanged":
                # Legacy daemons only emit the unversioned signal. New daemons
                # dual-emit it with StatusChanged2; once a v2 event was seen the
                # legacy signal carries no extra information (issue #31).
                if not self.status2_seen:
                    self._status_signal()
            elif signal == "StatusChanged2":
                self._status2_signal(_parameters)

        def _status_signal(self):
            now = time.monotonic()
            if now - self.status_signal_at >= STATUS_SIGNAL_INTERVAL:
                self.status_signal_at = now
                self._changed()
                return
            self.coalesced_signals += 1
            if self.status_signal_timer is None:
                delay = int((STATUS_SIGNAL_INTERVAL - (now - self.status_signal_at)) * 1000) + 1
                self.status_signal_timer = GLib.timeout_add(delay, self._status_trailing)

        def _status_trailing(self):
            self.status_signal_timer = None
            self.status_signal_at = time.monotonic()
            self._changed()
            return GLib.SOURCE_REMOVE

        def _status2_signal(self, parameters):
            """Consume one revisioned StatusChanged2 event (issue #31)."""
            try:
                (from_revision, to_revision, epoch, reset,
                 overview_changed, changes_json) = parameters.unpack()
            except (TypeError, ValueError, AttributeError):
                self._changed()
                return
            self.status2_events += 1
            self.status2_seen = bool(epoch)
            parsed = parse_status_changes(changes_json)
            if parsed is None or not epoch:
                self._changed()
                return
            changes, truncated = parsed
            full = self._status_discontinuous(
                from_revision, to_revision, epoch, reset, truncated)
            self.status_epoch = epoch
            self.status_revision = to_revision
            if full:
                self._changed()
                return
            if overview_changed:
                self._changed(requery_all=False)
                return
            affected = affected_tracked(changes, self.files.keys())
            if affected:
                self.pending.update(affected)
                self._schedule()

        def _status_discontinuous(self, from_revision, to_revision, epoch,
                                  reset, truncated):
            return bool(
                reset or truncated
                or self.status_epoch != epoch
                or self.status_revision is None
                or from_revision != self.status_revision
                or to_revision < from_revision
            )

        def _owner_changed(self, *_args):
            # A replacement daemon may be older and only speak the legacy
            # protocol; forget revision tracking so both paths start fresh.
            self.status2_seen = False
            self.status_epoch = None
            self.status_revision = None
            self._changed()

        def _changed(self, requery_all=True):
            self.generation += 1
            self.overview_queries += 1
            generation = self.generation

            def overview(data, failed):
                if generation != self.generation:
                    return
                old_root, old_connection, old_preview = self.root, self.connection, self.preview_only
                self.root = os.path.normpath(data["mount"]) if not failed else None
                self.connection = data.get("connection", "locked") if data else "locked"
                self.preview_only = data.get("preview_only") is True if data else False
                if old_root != self.root or old_preview != self.preview_only:
                    self.cache.clear()
                    self.pending.clear()
                for path, file in list(self.files.items()):
                    if self.root and self._contains(path):
                        if requery_all:
                            self.pending.add(path)
                        if old_root != self.root or old_connection != self.connection or old_preview != self.preview_only:
                            file.invalidate_extension_info()
                    elif old_root and os.path.commonpath((old_root, path)) == old_root:
                        # A changed/removed mount must also clear icons on old paths.
                        file.invalidate_extension_info()
                self._schedule()

            self._call("GetOverview", None, overview)

        def _candidate_path(self, file):
            if file.get_uri_scheme() != "file":
                return None
            path = Gio.File.new_for_uri(file.get_uri()).get_path()
            if path is None:
                return None
            return os.path.normpath(path)

        def _contains(self, path):
            return self.root and os.path.commonpath((self.root, path)) == self.root

        def _path(self, file):
            path = self._candidate_path(file)
            return path if path is not None and self._contains(path) else None

        def get_columns(self):
            return (
                FileManager.Column(
                    name="DriveNative::status",
                    attribute="drive_native_status",
                    label="Drive 状态",
                    description="本地可用性与最后已知的同步状态",
                ),
            )

        def _track(self, path, file):
            # GI wrappers can disappear even while Nemo keeps the native FileInfo.
            # Keep a bounded strong set so asynchronous results can invalidate the
            # actual visible file; weak references alone leave "loading" forever.
            self.files[path] = file
            self.files.move_to_end(path)
            while len(self.files) > 4096:
                old, _ = self.files.popitem(last=False)
                self.cache.pop(old, None)
                self.pending.discard(old)

        def update_file_info(self, file):
            path = self._candidate_path(file)
            if path is None:
                return
            # Nemo can ask for icons before the asynchronous bus connection is ready.
            # Retain bounded references so the first overview refreshes visible files
            # instead of requiring the user to reopen the directory.
            if not self.root:
                self._track(path, file)
                return
            if not self._contains(path):
                return
            self._track(path, file)
            state = self.cache.get(path)
            if state is None:
                file.add_string_attribute("drive_native_status", "正在读取状态…")
                self.pending.add(path)
                self._schedule()
                return
            label, emblem = STATES.get(state.get("status"), ("状态不可用", "drive-native-error"))
            if state.get("error"):
                label, emblem = "状态读取失败", "drive-native-error"
            if self.connection != "online":
                label += " · 未连接同步"
            file.add_string_attribute("drive_native_status", label)
            file.add_emblem(emblem)

        def _schedule(self):
            if self.timer is None and self.pending and not self.busy:
                self.timer = GLib.timeout_add(40, self._flush)

        def _flush(self):
            self.timer = None
            if self.busy or not self.pending:
                return GLib.SOURCE_REMOVE
            paths = sorted(self.pending)[:256]
            self.pending.difference_update(paths)
            generation = self.generation
            self.busy = True

            def received(data, failed):
                self.busy = False
                if generation != self.generation:
                    self.pending.update(path for path in paths if path in self.files and self._contains(path))
                else:
                    states = {e["path"]: e for e in data["entries"]} if data and not failed else {}
                    for path in paths:
                        previous = self.cache.get(path)
                        self.cache[path] = states.get(path, {"error": "state_unavailable"})
                        file = self.files.get(path)
                        if file is not None and previous != self.cache[path]:
                            file.invalidate_extension_info()
                self._schedule()

            self._call("GetStates", GLib.Variant("(as)", (paths,)), received)
            return GLib.SOURCE_REMOVE

        def get_file_items(self, *args):
            files = args[-1]
            paths = [self._path(file) for file in files]
            if not paths or any(path is None for path in paths):
                return ()
            return self._menus(paths)

        def get_background_items(self, *args):
            file = args[-1]
            path = self._path(file)
            return self._menus([path]) if path is not None else ()

        def _menus(self, paths):
            if self.preview_only:
                return ()
            root = FileManager.MenuItem(
                name="DriveNative::menu",
                label="Drive Native",
                tip="管理本地可用性",
                icon="drive-native-online",
            )
            submenu = FileManager.Menu()
            root.set_submenu(submenu)
            for key, label, method, value in [
                ("pin", "固定到本地", "SetPinned", True),
                ("unpin", "取消固定", "SetPinned", False),
                ("release", "释放空间", "ReleaseSpace", None),
                ("retry", "重试下载", "RetryDownloads", None),
            ]:
                item = FileManager.MenuItem(name="DriveNative::" + key, label=label, tip="", icon="")
                item.connect("activate", self._activate, paths, method, value)
                submenu.append_item(item)
            return (root,)

        def _activate(self, _item, paths, method, value):
            # A menu may outlive a mode transition; the backend also validates.
            if self.preview_only:
                return
            if method == "ReleaseSpace":
                self._release(paths)
                return
            for path in paths:
                arguments = (
                    GLib.Variant("(s)", (path,))
                    if value is None
                    else GLib.Variant("(sb)", (path, value))
                )

                def completed(data, failed):
                    if failed:
                        self._notify("操作未完成，请在 Drive Native 中检查状态。")
                    elif data and data.get("protected_or_failed"):
                        self._notify("部分文件仍在使用、已固定或有未同步修改，暂未释放。")
                    self._changed()

                self._call(method, arguments, completed)

        @staticmethod
        def _bytes_text(value):
            if not isinstance(value, int) or value <= 0:
                return "0 B"
            for unit, size in (("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)):
                if value >= size:
                    return f"{value / size:.2f} {unit}"
            return f"{value} B"

        def _release(self, paths):
            # Releasing a directory may take minutes. Ask for a background job and
            # poll it instead of holding one blocking D-Bus call for the whole run.
            if not paths:
                return
            self._call(
                "BeginReleaseSpace",
                GLib.Variant("(as)", (list(paths),)),
                self._release_started,
            )

        def _release_started(self, data, failed):
            job_id = data.get("job_id") if data else None
            if failed or not isinstance(job_id, str) or not job_id:
                self._notify("操作未完成，请在 Drive Native 中检查状态。")
                self._changed()
                return
            total = data.get("total")
            if isinstance(total, int) and total > 0:
                self._notify(f"正在释放 {total} 个文件，可在 Drive Native 中查看进度。")
            self._poll_release(job_id)

        def _poll_release(self, job_id):
            def received(data, failed):
                # A service restart forgets the in-memory job; stop polling then.
                if failed or not data or data.get("job_id") != job_id:
                    self._notify("释放未完成，请在 Drive Native 中检查状态。")
                    self._changed()
                    return
                if data.get("available") is False:
                    self._notify("释放未完成，请在 Drive Native 中检查状态。")
                    self._changed()
                    return
                if not data.get("finished"):
                    GLib.timeout_add(250, self._poll_release_once, job_id)
                    return
                self._release_finished(data)

            self._call("GetReleaseJob", GLib.Variant("(s)", (job_id,)), received)

        def _poll_release_once(self, job_id):
            self._poll_release(job_id)
            return False

        def _release_finished(self, data):
            done = data.get("done", 0)
            total = data.get("total", 0)
            reclaimed = self._bytes_text(data.get("reclaimed_bytes", 0))
            if data.get("error"):
                self._notify("释放未完成，请在 Drive Native 中检查状态。")
            elif data.get("cancelled"):
                self._notify(f"已取消释放：完成 {done}/{total}，已回收 {reclaimed}。")
            else:
                body = (
                    f"释放完成：已释放 {data.get('released_count', 0)} 个文件，"
                    f"跳过 {data.get('skipped_count', 0)} 个，回收 {reclaimed}。"
                )
                protected = data.get("protected_count", 0)
                if protected:
                    body += f"另有 {protected} 个文件受保护暂未释放。"
                self._notify(body)
            self._changed()

        def _notify(self, body):
            if not self.proxy:
                return
            parameters = GLib.Variant(
                "(susssasa{sv}i)", ("Drive Native", 0, "", "Drive Native", body, [], {}, -1)
            )
            self.proxy.get_connection().call(
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                "Notify",
                parameters,
                None,
                Gio.DBusCallFlags.NO_AUTO_START,
                3000,
                None,
                None,
            )


    class DriveNativeExtension(GObject.GObject, FileManager.InfoProvider, FileManager.ColumnProvider, FileManager.MenuProvider):
        """Discover task services locally; each child owns its own cache and commands."""
        def __init__(self):
            super().__init__()
            self.tasks = {}
            self.files = OrderedDict()
            Gio.bus_get(Gio.BusType.SESSION, None, self._bus_ready)

        def _bus_ready(self, _source, result):
            try:
                self.connection = Gio.bus_get_finish(result)
            except GLib.Error:
                return
            self.connection.signal_subscribe("org.freedesktop.DBus", "org.freedesktop.DBus", "NameOwnerChanged", "/org/freedesktop/DBus", None, Gio.DBusSignalFlags.NONE, self._owner_changed)
            self.connection.call("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "ListNames", None, None, Gio.DBusCallFlags.NONE, 3000, None, self._names)

        def _names(self, connection, result):
            try:
                for name in connection.call_finish(result).unpack()[0]: self._add(name)
            except GLib.Error:
                pass

        def _owner_changed(self, _bus, _sender, _path, _interface, _signal, params):
            name, old, new = params.unpack()
            if new: self._add(name)

        def _add(self, name):
            if name in self.tasks or not (name == NAME or re.fullmatch(re.escape(NAME) + r"\.i[0-9a-f]{32}", name)):
                return
            self.tasks[name] = _TaskExtension(name)
            for file in tuple(self.files.values()):
                self.tasks[name].update_file_info(file)

        def get_columns(self):
            return (FileManager.Column.new("DriveNative::status", "drive_native_status", "Drive 状态", "本地可用性与最后已知同步状态"),)

        def update_file_info(self, file):
            self.files[file.get_uri()] = file
            self.files.move_to_end(file.get_uri())
            while len(self.files) > 4096: self.files.popitem(last=False)
            for task in tuple(self.tasks.values()): task.update_file_info(file)

        def get_file_items(self, *args):
            files = args[-1]
            for task in tuple(self.tasks.values()):
                items = task.get_file_items(files)
                if items: return items
            return ()

        def get_background_items(self, *args):
            file = args[-1]
            for task in tuple(self.tasks.values()):
                items = task.get_background_items(file)
                if items: return items
            return ()
    return DriveNativeExtension, _TaskExtension
