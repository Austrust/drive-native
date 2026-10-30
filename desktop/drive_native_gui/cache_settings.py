"""Task-scoped cache controls backed by the local state owner."""
import json
import math
from PySide6.QtCore import QTimer
from PySide6.QtWidgets import (QDialog, QVBoxLayout, QHBoxLayout, QLabel,
    QCheckBox, QSpinBox, QComboBox, QPushButton, QFormLayout, QLayout, QSizePolicy)

MIB = 1024 ** 2
GIB = 1024 ** 3

def byte_text(value):
    value = max(0, int(value))
    for unit, factor in (("TiB", 1024**4), ("GiB", GIB), ("MiB", MIB), ("KiB", 1024)):
        if value >= factor:
            return f"{value / factor:.2f} {unit}"
    return f"{value} B"

class CacheSettingsDialog(QDialog):
    def __init__(self, bus, parent=None):
        super().__init__(parent)
        self.bus = bus
        self.task_id = None
        self.loaded = False
        self.loading = False
        self.saving = False
        self.dirty = False
        self.setWindowTitle("缓存设置 · Drive Native")
        self.resize(580, 540)
        layout = QVBoxLayout(self)
        layout.setSizeConstraint(QLayout.SizeConstraint.SetMinimumSize)
        layout.setContentsMargins(26, 24, 26, 24)
        layout.setSpacing(16)
        title = QLabel("管理本地空间")
        title.setObjectName("heading")
        layout.addWidget(title)
        explanation = QLabel("超过目标容量时，优先释放最久未打开的已同步文件。固定、未上传、冲突、更新未完成和正在使用的内容会保留。")
        explanation.setWordWrap(True)
        layout.addWidget(explanation)
        self.enabled = QCheckBox("自动管理缓存容量")
        layout.addWidget(self.enabled)
        row = QHBoxLayout()
        row.addWidget(QLabel("目标容量"))
        self.amount = QSpinBox()
        self.amount.setRange(1, 1048576)
        self.amount.setValue(20)
        self.amount.setAccessibleName("缓存目标容量")
        row.addWidget(self.amount, 1)
        self.unit = QComboBox()
        self.unit.addItems(["GiB", "MiB"])
        self.unit.setAccessibleName("容量单位")
        row.addWidget(self.unit)
        layout.addLayout(row)
        form = QFormLayout()
        self.used = QLabel("—")
        self.reclaimable = QLabel("—")
        self.protected = QLabel("—")
        form.addRow("本地正文占用", self.used)
        form.addRow("当前可自动释放", self.reclaimable)
        form.addRow("当前保留", self.protected)
        for row_index in range(form.rowCount()):
            for role in (QFormLayout.ItemRole.LabelRole, QFormLayout.ItemRole.FieldRole):
                label = form.itemAt(row_index, role).widget()
                label.setMinimumHeight(label.fontMetrics().height() + 8)
                label.setSizePolicy(QSizePolicy.Policy.Preferred, QSizePolicy.Policy.Minimum)
        layout.addLayout(form)
        self.note = QLabel("联网时自动管理；离线、暂停或锁定时保留现有缓存。刚下载的文件有 60 秒保留期。占用按正文大小计算，不含数据库。")
        self.note.setWordWrap(True)
        layout.addWidget(self.note)
        self.message = QLabel()
        self.message.setWordWrap(True)
        layout.addWidget(self.message)
        layout.addStretch()
        buttons = QHBoxLayout()
        self.refresh_button = QPushButton("刷新占用")
        self.refresh_button.clicked.connect(self.refresh)
        buttons.addWidget(self.refresh_button)
        buttons.addStretch()
        self.save_button = QPushButton("保存设置")
        self.save_button.clicked.connect(self.save)
        buttons.addWidget(self.save_button)
        close = QPushButton("关闭")
        close.clicked.connect(self.close)
        buttons.addWidget(close)
        layout.addLayout(buttons)
        self.enabled.toggled.connect(self.edited)
        self.amount.valueChanged.connect(self.edited)
        self.unit.currentIndexChanged.connect(self.edited)
        self.bus.changed.connect(self.schedule_refresh)
        self.bus.availability.connect(self.availability)
        self.timer = QTimer(self)
        self.timer.setSingleShot(True)
        self.timer.setInterval(300)
        self.timer.timeout.connect(self.refresh)
        self.update_controls()
        QTimer.singleShot(0, self.refresh)

    def edited(self, *_):
        self.dirty = True
        self.update_controls()

    def update_controls(self):
        self.amount.setMaximum(1024**3 if self.unit.currentText() == "MiB" else 1024**2)
        self.amount.setEnabled(self.enabled.isChecked() and not self.saving)
        self.unit.setEnabled(self.enabled.isChecked() and not self.saving)
        self.enabled.setEnabled(self.loaded and not self.saving)
        self.save_button.setEnabled(self.loaded and not self.saving and self.dirty)

    def availability(self, available):
        if not available:
            self.loaded = False
            self.message.setText("本地服务暂不可用，设置尚未更改。")
            self.update_controls()
        else:
            self.schedule_refresh()

    def schedule_refresh(self):
        if self.isVisible():
            self.timer.start()

    def refresh(self):
        if self.loading or self.saving:
            return
        self.loading = True
        self.bus.call("GetCacheSettings", callback=self.received)

    def received(self, data, error=None):
        self.loading = False
        if error or not isinstance(data, dict):
            self.loaded = False
            self.message.setText("无法读取缓存设置，请确认本地服务已启动。")
            self.update_controls()
            return
        try:
            usage = data["usage"]
            task_id = data["task_id"]
            limit = usage["limit_bytes"]
            fields = (limit, usage["content_bytes"], usage["reclaimable_bytes"], usage["protected_bytes"], usage["over_target_bytes"])
            if data.get("schema") != 1:
                raise ValueError()
            if not isinstance(task_id, str) or not task_id or any(type(n) is not int or n < 0 for n in fields):
                raise ValueError()
            if limit > 1024**5:
                raise ValueError()
        except (KeyError, TypeError, ValueError):
            self.loaded = False
            self.message.setText("本地服务返回的容量信息无效。")
            self.update_controls()
            return
        changed_task = self.task_id != task_id
        self.task_id = task_id
        if changed_task or not self.dirty:
            for widget in (self.enabled, self.amount, self.unit):
                widget.blockSignals(True)
            self.enabled.setChecked(limit != 0)
            factor = MIB if limit and (limit < GIB or limit % GIB) else GIB
            self.unit.setCurrentIndex(1 if factor == MIB else 0)
            self.amount.setMaximum(1024**3 if factor == MIB else 1024**2)
            self.amount.setValue(math.ceil(limit / factor) if limit else 20)
            for widget in (self.enabled, self.amount, self.unit):
                widget.blockSignals(False)
            self.dirty = False
        self.used.setText(byte_text(usage["content_bytes"]))
        self.reclaimable.setText(byte_text(usage["reclaimable_bytes"]))
        self.protected.setText(byte_text(usage["protected_bytes"]))
        if data.get("error"):
            self.message.setText("自动释放遇到本地错误，受保护内容仍保留。")
        elif usage["over_target_bytes"]:
            self.message.setText(f"当前超出目标 {byte_text(usage['over_target_bytes'])}。受保护内容允许超过目标，符合条件后自动释放。")
        elif data.get("connection") != "online":
            self.message.setText("当前没有在线同步，自动释放暂不运行。")
        else:
            self.message.setText("设置已由本地服务保存。" if not self.dirty else "修改尚未保存。")
        self.loaded = True
        self.update_controls()

    def save(self):
        if not self.loaded or self.saving:
            return
        factor = MIB if self.unit.currentText() == "MiB" else GIB
        limit = self.amount.value() * factor if self.enabled.isChecked() else 0
        body = json.dumps({"schema": 1, "task_id": self.task_id, "limit_bytes": limit})
        self.saving = True
        self.update_controls()
        self.bus.call("SetCacheSettings", [body], self.saved)

    def saved(self, data, error=None):
        self.saving = False
        if error:
            self.message.setText("设置未保存；请重新读取本地服务状态后再试。")
            self.update_controls()
            return
        self.dirty = False
        self.received(data)
