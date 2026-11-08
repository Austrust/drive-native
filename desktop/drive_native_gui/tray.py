"""One tray for current transfers and persistent, service-bound history."""
from pathlib import Path
import os
import select
import time
from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QAction, QColor, QCursor, QIcon, QPainter, QPixmap
from PySide6.QtDBus import QDBusMessage, QDBusPendingCallWatcher
from PySide6.QtWidgets import (
    QApplication, QDialog, QVBoxLayout, QLabel, QTreeWidget, QTreeWidgetItem,
    QPushButton, QMenu, QSystemTrayIcon, QHeaderView, QWidget, QTabWidget, QMessageBox,
)
from .window import status_icon
from .theme import status_color, status_icon as status_glyph
from .shutdown import STOP_DEADLINE_SECONDS, blocking_reason_text
from .transfers import PHASES, size_text
from .queue_health import uncertain_operations
from .history import HistoryPage


def tray_icon(kind):
    """Draw a tray icon from the shared semantic palette (theme.py owns the colors)."""
    try:
        color = QColor(status_color(kind))
        glyph = status_glyph(kind)
    except (KeyError, TypeError):
        # Keep the tray meaningful if theme.py ever loses a semantic kind.
        return status_icon({"idle": "local_complete", "attention": "conflict",
                            "paused": "mixed", "unknown": "pinning"}.get(kind, "local_complete"))
    pixmap = QPixmap(22, 22); pixmap.fill(Qt.transparent)
    painter = QPainter(pixmap)
    try:
        painter.setRenderHint(QPainter.Antialiasing)
        painter.setPen(Qt.NoPen); painter.setBrush(color)
        painter.drawEllipse(1, 1, 20, 20)
        painter.setPen(QColor("#20242b" if color.lightness() > 150 else "#ffffff"))
        font = painter.font(); font.setPixelSize(13); font.setBold(True); painter.setFont(font)
        painter.drawText(pixmap.rect(), Qt.AlignCenter, glyph)
    finally:
        painter.end()
    return QIcon(pixmap)


def count(value):
    return value if type(value) is int and 0 <= value < 1 << 64 else 0


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
        self.history = HistoryPage(manager.history_sources, self, recent=True, mount_actions=manager)
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
        self.stop_deadline = None
        self.notifications = None
        self.stop_poll = QTimer(manager); self.stop_poll.setInterval(500)
        self.stop_poll.timeout.connect(self.poll_stopping)
        manager.destroyed.connect(self.close_stop_fds)
        self.activity = CurrentActivity(manager)
        # "unknown" until the first overview arrives: never claim healthy early.
        self.icon = QSystemTrayIcon(tray_icon('unknown'), manager)
        self.menu = QMenu(manager)
        self.menu.aboutToShow.connect(self.rebuild_menu)
        self.icon.setContextMenu(self.menu)
        self.icon.activated.connect(self.activated)
        self.icon.messageClicked.connect(self.notification_clicked)
        self.icon.show()
        self.refresh()
        # A panel may disappear after startup. Never leave the only entry hidden.
        self.monitor = QTimer(manager); self.monitor.setInterval(3000)
        self.monitor.timeout.connect(self.ensure_reachable); self.monitor.start()

    def available(self):
        return QSystemTrayIcon.isSystemTrayAvailable() and self.icon.isVisible()

    def show_message(self, title, body, *, warning=False):
        # Notifications are a convenience: without a usable panel the window banner is the channel.
        if not self.available():
            return False
        icon = QSystemTrayIcon.Warning if warning else QSystemTrayIcon.Information
        self.icon.showMessage(title, body, icon, 8000)
        return True

    def notification_clicked(self):
        if self.notifications is not None:
            self.notifications.clicked()

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
        records = self.manager.records
        fresh = {slot: data for slot, data in self.manager.overviews.items()
                 if slot in records and data.get('task_id') == records[slot].get('task_id')}
        counts = self.manager.transfer_counts() if hasattr(self.manager, 'transfer_counts') else {}
        active = sum(counts.values())
        pending = sum(count(data.get('pending_operations')) for data in fresh.values())
        uncertain = sum(uncertain_operations(data) for data in fresh.values())
        issues = 0
        for data in fresh.values():
            value = data.get('issue_count')
            if type(value) is int:
                issues += max(0, value)
            else:
                # Older backends have no issue_count; count only known signals.
                issues += count(data.get('download_errors')) + uncertain_operations(data)
                issues += int(data.get('connection') in ('needs_attention', 'locked'))
                issues += int(bool(data.get('error')))
        attention = any(data.get('connection') in ('needs_attention', 'locked') or bool(data.get('error'))
                        for data in fresh.values())
        states = [data.get('connection') for data in fresh.values()]
        paused_any = any(state == 'paused' for state in states)
        paused_all = bool(states) and all(state == 'paused' for state in states)
        if not records:
            kind, label = 'idle', '尚未添加挂载'
        elif len(fresh) != len(records):
            kind, label = 'unknown', '正在检查状态'
        elif attention or issues > 0:
            kind = 'attention'
            label = '有任务需要检查' + (f'（{issues} 项问题）' if issues else '')
        elif any(state in ('connecting', 'offline', 'stopping') for state in states):
            kind, label = 'syncing', '连接或恢复中'
        elif active:
            kind, label = 'syncing', f'正在传输 {active} 项'
        elif paused_any:
            kind, label = 'paused', '全部已暂停' if paused_all else '部分已暂停'
        elif pending or uncertain:
            parts = []
            if pending: parts.append(f'{pending} 项待同步')
            if uncertain: parts.append(f'{uncertain} 项提交待确认')
            kind, label = 'syncing', ' · '.join(parts)
        else:
            kind, label = 'idle', '查看当前活动'
        self.icon.setIcon(tray_icon(kind))
        self.icon.setToolTip(self.tooltip_text(fresh, active, pending, uncertain, issues, label))

    def tooltip_text(self, fresh, active, pending, uncertain, issues, label):
        parts = [f"Drive Native · {len(self.manager.records)} 个挂载"]
        if fresh:
            parts.append(f"，{issues} 个问题")
        else:
            parts.append("，问题数暂不可用")
        if active: parts.append(f"，{active} 项传输中")
        if pending: parts.append(f"，{pending} 项待同步")
        if uncertain: parts.append(f"，{uncertain} 项提交待确认")
        return ''.join(parts) + f" · {label}"

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
            stop = self.action(sub, '停止此挂载并卸载…',
                               lambda _checked=False, slot=slot: self.manager.stop_task(slot))
            stop.setEnabled(state != 'stopping')
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
        self.stop_deadline = time.monotonic() + STOP_DEADLINE_SECONDS
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
        if self.stop_deadline is not None and time.monotonic() >= self.stop_deadline:
            # A cooperative stop that never settles must say so; it never force-kills.
            reasons = []
            for slot in self.stop_requested:
                watcher = self.manager.shutdown_watchers.get(slot)
                value = watcher.value if watcher is not None else None
                reason = blocking_reason_text(value) if isinstance(value, dict) else ''
                if reason and reason not in reasons: reasons.append(reason)
            for slot in list(self.stop_requested): self.stop_failures.add(slot)
            self.stop_poll.stop(); self.stopping = False
            self.stop_generation += 1
            self.stop_deadline = None
            self.close_stop_fds()
            self.manager.activate()
            detail = f"（阻塞原因：{'、'.join(reasons)}）" if reasons else ''
            self.manager.message.setText(
                f'等待后台确认停止超时（已等待超过 {STOP_DEADLINE_SECONDS} 秒）{detail}；界面保持打开，'
                '不会强制结束后台。请打开主窗口查看停止诊断并复制诊断信息。')
            return
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
            self.stop_deadline = None
            self.close_stop_fds()
            self.manager.message.setText('部分挂载尚未确认正常停止，界面保持打开。请检查停止阶段；不会强杀后台或清理数据。')
        elif not self.stop_requested:
            self.stop_poll.stop(); self.stopping = False
            self.stop_deadline = None
            self.quit_interface()
        elif self.stop_failures:
            self.manager.message.setText('正在停止其余挂载；部分挂载身份或停止结果尚未确认，全部完成前保持界面打开。')

    def quit_interface(self):
        self.stop_generation += 1
        self.stop_deadline = None
        self.stop_poll.stop()
        self.close_stop_fds()
        self.icon.hide()
        QApplication.instance().quit()

    def close_stop_fds(self, *_):
        for fd in self.stop_pidfds.values(): os.close(fd)
        self.stop_pidfds.clear()
