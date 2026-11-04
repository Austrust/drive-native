"""Manage independent mounts; credentials remain in each task's existing UI/keyring."""
import uuid
from pathlib import Path
from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QDesktopServices
from PySide6.QtCore import QUrl
from PySide6.QtWidgets import QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QTreeWidget, QTreeWidgetItem, QInputDialog, QStackedWidget, QApplication, QHeaderView, QSplitter, QTabWidget, QMenu
from .config import profile_records, config_path
from .bus import ClientBus
from .lifecycle import ServiceLauncher
from .config import service_unit
from .initialization import progress_text
from .shutdown import shutdown_text, ShutdownWatcher
from .theme import apply_window_theme
from .queue_health import queue_caption, queue_notice
from .history import HistoryPage, HistorySource

LABELS = {"locked":"等待解锁", "connecting":"连接中", "online":"已连接", "offline":"离线", "paused":"已暂停", "needs_attention":"需要处理", "stopping":"正在停止"}


class MountManager(QMainWindow):
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
        body.addWidget(self.navigation)
        self.stack = QStackedWidget(); body.addWidget(self.stack)
        self.all_page = QWidget(); self.stack.addWidget(self.all_page)
        self.settings_page = QWidget(); settings = QVBoxLayout(self.settings_page)
        settings.addWidget(QLabel("全局界面设置"))
        hint = QLabel("关闭主窗口时，托盘可用则隐藏窗口，后台继续同步。托盘不可用时保留窗口入口。\n证书、缓存、带宽和目录属于各个挂载，请先选择挂载。\n退出菜单可选择仅退出界面，或正常停止全部同步并退出。")
        hint.setWordWrap(True); settings.addWidget(hint); settings.addStretch()
        self.stack.addWidget(self.settings_page)
        self.navigation.currentItemChanged.connect(self.navigate)
        all_layout = QVBoxLayout(self.all_page)
        self.all_tabs = QTabWidget(); all_layout.addWidget(self.all_tabs)
        all_overview = QWidget(); self.all_tabs.addTab(all_overview, "概览")
        self.history_page = HistoryPage(self.history_sources, self)
        self.all_tabs.addTab(self.history_page, "同步历史")
        layout = QVBoxLayout(all_overview)
        title = QLabel("我的挂载"); title.setStyleSheet("font-size: 25px; font-weight: 600; padding: 12px 0;")
        layout.addWidget(title)
        layout.addWidget(QLabel("多个目录可以同时挂载。每个挂载独立连接、缓存和同步，关闭窗口不会停止挂载。"))
        bar = QHBoxLayout()
        for label, fn in [("查看挂载", self.open_task), ("启动挂载", self.start), ("暂停 / 恢复", self.pause), ("打开文件夹", self.open_folder)]:
            button = QPushButton(label); button.clicked.connect(fn); bar.addWidget(button)
        bar.addStretch(); layout.addLayout(bar)
        self.table = QTreeWidget(); self.table.setHeaderLabels(["挂载名称", "状态", "远端目录", "本地位置"])
        self.table.setRootIsDecorated(False); self.table.setAlternatingRowColors(True)
        self.table.header().setMinimumSectionSize(150)
        self.table.header().setSectionResizeMode(QHeaderView.Stretch)
        self.table.itemDoubleClicked.connect(lambda *_: self.open_task())
        layout.addWidget(self.table, 1)
        self.message = QLabel(); self.message.setWordWrap(True); layout.addWidget(self.message)
        body.setStretchFactor(1, 1); body.setSizes([190, 1000])
        self.timer = QTimer(self); self.timer.setInterval(2000); self.timer.timeout.connect(self.refresh_catalog); self.timer.start()
        self.refresh_catalog()
        from .tray import MountTray
        self.tray = MountTray(self)
        exit_menu = QMenu(self.exit_button)
        exit_menu.addAction("退出界面，继续同步", self.tray.quit_interface)
        exit_menu.addAction("停止全部同步并退出", self.tray.stop_all_and_quit)
        self.exit_button.setMenu(exit_menu)
        self.apply_theme()

    def apply_theme(self):
        apply_window_theme(self)
        self.setStyleSheet(self.styleSheet() + "QListWidget{background:palette(base);color:palette(text);border:1px solid palette(midlight);selection-background-color:palette(highlight);selection-color:palette(highlighted-text);} QListWidget::item{padding:12px 10px;}")
        for page in self.pages.values(): page.apply_theme()

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
        self.records = records
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
                watcher = ShutdownWatcher(bus, lambda key=slot: self.records.get(key, {}).get("task_id"), lambda value, mounted, key=slot: self.render_shutdown(key, value, mounted), bus)
                self.shutdown_watchers[slot] = watcher
                if hasattr(bus, "shutdown_changed"): watcher.refresh()
            self.refresh_status(slot)
        self.rebuild_navigation()
        self.history_page.refresh()
        for page in self.pages.values(): page.history_page.refresh()
        if hasattr(self, "tray"): self.tray.activity.history.refresh()
        if hasattr(self, "tray"): self.tray.refresh()
        if not records: self.message.setText("点击“添加挂载”，为第一个目录设置名称和本地位置。")
        else: self.message.setText(f"共 {len(records)} 个挂载。双击任务可登录、查看传输或处理错误。")


    def refresh_status(self, slot):
        if slot not in self.items: return
        if slot in self.status_pending:
            self.status_again.add(slot); return
        self.status_pending.add(slot)
        expected = self.records[slot]["task_id"]
        bus = self.buses[slot]
        def received(data, error):
            if self.buses.get(slot) is not bus: return
            self.status_pending.discard(slot)
            if slot in self.status_again:
                self.status_again.discard(slot)
                QTimer.singleShot(100, self, lambda: self.refresh_status(slot))
            if slot not in self.items: return
            profile = self.records[slot]
            if profile["task_id"] != expected: return
            self.overviews.pop(slot, None)
            self.items[slot].setData(1, Qt.UserRole, None)
            if error:
                self.items[slot].setText(1, "状态暂不可用")
                if hasattr(self, "tray"): self.tray.refresh()
                return
            if data.get("task_id") != profile["task_id"]:
                self.items[slot].setText(1, "任务不匹配"); return
            self.overviews[slot] = data
            self.items[slot].setText(1, LABELS.get(data.get("connection"), "状态未知"))
            self.items[slot].setData(1, Qt.UserRole, data.get("connection"))
            self.render_progress(slot, data.get("initialization"))
            watcher = self.shutdown_watchers.get(slot)
            stop = shutdown_text(watcher.value if watcher and watcher.value is not None else data.get("shutdown"))
            if stop:
                self.render_shutdown(slot, watcher.value if watcher and watcher.value is not None else data.get("shutdown"), watcher.mounted if watcher and watcher.value is not None else data.get("mounted"))
            if hasattr(self, "tray"): self.tray.refresh()
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

    def spawn(self, slot, name=None):
        # Kept as an internal compatibility entry point; never launches a process.
        self.activate_task(slot, name)

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

    def start(self):
        slot = self.selected()
        if slot is None: return
        launcher = ServiceLauncher(self, managed=True, unit=service_unit(slot), instance=slot)
        self.launchers.append(launcher)
        def done(error):
            if error: self.message.setText(error)
            self.refresh_status(slot)
            self.launchers.remove(launcher); launcher.deleteLater()
        launcher.completed.connect(done); launcher.start(enable=True)

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
