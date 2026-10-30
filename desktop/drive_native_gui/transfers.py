"""Live local transfer progress and task-scoped payload bandwidth settings."""

import json
from PySide6.QtCore import Qt, QTimer
from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QTreeWidget,
    QTreeWidgetItem,
    QHeaderView,
    QProgressBar,
    QPushButton,
    QSpinBox,
    QFormLayout,
)

PHASES = {
    "queued": "等待传输",
    "preparing": "准备内容",
    "transferring": "传输中",
    "verifying": "校验内容",
    "awaiting_confirmation": "等待服务器确认",
}


def size_text(value):
    value = max(0, float(value))
    for unit in ("B", "KiB", "MiB", "GiB", "TiB"):
        if value < 1024 or unit == "TiB":
            return f"{value:.1f} {unit}" if unit != "B" else f"{value:.0f} B"
        value /= 1024


class TransfersDialog(QDialog):
    def __init__(self, bus, parent=None):
        super().__init__(parent)
        self.bus = bus
        self.current = None
        self.loading = False
        self.saving = False
        self.setWindowTitle("传输与带宽 · Drive Native")
        self.resize(1040, 640)
        layout = QVBoxLayout(self)
        self.summary = QLabel("正在读取当前传输…")
        self.summary.setTextFormat(Qt.PlainText)
        layout.addWidget(self.summary)
        self.table = QTreeWidget()
        self.table.setRootIsDecorated(False)
        self.table.setHeaderLabels(["文件", "方向", "正文进度", "阶段", "平均速度"])
        self.table.header().setSectionResizeMode(0, QHeaderView.Stretch)
        for col, width in [(1, 70), (2, 230), (3, 160), (4, 115)]:
            self.table.setColumnWidth(col, width)
        layout.addWidget(self.table, 1)
        hint = QLabel(
            "正文传完后仍需校验或服务器确认。文件是否已同步，以文件列表中的同步状态为准。"
        )
        hint.setWordWrap(True)
        hint.setTextFormat(Qt.PlainText)
        layout.addWidget(hint)
        form = QFormLayout()
        self.download = self.rate_field()
        self.upload = self.rate_field()
        form.addRow("下载上限", self.download)
        form.addRow("上传上限", self.upload)
        layout.addLayout(form)
        hint = QLabel(
            "上限由当前任务同一方向的正文传输共享；目录和变更通知不计入。保存后对进行中的传输生效。"
        )
        hint.setWordWrap(True)
        hint.setTextFormat(Qt.PlainText)
        layout.addWidget(hint)
        self.message = QLabel()
        self.message.setWordWrap(True)
        self.message.setTextFormat(Qt.PlainText)
        layout.addWidget(self.message)
        buttons = QHBoxLayout()
        self.reload = QPushButton("重新读取设置")
        self.reload.clicked.connect(self.load_settings)
        self.save = QPushButton("保存带宽设置")
        self.save.setObjectName("primary")
        self.save.clicked.connect(self.apply)
        close = QPushButton("关闭")
        close.clicked.connect(self.close)
        buttons.addWidget(self.reload)
        buttons.addStretch()
        buttons.addWidget(self.save)
        buttons.addWidget(close)
        layout.addLayout(buttons)
        self.timer = QTimer(self)
        self.timer.setInterval(1000)
        self.timer.timeout.connect(self.refresh)
        self.bus.changed.connect(self.refresh_visible)
        self.bus.availability.connect(self.availability)
        self.load_settings()
        self.refresh()

    def rate_field(self):
        field = QSpinBox()
        field.setRange(0, 1048576)
        field.setSpecialValueText("不限速")
        field.setSuffix(" KiB/s")
        field.setSingleStep(64)
        return field

    def showEvent(self, event):
        super().showEvent(event)
        self.timer.start()
        self.refresh()

    def hideEvent(self, event):
        self.timer.stop()
        super().hideEvent(event)

    def refresh_visible(self, *_):
        if self.isVisible():
            self.refresh()

    def availability(self, available):
        self.current = None
        self.controls()
        self.table.setEnabled(False)
        self.summary.setText("服务已变化，请重新读取设置和进度。")
        if available:
            self.load_settings()
            self.refresh()

    def controls(self):
        enabled = self.current is not None and not self.saving
        self.save.setEnabled(enabled)
        self.download.setEnabled(enabled)
        self.upload.setEnabled(enabled)
        self.reload.setEnabled(not self.saving)

    def load_settings(self):
        if self.saving:
            return
        self.current = None
        self.controls()
        self.bus.call("GetTransferSettings", callback=self.settings_received)

    def settings_received(self, data, error):
        self.saving = False
        if not error:
            try:
                if (
                    data["schema"] != 1
                    or not isinstance(data["task_id"], str)
                    or not data["task_id"]
                    or not isinstance(data["version"], str)
                    or not data["version"]
                ):
                    raise ValueError()
                down, up = data["limits"]["download_bps"], data["limits"]["upload_bps"]
                if any(
                    type(v) is not int or v < 0 or v > 1073741824 or v % 1024 for v in (down, up)
                ):
                    raise ValueError()
                self.current = data
                self.download.setValue(down // 1024)
                self.upload.setValue(up // 1024)
                self.controls()
                return
            except (KeyError, TypeError, ValueError):
                pass
        self.current = None
        self.message.setText("无法读取带宽设置，请恢复服务后重新读取。")
        self.controls()

    def apply(self):
        if self.current is None or self.saving:
            return
        body = {
            "schema": 1,
            "task_id": self.current["task_id"],
            "expected_version": self.current["version"],
            "limits": {
                "download_bps": self.download.value() * 1024,
                "upload_bps": self.upload.value() * 1024,
            },
        }
        self.saving = True
        self.controls()
        self.bus.call("SetTransferSettings", [json.dumps(body)], self.saved)

    def saved(self, data, error):
        self.saving = False
        if error:
            self.current = None
            self.message.setText("未能确认应用设置。请重新读取当前任务的设置后重试。")
            self.controls()
            return
        self.settings_received(data, None)
        if self.current is not None:
            self.message.setText("带宽设置已保存；暂停的任务仍保持暂停。")

    def refresh(self):
        if self.loading:
            return
        self.loading = True
        self.bus.call("GetTransfers", callback=self.received)

    def received(self, data, error):
        self.loading = False
        if error or not isinstance(data, dict) or data.get("schema") != 1:
            self.summary.setText("无法读取实时进度，以下显示可能已过期。")
            self.table.setEnabled(False)
            return
        if self.current is not None and data.get("task_id") != self.current["task_id"]:
            self.summary.setText("当前同步任务已变化，请重新读取设置。")
            self.current = None
            self.controls()
            self.table.clear()
            return
        self.table.setEnabled(True)
        self.table.clear()
        entries = data.get("entries", [])
        self.summary.setText(
            f"当前 {len(entries)} 项正文传输"
            if entries
            else "当前没有正在传输的正文。待处理操作和冲突请查看主界面。"
        )
        for entry in entries:
            p = entry.get("progress")
            path = entry.get("path") or "文件路径暂不可用"
            if not p:
                row = QTreeWidgetItem([path, "—", "等待网络", "等待传输", "—"])
                self.table.addTopLevelItem(row)
                continue
            done, total = p["bytes"], p["total"]
            if type(done) is not int or type(total) is not int or done < 0 or total < done:
                continue
            elapsed = max(1, entry.get("elapsed_ms", 1)) / 1000
            speed = size_text(done / elapsed) + "/s" if done else "—"
            row = QTreeWidgetItem(
                [
                    path,
                    "下载" if p["direction"] == "download" else "上传",
                    "",
                    PHASES.get(p["phase"], "等待状态"),
                    speed,
                ]
            )
            row.setToolTip(0, path)
            self.table.addTopLevelItem(row)
            progress = QProgressBar()
            progress.setRange(0, 1000)
            progress.setValue(int(done * 1000 / total) if total else 0)
            progress.setFormat(f"{size_text(done)} / {size_text(total)}")
            self.table.setItemWidget(row, 2, progress)
