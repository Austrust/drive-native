"""Resolve current identity before opening; never use the historical path."""
import json
import sys
from pathlib import PurePosixPath
from PySide6.QtCore import QObject, Signal, QUrl, QProcess, QTimer
from PySide6.QtGui import QDesktopServices
from PySide6.QtDBus import QDBusConnection, QDBusMessage, QDBusPendingCallWatcher

REASONS = {"deleted": "该文件已删除。", "offline": "挂载离线且没有可用的本地完整内容。",
           "online_only": "文件仅在线，需要下载后打开。", "unavailable": "挂载尚不可用。",
           "not_found": "历史记录对应的文件已不可用。", "permission_denied": "当前账号没有访问此文件的权限。"}
FALLBACK_REASON = "文件暂不可用。"

# Controlled failure classifications persisted with a history record. Users must
# see Chinese text; the raw token is kept only for the diagnostics/detail view.
ERROR_TEXT = {
    "transport": "网络传输失败，请检查网络连接后重试。",
    "network": "网络连接失败，请检查网络后重试。",
    "session_expired": "登录已失效，请在该挂载的设置中重新登录后重试。",
    "server_rejected": "服务器拒绝了该操作，请检查文件权限或服务器状态。",
    "invalid_content": "下载内容校验失败，文件可能已发生变化，请重试。",
    "local_storage": "本地存储不可用或空间不足，请清理空间后重试。",
    "download_failed": "下载失败，请检查连接后重试。",
    "protocol": "上传协议错误，请重试；若持续出现请复制诊断信息。",
    "uncertain_commit": "尚未确认服务器是否完成该操作，请稍后刷新核对。",
    "conflict": "服务器上的文件已发生变化，请先在同步问题中处理冲突。",
    "permission_commit_uncertain": "服务器返回权限相关响应，提交结果待核对。",
    "permission_denied": "当前账号没有权限完成该操作。",
    "busy": "后台正在提交本地状态，请稍后重试。",
    "service_unavailable": "同步服务暂不可用，请稍后重试。",
}
UNKNOWN_METHOD_TEXT = "当前后台版本尚不支持该操作，请应用后台更新。"
UNKNOWN_ERROR_TEXT = "同步失败，请在同步问题页查看详情（代码：{code}）。"


def error_text_for_token(token):
    """Chinese text for one backend/classification token."""
    if token is None:
        return ""
    text = str(token)
    if text in ERROR_TEXT:
        return ERROR_TEXT[text]
    if "UnknownMethod" in text:
        return UNKNOWN_METHOD_TEXT
    return UNKNOWN_ERROR_TEXT.format(code=text)


def record_error_text(row):
    """Human text for a history row; never returns a raw token it cannot map."""
    kind = row.get("error_kind")
    if isinstance(kind, str) and kind:
        return error_text_for_token(kind)
    summary = row.get("error_summary")
    if isinstance(summary, str) and summary:
        if all(ord(ch) < 128 for ch in summary):
            return error_text_for_token(summary)
        return summary  # legacy rows may already store a controlled Chinese summary
    return ""


def record_error_code(row):
    """Raw controlled token, for the diagnostics/detail view only."""
    kind = row.get("error_kind")
    if isinstance(kind, str) and kind:
        return kind
    summary = row.get("error_summary")
    return summary if isinstance(summary, str) else ""


def reason_text(reason):
    return REASONS.get(reason, FALLBACK_REASON)


def safe_path(mount, value):
    # Lexical only: resolve()/exists()/stat() on FUSE would block the GUI.
    if not isinstance(value, str) or not value.startswith("/") or "\0" in value or "\\" in value:
        return None
    path, root = PurePosixPath(value), PurePosixPath(mount)
    if not root.is_absolute() or ".." in path.parts or ".." in root.parts:
        return None
    return str(path) if path == root or root in path.parents else None


class HistoryActions(QObject):
    message = Signal(str)

    def __init__(self, sources, parent=None, *, mount_actions=None):
        super().__init__(parent)
        self.sources = sources
        self.mount_actions = mount_actions
        self.jobs = set()
        self.locators = set()

    def current(self, source):
        current = self.sources().get(source.slot)
        return current is not None and current.bus is source.bus and current.task_id == source.task_id

    def resolve_target(self, source, row, callback):
        """Resolve current identity and hand the payload to callback(data, error)."""
        if not self.current(source):
            callback(None, "service_changed")
            return
        def resolved(data, error):
            if not self.current(source):
                return
            callback(data, error)
        source.bus.call("ResolveHistoryTarget", [json.dumps({"id": row["id"]})], resolved)

    def open(self, source, row, *, locate=False):
        if not self.current(source):
            self.message.emit("挂载身份已变化，请刷新历史后重试。"); return
        self.message.emit("正在核对文件当前位置…")
        def resolved(data, error):
            if not self.current(source): return
            if error or not isinstance(data, dict) or data.get("error"):
                self.message.emit("后台忙或文件位置暂不可用，请稍后重试。"); return
            path = safe_path(source.mount, data.get("path"))
            parent = safe_path(source.mount, data.get("parent_path"))
            reason = reason_text(data.get("reason"))
            cached = data.get("cached")
            if not isinstance(cached, bool):
                cached = data.get("offline_available") is True
            if locate:
                if path and data.get("exists"):
                    self.locate(path)
                elif parent:
                    self.message.emit(reason + " 已打开当前可用的父目录。")
                    self.open_url(parent)
                else: self.message.emit(reason)
            elif not path or not data.get("exists"):
                hint = " 可选择“在文件管理器中定位”打开可用父目录。" if parent else ""
                self.message.emit(reason + hint)
            elif cached:
                self.open_url(path)
            elif data.get("mount_online") is True:
                self.hydrate(source, row, path, data.get("node"))
            else:
                self.message.emit(reason)
        source.bus.call("ResolveHistoryTarget", [json.dumps({"id": row["id"]})], resolved)

    def start_mount(self, source, resolved=None):
        """Start the mount when the owner exposes a start hook; never pretend success."""
        slot = (resolved or {}).get("mount_slot") or source.slot
        actions = self._mount_actions()
        if actions is not None:
            for name in ("start_mount", "start_slot", "start_task"):
                method = getattr(actions, name, None)
                if callable(method):
                    try:
                        method(slot)
                    except TypeError:
                        method()
                    self.message.emit("正在启动挂载；启动完成后请再次点击「启动挂载后重试」。")
                    return True
        self.message.emit("请在挂载管理中启动该挂载，然后点击「重试」。")
        return False

    def _mount_actions(self):
        if self.mount_actions is not None:
            return self.mount_actions
        parent = self.parent()
        seen = set()
        while parent is not None and id(parent) not in seen:
            seen.add(id(parent))
            if any(callable(getattr(parent, name, None))
                   for name in ("start_mount", "start_slot", "start_task")):
                return parent
            parent = parent.parent() if hasattr(parent, "parent") else None
        return None

    def open_url(self, path):
        if QDesktopServices.openUrl(QUrl.fromLocalFile(path)):
            self.message.emit("已交给桌面默认应用。")
        else:
            self.message.emit("默认应用无法打开，请检查桌面应用设置。")

    def locate(self, path):
        message = QDBusMessage.createMethodCall("org.freedesktop.FileManager1", "/org/freedesktop/FileManager1",
                                               "org.freedesktop.FileManager1", "ShowItems")
        message.setArguments([[QUrl.fromLocalFile(path).toString()], ""])
        watcher = QDBusPendingCallWatcher(QDBusConnection.sessionBus().asyncCall(message, 15000), self)
        self.locators.add(watcher)
        def done(*_):
            self.locators.discard(watcher)
            if watcher.isError(): self.open_url(str(PurePosixPath(path).parent))
            else: self.message.emit("已在文件管理器中定位。")
            watcher.deleteLater()
        watcher.finished.connect(done)

    def hydrate(self, source, row, path, node):
        # Only an explicit click starts this helper. Reading uses the existing
        # FUSE foreground-download path, without pinning or storing another copy.
        # The helper is isolated from Qt and never returns content to the UI.
        if any(job.property("history_key") == (source.slot, row["id"]) for job in self.jobs):
            self.message.emit("此文件正在下载，完成后会打开。"); return
        process = QProcess(self)
        process.setProperty("history_key", (source.slot, row["id"]))
        process.setProcessChannelMode(QProcess.SeparateChannels)
        process.setStandardErrorFile(QProcess.nullDevice())
        process.setStandardOutputFile(QProcess.nullDevice())
        timer = QTimer(process); timer.setSingleShot(True); timer.setInterval(300000)
        timer.timeout.connect(process.kill)
        self.jobs.add(process)
        completed = [False]
        def finish(code=1, status=QProcess.CrashExit):
            if completed[0]: return
            completed[0] = True; timer.stop(); self.jobs.discard(process); process.deleteLater()
            if not self.current(source): return
            if code != 0 or status != QProcess.NormalExit:
                self.message.emit("下载尚未完成，文件未打开；请检查同步问题后重试。"); return
            def checked(data, error):
                if not self.current(source): return
                if (not error and isinstance(data, dict) and data.get("exists") and data.get("offline_available")
                        and data.get("node") == node and safe_path(source.mount, data.get("path")) == path):
                    self.open_url(path)
                else: self.message.emit("下载后文件状态或位置已变化，请重新选择记录。")
            source.bus.call("ResolveHistoryTarget", [json.dumps({"id": row["id"]})], checked)
        process.finished.connect(finish)
        process.errorOccurred.connect(lambda _error: finish())
        self.message.emit("正在下载；完成并核对当前位置后打开…")
        script = "import os,sys\nf=os.open(sys.argv[1],os.O_RDONLY|os.O_NOFOLLOW)\ntry:\n while os.read(f,1048576): pass\nfinally: os.close(f)\n"
        process.start(sys.executable, ["-c", script, path]); timer.start()
