import json
from collections import deque
from PySide6.QtCore import QObject, QProcess, QTimer
from .config import binary

ERRORS = {
    "login_rejected": "NAS 拒绝了本次登录。请核对账号密码、账号状态和 Drive 应用权限，避免连续重试。",
    "configuration": "连接配置无效，请检查地址、证书设置和所选目录。",
    "session_expired": "登录会话已失效，请重新登录；已保存的本地文件会保留。",
    "protocol": "服务器响应或协议版本无法处理，请检查兼容性与诊断信息，无需反复输入密码。",
    "server_rejected": "服务器拒绝了本次请求，请检查账号权限、服务器状态和诊断信息。",
    "login_required": "请先完成登录，再选择同步目录。",
    "certificate": "服务器证书验证失败，请检查证书域名和“可信证书”设置。",
    "network": "无法连接 NAS，请检查地址、网络和 Drive 服务。",
    "configuration_or_permission": "账号设置或 Drive 应用权限需要检查。",
    "download_permission": "当前账号没有 Drive 下载权限。",
    "authentication_or_protocol": "登录未通过，请检查账号密码及服务器认证要求。",
}


class SetupClient(QObject):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.process = QProcess(self)
        self.buffer = bytearray()
        self.callbacks = deque()
        self.timeout = QTimer(self)
        self.timeout.setSingleShot(True)
        self.timeout.setInterval(90000)
        self.timeout.timeout.connect(lambda: self._failed("连接超时，请重新尝试。"))
        self.process.readyReadStandardOutput.connect(self._read)
        self.process.readyReadStandardError.connect(lambda: self.process.readAllStandardError())
        self.process.errorOccurred.connect(lambda _error: self._failed("无法启动登录组件。"))
        self.process.finished.connect(lambda *_: self._failed("登录连接已结束。", kill=False))

    def request(self, payload, callback):
        if self.process.state() == QProcess.NotRunning:
            self.process.setProgram(binary("drive-native-setup"))
            self.process.setArguments(["--private-stdin"])
            self.process.start()
        encoded = (json.dumps(payload, ensure_ascii=False) + "\n").encode("utf-8")
        if len(encoded) > 65536:
            callback(None, "输入内容过长。")
            return
        self.callbacks.append(callback)
        self.process.write(encoded)
        self.timeout.start()

    def _read(self):
        self.buffer.extend(bytes(self.process.readAllStandardOutput()))
        if len(self.buffer) > 8 * 1024 * 1024:
            self._failed("服务器目录过大，当前选择器无法载入。")
            return
        while b"\n" in self.buffer:
            line, _, remaining = self.buffer.partition(b"\n")
            self.buffer = bytearray(remaining)
            if not self.callbacks:
                self._failed("登录组件返回了意外响应。")
                return
            callback = self.callbacks.popleft()
            try:
                response = json.loads(line)
                if response.get("schema") != 1:
                    raise ValueError()
                if response.get("ok"):
                    callback(response["data"], None)
                else:
                    callback(None, ERRORS.get(response.get("error"), "登录请求未完成。"))
            except (ValueError, KeyError, TypeError):
                callback(None, "无法读取登录响应。")
        if self.callbacks:
            self.timeout.start()
        else:
            self.timeout.stop()

    def _failed(self, error, kill=True):
        self.timeout.stop()
        callbacks, self.callbacks = self.callbacks, deque()
        self.buffer.clear()
        if kill:
            self.process.kill()
        for callback in callbacks:
            callback(None, error)

    def close(self):
        self.callbacks.clear()
        self.timeout.stop()
        self.process.kill()
