"""Manage independent mounts; credentials remain in each task's existing UI/keyring."""
import json
import time
import uuid
from pathlib import Path
from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import QDesktopServices, QKeySequence, QShortcut
from PySide6.QtCore import QUrl
from PySide6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QTreeWidget,
    QTreeWidgetItem, QInputDialog, QStackedWidget, QApplication, QHeaderView, QSplitter,
    QTabWidget, QMenu, QCheckBox, QGroupBox, QMessageBox,
)
from .config import profile_records, config_path, load_gui_settings, save_gui_settings
from .bus import ClientBus
from .lifecycle import ServiceLauncher
from .config import service_unit
from .initialization import progress_text
from .shutdown import STOP_DEADLINE_SECONDS, blocking_reason_text, shutdown_text, ShutdownWatcher
from .theme import apply_window_theme
from .queue_health import queue_caption, queue_notice, uncertain_operations
from .history import HistoryPage, HistorySource
from .transfers import PHASES as TRANSFER_PHASES, transfer_row
from .notifications import NotificationCenter

LABELS = {"locked":"等待解锁", "connecting":"连接中", "online":"已连接", "offline":"离线", "paused":"已暂停", "needs_attention":"需要处理", "stopping":"正在停止"}
STOP_TIMEOUT_SECONDS = STOP_DEADLINE_SECONDS
SUCCESS_POLL_MS = 30 * 1000
TRANSFER_POLL_MS = 2000
TRANSFER_HIDDEN_POLL_MS = 5000
STATUS_SIGNAL_INTERVAL = 1.0
TRANSFER_LIMIT = 128


def count(value):
    return value if type(value) is int and 0 <= value < 1 << 64 else 0


def active_transfer(entry):
    """A cache entry only counts while it describes a real in-flight transfer."""
    if not isinstance(entry, dict):
        return False
    progress = entry.get("progress")
    if not isinstance(progress, dict) or not progress:
        return False
    if progress.get("phase") not in TRANSFER_PHASES:
        return False
    if progress.get("direction") not in ("upload", "download"):
        return False
    done, total = progress.get("bytes"), progress.get("total")
    return type(done) is int and type(total) is int and 0 <= done <= total


class MountManager(QMainWindow):
    transfers_changed = Signal()

    def __init__(self):
        super().__init__()
        self.setWindowTitle("Drive Native")
        self.resize(1240, 780)
        self.setMinimumSize(720, 420)
        self.pages = {}
        self.nav_items = {}
        self.overviews = {}
        self.shutdown_watchers = {}
        self.status_pending = set()
        self.status_again = set()
        self.records = {}
        self.buses = {}
        self.items = {}
        self.launchers = []
        self.signature = None
        self.progress_pending = set()
        self.progress_again = set()
        self.gui_settings = load_gui_settings()
        self.stop_pending = {}
        self.shortcuts = []
        self.transfer_cache = {}
        self.transfer_pending = set()
        self.transfer_again = set()
        root = QWidget(); self.setCentralWidget(root)
        shell = QVBoxLayout(root)
        navigation = QHBoxLayout()
        add_button = QPushButton("添加挂载…"); add_button.clicked.connect(self.add)
        navigation.addWidget(add_button)
        navigation.addWidget(QLabel("关闭界面不会停止后台同步")); navigation.addStretch()
        self.exit_button = QPushButton("退出…")
        navigation.addWidget(self.exit_button)
        shell.addLayout(navigation)
        body = QSplitter(Qt.Horizontal); shell.addWidget(body, 1)
        self.navigation = QTreeWidget(); self.navigation.setHeaderHidden(True)
        self.navigation.setMinimumWidth(150); self.navigation.setMaximumWidth(300)
        self.navigation.setAccessibleName("挂载导航")
        self.navigation.itemActivated.connect(self.open_navigation_item)
        body.addWidget(self.navigation)
        self.stack = QStackedWidget(); body.addWidget(self.stack)
        self.all_page = QWidget(); self.stack.addWidget(self.all_page)
        self.build_settings_page()
        self.stack.addWidget(self.settings_page)
        self.navigation.currentItemChanged.connect(self.navigate)
        all_layout = QVBoxLayout(self.all_page)
        self.all_tabs = QTabWidget(); all_layout.addWidget(self.all_tabs)
        all_overview = QWidget(); self.all_tabs.addTab(all_overview, "概览")
        self.history_page = HistoryPage(self.history_sources, self, mount_actions=self)
        self.all_tabs.addTab(self.history_page, "同步历史")
        layout = QVBoxLayout(all_overview)
        title = QLabel("我的挂载"); title.setStyleSheet("font-size: 25px; font-weight: 600; padding: 12px 0;")
        layout.addWidget(title)
        layout.addWidget(QLabel("多个目录可以同时挂载。每个挂载独立连接、缓存和同步，关闭窗口不会停止挂载。"))
        bar = QHBoxLayout()
        for label, fn in [("查看挂载", self.open_task), ("启动挂载", self.start), ("暂停 / 恢复", self.pause), ("打开文件夹", self.open_folder)]:
            button = QPushButton(label); button.clicked.connect(fn); bar.addWidget(button)
        bar.addStretch(); layout.addLayout(bar)
        self.table = QTreeWidget()
        self.table.setHeaderLabels(["挂载名称", "状态", "同步活动", "问题", "远端目录", "本地位置"])
        self.table.setRootIsDecorated(False); self.table.setAlternatingRowColors(True)
        self.table.header().setMinimumSectionSize(120)
        self.table.header().setSectionResizeMode(QHeaderView.Stretch)
        self.table.itemDoubleClicked.connect(lambda *_: self.open_task())
        self.table.setAccessibleName("所有挂载列表")
        self.table.setContextMenuPolicy(Qt.CustomContextMenu)
        self.table.customContextMenuRequested.connect(self.show_mount_menu)
        layout.addWidget(self.table, 1)
        self.transfer_summary = QLabel("当前没有正在传输的文件")
        self.transfer_summary.setObjectName("transferSummary")
        self.transfer_summary.setAccessibleName("所有挂载的当前传输摘要")
        self.transfer_summary.setWordWrap(True)
        layout.addWidget(self.transfer_summary)
        self.transfer_table = QTreeWidget()
        self.transfer_table.setObjectName("transferTable")
        self.transfer_table.setAccessibleName("所有挂载的当前传输")
        self.transfer_table.setHeaderLabels(["挂载", "文件", "阶段", "正文进度"])
        self.transfer_table.setRootIsDecorated(False)
        self.transfer_table.setAlternatingRowColors(True)
        self.transfer_table.header().setMinimumSectionSize(120)
        self.transfer_table.header().setSectionResizeMode(QHeaderView.Stretch)
        self.transfer_table.setMaximumHeight(180)
        self.transfer_table.setVisible(False)
        layout.addWidget(self.transfer_table)
        self.message = QLabel(); self.message.setWordWrap(True); layout.addWidget(self.message)
        body.setStretchFactor(1, 1); body.setSizes([190, 1000])
        self.timer = QTimer(self); self.timer.setInterval(2000); self.timer.timeout.connect(self.refresh_catalog); self.timer.start()
        self.stop_timer = QTimer(self); self.stop_timer.setInterval(500); self.stop_timer.timeout.connect(self.poll_stop_tasks)
        # Transfers are polled even while hidden: the tray must stay accurate
        # without the main window; the backend call is cheap and per mount.
        self.transfer_timer = QTimer(self); self.transfer_timer.setInterval(TRANSFER_POLL_MS)
        self.transfer_timer.timeout.connect(self.refresh_transfers); self.transfer_timer.start()
        # Signal-driven overview refreshes are coalesced per mount (issue #29);
        # the polled fallback in refresh_catalog stays unthrottled.
        self.status_queried = {}
        self.status_timers = {}
        self.overview_queries = 0
        self.refresh_catalog()
        from .tray import MountTray
        self.tray = MountTray(self)
        # Delivery policy is injected so the GUI and tests share one center.
        self.notifications = NotificationCenter(self.tray, self.notification_target, parent=self)
        self.notifications.set_enabled(self.gui_settings["notifications"])
        self.tray.notifications = self.notifications
        self.success_timer = QTimer(self); self.success_timer.setInterval(SUCCESS_POLL_MS)
        self.success_timer.timeout.connect(self.poll_success_notifications)
        if self.notifications.enabled_success():
            self.success_timer.start()
        self.exit_menu = QMenu(self.exit_button)
        self.exit_menu.addAction("退出界面，继续同步", self.tray.quit_interface)
        self.exit_menu.addAction("停止全部同步并退出", self.tray.stop_all_and_quit)
        self.exit_button.setMenu(self.exit_menu)
        self.install_shortcuts()
        self.apply_theme()

    def build_settings_page(self):
        self.settings_page = QWidget()
        self.settings_page.setAccessibleName("全局界面设置")
        settings = QVBoxLayout(self.settings_page)
        heading = QLabel("全局界面设置")
        heading.setStyleSheet("font-size: 25px; font-weight: 600; padding: 12px 0;")
        settings.addWidget(heading)
        intro = QLabel("这里的设置只影响桌面通知和界面行为，不会改变任何挂载的连接、目录、缓存或远端数据；修改会立即保存。")
        intro.setWordWrap(True); settings.addWidget(intro)

        notifications_box = QGroupBox("桌面通知")
        notifications_layout = QVBoxLayout(notifications_box)
        self.problems_check = QCheckBox("同步问题通知（需要登录、需要处理、持续同步失败）")
        self.problems_check.setAccessibleName("同步问题通知")
        notifications_layout.addWidget(self.problems_check)
        self.success_check = QCheckBox("同步成功通知（默认关闭；按挂载合并，不逐个文件提示）")
        self.success_check.setAccessibleName("同步成功通知")
        notifications_layout.addWidget(self.success_check)
        notification_note = QLabel("托盘不可用时只在界面内显示状态，不发送系统通知。同一挂载的同类问题 30 分钟内不会重复提示；点击通知可打开对应挂载。")
        notification_note.setWordWrap(True); notifications_layout.addWidget(notification_note)
        settings.addWidget(notifications_box)

        history_box = QGroupBox("同步历史")
        history_layout = QVBoxLayout(history_box)
        history_note = QLabel("同步历史默认保留 30 天 / 10,000 条，用于查看已完成的同步记录；清理只删除历史记录，不会删除本地或远端文件。保留期限可在各挂载的“同步历史”页面调整。")
        history_note.setWordWrap(True); history_layout.addWidget(history_note)
        self.open_history_button = QPushButton("打开同步历史")
        self.open_history_button.setAccessibleName("打开同步历史")
        self.open_history_button.clicked.connect(lambda: self.show_history())
        history_layout.addWidget(self.open_history_button)
        settings.addWidget(history_box)

        behavior_box = QGroupBox("行为与键盘")
        behavior_layout = QVBoxLayout(behavior_box)
        behavior = QLabel("关闭主窗口时，托盘可用则隐藏窗口并继续后台同步；托盘不可用时保留窗口入口。\n退出菜单可选择仅退出界面，或正常停止全部同步并退出。\n键盘：Ctrl+1…Ctrl+5 切换当前挂载页面的标签页；Ctrl+, 打开全局设置；F6 在导航树与右侧页面之间切换焦点；在导航树中按 Enter 打开所选项。")
        behavior.setWordWrap(True); behavior_layout.addWidget(behavior)
        settings.addWidget(behavior_box)

        self.settings_message = QLabel(); self.settings_message.setWordWrap(True)
        settings.addWidget(self.settings_message)
        settings.addStretch()

        self.problems_check.setChecked(bool(self.gui_settings["notifications"]["problems"]))
        self.success_check.setChecked(bool(self.gui_settings["notifications"]["success"]))
        # Connect after the stored state is displayed, so startup never rewrites it.
        self.problems_check.toggled.connect(self.set_problem_notifications)
        self.success_check.toggled.connect(self.set_success_notifications)

    def install_shortcuts(self):
        for index in range(1, 6):
            shortcut = QShortcut(QKeySequence(f"Ctrl+{index}"), self)
            shortcut.setObjectName(f"action_tab_{index}")
            shortcut.setContext(Qt.WindowShortcut)
            shortcut.activated.connect(lambda index=index: self.activate_tab(index))
            self.shortcuts.append(shortcut)
        shortcut = QShortcut(QKeySequence("Ctrl+,"), self)
        shortcut.setObjectName("action_show_settings")
        shortcut.setContext(Qt.WindowShortcut)
        shortcut.activated.connect(self.show_settings)
        self.shortcuts.append(shortcut)
        extra = [
            ("action_add_mount", "Ctrl+Shift+A", self.add),
            ("action_show_history", "Ctrl+H", self.show_history),
            ("action_hide_window", "Ctrl+W", self.hide_to_tray),
            ("action_exit_menu", "Ctrl+Q", self.open_exit_menu),
            ("action_focus_cycle", "F6", self.focus_cycle),
        ]
        for name, keys, callback in extra:
            shortcut = QShortcut(QKeySequence(keys), self)
            shortcut.setObjectName(name)
            shortcut.setContext(Qt.WindowShortcut)
            shortcut.activated.connect(callback)
            self.shortcuts.append(shortcut)

    def apply_theme(self):
        apply_window_theme(self)
        self.setStyleSheet(self.styleSheet() + "QListWidget{background:palette(base);color:palette(text);border:1px solid palette(midlight);selection-background-color:palette(highlight);selection-color:palette(highlighted-text);} QListWidget::item{padding:12px 10px;}")
        for page in self.pages.values(): page.apply_theme()

    def hide_to_tray(self):
        """Ctrl+W hides only when the tray keeps the app reachable."""
        if self.tray.available():
            self.hide()
            return True
        self.showNormal(); self.raise_(); self.activateWindow()
        self.message.setText("托盘不可用，窗口保持打开；如需仅退出界面并继续同步，请使用“退出…”菜单。")
        return False

    def open_exit_menu(self):
        """Ctrl+Q shows the two explicit exit choices instead of quitting directly."""
        self.activate()
        self.exit_menu.popup(self.exit_button.mapToGlobal(self.exit_button.rect().bottomLeft()))
        return self.exit_menu

    def render_shutdown(self, slot, value, mounted):
        if slot not in self.items: return
        text = shutdown_text(value)
        if text:
            self.items[slot].setText(1, text)
            self.items[slot].setData(1, Qt.UserRole, 'stopping')
            if slot in self.overviews and type(mounted) is bool:
                self.overviews[slot]['mounted'] = mounted

    def selected(self):
        item = self.table.currentItem()
        return item.data(0, Qt.UserRole) if item else None

    def refresh_catalog(self):
        try:
            records = dict(profile_records())
        except (OSError, ValueError, KeyError) as error:
            self.message.setText(str(error)); return
        signature = tuple((key, p["task_id"], p["mount"], p["scope"]["path"]) for key, p in records.items())
        if signature == self.signature:
            return
        selected = self.selected(); self.signature = signature
        for slot in set(self.records) | set(self.pages):
            old, current = self.records.get(slot), records.get(slot)
            page = self.pages.get(slot)
            changed = (old is not None and (current is None or old["task_id"] != current["task_id"]))
            arrived = old is None and current is not None and page is not None
            if changed or arrived:
                old_task = old["task_id"] if old else None
                keep_page = page is not None and (
                    page.owns_catalog_transition(old_task, current)
                    or (old is None and current is not None and page.profile is not None
                        and page.profile.get("task_id") == current["task_id"]))
                if keep_page and old is None:
                    page.catalog_replacement = None
                if page is not None and not keep_page:
                    self.pages.pop(slot)
                    self.stack.removeWidget(page); page.hide(); page.deleteLater()
                self.overviews.pop(slot, None)
                self.shutdown_watchers.pop(slot, None)
                old_bus = self.buses.pop(slot, None)
                if old_bus is not None: old_bus.deleteLater()
                self.status_pending.discard(slot); self.status_again.discard(slot)
                self.progress_pending.discard(slot); self.progress_again.discard(slot)
                self.transfer_pending.discard(slot); self.transfer_again.discard(slot)
                self.transfer_cache.pop(slot, None)
                self.stop_pending.pop(slot, None)
                if hasattr(self, "notifications"): self.notifications.forget(slot)
        self.records = records
        self.table.clear(); self.items = {}
        for slot, profile in records.items():
            item = QTreeWidgetItem([Path(profile["mount"]).name, "正在读取状态…", "—", "—",
                                    profile["scope"]["path"], profile["mount"]])
            item.setData(0, Qt.UserRole, slot); self.table.addTopLevelItem(item); self.items[slot] = item
            if slot == selected: self.table.setCurrentItem(item)
            if slot not in self.buses:
                bus = ClientBus(self, instance=slot); self.buses[slot] = bus
                bus.changed.connect(lambda key=slot: self.schedule_status(key))
                bus.progress_changed.connect(lambda key=slot: self.refresh_progress(key))
                bus.availability.connect(lambda _up, key=slot: self.schedule_status(key))
                watcher = ShutdownWatcher(bus, lambda key=slot: self.records.get(key, {}).get("task_id"), lambda value, mounted, key=slot: self.render_shutdown(key, value, mounted), bus)
                self.shutdown_watchers[slot] = watcher
                if hasattr(bus, "shutdown_changed"): watcher.refresh()
            if slot in self.overviews:
                self.update_activity(slot)
            self.refresh_status(slot)
        self.rebuild_navigation()
        self.render_transfers()
        self.history_page.refresh()
        for page in self.pages.values(): page.history_page.refresh()
        if hasattr(self, "tray"): self.tray.activity.history.refresh()
        if hasattr(self, "tray"): self.tray.refresh()
        if not records: self.message.setText("点击“添加挂载”，为第一个目录设置名称和本地位置。")
        else: self.message.setText(f"共 {len(records)} 个挂载。双击任务可登录、查看传输或处理错误。")


    def schedule_status(self, slot):
        """Coalesce StatusChanged/availability bursts into one query per second.

        The old path answered every signal and then re-queried 100 ms after any
        overlapping reply, which kept an unbounded signal/query loop alive
        (issue #29).
        """
        if slot not in self.items: return
        if slot in self.status_pending:
            self.status_again.add(slot); return
        elapsed = time.monotonic() - self.status_queried.get(slot, 0.0)
        delay = int((STATUS_SIGNAL_INTERVAL - elapsed) * 1000)
        if delay <= 0:
            self.refresh_status(slot)
            return
        timer = self.status_timers.get(slot)
        if timer is None:
            timer = QTimer(self); timer.setSingleShot(True)
            timer.timeout.connect(lambda key=slot: self._scheduled_status(key))
            self.status_timers[slot] = timer
        if not timer.isActive():
            timer.start(delay)

    def _scheduled_status(self, slot):
        timer = self.status_timers.get(slot)
        if timer is not None: timer.stop()
        self.refresh_status(slot)

    def refresh_status(self, slot):
        if slot not in self.items: return
        if slot in self.status_pending:
            self.status_again.add(slot); return
        self.status_pending.add(slot)
        self.status_queried[slot] = time.monotonic()
        self.overview_queries += 1
        expected = self.records[slot]["task_id"]
        bus = self.buses[slot]
        def received(data, error):
            if self.buses.get(slot) is not bus: return
            self.status_pending.discard(slot)
            if slot in self.status_again:
                self.status_again.discard(slot)
                self.schedule_status(slot)
            if slot not in self.items: return
            profile = self.records[slot]
            if profile["task_id"] != expected: return
            self.overviews.pop(slot, None)
            self.items[slot].setData(1, Qt.UserRole, None)
            if error:
                self.items[slot].setText(1, "状态暂不可用")
                self.items[slot].setText(2, "—"); self.items[slot].setText(3, "—")
                if hasattr(self, "tray"): self.tray.refresh()
                return
            if data.get("task_id") != profile["task_id"]:
                self.items[slot].setText(1, "任务不匹配"); return
            self.overviews[slot] = data
            self.items[slot].setText(1, LABELS.get(data.get("connection"), "状态未知"))
            self.items[slot].setData(1, Qt.UserRole, data.get("connection"))
            self.render_progress(slot, data.get("initialization"))
            self.update_activity(slot)
            if hasattr(self, "notifications"):
                self.notifications.observe_overview(slot, Path(profile["mount"]).name, data)
            watcher = self.shutdown_watchers.get(slot)
            stop = shutdown_text(watcher.value if watcher and watcher.value is not None else data.get("shutdown"))
            if stop:
                self.render_shutdown(slot, watcher.value if watcher and watcher.value is not None else data.get("shutdown"), watcher.mounted if watcher and watcher.value is not None else data.get("mounted"))
            if hasattr(self, "tray"): self.tray.refresh()
        self.buses[slot].call("GetOverview", callback=received)

    def update_activity(self, slot):
        """Show real in-flight transfers first, then pending work from the overview."""
        item = self.items.get(slot)
        if item is None: return
        data = self.overviews.get(slot)
        if not isinstance(data, dict):
            item.setText(2, "—"); item.setText(3, "—")
            item.setToolTip(2, ""); item.setToolTip(3, "")
            return
        active = self.transfer_counts().get(slot, 0)
        pending = count(data.get("pending_operations"))
        if active:
            activity = f"{active} 项传输中"
        elif pending:
            activity = f"{pending} 项待同步"
        else:
            activity = "无"
        issues = (count(data.get("download_errors")) + uncertain_operations(data)
                  + int(data.get("connection") in ("needs_attention", "locked"))
                  + int(bool(data.get("background_error")) or bool(data.get("error")))
                  + int(bool(data.get("upload_block"))))
        item.setText(2, activity)
        item.setText(3, "无" if not issues else f"{issues} 项待处理")
        item.setToolTip(2, "当前在途传输来自后台实时状态；待同步操作来自概览。" if activity != "无" else "当前没有传输或待同步操作。")
        item.setToolTip(3, "打开同步问题可查看诊断和恢复操作。" if issues else "当前没有待处理的同步问题。")

    def refresh_transfers(self):
        """Poll in-flight transfers per mount; the cache keeps the last good rows."""
        if not self.records: return
        for slot, profile in list(self.records.items()):
            bus = self.buses.get(slot)
            if bus is None: continue
            if slot in self.transfer_pending:
                self.transfer_again.add(slot); continue
            self.transfer_pending.add(slot)
            expected = profile["task_id"]
            def received(data, error, slot=slot, expected=expected, bus=bus):
                self.transfer_pending.discard(slot)
                if self.buses.get(slot) is not bus or self.records.get(slot, {}).get("task_id") != expected:
                    self.transfer_again.discard(slot)
                    return
                self.transfer_received(slot, data, error)
                if slot in self.transfer_again:
                    self.transfer_again.discard(slot)
                    QTimer.singleShot(0, self, self.refresh_transfers)
            bus.call("GetTransfers", callback=received)

    def transfer_received(self, slot, data, error):
        task = self.records.get(slot, {}).get("task_id")
        if task is None: return
        previous = self.transfer_cache.get(slot, {})
        entries = previous.get("entries", []) if previous.get("task_id") == task else []
        state = {"task_id": task, "entries": entries, "error": True, "at": time.monotonic()}
        if not error and isinstance(data, dict) and data.get("schema") == 1 and data.get("task_id") == task:
            raw = data.get("entries")
            state["entries"] = list(raw)[:TRANSFER_LIMIT] if isinstance(raw, list) else []
            state["error"] = bool(data.get("error"))
        self.transfer_cache[slot] = state
        self.render_transfers()
        self.update_activity(slot)
        if hasattr(self, "tray"): self.tray.refresh()
        self.transfers_changed.emit()

    def transfer_counts(self):
        """Number of real in-flight transfers per mount; unreadable data counts as 0."""
        counts = {}
        for slot, cached in self.transfer_cache.items():
            if slot not in self.records: continue
            if cached.get("error"): counts[slot] = 0; continue
            counts[slot] = sum(1 for entry in (cached.get("entries") or []) if active_transfer(entry))
        return counts

    def transfer_total(self):
        return sum(self.transfer_counts().values())

    def render_transfers(self):
        """Summarize all mounts' in-flight transfers for the global overview."""
        if not hasattr(self, "transfer_table"): return
        total = self.transfer_total()
        errors = any(cached.get("error") for slot, cached in self.transfer_cache.items()
                     if slot in self.records and cached.get("task_id") == self.records[slot].get("task_id"))
        summary = f"正在传输 {total} 项" if total else "当前没有正在传输的文件"
        if errors:
            summary += "（部分挂载状态暂不可用）"
        self.transfer_summary.setText(summary)
        self.transfer_table.clear()
        rows = 0
        for slot, profile in self.records.items():
            cached = self.transfer_cache.get(slot)
            if not cached or cached.get("task_id") != profile["task_id"]: continue
            for entry in (cached.get("entries") or [])[:TRANSFER_LIMIT]:
                row = transfer_row(entry) if isinstance(entry, dict) else None
                if row is None: continue
                item = QTreeWidgetItem([Path(profile["mount"]).name, row["path"],
                                        row["phase"], row["progress_text"] or "—"])
                self.transfer_table.addTopLevelItem(item)
                rows += 1
                if rows >= 8: break
            if rows >= 8: break
        self.transfer_table.setVisible(bool(rows))
        self.transfer_table.setToolTip("仅显示部分在途传输；打开各挂载可查看完整列表。" if rows else "")

    def render_progress(self, slot, progress):
        title, detail = progress_text(progress)
        item = self.items[slot]
        if title:
            item.setToolTip(1, detail)
            if item.data(1, Qt.UserRole) in (None, "connecting", "online"):
                item.setText(1, title)
        else:
            item.setToolTip(1, "")
        overview = self.overviews.get(slot, {})
        caption = queue_caption(overview)
        if caption:
            state = overview.get("connection")
            item.setText(1, f"{LABELS.get(state, '状态未知')} · {caption}")
            item.setToolTip(1, queue_notice(overview) + ("\n" + detail if detail else ""))

    def refresh_progress(self, slot):
        if not self.isVisible() or slot not in self.items:
            return
        if slot in self.progress_pending:
            self.progress_again.add(slot)
            return
        self.progress_pending.add(slot)
        bus = self.buses[slot]
        def received(data, error):
            if self.buses.get(slot) is not bus: return
            self.progress_pending.discard(slot)
            if slot in self.progress_again:
                self.progress_again.discard(slot)
                QTimer.singleShot(100, self, lambda: self.refresh_progress(slot))
            if error or slot not in self.items or data.get("task_id") != self.records[slot]["task_id"]:
                return
            self.render_progress(slot, data.get("initialization"))
        self.buses[slot].call("GetInitialization", callback=received)

    def rebuild_navigation(self):
        current = self.navigation.currentItem()
        selected = current.data(0, Qt.UserRole) if current else ("all", "")
        self.navigation.blockSignals(True)
        self.navigation.clear(); self.nav_items = {}
        group = QTreeWidgetItem(["我的挂载"])
        group.setFlags(group.flags() & ~Qt.ItemIsSelectable)
        for key, title in [(('all', ''), '所有挂载')] + [
            (('task', slot), Path(profile['mount']).name) for slot, profile in self.records.items()
        ] + [(('task', slot), page.new_name or '新挂载') for slot, page in self.pages.items() if slot not in self.records] + [(('settings', ''), '全局设置')]:
            item = QTreeWidgetItem([title]); item.setData(0, Qt.UserRole, key)
            if key[0] == 'task': group.addChild(item)
            else: self.navigation.addTopLevelItem(item)
            self.nav_items[key] = item
        self.navigation.insertTopLevelItem(1, group); group.setExpanded(True)
        self.navigation.setCurrentItem(self.nav_items.get(selected, self.nav_items[('all', '')]))
        self.navigation.blockSignals(False)
        self.navigate(self.navigation.currentItem(), None)

    def navigate(self, item, _previous):
        if item is None or item.data(0, Qt.UserRole) is None: return
        kind, slot = item.data(0, Qt.UserRole)
        if kind == 'task':
            self.activate_task(slot)
        else:
            self.stack.setCurrentWidget(self.all_page if kind == 'all' else self.settings_page)

    def open_navigation_item(self, item, _column=0):
        # Enter on the navigation tree behaves like clicking the entry.
        if item is None: return
        self.navigate(item, None)

    def activate_tab(self, index):
        """Ctrl+1..Ctrl+5: tabs of the current view; global view maps 3 to history."""
        page = self.stack.currentWidget()
        for candidate in self.pages.values():
            if candidate is page:
                candidate.tabs.setCurrentIndex(index - 1)
                return "mount"
        if page is self.all_page:
            if index == 1: self.all_tabs.setCurrentIndex(0)
            elif index in (2, 3): self.all_tabs.setCurrentIndex(1)
            return "overview" if index == 1 else "history" if index in (2, 3) else "global"
        return "settings" if page is self.settings_page else None

    def page_focus_target(self):
        page = self.stack.currentWidget()
        widget = getattr(page, "tabs", None)
        if page is self.all_page: widget = self.all_tabs
        elif page is self.settings_page: widget = self.problems_check
        if widget is not None: return widget
        for child in page.findChildren(QWidget):
            if child.focusPolicy() & Qt.TabFocus and child.isVisibleTo(page) and child.isEnabled():
                return child
        return page

    def focus_cycle(self):
        focus = QApplication.focusWidget()
        if focus is not None and (focus is self.navigation or self.navigation.isAncestorOf(focus)):
            self.page_focus_target().setFocus(Qt.TabFocusReason)
            return "page"
        self.navigation.setFocus(Qt.TabFocusReason)
        return "navigation"

    def activate_task(self, slot=None, name=None):
        if slot is None:
            self.navigation.setCurrentItem(self.nav_items[('all', '')])
            self.stack.setCurrentWidget(self.all_page)
            return
        config_path(slot)  # Validate before constructing any task-bound component.
        if slot not in self.pages:
            from .window import MainWindow
            page = MainWindow(self.stack, instance=slot, new_name=name)
            page.setWindowFlags(Qt.Widget)
            page.ready.connect(self.refresh_catalog)
            page.pause_intent.connect(lambda key=slot: self.tray.manual_pause_intent(key))
            # window.py confirms once on its button; the manager must not ask again.
            if hasattr(page, "stop_requested"):
                page.stop_requested.connect(lambda key=slot: self.begin_stop(key))
            self.pages[slot] = page; self.stack.addWidget(page)
            self.rebuild_navigation()
        page = self.pages[slot]
        self.stack.setCurrentWidget(page)
        item = self.nav_items.get(('task', slot))
        if item and item is not self.navigation.currentItem():
            self.navigation.setCurrentItem(item)

    def activate(self, task='', name=''):
        # The CLI uses 'legacy' for the old profile, '' for the global overview.
        self.activate_task(None if task == '' else '' if task == 'legacy' else task, name or None)
        self.showNormal(); self.raise_(); self.activateWindow()

    def notification_target(self, slot):
        # Tray clicks carry no payload; reopen the mount behind the last notice.
        if slot not in self.records:
            self.activate('')
        elif slot == '':
            self.activate('legacy')
        else:
            self.activate(slot)

    def spawn(self, slot, name=None):
        # Kept as an internal compatibility entry point; never launches a process.
        self.activate_task(slot, name)

    def showEvent(self, event):
        # Foreground: keep the fast transfer poll that feeds the table.
        self.transfer_timer.setInterval(TRANSFER_POLL_MS)
        super().showEvent(event)

    def hideEvent(self, event):
        # Hidden (tray-only): the tray still needs activity, but the full table
        # does not. Slow the poll instead of stopping it (issue #29).
        self.transfer_timer.setInterval(TRANSFER_HIDDEN_POLL_MS)
        super().hideEvent(event)

    def closeEvent(self, event):
        if self.tray.available():
            self.hide(); event.ignore()
        else:
            # Keep the last reachable UI when no system tray is available.
            self.showNormal(); event.ignore()
            self.message.setText("托盘不可用，窗口保持打开。需要退出时请使用“退出界面，继续同步”。")

    def history_sources(self):
        return {slot: HistorySource(slot, Path(profile['mount']).name, self.buses[slot],
                                   profile['task_id'], profile['mount'])
                for slot, profile in self.records.items() if slot in self.buses}

    def show_history(self, slot=None):
        self.activate('' if slot is None else slot or 'legacy')
        if slot is None:
            self.all_tabs.setCurrentWidget(self.history_page)
        elif slot in self.pages:
            self.pages[slot].show_history()

    def show_settings(self):
        self.activate()
        self.navigation.setCurrentItem(self.nav_items[('settings', '')])

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

    def mount_menu(self, slot):
        """Row context menu; also the keyboard/mouse entry to stop one mount."""
        if slot not in self.records:
            return None
        overview = self.overviews.get(slot, {})
        state = overview.get("connection")
        menu = QMenu(self.table)
        action = menu.addAction("打开管理")
        action.triggered.connect(lambda _checked=False, slot=slot: self.activate_task(slot))
        action = menu.addAction("打开文件夹")
        action.triggered.connect(lambda _checked=False, slot=slot: self.open_task_folder(slot))
        action = menu.addAction("恢复同步" if state == "paused" else "暂停同步")
        action.triggered.connect(lambda _checked=False, slot=slot: self.toggle_pause_slot(slot))
        action.setEnabled(state not in (None, "stopping"))
        action = menu.addAction("停止此挂载并卸载…")
        action.triggered.connect(lambda _checked=False, slot=slot: self.stop_task(slot))
        action.setEnabled(state != "stopping" and slot not in self.stop_pending)
        action = menu.addAction("同步历史")
        action.triggered.connect(lambda _checked=False, slot=slot: self.show_history(slot))
        return menu

    def show_mount_menu(self, position):
        item = self.table.itemAt(position)
        if item is None: return
        slot = item.data(0, Qt.UserRole)
        menu = self.mount_menu(slot)
        if menu is not None:
            menu.exec(self.table.viewport().mapToGlobal(position))

    def toggle_pause_slot(self, slot):
        if slot not in self.items or getattr(self.tray, "stopping", False): return
        self.table.setCurrentItem(self.items[slot])
        self.pause()

    def stop_task(self, slot):
        """Stop and unmount one mount; never force-kill or delete local data."""
        if slot in self.stop_pending:
            self.message.setText("该挂载正在停止，请等待后台确认。"); return False
        if slot not in self.records or slot not in self.buses:
            self.message.setText("挂载已不存在，无法停止。"); return False
        name = Path(self.records[slot]["mount"]).name
        text = (f"「{name}」可以选择：\n"
                "暂停同步（保留挂载）：只暂停传输，挂载保留，可随时恢复。\n"
                "停止并卸载（结束挂载和服务）：正常结束该挂载的后台、保存状态并卸载目录。\n\n"
                "停止不会删除缓存、草稿或远端文件，也不会强制结束进程；取消则保持当前状态。")
        answer = QMessageBox.question(self, "停止此挂载并卸载", text,
                                      QMessageBox.Yes | QMessageBox.Cancel, QMessageBox.Cancel)
        if answer != QMessageBox.Yes:
            self.message.setText("已取消停止，挂载保持当前状态。")
            return False
        return self.begin_stop(slot)

    def begin_stop(self, slot):
        bus = self.buses[slot]
        profile = self.records[slot]
        task = profile["task_id"]
        name = Path(profile["mount"]).name
        self.stop_pending[slot] = {"bus": bus, "task": task,
                                   "deadline": time.monotonic() + STOP_TIMEOUT_SECONDS}
        self.items[slot].setText(1, "正在停止…")
        self.items[slot].setData(1, Qt.UserRole, "stopping")
        self.message.setText(f"正在正常停止「{name}」：已请求停止，等待后台确认。不会强制结束后台。")
        page = self.pages.get(slot)
        if page is not None and hasattr(page, "set_stopping"):
            page.set_stopping(True)
        if hasattr(self, "tray"): self.tray.refresh()

        def done(data, error):
            if self.buses.get(slot) is not bus or self.records.get(slot, {}).get("task_id") != task:
                return
            if (error and error != "service_changed") or (isinstance(data, dict) and data.get("error")):
                self.finish_stop(slot, False, "停止请求未被后台确认，挂载状态未知。")
        bus.call("StopMount", callback=done)
        watcher = self.shutdown_watchers.get(slot)
        if watcher is not None: watcher.refresh()
        self.stop_timer.start()
        return True

    def poll_stop_tasks(self):
        if not self.stop_pending:
            self.stop_timer.stop(); return
        now = time.monotonic()
        for slot, state in list(self.stop_pending.items()):
            if self.buses.get(slot) is not state["bus"] or self.records.get(slot, {}).get("task_id") != state["task"]:
                self.finish_stop(slot, False, "挂载或任务已经变化。"); continue
            watcher = self.shutdown_watchers.get(slot)
            value = watcher.value if watcher is not None else None
            # Older backends may omit "mounted" from GetShutdown; the overview
            # still reports it, so never turn a confirmed stop into a timeout.
            mounted = watcher.mounted if watcher is not None else None
            if mounted is None:
                mounted = self.overviews.get(slot, {}).get("mounted")
            if value and value.get("phase") == "complete" and mounted is False:
                self.finish_stop(slot, True); continue
            if value and value.get("phase") == "failed":
                self.finish_stop(slot, False, "后台报告停止未完成：" + (shutdown_text(value) or "停止失败") + "。"); continue
            if now >= state["deadline"]:
                reason = f"等待后台确认停止超时（已等待超过 {STOP_DEADLINE_SECONDS} 秒）。"
                blocking = blocking_reason_text(value) if isinstance(value, dict) else ""
                if blocking: reason += f"阻塞原因：{blocking}；"
                reason += "不会强制结束后台；请打开诊断并检查正在使用该目录的程序。"
                self.finish_stop(slot, False, reason); continue
            if watcher is not None: watcher.refresh()
            name = Path(self.records[slot]["mount"]).name
            stage = shutdown_text(value) if value else "已请求停止，等待后台确认"
            self.items[slot].setText(1, stage)
            self.message.setText(f"正在停止「{name}」：{stage}。不会强制结束后台或删除数据。")

    def finish_stop(self, slot, stopped, reason=""):
        self.stop_pending.pop(slot, None)
        if not self.stop_pending:
            self.stop_timer.stop()
        if slot not in self.records:
            return
        name = Path(self.records[slot]["mount"]).name
        page = self.pages.get(slot)
        if stopped:
            text = f"已停止并卸载「{name}」，可稍后启动挂载。"
            self.message.setText(text)
            if page is not None and hasattr(page, "set_stopping"):
                page.set_stopping(False, text)
        else:
            text = f"「{name}」停止未确认：{reason}界面保持打开；不会强制结束后台，也不会清理缓存或草稿。"
            self.message.setText(text)
            if page is not None and hasattr(page, "set_stopping"):
                page.set_stopping(False, text)
            self.open_stop_diagnostics(slot)
        if hasattr(self, "tray"): self.tray.refresh()

    def open_stop_diagnostics(self, slot):
        if slot not in self.records: return
        self.activate_task(slot)
        page = self.pages.get(slot)
        if page is not None and getattr(page, "overview", None) is not None:
            page.show_issues()

    def poll_success_notifications(self):
        if not self.notifications.enabled_success(): return
        for slot, profile in list(self.records.items()):
            bus = self.buses.get(slot)
            if bus is None: continue
            expected = profile["task_id"]
            title = Path(profile["mount"]).name

            def received(data, error, slot=slot, expected=expected, bus=bus, title=title):
                if self.buses.get(slot) is not bus or self.records.get(slot, {}).get("task_id") != expected:
                    return
                if error or not isinstance(data, dict): return
                self.notifications.observe_activity(slot, title, data.get("records", []))
            if hasattr(bus, "get_recent_activity"):
                bus.get_recent_activity(20, received)
            else:
                bus.call("GetRecentActivity", [json.dumps({"limit": 20})], received)

    def set_problem_notifications(self, checked):
        checked = bool(checked)
        self.gui_settings["notifications"]["problems"] = checked
        if hasattr(self, "notifications"):
            self.notifications.set_enabled(problems=checked)
        self.persist_gui_settings()

    def set_success_notifications(self, checked):
        checked = bool(checked)
        self.gui_settings["notifications"]["success"] = checked
        if hasattr(self, "notifications"):
            self.notifications.set_enabled(success=checked)
        timer = getattr(self, "success_timer", None)
        if timer is not None:
            if checked: timer.start()
            else: timer.stop()
        self.persist_gui_settings()

    def persist_gui_settings(self):
        try:
            save_gui_settings(self.gui_settings)
        except (OSError, ValueError) as error:
            message = f"无法保存全局界面设置：{error}"
            self.settings_message.setText(message)
            self.message.setText(message)

    def start_slot(self, slot=None):
        """Start one mount's service; history pages reuse this for "start and retry"."""
        if slot is None: slot = self.selected()
        if slot is None:
            self.message.setText("请先在挂载列表中选择一个挂载。"); return False
        if slot not in self.records or slot not in self.buses:
            self.message.setText("该挂载未配置或已移除，无法启动；请先在“添加挂载”中完成设置。"); return False
        name = Path(self.records[slot]["mount"]).name
        launcher = ServiceLauncher(self, managed=True, unit=service_unit(slot), instance=slot)
        self.launchers.append(launcher)
        self.message.setText(f"正在启动「{name}」的挂载服务；完成后可重试打开文件。")
        def done(error):
            if error: self.message.setText(f"启动「{name}」失败：{error}")
            else: self.message.setText(f"已请求启动「{name}」；启动完成后可再次重试。")
            self.refresh_status(slot)
            self.launchers.remove(launcher); launcher.deleteLater()
        launcher.completed.connect(done); launcher.start(enable=True)
        return True

    def start_mount(self, slot=None):
        """Duck-typed hook used by HistoryActions (same contract as window.py)."""
        return self.start_slot(slot)

    def start(self):
        self.start_slot(self.selected())

    def pause(self):
        slot = self.selected()
        if slot is None: return
        self.tray.manual_pause_intent(slot)
        state = self.items[slot].data(1, Qt.UserRole)
        if state is None:
            self.message.setText("状态未确认，请稍后重试。"); return
        self.buses[slot].call("Resume" if state == "paused" else "Pause", callback=lambda _data, _error: self.refresh_status(slot))

    def open_folder(self):
        slot = self.selected()
        if slot is not None: self.open_task_folder(slot)

    def open_task_folder(self, slot):
        data = self.overviews.get(slot, {})
        if slot not in self.records or not data.get("mounted"):
            self.activate()
            self.message.setText("挂载尚不可用，请先检查该任务的连接和初始化状态。"); return
        if not QDesktopServices.openUrl(QUrl.fromLocalFile(self.records[slot]["mount"])):
            self.activate()
            self.message.setText("无法打开文件管理器，请检查桌面默认应用设置。")
