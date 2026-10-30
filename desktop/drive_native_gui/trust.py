"""Task-scoped public certificate import. The native service remains authoritative."""

import hashlib
import json
from pathlib import Path
from PySide6.QtCore import Qt
from PySide6.QtNetwork import QSslCertificate, QSsl
from PySide6.QtWidgets import (
    QWidget,
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QTreeWidget,
    QTreeWidgetItem,
    QHeaderView,
    QPushButton,
    QFileDialog,
    QDialogButtonBox,
)


class TrustEditor(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.certificates = []
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        hint = QLabel(
            "默认使用系统可信证书。只有 NAS 使用私有 CA 或自签名证书时，才需要在此添加其公钥证书。服务器名称仍会验证。"
        )
        hint.setWordWrap(True)
        hint.setTextFormat(Qt.PlainText)
        layout.addWidget(hint)
        self.table = QTreeWidget()
        self.table.setRootIsDecorated(False)
        self.table.setHeaderLabels(["证书", "有效期至", "SHA-256 指纹"])
        self.table.header().setSectionResizeMode(0, QHeaderView.Stretch)
        self.table.setColumnWidth(1, 115)
        self.table.setColumnWidth(2, 250)
        layout.addWidget(self.table, 1)
        buttons = QHBoxLayout()
        self.add = QPushButton("导入证书…")
        self.add.clicked.connect(self.choose)
        self.remove = QPushButton("移除选中证书")
        self.remove.clicked.connect(self.remove_selected)
        buttons.addWidget(self.add)
        buttons.addWidget(self.remove)
        buttons.addStretch()
        layout.addLayout(buttons)
        self.message = QLabel("没有额外信任的证书")
        self.message.setTextFormat(Qt.PlainText)
        self.message.setWordWrap(True)
        layout.addWidget(self.message)
        self.table.itemSelectionChanged.connect(
            lambda: self.remove.setEnabled(self.table.currentItem() is not None)
        )
        self.remove.setEnabled(False)

    @staticmethod
    def decode(data):
        if len(data) > 32768 or b"PRIVATE KEY" in data:
            raise ValueError("请选择公钥证书；不接受私钥，证书内容不得超过 32 KiB。")
        certs = QSslCertificate.fromData(data, QSsl.Pem if b"-----BEGIN" in data else QSsl.Der)
        if not certs or any(cert.isNull() for cert in certs):
            raise ValueError("无法读取证书，请选择 PEM 或 DER 格式的 X.509 公钥证书。")
        return certs

    def set_certificates(self, values):
        certs = []
        for value in values:
            parsed = self.decode(value.encode("ascii"))
            if len(parsed) != 1:
                raise ValueError("每个条目必须包含一张证书。")
            certs.extend(parsed)
        self.replace(certs)

    def replace(self, certs):
        pem = [bytes(cert.toPem()).decode("ascii") for cert in certs]
        fingerprints = [hashlib.sha256(bytes(cert.toDer())).hexdigest() for cert in certs]
        if len(certs) > 8 or sum(len(value) for value in pem) > 32768:
            raise ValueError("最多添加 8 张证书，总内容不得超过 32 KiB。")
        if len(set(fingerprints)) != len(fingerprints):
            raise ValueError("该证书已在列表中。")
        self.certificates = certs
        self.table.clear()
        for cert, fingerprint in zip(certs, fingerprints):
            subject = ", ".join(cert.subjectInfo(QSslCertificate.CommonName)) or "未命名证书"
            issuer = ", ".join(cert.issuerInfo(QSslCertificate.CommonName)) or "未命名颁发者"
            row = QTreeWidgetItem([subject, cert.expiryDate().toString("yyyy-MM-dd"), fingerprint])
            row.setToolTip(0, f"{subject}\n颁发者：{issuer}")
            row.setToolTip(2, fingerprint)
            self.table.addTopLevelItem(row)
        self.message.setText(
            f"额外信任 {len(certs)} 张证书 · 仅作用于当前同步任务"
            if certs
            else "没有额外信任的证书"
        )

    def values(self):
        return [bytes(cert.toPem()).decode("ascii") for cert in self.certificates]

    def choose(self):
        path, _ = QFileDialog.getOpenFileName(
            self, "导入 NAS 的公钥证书", "", "证书 (*.pem *.crt *.cer *.der);;所有文件 (*)"
        )
        if not path:
            return
        try:
            with Path(path).open("rb") as stream:
                data = stream.read(32769)
            self.replace(self.certificates + self.decode(data))
        except (OSError, ValueError) as error:
            self.message.setText(str(error))

    def remove_selected(self):
        item = self.table.currentItem()
        if item is not None:
            index = self.table.indexOfTopLevelItem(item)
            self.replace(self.certificates[:index] + self.certificates[index + 1 :])


class TrustImportDialog(QDialog):
    def __init__(self, certificates, parent=None):
        super().__init__(parent)
        self.setWindowTitle("连接证书 · Drive Native")
        self.resize(860, 470)
        layout = QVBoxLayout(self)
        self.editor = TrustEditor(self)
        self.editor.set_certificates(certificates)
        layout.addWidget(self.editor)
        buttons = QDialogButtonBox(QDialogButtonBox.Ok | QDialogButtonBox.Cancel)
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        layout.addWidget(buttons)


class TrustSettingsDialog(QDialog):
    def __init__(self, bus, parent=None):
        super().__init__(parent)
        self.bus = bus
        self.current = None
        self.busy = False
        self.setWindowTitle("连接证书 · Drive Native")
        self.resize(900, 560)
        layout = QVBoxLayout(self)
        self.identity = QLabel("正在读取当前任务…")
        self.identity.setTextFormat(Qt.PlainText)
        self.identity.setWordWrap(True)
        layout.addWidget(self.identity)
        self.editor = TrustEditor(self)
        layout.addWidget(self.editor, 1)
        self.message = QLabel("保存后只重连网络；挂载和本地内容会保留，已暂停的任务仍保持暂停。")
        self.message.setTextFormat(Qt.PlainText)
        self.message.setWordWrap(True)
        layout.addWidget(self.message)
        actions = QHBoxLayout()
        self.refresh = QPushButton("重新读取")
        self.refresh.clicked.connect(self.load)
        self.save = QPushButton("保存并重连")
        self.save.setObjectName("primary")
        self.save.clicked.connect(self.apply)
        close = QPushButton("关闭")
        close.clicked.connect(self.close)
        actions.addWidget(self.refresh)
        actions.addStretch()
        actions.addWidget(self.save)
        actions.addWidget(close)
        layout.addLayout(actions)
        self.load()

    def controls(self):
        self.editor.setEnabled(not self.busy and self.current is not None)
        self.save.setEnabled(not self.busy and self.current is not None)
        self.refresh.setEnabled(not self.busy)

    def load(self):
        if self.busy:
            return
        self.busy = True
        self.controls()
        self.bus.call("GetTrustSettings", callback=self.received)

    def received(self, data, error):
        self.busy = False
        if not error:
            try:
                if data["schema"] != 1 or data["settings"]["schema"] != 1:
                    raise ValueError()
                self.editor.set_certificates(data["settings"]["trusted_ca_pem"])
                self.identity.setText(f"NAS：{data['host']}\n验证的服务器名称：{data['tls_name']}")
                self.current = data
                self.controls()
                return
            except (ValueError, KeyError, TypeError):
                error = True
        self.current = None
        self.message.setText("无法读取可信证书，请恢复服务后重新读取。")
        self.controls()

    def apply(self):
        if self.busy or self.current is None:
            return
        body = {
            "expected_version": self.current["version"],
            "settings": {
                "schema": 1,
                "task_id": self.current["settings"]["task_id"],
                "trusted_ca_pem": self.editor.values(),
            },
        }
        self.busy = True
        self.controls()
        self.bus.call("SetTrustSettings", [json.dumps(body)], self.saved)

    def saved(self, data, error):
        self.busy = False
        if error:
            self.message.setText("未能确认保存。请重新读取当前任务的证书设置，检查后再次保存。")
            self.controls()
            return
        self.received(data, None)
        if self.current is not None:
            self.message.setText("可信证书已保存，网络连接将使用新设置。")
