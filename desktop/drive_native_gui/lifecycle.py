"""One user service lifecycle for installed builds; explicit development isolation."""

import os
from PySide6.QtCore import QObject, QProcess, QProcessEnvironment, QTimer, Signal
from .config import binary, config_path, service_unit


def _start_detached(process):
    return process.startDetached()


class ServiceLauncher(QObject):
    completed = Signal(object)

    def __init__(self, parent=None, *, managed=None, unit=None, instance=None):
        super().__init__(parent)
        self.managed = "DRIVE_NATIVE_BIN_DIR" not in os.environ if managed is None else managed
        self.instance = os.environ.get("DRIVE_NATIVE_INSTANCE", "") if instance is None else instance
        self.unit = unit or service_unit(self.instance)
        self.process = QProcess(self)
        self.timer = QTimer(self)
        self.timer.setSingleShot(True)
        self.timer.setInterval(20000)
        self.active = False
        self.checking = False
        self.process.finished.connect(self._finished)
        self.process.errorOccurred.connect(
            lambda _error: self._finish("后台服务未能启动，请检查用户服务状态。")
        )
        self.timer.timeout.connect(self._timed_out)

    def start(self, *, enable=False):
        if self.active:
            return
        self.active = True
        self.checking = False
        if not self.managed:
            try:
                child = QProcess(self)
                environment = QProcessEnvironment.systemEnvironment()
                if self.instance:
                    environment.insert("DRIVE_NATIVE_INSTANCE", self.instance)
                else:
                    environment.remove("DRIVE_NATIVE_INSTANCE")
                child.setProcessEnvironment(environment)
                child.setProgram(binary("drive-native-fsd"))
                child.setArguments(["--config", str(config_path(self.instance))])
                started = _start_detached(child)
                child.deleteLater()
                self._finish(None if started else "后台组件无法启动。")
            except (OSError, ValueError):
                self._finish("后台组件未安装完整。")
            return
        arguments = (
            ["--user", "enable", "--now", self.unit] if enable else ["--user", "start", self.unit]
        )
        self.process.start("/usr/bin/systemctl", arguments)
        self.timer.start()

    def _finished(self, code, status):
        output = bytes(self.process.readAllStandardOutput()).decode("utf-8", "replace").strip()
        self.process.readAllStandardError()
        if not self.active:
            return
        if code != 0 or status != QProcess.NormalExit:
            self._finish("用户同步服务未能启动，请检查配置和服务状态。")
        elif not self.checking:
            # A failed ConditionPathExists can make systemctl start exit 0.
            self.checking = True
            self.process.start(
                "/usr/bin/systemctl",
                ["--user", "show", self.unit, "--property=ActiveState", "--value"],
            )
        else:
            self._finish(None if output == "active" else "后台服务尚未运行，请检查配置和服务状态。")

    def _timed_out(self):
        # Terminate only the controller. An already-started mount belongs to systemd.
        self.process.kill()
        self._finish("服务启动尚未确认，请检查后台状态。")

    def _finish(self, error):
        if not self.active:
            return
        self.active = False
        self.timer.stop()
        self.completed.emit(error)


def same_task(profile, overview):
    return bool(
        profile
        and overview
        and profile.get("task_id")
        and profile["task_id"] == overview.get("task_id")
        and profile.get("host") == overview.get("host")
        and profile.get("username") == overview.get("username")
    )
