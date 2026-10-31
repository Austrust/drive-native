"""Local executable detection and cooperative, owner-bound backend handover."""
import os
import select
from PySide6.QtCore import QObject, Signal, QTimer
from PySide6.QtDBus import QDBusMessage, QDBusPendingCallWatcher
from .config import binary
from .pause_handover import preserve_pause


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
        self.pidfd = None
        self.owner = None
        self.active = False
        self.waiting_exit = False
        self.elapsed = 0
        self.timer = QTimer(self)
        self.timer.setInterval(250)
        self.timer.timeout.connect(self._wait_exit)
        self.detect_timer = QTimer(self)
        self.detect_timer.setInterval(30000)
        self.detect_timer.timeout.connect(self.check)
        self.detect_timer.start()
        self.bus.availability.connect(lambda _: self.check())
        self.launcher.completed.connect(self._started)
        QTimer.singleShot(0, self.check)

    def check(self):
        if self.active or self.pending:
            return
        if not self.bus.owner or not getattr(self.bus, "bus", None):
            self._detected(False)
            return
        owner = self.bus.owner
        message = QDBusMessage.createMethodCall("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "GetConnectionUnixProcessID")
        message.setArguments([owner])
        watcher = QDBusPendingCallWatcher(self.bus.bus.asyncCall(message, 3000), self)
        self.pending = watcher

        def done(*_):
            self.pending = None
            if self.pidfd is not None:
                os.close(self.pidfd)
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
        if self.pidfd is not None and select.select([self.pidfd], [], [], 0)[0]:
            self.timer.stop()
            self.waiting_exit = False
            os.close(self.pidfd)
            self.pidfd = None
            self.launcher.start()
            return
        self.elapsed += 1
        if self.elapsed >= 240:
            # No SIGKILL or systemctl restart: unfinished local writes keep their process.
            self._finish("后台仍在结束本地操作，尚未切换。请关闭打开的文件后重试；缓存与草稿保留。")

    def _started(self, error):
        if self.active and not self.waiting_exit:
            self._finish(error)

    def _finish(self, error):
        self.timer.stop()
        self.active = False
        self.waiting_exit = False
        self.finished.emit(error)
        self.check()
