"""Nemo integration: local asynchronous D-Bus only, never opens file contents."""

import json
import os
import re
from collections import OrderedDict
import gi

gi.require_version("Nemo", "3.0")
from gi.repository import GObject, Gio, GLib, Nemo

NAME = "org.drivenative.Client1"
PATH = "/org/drivenative/Client1"
STATES = {
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


class _TaskExtension(GObject.GObject):
    def __init__(self, service=NAME):
        super().__init__()
        self.proxy = None
        self.root = None
        self.connection = "locked"
        self.cache = {}
        self.files = OrderedDict()
        self.pending = set()
        self.timer = None
        self.busy = False
        self.generation = 0
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
            self.proxy.connect("notify::g-name-owner", lambda *_: self._changed())
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
            self._changed()

    def _changed(self):
        self.generation += 1
        generation = self.generation
        self.cache.clear()

        def overview(data, failed):
            if generation != self.generation:
                return
            self.root = os.path.normpath(data["mount"]) if not failed else None
            self.connection = data.get("connection", "locked") if data else "locked"
            for path, file in list(self.files.items()):
                if self.root and self._contains(path):
                    self.pending.add(path)
                    file.invalidate_extension_info()
                elif not self.root:
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
            Nemo.Column(
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
                self.pending.update(path for path in paths if path in self.files)
            else:
                states = {e["path"]: e for e in data["entries"]} if data and not failed else {}
                for path in paths:
                    self.cache[path] = states.get(path, {"error": "state_unavailable"})
                    file = self.files.get(path)
                    if file is not None:
                        file.invalidate_extension_info()
            self._schedule()

        self._call("GetStates", GLib.Variant("(as)", (paths,)), received)
        return GLib.SOURCE_REMOVE

    def get_file_items(self, _window, files):
        paths = [self._path(file) for file in files]
        if not paths or any(path is None for path in paths):
            return ()
        return self._menus(paths)

    def get_background_items(self, _window, file):
        path = self._path(file)
        return self._menus([path]) if path is not None else ()

    def _menus(self, paths):
        root = Nemo.MenuItem(
            name="DriveNative::menu",
            label="Drive Native",
            tip="管理本地可用性",
            icon="drive-native-online",
        )
        submenu = Nemo.Menu()
        root.set_submenu(submenu)
        for key, label, method, value in [
            ("pin", "固定到本地", "SetPinned", True),
            ("unpin", "取消固定", "SetPinned", False),
            ("release", "释放空间", "ReleaseSpace", None),
            ("retry", "重试下载", "RetryDownloads", None),
        ]:
            item = Nemo.MenuItem(name="DriveNative::" + key, label=label, tip="", icon="")
            item.connect("activate", self._activate, paths, method, value)
            submenu.append_item(item)
        return (root,)

    def _activate(self, _item, paths, method, value):
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


class DriveNativeExtension(GObject.GObject, Nemo.InfoProvider, Nemo.ColumnProvider, Nemo.MenuProvider):
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
        return (Nemo.Column.new("DriveNative::status", "drive_native_status", "Drive 状态", "本地可用性与最后已知同步状态"),)

    def update_file_info(self, file):
        self.files[file.get_uri()] = file
        self.files.move_to_end(file.get_uri())
        while len(self.files) > 4096: self.files.popitem(last=False)
        for task in tuple(self.tasks.values()): task.update_file_info(file)

    def get_file_items(self, window, files):
        for task in tuple(self.tasks.values()):
            items = task.get_file_items(window, files)
            if items: return items
        return ()

    def get_background_items(self, window, file):
        for task in tuple(self.tasks.values()):
            items = task.get_background_items(window, file)
            if items: return items
        return ()
