import os
import uuid
from pathlib import Path
from PySide6.QtCore import Qt, Signal
from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QFormLayout,
    QLineEdit,
    QLabel,
    QPushButton,
    QStackedWidget,
    QWidget,
    QTreeWidget,
    QTreeWidgetItem,
    QFileDialog,
    QCheckBox,
)
from .setup_client import SetupClient
from .trust import TrustImportDialog
from .config import save_profile
from .credentials import secret_job


class SetupWizard(QDialog):
    configured = Signal(dict, str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("连接你的 Synology Drive")
        self.resize(680, 600)
        self.client = SetupClient(self)
        self.device_id = str(uuid.uuid4())
        self.password_value = ""
        self.profile = None
        self.trusted_ca_pem = []
        self.connection_snapshot = None
        self.busy = False
        layout = QVBoxLayout(self)
        layout.setContentsMargins(28, 24, 28, 24)
        layout.setSpacing(16)
        self.title = QLabel("让文件随手可用")
        self.title.setObjectName("heading")
        layout.addWidget(self.title)
        self.subtitle = QLabel("连接 NAS，选择一个 Drive 文件夹。文件将在打开时下载。")
        self.subtitle.setWordWrap(True)
        layout.addWidget(self.subtitle)
        self.pages = QStackedWidget()
        layout.addWidget(self.pages, 1)
        account = QWidget()
        form = QFormLayout(account)
        form.setSpacing(14)
        self.host = QLineEdit()
        self.host.setPlaceholderText("域名或 IP 地址")
        self.tls = QLineEdit()
        self.tls.setPlaceholderText("留空时使用上面的地址")
        self.username = QLineEdit()
        self.password = QLineEdit()
        self.password.setEchoMode(QLineEdit.Password)
        for label, field in [
            ("NAS 地址", self.host),
            ("证书域名", self.tls),
            ("账号", self.username),
            ("密码", self.password),
        ]:
            form.addRow(label, field)
        hint = QLabel("通过 IP 连接时，可在“证书域名”填入 NAS 证书上的域名。")
        hint.setWordWrap(True)
        form.addRow(hint)
        self.trust_button = QPushButton("可信证书…（使用系统信任）")
        self.trust_button.clicked.connect(self.edit_trust)
        form.addRow(self.trust_button)
        self.pages.addWidget(account)
        choose = QWidget()
        choose_layout = QVBoxLayout(choose)
        choose_layout.setContentsMargins(0, 0, 0, 0)
        self.tree = QTreeWidget()
        self.tree.setHeaderLabel("选择要同步的 Drive 文件夹")
        choose_layout.addWidget(self.tree, 1)
        self.tree.itemExpanded.connect(self.expand)
        location = QHBoxLayout()
        name = os.environ.get("DRIVE_NATIVE_NEW_NAME", "Drive")
        self.mount = QLineEdit(str(Path.home() / "DriveMounts" / name) if os.environ.get("DRIVE_NATIVE_INSTANCE") else str(Path.home() / name))
        self.mount_name = QLineEdit(name)
        name_row = QHBoxLayout()
        name_row.addWidget(QLabel("挂载名称"))
        name_row.addWidget(self.mount_name, 1)
        choose_layout.addLayout(name_row)
        self.mount_name.setPlaceholderText("本地文件夹显示的名称")
        self.mount_name.textEdited.connect(self.rename_mount)
        self.mount.textChanged.connect(self.mount_changed)
        browse = QPushButton("选择位置…")
        browse.clicked.connect(self.browse)
        location.addWidget(QLabel("本地位置"))
        location.addWidget(self.mount, 1)
        location.addWidget(browse)
        choose_layout.addLayout(location)
        self.remember = QCheckBox("在系统钥匙环中保存登录凭据")
        self.remember.setChecked(True)
        choose_layout.addWidget(self.remember)
        self.pages.addWidget(choose)
        self.message = QLabel()
        self.message.setWordWrap(True)
        self.message.setObjectName("message")
        layout.addWidget(self.message)
        buttons = QHBoxLayout()
        buttons.addStretch()
        cancel = QPushButton("取消")
        cancel.clicked.connect(self.reject)
        self.next = QPushButton("连接 NAS")
        self.next.setObjectName("primary")
        self.next.clicked.connect(self.advance)
        buttons.addWidget(cancel)
        buttons.addWidget(self.next)
        layout.addLayout(buttons)
        self.password.returnPressed.connect(self.advance)

    def edit_trust(self):
        if self.busy:
            return
        dialog = TrustImportDialog(self.trusted_ca_pem, self)
        if dialog.exec() == QDialog.Accepted:
            self.trusted_ca_pem = dialog.editor.values()
            count = len(self.trusted_ca_pem)
            self.trust_button.setText(
                f"可信证书…（额外 {count} 张）" if count else "可信证书…（使用系统信任）"
            )

    def rename_mount(self, name):
        if name and name not in (".", "..") and "/" not in name and not any(ord(c) < 32 for c in name):
            self.mount.setText(str(Path(self.mount.text()).parent / name))

    def mount_changed(self, text):
        self.mount_name.setText(Path(text).name)

    def browse(self):
        path = QFileDialog.getExistingDirectory(self, "选择空的本地文件夹", self.mount.text())
        if path:
            self.mount.setText(path)

    def advance(self):
        if self.busy:
            return
        if self.pages.currentIndex() == 0:
            if not all(
                (self.host.text().strip(), self.username.text().strip(), self.password.text())
            ):
                self.message.setText("请填写 NAS 地址、账号和密码。")
                return
            self.connection_snapshot = {
                "host": self.host.text().strip(),
                "tls_name": self.tls.text().strip() or self.host.text().strip(),
                "username": self.username.text().strip(),
                "trusted_ca_pem": list(self.trusted_ca_pem),
            }
            self.password_value = self.password.text()
            self.busy = True
            self.next.setEnabled(False)
            self.message.setText("正在验证连接并读取 Drive 文件夹…")
            try:
                self.client.request(
                    {
                        "command": "login",
                        **self.connection_snapshot,
                        "password": self.password_value,
                        "device_id": self.device_id,
                    },
                    self.logged_in,
                )
            except Exception:
                self.logged_in(None, "后台登录组件不可用，请检查安装。")
        else:
            self.finish_setup()

    def logged_in(self, data, error):
        self.busy = False
        self.next.setEnabled(True)
        if error:
            self.password_value = ""
            self.message.setText(error)
            return
        self.password.clear()
        self.tree.clear()
        root = QTreeWidgetItem(["Synology Drive"])
        root.setData(0, Qt.UserRole, {"loaded": True})
        self.tree.addTopLevelItem(root)
        spaces = sorted(data.get("spaces", []), key=lambda r: (not r.get("personal", False), r["name"].casefold(), r["view_id"]))
        for space in spaces:
            item = QTreeWidgetItem([space["name"]])
            item.setData(0, Qt.UserRole, {**space, "loaded": False})
            if space.get("available"):
                item.setChildIndicatorPolicy(QTreeWidgetItem.ShowIndicator)
            else:
                item.setDisabled(True)
                item.setToolTip(0, "当前空间没有下载或同步权限。")
            root.addChild(item)
        root.setExpanded(True)
        self.tree.setCurrentItem(root)
        self.pages.setCurrentIndex(1)
        self.next.setText("开始同步")
        warnings = data.get("warnings", [])
        self.message.setText(
            "团队文件夹列表读取失败，请重新连接后重试。" if "team_folders_unavailable" in warnings else
            "未找到可同步空间，请检查 Drive 权限。" if not spaces else
            "展开“我的文件”或团队文件夹，再选择同步目录。"
        )
        self.subtitle.setText("从 Drive 顶层选择一个空间或子文件夹；正文仍在打开时下载。")

    def add_folders(self, parent, folders):
        for folder in folders:
            item = QTreeWidgetItem([folder["name"]])
            item.setData(0, Qt.UserRole, {**folder, "loaded": False})
            item.setChildIndicatorPolicy(QTreeWidgetItem.ShowIndicator)
            parent.addChild(item)

    def expand(self, item):
        record = item.data(0, Qt.UserRole)
        if not record or record.get("loaded") or record.get("loading"):
            return
        record["loading"] = True
        item.setData(0, Qt.UserRole, record)

        def received(data, error):
            record["loading"] = False
            if not error and (data.get("view_id") != record["view_id"] or data.get("path") != record["path"]):
                error = "目录响应与所选空间不一致，请重新连接。"
            if error:
                self.message.setText(error)
            else:
                record["loaded"] = True
                if record.get("path") == "/":
                    record["scope"] = data.get("root_scope")
                self.add_folders(item, data["folders"])
                if not item.childCount():
                    item.setChildIndicatorPolicy(QTreeWidgetItem.DontShowIndicator)
            item.setData(0, Qt.UserRole, record)

        self.client.request({"command": "list", "view_id": record["view_id"], "path": record["path"]}, received)

    def finish_setup(self):
        if self.profile is not None:
            self.complete()
            return
        item = self.tree.currentItem()
        selection = item.data(0, Qt.UserRole) if item else None
        if not selection or not selection.get("scope"):
            self.message.setText("请选择一个已验证的 Drive 文件夹。")
            return
        mount = Path(self.mount.text()).expanduser()
        try:
            name = self.mount_name.text()
            if not name or name in (".", "..") or "/" in name or any(ord(c) < 32 for c in name) or len(name.encode("utf-8")) > 255:
                raise ValueError("请输入有效的挂载名称，不能含斜杠或控制字符。")
            if not mount.is_absolute():
                raise ValueError("本地位置必须是绝对路径。")
            if mount.exists() and (not mount.is_dir() or any(mount.iterdir())):
                raise ValueError("请选择空文件夹，避免覆盖已有文件。")
            mount.mkdir(parents=True, exist_ok=True, mode=0o700)
            profile = {
                "schema": 1,
                "task_id": str(uuid.uuid4()),
                **self.connection_snapshot,
                "device_id": self.device_id,
                "scope": selection["scope"],
                "mount": str(mount.resolve()),
                "writable": bool(selection.get("writable", False)),
            }
            save_profile(profile)
            self.profile = profile
        except (OSError, ValueError) as error:
            self.message.setText(str(error))
            return
        if self.remember.isChecked():
            self.busy = True
            self.next.setEnabled(False)
            self.message.setText("正在保存到系统钥匙环…")
            self.keyring_job = secret_job(profile["task_id"], self.stored, self.password_value)
        else:
            self.complete()

    def stored(self, _value, error):
        self.busy = False
        self.next.setEnabled(True)
        if error:
            self.message.setText("配置已保存，但钥匙环尚不可用。本次仍可连接，下次需要输入密码。")
            self.next.setText("继续连接")
            return
        self.complete()

    def complete(self):
        password = self.password_value
        self.password_value = ""
        self.client.close()
        self.configured.emit(self.profile, password)
        self.accept()

    def reject(self):
        self.password.clear()
        self.password_value = ""
        self.client.close()
        super().reject()
