"""One tray for the manager; current transfers are not completion history."""
from pathlib import Path
from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QAction, QCursor
from PySide6.QtWidgets import (
    QApplication, QDialog, QVBoxLayout, QLabel, QTreeWidget, QTreeWidgetItem,
    QPushButton, QMenu, QSystemTrayIcon, QHeaderView,
)
from .window import status_icon
from .transfers import PHASES, size_text


class CurrentActivity(QDialog):
    def __init__(self, manager):
        super().__init__(manager, Qt.Tool)
        self.manager = manager
        self.setWindowTitle("当前同步活动 · Drive Native")
        self.resize(920, 420)
        self.pending = {}
        self.generation = 0
        layout = QVBoxLayout(self)
        note = QLabel("仅显示当前传输，不是持久同步历史。完成记录、历史文件定位和筛选尚未提供。")
        note.setWordWrap(True); layout.addWidget(note)
        self.table = QTreeWidget()
        self.table.setHeaderLabels(["挂载", "文件", "阶段", "正文进度"])
        self.table.setRootIsDecorated(False)
        self.table.header().setMinimumSectionSize(150)
        self.table.header().setSectionResizeMode(1, QHeaderView.Stretch)
        self.table.setColumnWidth(0, 160); self.table.setColumnWidth(2, 170); self.table.setColumnWidth(3, 180)
        self.table.itemActivated.connect(self.open_task)
        layout.addWidget(self.table)
        button = QPushButton("打开主窗口"); button.clicked.connect(lambda: manager.activate())
        layout.addWidget(button)
        self.timer = QTimer(self); self.timer.setInterval(2000)
        self.timer.timeout.connect(self.refresh)

    def showEvent(self, event):
        super().showEvent(event)
        self.refresh(); self.timer.start()

    def hideEvent(self, event):
        self.timer.stop(); self.generation += 1; self.pending.clear(); self.table.clear()
        super().hideEvent(event)

    def open_task(self, item, _column):
        slot = item.data(0, Qt.UserRole)
        if slot in self.manager.records:
            self.manager.activate(slot or 'legacy')

    def refresh(self):
        if not self.isVisible(): return
        self.pending = {slot: bus for slot, bus in self.pending.items() if self.manager.buses.get(slot) is bus}
        for index in reversed(range(self.table.topLevelItemCount())):
            if self.table.topLevelItem(index).data(0, Qt.UserRole) not in self.manager.records:
                self.table.takeTopLevelItem(index)
        generation = self.generation
        for slot, profile in list(self.manager.records.items()):
            if slot in self.pending: continue
            bus = self.manager.buses[slot]
            self.pending[slot] = bus
            expected = profile['task_id']
            title = Path(profile['mount']).name
            def received(data, error, slot=slot, expected=expected, title=title, bus=bus):
                if generation != self.generation or self.pending.get(slot) is not bus: return
                self.pending.pop(slot, None)
                if not self.isVisible(): return
                if self.manager.records.get(slot, {}).get('task_id') != expected: return
                if error or not isinstance(data, dict) or data.get('task_id') != expected:
                    rows = [("", "状态暂不可用", "")]
                else:
                    rows = []
                    for entry in data.get('entries', [])[:128]:
                        progress = entry.get('progress') or {}
                        done, total = progress.get('bytes'), progress.get('total')
                        text = (f"{size_text(done)} / {size_text(total)}" if type(done) is int and type(total) is int and 0 <= done <= total else "")
                        rows.append((entry.get('path') or '路径暂不可用', PHASES.get(progress.get('phase'), '等待传输'), text))
                    if not rows: rows = [("", "当前无正文传输", "")]
                for index in reversed(range(self.table.topLevelItemCount())):
                    if self.table.topLevelItem(index).data(0, Qt.UserRole) == slot:
                        self.table.takeTopLevelItem(index)
                if not error and isinstance(data, dict) and len(data.get('entries', [])) > 128:
                    rows.append(('', '仅显示前 128 项；打开此挂载查看传输', ''))
                for path, phase, progress in rows:
                    row = QTreeWidgetItem([title, path, phase, progress]); row.setData(0, Qt.UserRole, slot)
                    self.table.addTopLevelItem(row)
            self.manager.buses[slot].call('GetTransfers', callback=received)


class MountTray:
    def __init__(self, manager):
        self.manager = manager
        self.activity = CurrentActivity(manager)
        self.icon = QSystemTrayIcon(status_icon('local_complete'), manager)
        self.menu = QMenu(manager)
        self.menu.aboutToShow.connect(self.rebuild_menu)
        self.icon.setContextMenu(self.menu)
        self.icon.activated.connect(self.activated)
        self.icon.show()
        self.refresh()
        # A panel may disappear after startup. Never leave the only entry hidden.
        self.monitor = QTimer(manager); self.monitor.setInterval(3000)
        self.monitor.timeout.connect(self.ensure_reachable); self.monitor.start()

    def available(self):
        return QSystemTrayIcon.isSystemTrayAvailable() and self.icon.isVisible()

    def ensure_reachable(self):
        if not self.available() and not self.manager.isVisible():
            self.manager.showNormal()

    def activated(self, reason):
        if reason == QSystemTrayIcon.Trigger:
            if self.activity.isVisible(): self.activity.hide()
            else: self.show_activity()
        elif reason == QSystemTrayIcon.DoubleClick:
            self.manager.activate()

    def show_activity(self):
        self.activity.show(); self.activity.raise_()
        # Follow the panel icon; cursor fallback supports desktops without geometry.
        geometry = self.icon.geometry()
        anchor = geometry.center() if geometry.isValid() else QCursor.pos()
        screen = QApplication.screenAt(anchor) or QApplication.primaryScreen()
        if screen:
            bounds = screen.availableGeometry()
            self.activity.move(max(bounds.left(), min(anchor.x(), bounds.right() - self.activity.width())),
                               max(bounds.top(), min(anchor.y(), bounds.bottom() - self.activity.height())))

    def refresh(self):
        states = [data.get('connection') for data in self.manager.overviews.values()]
        unknown = len(states) != len(self.manager.records)
        if unknown or any(s in ('needs_attention', 'locked') for s in states):
            status, label = 'conflict', '有任务需要检查'
        elif any(s in ('connecting', 'offline', 'stopping') for s in states):
            status, label = 'pinning', '连接或恢复中'
        elif states and all(s == 'paused' for s in states):
            status, label = 'mixed', '全部已暂停'
        elif any(data.get('pending_operations', 0) for data in self.manager.overviews.values()):
            status, label = 'pending_upload', '有待处理操作'
        else:
            status, label = 'local_complete', '查看当前活动'
        self.icon.setIcon(status_icon(status))
        self.icon.setToolTip(f"Drive Native · {len(self.manager.records)} 个挂载 · {label}")

    def action(self, menu, title, callback):
        action = QAction(title, menu); action.triggered.connect(callback); menu.addAction(action)
        return action

    def rebuild_menu(self):
        self.menu.clear()
        self.action(self.menu, '打开主窗口', lambda: self.manager.activate())
        self.action(self.menu, '当前同步活动…', self.show_activity)
        for slot, profile in self.manager.records.items():
            sub = self.menu.addMenu(Path(profile['mount']).name)
            self.action(sub, '管理此挂载', lambda _checked=False, slot=slot: self.manager.activate(slot or 'legacy'))
            self.action(sub, '打开文件夹', lambda _checked=False, slot=slot: self.manager.open_task_folder(slot))
            state = self.manager.overviews.get(slot, {}).get('connection')
            action = self.action(sub, '恢复同步' if state == 'paused' else '暂停同步',
                                 lambda _checked=False, slot=slot: self.pause(slot))
            action.setEnabled(state not in (None, 'stopping'))
        self.menu.addSeparator()
        self.action(self.menu, '退出界面，继续同步', self.quit_interface)

    def pause(self, slot):
        if slot not in self.manager.records: return
        state = self.manager.overviews.get(slot, {}).get('connection')
        if state is None or state == 'stopping': return
        def done(_data, error):
            if error:
                self.manager.message.setText('暂停／恢复尚未确认，请查看该挂载状态。')
                self.manager.activate(slot or 'legacy')
            self.manager.refresh_status(slot)
        self.manager.buses[slot].call('Resume' if state == 'paused' else 'Pause', callback=done)

    def quit_interface(self):
        self.icon.hide()
        QApplication.instance().quit()
