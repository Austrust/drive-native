"""Manage independent mounts; credentials remain in each task's existing UI/keyring."""
import os
import sys
import uuid
from pathlib import Path
from PySide6.QtCore import Qt, QTimer, QProcess
from PySide6.QtGui import QDesktopServices
from PySide6.QtCore import QUrl
from PySide6.QtWidgets import QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QTreeWidget, QTreeWidgetItem, QInputDialog
from .config import profile_records
from .bus import ClientBus
from .lifecycle import ServiceLauncher
from .config import service_unit
from .initialization import progress_text

LABELS = {"locked":"等待解锁", "connecting":"连接中", "online":"已连接", "offline":"离线", "paused":"已暂停", "needs_attention":"需要处理", "stopping":"正在停止"}


class MountManager(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Drive Native · 挂载管理")
        self.resize(980, 600)
        self.setMinimumSize(720, 420)
        self.records = {}
        self.buses = {}
        self.items = {}
        self.launchers = []
        self.signature = None
        self.progress_pending = set()
        self.progress_again = set()
        root = QWidget(); self.setCentralWidget(root)
        layout = QVBoxLayout(root)
        title = QLabel("我的挂载"); title.setStyleSheet("font-size: 25px; font-weight: 600; padding: 12px 0;")
        layout.addWidget(title)
        layout.addWidget(QLabel("多个目录可以同时挂载。每个挂载独立连接、缓存和同步，关闭窗口不会停止挂载。"))
        bar = QHBoxLayout()
        for label, fn in [("添加挂载…", self.add), ("打开管理", self.open_task), ("启动挂载", self.start), ("暂停 / 恢复", self.pause), ("打开文件夹", self.open_folder)]:
            button = QPushButton(label); button.clicked.connect(fn); bar.addWidget(button)
        bar.addStretch(); layout.addLayout(bar)
        self.table = QTreeWidget(); self.table.setHeaderLabels(["挂载名称", "状态", "远端目录", "本地位置"])
        self.table.setRootIsDecorated(False); self.table.setAlternatingRowColors(True)
        self.table.itemDoubleClicked.connect(lambda *_: self.open_task())
        layout.addWidget(self.table, 1)
        self.message = QLabel(); self.message.setWordWrap(True); layout.addWidget(self.message)
        self.timer = QTimer(self); self.timer.setInterval(2000); self.timer.timeout.connect(self.refresh_catalog); self.timer.start()
        self.refresh_catalog()

    def selected(self):
        item = self.table.currentItem()
        return item.data(0, Qt.UserRole) if item else None

    def refresh_catalog(self):
        try:
            records = dict(profile_records())
        except (OSError, ValueError, KeyError) as error:
            self.message.setText(str(error)); return
        signature = tuple((key, p["task_id"], p["mount"]) for key, p in records.items())
        if signature == self.signature:
            return
        selected = self.selected(); self.signature = signature; self.records = records
        self.table.clear(); self.items = {}
        for slot, profile in records.items():
            item = QTreeWidgetItem([Path(profile["mount"]).name, "正在读取状态…", profile["scope"]["path"], profile["mount"]])
            item.setData(0, Qt.UserRole, slot); self.table.addTopLevelItem(item); self.items[slot] = item
            if slot == selected: self.table.setCurrentItem(item)
            if slot not in self.buses:
                bus = ClientBus(self, instance=slot); self.buses[slot] = bus
                bus.changed.connect(lambda key=slot: self.refresh_status(key))
                bus.progress_changed.connect(lambda key=slot: self.refresh_progress(key))
                bus.availability.connect(lambda _up, key=slot: self.refresh_status(key))
            self.refresh_status(slot)
        if not records: self.message.setText("点击“添加挂载”，为第一个目录设置名称和本地位置。")
        else: self.message.setText(f"共 {len(records)} 个挂载。双击任务可登录、查看传输或处理错误。")
        for col in range(3): self.table.resizeColumnToContents(col)

    def refresh_status(self, slot):
        if slot not in self.items: return
        def received(data, error):
            if slot not in self.items: return
            profile = self.records[slot]
            if error:
                self.items[slot].setText(1, "未启动"); return
            if data.get("task_id") != profile["task_id"]:
                self.items[slot].setText(1, "任务不匹配"); return
            self.items[slot].setText(1, LABELS.get(data.get("connection"), "状态未知"))
            self.items[slot].setData(1, Qt.UserRole, data.get("connection"))
            self.render_progress(slot, data.get("initialization"))
        self.buses[slot].call("GetOverview", callback=received)

    def render_progress(self, slot, progress):
        title, detail = progress_text(progress)
        item = self.items[slot]
        if title:
            item.setToolTip(1, detail)
            if item.data(1, Qt.UserRole) in (None, "connecting", "online"):
                item.setText(1, title)
        else:
            item.setToolTip(1, "")

    def refresh_progress(self, slot):
        if not self.isVisible() or slot not in self.items:
            return
        if slot in self.progress_pending:
            self.progress_again.add(slot)
            return
        self.progress_pending.add(slot)
        def received(data, error):
            self.progress_pending.discard(slot)
            if slot in self.progress_again:
                self.progress_again.discard(slot)
                QTimer.singleShot(100, lambda: self.refresh_progress(slot))
            if error or slot not in self.items or data.get("task_id") != self.records[slot]["task_id"]:
                return
            self.render_progress(slot, data.get("initialization"))
        self.buses[slot].call("GetInitialization", callback=received)

    def spawn(self, slot, name=None):
        args = ["--task", slot or "legacy"]
        if name: args += ["--new-name", name]
        if getattr(sys, "frozen", False): program = sys.executable
        else:
            program = sys.executable; args = ["-m", "drive_native_gui"] + args
        ok, _ = QProcess.startDetached(program, args)
        if not ok: self.message.setText("无法打开挂载配置窗口。")

    def add(self):
        name, ok = QInputDialog.getText(self, "添加挂载", "挂载名称（本地文件夹名称，可在配置时修改）")
        if not ok: return
        name = name.strip()
        if not name or name in (".", "..") or "/" in name or any(ord(c) < 32 for c in name) or len(name.encode()) > 255:
            self.message.setText("请输入有效名称，不能含斜杠或控制字符。"); return
        self.spawn(uuid.uuid4().hex, name)

    def open_task(self):
        slot = self.selected()
        if slot is not None: self.spawn(slot)

    def start(self):
        slot = self.selected()
        if slot is None: return
        launcher = ServiceLauncher(self, managed=True, unit=service_unit(slot))
        self.launchers.append(launcher)
        def done(error):
            if error: self.message.setText(error)
            self.refresh_status(slot)
            self.launchers.remove(launcher); launcher.deleteLater()
        launcher.completed.connect(done); launcher.start(enable=True)

    def pause(self):
        slot = self.selected()
        if slot is None: return
        state = self.items[slot].data(1, Qt.UserRole)
        self.buses[slot].call("Resume" if state == "paused" else "Pause", callback=lambda _data, _error: self.refresh_status(slot))

    def open_folder(self):
        slot = self.selected()
        if slot is not None: QDesktopServices.openUrl(QUrl.fromLocalFile(self.records[slot]["mount"]))
