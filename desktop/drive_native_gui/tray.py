"""One tray for current transfers and persistent, service-bound history."""
from pathlib import Path
import os
import select
from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QAction, QCursor
from PySide6.QtDBus import QDBusMessage, QDBusPendingCallWatcher
from PySide6.QtWidgets import (
    QApplication, QDialog, QVBoxLayout, QLabel, QTreeWidget, QTreeWidgetItem,
    QPushButton, QMenu, QSystemTrayIcon, QHeaderView, QWidget, QTabWidget, QMessageBox,
)
from .window import status_icon
from .transfers import PHASES, size_text
from .queue_health import uncertain_operations
from .history import HistoryPage


class CurrentActivity(QDialog):
    def __init__(self, manager):
        super().__init__(manager, Qt.Tool)
        self.manager = manager
        self.setWindowTitle("最近活动 · Drive Native")
        self.resize(920, 420)
        self.pending = {}
        self.generation = 0
        layout = QVBoxLayout(self)
        note = QLabel("当前传输与已保存的同步历史分别显示；历史记录可打开或定位当前文件。")
        note.setWordWrap(True); layout.addWidget(note)
        tabs = QTabWidget(); layout.addWidget(tabs, 1)
        current = QWidget(); current_layout = QVBoxLayout(current)
        tabs.addTab(current, "当前传输")
        self.history = HistoryPage(manager.history_sources, self, recent=True)
        tabs.addTab(self.history, "近期历史")
        self.table = QTreeWidget()
        self.table.setHeaderLabels(["挂载", "文件", "阶段", "正文进度"])
        self.table.setRootIsDecorated(False)
        self.table.header().setMinimumSectionSize(150)
        self.table.header().setSectionResizeMode(1, QHeaderView.Stretch)
        self.table.setColumnWidth(0, 160); self.table.setColumnWidth(2, 170); self.table.setColumnWidth(3, 180)
        self.table.itemActivated.connect(self.open_task)
        current_layout.addWidget(self.table)
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
        self.paused_by_all = {}
        self.pause_intents = {}
        self.bulk_pending = set()
        self.stop_requested = {}
        self.stop_failures = set()
        self.stop_owners = {}
        self.stop_pidfds = {}
        self.stop_accepted = set()
        self.stop_identity_pending = set()
        self.stop_checks = set()
        self.stopping = False
        self.stop_generation = 0
        self.stop_poll = QTimer(manager); self.stop_poll.setInterval(500)
        self.stop_poll.timeout.connect(self.poll_stopping)
        manager.destroyed.connect(self.close_stop_fds)
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
        if (unknown or any(s in ('needs_attention', 'locked') for s in states)
                or any(uncertain_operations(data) for data in self.manager.overviews.values())):
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
        uploads = sum(count(data.get('uploading', data.get('pending_operations', 0)))
                      for data in self.manager.overviews.values())
        issues = sum(count(data.get('download_errors')) + uncertain_operations(data)
                     + int(data.get('connection') in ('needs_attention', 'locked'))
                     for data in self.manager.overviews.values())
        self.icon.setToolTip(f"Drive Native · {len(self.manager.records)} 个挂载，{uploads} 项上传/待同步，{issues} 个问题 · {label}")

    def action(self, menu, title, callback):
        action = QAction(title, menu); action.triggered.connect(callback); menu.addAction(action)
        return action

    def rebuild_menu(self):
        self.menu.clear()
        self.action(self.menu, '打开主窗口', lambda: self.manager.activate())
        self.action(self.menu, '当前同步活动…', self.show_activity)
        self.action(self.menu, '同步历史', lambda: self.manager.show_history())
        for slot, profile in self.manager.records.items():
            sub = self.menu.addMenu(Path(profile['mount']).name)
            self.action(sub, '管理此挂载', lambda _checked=False, slot=slot: self.manager.activate(slot or 'legacy'))
            self.action(sub, '打开文件夹', lambda _checked=False, slot=slot: self.manager.open_task_folder(slot))
            state = self.manager.overviews.get(slot, {}).get('connection')
            action = self.action(sub, '恢复同步' if state == 'paused' else '暂停同步',
                                 lambda _checked=False, slot=slot: self.pause(slot))
            action.setEnabled(state not in (None, 'stopping'))
        self.menu.addSeparator()
        pause_all = self.action(self.menu, '全部暂停', self.pause_all)
        pause_all.setEnabled(not self.bulk_pending and not self.stopping)
        resume_all = self.action(self.menu, '恢复由“全部暂停”暂停的挂载', self.resume_all)
        resume_all.setEnabled(bool(self.paused_by_all) and not self.bulk_pending and not self.stopping)
        self.action(self.menu, '全局设置', self.manager.show_settings)
        self.menu.addSeparator()
        self.action(self.menu, '退出界面，继续同步', self.quit_interface)
        self.action(self.menu, '停止全部同步并退出', self.stop_all_and_quit)

    def pause(self, slot):
        if slot not in self.manager.records or self.stopping or slot in self.bulk_pending: return
        self.manual_pause_intent(slot)
        state = self.manager.overviews.get(slot, {}).get('connection')
        if state is None or state == 'stopping': return
        def done(_data, error):
            if error:
                self.manager.message.setText('暂停／恢复尚未确认，请查看该挂载状态。')
                self.manager.activate(slot or 'legacy')
            self.manager.refresh_status(slot)
        self.manager.buses[slot].call('Resume' if state == 'paused' else 'Pause', callback=done)

    def manual_pause_intent(self, slot):
        # A later manual action owns the pause even when the earlier bulk
        # request has not replied yet. Its late callback cannot reclaim it.
        self.pause_intents[slot] = self.pause_intents.get(slot, 0) + 1
        self.paused_by_all.pop(slot, None)

    def pause_all(self):
        if self.stopping or self.bulk_pending: return
        for slot, profile in list(self.manager.records.items()):
            state = self.manager.overviews.get(slot, {}).get('connection')
            if state in (None, 'paused', 'stopping', 'locked'): continue
            self.bulk_action(slot, profile['task_id'], 'Pause')

    def resume_all(self):
        if self.stopping or self.bulk_pending: return
        for slot, task in list(self.paused_by_all.items()):
            if self.manager.records.get(slot, {}).get('task_id') != task:
                self.paused_by_all.pop(slot, None); continue
            if self.manager.overviews.get(slot, {}).get('connection') == 'paused':
                self.bulk_action(slot, task, 'Resume')

    def bulk_action(self, slot, task, method):
        bus = self.manager.buses[slot]
        intent = self.pause_intents.get(slot, 0)
        self.bulk_pending.add(slot)
        def done(data, error):
            self.bulk_pending.discard(slot)
            if self.manager.buses.get(slot) is not bus or self.manager.records.get(slot, {}).get('task_id') != task: return
            if intent != self.pause_intents.get(slot, 0):
                self.manager.refresh_status(slot)
                return
            if error or (isinstance(data, dict) and data.get('error')):
                self.manager.message.setText('部分挂载暂停／恢复尚未确认，请查看各挂载状态。')
            elif method == 'Pause': self.paused_by_all[slot] = task
            else: self.paused_by_all.pop(slot, None)
            self.manager.refresh_status(slot)
        bus.call(method, callback=done)

    def stop_all_and_quit(self):
        if self.stopping: return
        if QMessageBox.question(self.manager, '停止全部同步并退出',
                '请先保存并关闭挂载中的文件。将正常停止全部挂载，确认本地数据保存与挂载卸载后退出。',
                QMessageBox.Yes | QMessageBox.No, QMessageBox.No) != QMessageBox.Yes:
            return
        self.stopping = True
        self.stop_generation += 1
        self.stop_identity_pending.clear()
        self.close_stop_fds()
        self.stop_requested = {slot: (p['task_id'], self.manager.buses[slot])
                               for slot, p in self.manager.records.items() if slot in self.manager.buses}
        self.stop_failures.clear()
        self.stop_accepted.clear(); self.stop_owners.clear()
        self.manager.activate()
        self.manager.message.setText('正在正常停止全部挂载，确认停止后退出；不会强制结束后台。')
        for slot, (task, bus) in list(self.stop_requested.items()):
            if not getattr(bus, 'owner', None):
                # Losing the bus name is not evidence of process exit or
                # unmount. Keep the unresolved task and leave the UI reachable.
                self.stop_failures.add(slot)
                continue
            self.stop_identity_pending.add(slot)
            self.capture_stop_identity(slot, task, bus)
        self.stop_poll.start(); self.poll_stopping()

    def capture_stop_identity(self, slot, task, bus):
        generation = self.stop_generation
        owner = bus.owner
        if not getattr(bus, 'bus', None):
            self.stop_identity_pending.discard(slot); self.stop_failures.add(slot); return
        message = QDBusMessage.createMethodCall('org.freedesktop.DBus', '/org/freedesktop/DBus',
                                               'org.freedesktop.DBus', 'GetConnectionUnixProcessID')
        message.setArguments([owner])
        check = QDBusPendingCallWatcher(bus.bus.asyncCall(message, 3000), self.manager)
        self.stop_checks.add(check)
        def identified(*_):
            self.stop_checks.discard(check)
            if generation != self.stop_generation:
                check.deleteLater(); return
            self.stop_identity_pending.discard(slot)
            if not self.stopping or check.isError() or owner != bus.owner:
                self.stop_failures.add(slot); check.deleteLater(); return
            try:
                pid = int(check.reply().arguments()[0])
                self.stop_pidfds[slot] = os.pidfd_open(pid)
                self.stop_owners[slot] = owner
            except (OSError, ValueError, AttributeError, IndexError):
                self.stop_failures.add(slot); check.deleteLater(); return
            check.deleteLater()
            def done(data, error, slot=slot, task=task, bus=bus):
                if generation != self.stop_generation: return
                if self.manager.records.get(slot, {}).get('task_id') != task or self.manager.buses.get(slot) is not bus:
                    self.stop_failures.add(slot)
                elif (error and error != 'service_changed') or (isinstance(data, dict) and data.get('error')):
                    self.stop_failures.add(slot)
                else:
                    self.stop_accepted.add(slot)
                watcher = self.manager.shutdown_watchers.get(slot)
                if watcher: watcher.refresh()
            bus.call('StopMount', callback=done)
        check.finished.connect(identified)

    def poll_stopping(self):
        if not self.stopping: return
        complete = []
        for slot, (task, bus) in self.stop_requested.items():
            if slot in self.stop_failures: continue
            if self.manager.records.get(slot, {}).get('task_id') != task or self.manager.buses.get(slot) is not bus:
                self.stop_failures.add(slot); continue
            if slot in self.stop_identity_pending: continue
            owner = self.stop_owners.get(slot)
            if owner and bus.owner and owner != bus.owner:
                self.stop_failures.add(slot); continue
            watcher = self.manager.shutdown_watchers.get(slot)
            if watcher and watcher.value and watcher.value.get('phase') == 'complete' and watcher.mounted is False:
                complete.append(slot)
            elif watcher and watcher.value and watcher.value.get('phase') == 'failed':
                self.stop_failures.add(slot)
            elif slot in self.stop_accepted and slot in self.stop_pidfds and select.select([self.stop_pidfds[slot]], [], [], 0)[0]:
                # Like BackendUpgrade: owner-bound cooperative StopMount plus
                # pidfd exit is the barrier, never the request receipt alone.
                complete.append(slot)
            elif watcher: watcher.refresh()
        for slot in complete:
            self.stop_requested.pop(slot, None)
            fd = self.stop_pidfds.pop(slot, None)
            if fd is not None: os.close(fd)
        if self.stop_failures and not (set(self.stop_requested) - self.stop_failures):
            self.stop_poll.stop(); self.stopping = False
            self.stop_generation += 1
            self.close_stop_fds()
            self.manager.message.setText('部分挂载尚未确认正常停止，界面保持打开。请检查停止阶段；不会强杀后台或清理数据。')
        elif not self.stop_requested:
            self.stop_poll.stop(); self.stopping = False; self.quit_interface()
        elif self.stop_failures:
            self.manager.message.setText('正在停止其余挂载；部分挂载身份或停止结果尚未确认，全部完成前保持界面打开。')

    def quit_interface(self):
        self.stop_generation += 1
        self.stop_poll.stop()
        self.close_stop_fds()
        self.icon.hide()
        QApplication.instance().quit()

    def close_stop_fds(self, *_):
        for fd in self.stop_pidfds.values(): os.close(fd)
        self.stop_pidfds.clear()


def count(value):
    return value if type(value) is int and 0 <= value < 1 << 64 else 0
