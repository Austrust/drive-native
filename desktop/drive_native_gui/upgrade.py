"""Local executable detection and cooperative, owner-bound backend handover."""
import os
import select
from PySide6.QtCore import QObject, Signal, Slot, QTimer
from PySide6.QtDBus import QDBusMessage, QDBusPendingCallWatcher
from .config import binary
from .pause_handover import preserve_pause


class _OwnedPidfd:
    """Can outlive the QObject wrapper long enough to close its descriptor."""
    def __init__(self):
        self.fd = None

    def close(self, *_destroyed):
        fd, self.fd = self.fd, None
        if fd is not None:
            os.close(fd)


class BackendUpgrade(QObject):
    changed = Signal(bool)
    finished = Signal(object)

    def __init__(self, bus, launcher, parent=None, *, save_pause=None):
        super().__init__(parent)
        self.bus, self.launcher = bus, launcher
        self.save_pause = save_pause
        self.intent_profile = None
        self.available = False
        self.pending = None
        self._pidfd = _OwnedPidfd()
        # A slot on the dying QObject itself is not a reliable destruction hook.
        # This plain Python owner has no reference back to the QObject.
        self.destroyed.connect(self._pidfd.close)
        self.owner = None
        self.active = False
        self.waiting_exit = False
        self.confirming = False
        self.confirm_pending = False
        self.elapsed = 0
        self.timer = QTimer(self)
        self.timer.setInterval(250)
        self.timer.timeout.connect(self._wait_exit)
        self.detect_timer = QTimer(self)
        self.detect_timer.setInterval(30000)
        self.detect_timer.timeout.connect(self.check)
        self.detect_timer.start()
        self.bus.availability.connect(self._availability_changed)
        self.launcher.completed.connect(self._started)
        QTimer.singleShot(0, self.check)

    @property
    def pidfd(self):
        return self._pidfd.fd

    @pidfd.setter
    def pidfd(self, fd):
        if fd != self._pidfd.fd:
            self._pidfd.close()
            self._pidfd.fd = fd

    @Slot(bool)
    def _availability_changed(self, _available):
        self.check()

    def check(self):
        if self.active or self.pending:
            return
        if not self.bus.owner or not getattr(self.bus, "bus", None):
            self.pidfd = None
            self.owner = None
            self._detected(False)
            return
        owner = self.bus.owner
        message = QDBusMessage.createMethodCall("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "GetConnectionUnixProcessID")
        message.setArguments([owner])
        watcher = QDBusPendingCallWatcher(self.bus.bus.asyncCall(message, 3000), self)
        self.pending = watcher

        def done(*_):
            self.pending = None
            self.pidfd = None
            changed = False
            if owner == self.bus.owner and not watcher.isError():
                try:
                    pid = int(watcher.reply().arguments()[0])
                    # pidfd prevents PID reuse from confusing the exit barrier.
                    fd = os.pidfd_open(pid)
                    try:
                        running = os.stat(f"/proc/{pid}/exe")
                        installed = os.stat(binary("drive-native-fsd"))
                        changed = (running.st_dev, running.st_ino) != (installed.st_dev, installed.st_ino)
                        if changed:
                            self.pidfd, self.owner, fd = fd, owner, None
                    finally:
                        if fd is not None:
                            os.close(fd)
                except (OSError, ValueError, AttributeError):
                    pass
            self._detected(changed)
            watcher.deleteLater()
        watcher.finished.connect(done)

    def _detected(self, value):
        self.available = value
        self.changed.emit(value)

    def begin(self, profile=None):
        if self.active or not self.available or self.owner != self.bus.owner:
            self.check()
            return False
        self.active = True
        self.intent_profile = dict(profile) if profile else None
        self.bus.call("Pause", callback=self._paused)
        return self.active

    def _paused(self, _data, error):
        if error or self.owner != self.bus.owner:
            self._finish("暂停未确认，尚未切换后台。")
            return
        try:
            if self.save_pause:
                self.save_pause()
            else:
                preserve_pause(self.intent_profile)
        except (OSError, ValueError, KeyError, TypeError):
            self._finish("暂停状态未能持久保存，尚未停止旧后台。请检查本地存储权限和空间后重试。")
            return
        self.waiting_exit = True
        self.elapsed = 0
        self.bus.call("StopMount", callback=self._stopped)
        self.timer.start()

    def _stopped(self, _data, error):
        # Owner disappearance is expected during normal stop. The pidfd is authoritative.
        if error and error != "service_changed":
            self._finish("停止挂载未确认。已保留数据，请关闭使用该目录的应用后重试。")

    def _wait_exit(self):
        if self.confirming:
            self.elapsed += 1
            if self.elapsed >= 240:
                self._finish("新后台尚未确认接管，数据仍保留。请检查挂载状态后重试。")
            else:
                self._confirm_started()
            return
        if self.pidfd is not None and select.select([self.pidfd], [], [], 0)[0]:
            self.timer.stop()
            self.waiting_exit = False
            self.pidfd = None
            self.launcher.start()
            return
        self.elapsed += 1
        if self.elapsed >= 240:
            # No SIGKILL or systemctl restart: unfinished local writes keep their process.
            self._finish("后台仍在结束本地操作，尚未切换。请关闭打开的文件后重试；缓存与草稿保留。")

    def _started(self, error):
        if self.active and not self.waiting_exit:
            if error:
                self._finish(error)
                return
            self.confirming = True
            self.elapsed = 0
            self.timer.start()
            self._confirm_started()

    def _confirm_started(self):
        owner = self.bus.owner
        if self.confirm_pending or not owner or owner == self.owner:
            return
        self.confirm_pending = True

        def received(data, error):
            if not self.active or not self.confirming:
                return
            if error or self.bus.owner != owner or not isinstance(data, dict):
                self.confirm_pending = False
                return
            if not self.intent_profile or data.get("task_id") != self.intent_profile.get("task_id"):
                self._finish("新后台的任务身份不匹配，未确认接管；数据仍保留。")
                return
            if data.get("connection") != "paused" or not data.get("mounted"):
                self.confirm_pending = False
                return
            self._verify_installed_owner(owner, self._identity_confirmed)

        self.bus.call("GetOverview", callback=received)

    def _verify_installed_owner(self, owner, callback):
        message = QDBusMessage.createMethodCall("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "GetConnectionUnixProcessID")
        message.setArguments([owner])
        watcher = QDBusPendingCallWatcher(self.bus.bus.asyncCall(message, 3000), self)

        def done(*_):
            matches = False
            if owner == self.bus.owner and not watcher.isError():
                try:
                    pid = int(watcher.reply().arguments()[0])
                    fd = os.pidfd_open(pid)
                    try:
                        running = os.stat(f"/proc/{pid}/exe")
                        installed = os.stat(binary("drive-native-fsd"))
                        matches = (running.st_dev, running.st_ino) == (installed.st_dev, installed.st_ino) and not select.select([fd], [], [], 0)[0]
                    finally:
                        os.close(fd)
                except (OSError, ValueError, AttributeError):
                    pass
            callback(owner, matches)
            watcher.deleteLater()
        watcher.finished.connect(done)

    def _identity_confirmed(self, owner, matches):
        self.confirm_pending = False
        if self.active and self.confirming and matches and self.bus.owner == owner:
            self._finish(None)

    def _finish(self, error):
        self.timer.stop()
        self.active = False
        self.waiting_exit = False
        self.confirming = False
        self.confirm_pending = False
        self.finished.emit(error)
        self.check()
