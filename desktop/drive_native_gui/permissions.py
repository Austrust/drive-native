"""Explain effective decisions without turning unknown into a grant."""
import json
from PySide6.QtCore import Qt
from PySide6.QtWidgets import QDialog, QVBoxLayout, QLabel, QComboBox, QDialogButtonBox


def permission_text(overview, directory=None):
    policy = overview or {}
    if policy.get("local_policy") == "explicit_read_only":
        return "用户设为只读 · 本地修改不会上传"
    if policy.get("server_capability") == "space_forbidden":
        return "服务禁止写入 · 请检查 Drive 空间设置"
    actions = (directory or {}).get("actions", {})
    if any(value.get("state") == "denied" for value in actions.values() if isinstance(value, dict)):
        return "当前目录权限不足 · 仅限制被拒绝的操作，可重新核对"
    if not actions or any(value.get("state") == "unknown" for value in actions.values() if isinstance(value, dict)):
        return "权限待验证 · 按远端权限尝试，由服务器确认"
    return "已验证的操作可用 · 按远端权限同步"


class PermissionSettings(QDialog):
    def __init__(self, bus, overview, parent=None):
        super().__init__(parent)
        self.setWindowTitle("读写设置")
        self.bus, self.overview = bus, dict(overview)
        layout = QVBoxLayout(self)
        self.choice = QComboBox()
        self.choice.addItem("按远端目录权限同步", "follow_remote")
        self.choice.addItem("仅供读取", "explicit_read_only")
        self.choice.setCurrentIndex(1 if overview.get("local_policy") == "explicit_read_only" else 0)
        layout.addWidget(self.choice)
        self.message = QLabel("保存后重新挂载以完整应用读写设置。重新核对权限不会更改此选择。")
        self.message.setWordWrap(True)
        self.message.setTextFormat(Qt.PlainText)
        layout.addWidget(self.message)
        self.buttons = QDialogButtonBox(QDialogButtonBox.Save | QDialogButtonBox.Cancel)
        self.buttons.accepted.connect(self.save)
        self.buttons.rejected.connect(self.reject)
        layout.addWidget(self.buttons)

    def save(self):
        self.buttons.setEnabled(False)
        body = {"task_id": self.overview["task_id"],
                "expected_revision": self.overview["write_policy_revision"],
                "local_policy": self.choice.currentData()}
        def done(data, error):
            self.buttons.setEnabled(True)
            if error:
                self.message.setText("未能保存，请刷新状态后重试。")
            elif data and data.get("saved"):
                self.message.setText("已保存。请停止并重新启动此挂载；本地文件会保留。")
                self.buttons.button(QDialogButtonBox.Save).setEnabled(False)
        self.bus.call("SetWritePolicy", (json.dumps(body),), callback=done)
