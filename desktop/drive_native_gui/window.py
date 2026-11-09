import os
import json
import time
from pathlib import Path
from PySide6.QtCore import Qt, QTimer, QUrl, Signal, QProcess
from PySide6.QtGui import QDesktopServices, QAction, QIcon, QColor, QPainter, QPixmap, QPalette
from PySide6.QtWidgets import (
    QMainWindow,
    QWidget,
    QHBoxLayout,
    QVBoxLayout,
    QLabel,
    QPushButton,
    QFrame,
    QTreeWidget,
    QTreeWidgetItem,
    QHeaderView,
    QLineEdit,
    QDialog,
    QDialogButtonBox,
    QMenu,
    QInputDialog,
    QStyle,
    QMessageBox,
    QTabWidget,
)
from .issues import IssuesDialog
from .trust import TrustSettingsDialog
from .transfers import TransfersDialog, transfer_overview_summary, transfer_rows
from .cache_settings import CacheSettingsDialog, byte_text
from .bus import ClientBus
from .config import load_profile, config_path, binary, archive_profile, restore_profile
from .wizard import SetupWizard
from .credentials import secret_job
from .lifecycle import ServiceLauncher, same_task
from .upgrade import BackendUpgrade
from .initialization import progress_text
from .shutdown import shutdown_text, ShutdownWatcher
from .theme import apply_window_theme
from .queue_health import queue_notice
from .history import HistoryPage, HistorySource
from .permissions import permission_text, PermissionSettings

STATUS = {
    "initializing": ("初始化预览", "#bc8425"),
    "download_retry": ("下载失败 · 将重试", "#bc8425"),
    "update_retry": ("更新失败 · 将重试", "#bc8425"),
    "download_failed": ("下载失败", "#c14c58"),
    "update_failed": ("更新失败 · 旧版可用", "#c14c58"),
    "online_only": ("仅在线", "#4c78c5"),
    "local_complete": ("本地可用", "#27966f"),
    "pinned_complete": ("已固定", "#16835e"),
    "pinning": ("正在固定", "#497ac2"),
    "downloading": ("正在下载", "#497ac2"),
    "updating": ("正在更新", "#497ac2"),
    "pending_upload": ("等待同步", "#bc8425"),
    "conflict": ("需要处理冲突", "#c14c58"),
    "mixed": ("部分本地可用", "#748094"),
    "deleted": ("已移除", "#748094"),
}
CONNECTION = {
    "locked": "等待解锁",
    "connecting": "正在连接",
    "online": "已连接",
    "offline": "暂时离线",
    "paused": "已暂停",
    "needs_attention": "需要处理",
    "stopping": "正在停止",
}
ERRORS = {
    "worker_image_unavailable": "后台组件不完整，网络同步已暂停。本地内容仍可用；请修复安装后重新启动挂载，无需清除缓存。",
    "authentication_or_protocol_rejected": "账号认证未通过，请重新输入密码并检查 Drive 权限。",
    "trust_settings_not_durable": "证书设置未确认保存。请在连接证书中重新读取并保存；本地缓存仍可使用。",
    "certificate_or_configuration": "证书或连接配置需要检查。",
    "native_root_unavailable": "远端同步根已不可用或身份发生变化，同步已停止。缓存和未上传修改均保留；请检查远端目录及权限，恢复原目录后重新登录连接。",
    "native_index_capacity": "目录历史或索引超过当前处理预算；本地内容已保留。请在“同步问题”复制诊断摘要，重新登录不会解决此问题。",
    "native_response_capacity": "服务器元数据响应超过当前解码预算；本地内容已保留。请在“同步问题”复制诊断摘要。",
    "native_state_recovery_required": "远端状态需要恢复，已保存的本地内容会保留。",
    "network_unavailable": "暂时无法连接 NAS，已有缓存仍可使用。",
    "local_or_protocol_error": "同步遇到问题，已保存的本地内容会保留。",
}

# Connection, sync activity and local availability are three independent axes.
# Keep their wording in one place so no card can overwrite another one.
CHANNEL = {
    "locked": "等待登录",
    "connecting": "正在连接",
    "online": "已连接",
    "offline": "暂时离线",
    "paused": "未连接（同步已暂停）",
    "needs_attention": "需要处理",
    "stopping": "正在停止",
}
SYNC = {
    "needs_attention": "需要处理",
    "paused": "已暂停同步",
    "transferring": "正在传输 {n} 项",
    "checking": "正在检查变更",
    "pending": "{n} 项等待同步",
    "idle": "暂无待处理变更",
    "unknown": "尚未核对",
}
CACHE = {
    "empty": "目录暂无文件",
    "online_only": "仅在线",
    "partial": "部分本地可用",
    "complete": "已缓存可离线打开",
    "pinned": "已固定",
    "unknown": "尚未核对",
}
SYNC_ROW = {
    "initializing": "—",
    "download_retry": "需要检查",
    "update_retry": "需要检查",
    "download_failed": "需要检查",
    "update_failed": "需要检查",
    "pinning": "正在固定",
    "downloading": "正在传输",
    "updating": "正在传输",
    "pending_upload": "等待同步",
    "conflict": "需要处理冲突",
    "deleted": "已移除",
    "online_only": "暂无待处理变更",
    "local_complete": "暂无待处理变更",
    "pinned_complete": "暂无待处理变更",
    "mixed": "暂无待处理变更",
}


def sync_row_text(row):
    if row.get("error"):
        return "状态读取失败"
    return SYNC_ROW.get(row.get("status"), "尚未核对")


def cache_row_text(row):
    if row.get("status") == "initializing":
        return "尚未可用"
    if row.get("status") == "mixed":
        return "部分本地可用"
    if row.get("offline_available"):
        return "已固定" if row.get("pinned") else "已缓存"
    return "仅在线"


# Status icons never change for a given status: build each one once and reuse
# the QIcon. The old code painted a fresh 20x20 pixmap for every row on every
# four-per-second signal burst (issue #29).
_STATUS_ICONS = {}
OVERVIEW_SIGNAL_INTERVAL = 1.0
# A membership change (listing/subtree) reloads the current directory page at
# most this often, so a rename burst opens one page request, not one per event.
MEMBERSHIP_REFRESH_MS = 250


def status_icon(status):
    cached = _STATUS_ICONS.get(status)
    if cached is not None:
        return cached
    color = QColor(STATUS.get(status, ("", "#c14c58"))[1])
    pix = QPixmap(20, 20)
    pix.fill(Qt.transparent)
    painter = QPainter(pix)
    painter.setRenderHint(QPainter.Antialiasing)
    painter.setPen(Qt.NoPen)
    painter.setBrush(color)
    painter.drawEllipse(3, 3, 14, 14)
    painter.setPen(Qt.white)
    if status in ("local_complete", "pinned_complete"):
        painter.drawText(pix.rect(), Qt.AlignCenter, "✓")
    elif status in ("conflict", "download_failed", "update_failed"):
        painter.drawText(pix.rect(), Qt.AlignCenter, "!")
    elif status == "online_only":
        painter.drawEllipse(7, 7, 6, 6)
    else:
        painter.drawText(pix.rect(), Qt.AlignCenter, "·")
    painter.end()
    icon = QIcon(pix)
    _STATUS_ICONS[status] = icon
    return icon


class _MountActions:
    """Start hook handed to the history page: one page owns exactly one mount."""

    def __init__(self, window):
        self.window = window

    def start_mount(self, slot=None):
        return self.window.start_mount(slot)


class MainWindow(QMainWindow):
    ready = Signal()
    pause_intent = Signal()
    stop_requested = Signal()

    def __init__(self, parent=None, *, instance=None, new_name=None):
        super().__init__(parent)
        self.setWindowTitle("Drive Native")
        self.resize(1120, 740)
        self.setMinimumSize(540, 380)
        self.instance = os.environ.get("DRIVE_NATIVE_INSTANCE", "") if instance is None else instance
        config_path(self.instance)
        self.new_name = new_name
        self.bus = ClientBus(self) if instance is None else ClientBus(self, instance=self.instance)
        self.launcher = ServiceLauncher(self) if instance is None else ServiceLauncher(self, instance=self.instance)
        self.launcher.completed.connect(self.service_started)
        self.profile = None
        self.profile_transition = None
        self.catalog_replacement = None
        self.overview = {}
        self.folder = ""
        self.display_limit = 256
        self.view_generation = 0
        self.rows = []
        # Bulk release runs as a background job so the service is never held by
        # one long D-Bus call; the window polls a compact progress snapshot.
        self.release_job_id = None
        self.release_total = 0
        # Directory page accumulator: a GetDirectory reply may arrive before
        # received_overview creates it for the first time.
        self.page_rows = []
        # #29 counters: asserted by tests and reported as evidence.
        self.overview_queries = 0
        self.directory_queries = 0
        self.icon_sets = 0
        self.filter_passes = 0
        self.filter_row_updates = 0
        self.last_overview_at = 0.0
        self._directory_loading = False
        self._dir_icon = None
        self._file_icon = None
        # #31 revisioned change-set consumer: a StatusChanged2 storm patches the
        # visible rows with one GetStates batch instead of refreshing everything.
        self.status2_seen = False
        self.status_epoch = None
        self.status_revision = None
        self.status_events = 0
        self.status_getstates = 0
        self.status_patched_rows = 0
        self.status_dropped_rows = 0
        self.membership_refreshes = 0
        self._status_loading = False
        self._status_pending = set()
        self._status_generation = None
        self._membership_again = False
        self.loading = False
        self.progress_loading = False
        self.progress_again = False
        self.refresh_again = False
        self.pending_password = None
        self.auto_unlock_attempted = False
        self.keyring_job = None
        # In-flight transfer summary state (see refresh_transfers): a reply is
        # only applied while it still belongs to the visible page generation.
        self.transfer_loading = False
        self.transfer_again = False
        self.transfer_generation = 0
        self.transfer_rows = []
        self.transfer_count = 0
        # The embedded page is hidden whenever the owner window goes to the
        # tray; remember the owner's maximized state across that hide/show.
        self.restore_maximized = False
        self.cache_from_backend = False
        root = QWidget()
        self.setCentralWidget(root)
        layout = QVBoxLayout(root)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        self.tabs = QTabWidget()
        self.tabs.setAccessibleName("当前挂载页面")
        layout.addWidget(self.tabs)
        self.overview_page = QWidget()
        self.tabs.addTab(self.overview_page, "概览")
        side = QFrame()
        side.setObjectName("sidebar")
        self.settings_page = side
        left = QVBoxLayout(side)
        left.setContentsMargins(24, 30, 24, 22)
        left.setSpacing(12)
        brand = QLabel("Drive Native")
        brand.setObjectName("brand")
        left.addWidget(brand)
        caption = QLabel("当前挂载设置")
        caption.setObjectName("muted")
        left.addWidget(caption)
        left.addSpacing(26)
        files = QPushButton("查看文件状态")
        self.files_button = files
        files.setCheckable(True)
        files.setObjectName("navigation")
        files.toggled.connect(self.toggle_files)
        left.addWidget(files)
        self.transfers_dialog = None
        self.transfers_button = QPushButton("传输与带宽")
        self.transfers_button.clicked.connect(self.show_transfers)
        left.addWidget(self.transfers_button)
        self.issues_button = QPushButton("同步问题")
        self.issues_button.clicked.connect(self.show_issues)
        left.addWidget(self.issues_button)
        self.issues_dialog = None
        self.cache_dialog = None
        self.trust_dialog = None
        self.trust_settings_button = QPushButton("连接证书")
        self.trust_settings_button.clicked.connect(self.show_trust_settings)
        left.addWidget(self.trust_settings_button)
        self.cache_settings_button = QPushButton("缓存设置")
        self.cache_settings_button.clicked.connect(self.show_cache_settings)
        left.addWidget(self.cache_settings_button)
        self.open_folder = QPushButton("在文件管理器中打开")
        self.open_folder.clicked.connect(self.open_in_files)
        left.addWidget(self.open_folder)
        self.stop_mount = QPushButton("停止此挂载并卸载…")
        self.stop_mount.setAccessibleName("停止此挂载并卸载")
        self.stop_mount.clicked.connect(self.stop_and_unmount)
        left.addWidget(self.stop_mount)
        left.addStretch()
        self.account = QLabel("尚未连接 NAS")
        self.account.setWordWrap(True)
        left.addWidget(self.account)
        self.configure = QPushButton("连接 NAS")
        self.configure.clicked.connect(self.configure_account)
        left.addWidget(self.configure)
        self.change_folder = QPushButton("更换同步目录…")
        self.change_folder.clicked.connect(self.choose_other_folder)
        left.addWidget(self.change_folder)
        content = QVBoxLayout(self.overview_page)
        content.setContentsMargins(30, 28, 30, 24)
        content.setSpacing(18)
        head = QHBoxLayout()
        titles = QVBoxLayout()
        title = QLabel("同步文件")
        self.files_title = title
        title.setObjectName("heading")
        titles.addWidget(title)
        self.subtitle = QLabel("连接后，在这里查看文件与同步状态。")
        self.subtitle.setObjectName("muted")
        titles.addWidget(self.subtitle)
        head.addLayout(titles, 1)
        self.pause = QPushButton("暂停同步")
        self.pause.clicked.connect(self.toggle_pause)
        head.addWidget(self.pause)
        self.update_backend = QPushButton("应用后台更新…")
        self.update_backend.hide()
        self.update_backend.clicked.connect(self.apply_backend_update)
        head.addWidget(self.update_backend)
        self.backend_upgrade = BackendUpgrade(self.bus, self.launcher, self)
        self.backend_upgrade.changed.connect(self.update_backend.setVisible)
        self.backend_upgrade.finished.connect(self.backend_updated)
        content.addLayout(head)
        cards = QHBoxLayout()
        cards.setSpacing(14)
        self.connection_value = self.card(cards, "连接状态", "尚未启动")
        self.sync_value = self.card(cards, "同步活动", "尚未核对")
        # Compatibility alias: existing pages/tests still address the card by
        # its historical attribute name.
        self.pending_value = self.sync_value
        self.availability_value = self.card(cards, "本地可用性", "尚未核对")
        content.addLayout(cards)
        self.progress_label = QLabel()
        self.progress_label.setWordWrap(True)
        self.progress_label.hide()
        content.addWidget(self.progress_label)
        self.transfer_title = QLabel("当前传输")
        self.transfer_title.setObjectName("muted")
        content.addWidget(self.transfer_title)
        self.transfer_summary = QLabel("当前没有正在传输的文件")
        self.transfer_summary.setWordWrap(True)
        content.addWidget(self.transfer_summary)
        self.transfer_table = QTreeWidget()
        self.transfer_table.setHeaderLabels(["文件", "方向", "阶段", "正文进度"])
        self.transfer_table.setRootIsDecorated(False)
        self.transfer_table.setSelectionMode(QTreeWidget.NoSelection)
        self.transfer_table.setMaximumHeight(8 * 24 + 26)
        self.transfer_table.hide()
        content.addWidget(self.transfer_table)
        self.transfer_more = QPushButton()
        self.transfer_more.clicked.connect(self.show_transfers)
        self.transfer_more.hide()
        content.addWidget(self.transfer_more, 0, Qt.AlignLeft)
        self.shutdown_label = QLabel(); self.shutdown_label.setWordWrap(True)
        self.shutdown_label.hide(); content.addWidget(self.shutdown_label)
        self.shutdown_watcher = ShutdownWatcher(self.bus, lambda: self.profile.get("task_id") if self.profile else None, self.render_shutdown, self)
        self.preview_label = QLabel("目录预览 · 已确认的目录可浏览，文件正文暂不可用。初始化完成后自动开放正常操作。")
        self.preview_label.setWordWrap(True)
        self.preview_label.setObjectName("message")
        self.preview_label.hide()
        content.addWidget(self.preview_label)
        self.banner = QLabel()
        self.banner.setWordWrap(True)
        self.banner.setObjectName("message")
        self.banner.hide()
        content.addWidget(self.banner)
        self.release_label = QLabel()
        self.release_label.setWordWrap(True)
        self.release_label.setObjectName("message")
        self.release_label.hide()
        content.addWidget(self.release_label)
        self.release_cancel = QPushButton("取消释放")
        self.release_cancel.clicked.connect(self.cancel_release)
        self.release_cancel.hide()
        content.addWidget(self.release_cancel, 0, Qt.AlignLeft)
        self.upload_block_label = QLabel()
        self.upload_block_label.setTextFormat(Qt.PlainText)
        self.upload_block_label.setWordWrap(True)
        self.upload_block_label.hide()
        content.addWidget(self.upload_block_label)
        self.upload_block_action = QPushButton("查看上传阻塞与恢复方式")
        self.upload_block_action.clicked.connect(self.show_issues)
        self.upload_block_action.hide()
        content.addWidget(self.upload_block_action)
        self.permission_label = QLabel()
        self.permission_label.setTextFormat(Qt.PlainText)
        self.permission_label.setWordWrap(True)
        content.addWidget(self.permission_label)
        permission_actions = QHBoxLayout()
        self.permission_settings = QPushButton("读写设置…")
        self.permission_settings.clicked.connect(self.edit_permissions)
        permission_actions.addWidget(self.permission_settings)
        self.permission_recheck = QPushButton("重新核对远端权限")
        self.permission_recheck.clicked.connect(self.show_issues)
        permission_actions.addWidget(self.permission_recheck)
        content.addLayout(permission_actions)
        self.overview_hint = QLabel("文件可直接从文件管理器打开。\n选择“文件”页，按需载入当前目录的状态列表。")
        self.overview_hint.setObjectName("muted")
        self.overview_hint.setAlignment(Qt.AlignCenter)
        self.overview_hint.setWordWrap(True)
        content.addWidget(self.overview_hint, 1)
        self.file_details = QWidget()
        details = QVBoxLayout(self.file_details)
        details.setContentsMargins(0, 0, 0, 0)
        self.tabs.addTab(self.file_details, "文件")
        toolbar = QHBoxLayout()
        self.up = QPushButton("↑ 上一级")
        self.up.clicked.connect(self.parent_folder)
        toolbar.addWidget(self.up)
        self.breadcrumb = QLabel("同步目录")
        self.breadcrumb.setTextInteractionFlags(Qt.TextSelectableByMouse)
        toolbar.addWidget(self.breadcrumb, 1)
        self.filter = QLineEdit()
        self.filter.setPlaceholderText("筛选当前目录")
        self.filter.setClearButtonEnabled(True)
        self.filter.setMaximumWidth(220)
        self.filter.textChanged.connect(self.apply_filter)
        toolbar.addWidget(self.filter)
        details.addLayout(toolbar)
        self.table = QTreeWidget()
        self.table.setHeaderLabels(["名称", "同步状态", "本地可用性"])
        self.table.setRootIsDecorated(False)
        self.table.setAlternatingRowColors(True)
        self.table.setSelectionMode(QTreeWidget.ExtendedSelection)
        self.table.header().setSectionResizeMode(0, QHeaderView.Stretch)
        self.table.setColumnWidth(1, 160)
        self.table.setColumnWidth(2, 110)
        self.table.itemDoubleClicked.connect(self.activate)
        self.table.setContextMenuPolicy(Qt.CustomContextMenu)
        self.table.customContextMenuRequested.connect(self.context_menu)
        details.addWidget(self.table, 1)
        footer = QHBoxLayout()
        self.detail = QLabel("文件状态来自本地同步服务")
        self.detail.setObjectName("muted")
        footer.addWidget(self.detail, 1)
        self.more = QPushButton("加载更多")
        self.more.clicked.connect(self.load_more)
        self.more.hide()
        footer.addWidget(self.more)
        details.addLayout(footer)
        self.history_page = HistoryPage(
            self.history_sources, self, mount_actions=_MountActions(self))
        self.tabs.addTab(self.history_page, "同步历史")
        self.issues_page = QWidget()
        issue_layout = QVBoxLayout(self.issues_page)
        issue_hint = QLabel("查看冲突、失败原因与可执行的恢复方式。")
        issue_layout.addWidget(issue_hint)
        issue_open = QPushButton("查看同步问题")
        issue_open.clicked.connect(self.show_issues)
        issue_layout.addWidget(issue_open); issue_layout.addStretch()
        self.tabs.addTab(self.issues_page, "同步问题")
        self.tabs.addTab(side, "当前挂载设置")
        self.tabs.currentChanged.connect(self.tab_changed)
        self.refresh_timer = QTimer(self)
        self.refresh_timer.setSingleShot(True)
        self.refresh_timer.setInterval(int(OVERVIEW_SIGNAL_INTERVAL * 1000))
        self.refresh_timer.timeout.connect(self.refresh)
        self.bus.changed.connect(self._legacy_status_changed)
        if hasattr(self.bus, "changed2"):
            self.bus.changed2.connect(self.received_status2)
        self.membership_timer = QTimer(self)
        self.membership_timer.setSingleShot(True)
        self.membership_timer.setInterval(MEMBERSHIP_REFRESH_MS)
        self.membership_timer.timeout.connect(self._refresh_membership)
        self.bus.progress_changed.connect(self.refresh_progress)
        # This reads only the local monotonic clocks while a phase is active.
        # It never increments completed work or starts a NAS request.
        self.progress_timer = QTimer(self)
        self.progress_timer.setInterval(2000)
        self.progress_timer.timeout.connect(self.refresh_progress)
        # The transfer summary refreshes only while the overview page is
        # visible; replies that arrive after hiding are discarded.
        self.transfer_timer = QTimer(self)
        self.transfer_timer.setInterval(2000)
        self.transfer_timer.timeout.connect(self.refresh_transfers)
        self.release_timer = QTimer(self)
        self.release_timer.setInterval(250)
        self.release_timer.timeout.connect(self.poll_release_job)
        self.bus.availability.connect(self.service_availability)
        for label in self.findChildren(QLabel):
            label.setTextFormat(Qt.PlainText)
        self.apply_theme()
        QTimer.singleShot(0, self, self.start)

    def toggle_files(self, visible):
        self.tabs.setCurrentWidget(self.file_details if visible else self.overview_page)
        self.files_button.setText("收起文件状态" if visible else "查看文件状态")
        self.view_generation += 1
        if visible:
            self.stop_transfer_timer()
            self.schedule_refresh()
        else:
            self.start_transfer_timer()

    def tab_changed(self, _index):
        visible = self.tabs.currentWidget() is self.file_details
        self.files_button.blockSignals(True)
        self.files_button.setChecked(visible)
        self.files_button.blockSignals(False)
        self.view_generation += 1
        if visible:
            self.stop_transfer_timer()
            self.schedule_refresh()
        elif self.tabs.currentWidget() is self.overview_page:
            self.start_transfer_timer()
        else:
            self.stop_transfer_timer()
        if self.tabs.currentWidget() is self.issues_page: self.show_issues()

    def history_sources(self):
        profile = self.profile or {}
        if not profile.get("task_id") or not profile.get("mount"): return {}
        return {self.instance: HistorySource(self.instance, Path(profile["mount"]).name,
            self.bus, profile["task_id"], profile["mount"])}

    def show_history(self):
        self.tabs.setCurrentWidget(self.history_page)

    def show_settings(self):
        self.tabs.setCurrentWidget(self.settings_page)

    def showEvent(self, event):
        super().showEvent(event)
        self.schedule_refresh()
        self.refresh_progress()
        self.start_transfer_timer()
        if hasattr(self.bus, "shutdown_changed"): self.shutdown_watcher.refresh()
        if self.restore_maximized:
            QTimer.singleShot(0, self, self.restore_owner_maximized)

    def hideEvent(self, event):
        self.refresh_timer.stop()
        self.progress_timer.stop()
        self.stop_transfer_timer()
        self.refresh_again = False
        self.view_generation += 1
        owner = self.window()
        self.restore_maximized = bool(owner is not self and owner.isMaximized())
        super().hideEvent(event)

    def start_transfer_timer(self):
        if not self.transfer_page_visible():
            return
        if not self.transfer_timer.isActive():
            self.transfer_timer.start()
        self.refresh_transfers()

    def stop_transfer_timer(self):
        self.transfer_timer.stop()
        self.transfer_generation += 1
        self.transfer_loading = False
        self.transfer_again = False

    def transfer_page_visible(self):
        return self.isVisible() and self.tabs.currentWidget() is self.overview_page

    def refresh_transfers(self):
        if not self.transfer_page_visible():
            return
        if self.transfer_loading:
            self.transfer_again = True
            return
        self.transfer_loading = True
        generation = self.transfer_generation
        task = (self.profile or {}).get("task_id")

        def received(data, error):
            self.transfer_loading = False
            if generation != self.transfer_generation or not self.transfer_page_visible():
                self.transfer_again = False
                return
            if task and (self.profile or {}).get("task_id") != task:
                self.transfer_again = False
                return
            self.render_transfers(data, error)
            if self.transfer_again:
                self.transfer_again = False
                QTimer.singleShot(100, self.refresh_transfers)

        if not self.bus.owner:
            self.transfer_loading = False
            self.render_transfers(None, "service_unavailable")
            return
        self.bus.call("GetTransfers", callback=received)

    def render_transfers(self, data, error):
        if error or not isinstance(data, dict) or data.get("schema") != 1:
            self.transfer_summary.setText("当前传输状态暂不可用。")
            self.transfer_table.clear()
            self.transfer_table.hide()
            self.transfer_more.hide()
            return
        task = (self.profile or {}).get("task_id")
        if task and data.get("task_id") != task:
            self.transfer_summary.setText("任务身份不一致，等待刷新后重试。")
            self.transfer_table.clear()
            self.transfer_table.hide()
            self.transfer_more.hide()
            return
        entries = data.get("entries", [])
        self.transfer_rows = transfer_rows(entries)
        self.transfer_count = len(entries) if isinstance(entries, list) else 0
        self.transfer_summary.setText(transfer_overview_summary(entries))
        self.transfer_table.clear()
        for row in self.transfer_rows[:8]:
            item = QTreeWidgetItem([
                Path(row["path"]).name or row["path"],
                row["direction"],
                row["phase"],
                row["progress_text"],
            ])
            item.setToolTip(0, row["path"])
            self.transfer_table.addTopLevelItem(item)
        visible = bool(self.transfer_rows)
        more = visible and self.transfer_count > 8
        self.transfer_table.setVisible(visible)
        self.transfer_more.setVisible(more)
        if more:
            self.transfer_more.setText(f"查看全部（共 {self.transfer_count} 项）")

    def restore_owner_maximized(self):
        owner = self.window()
        if (self.restore_maximized and owner is not self
                and owner.isVisible() and not owner.isMaximized()):
            owner.showMaximized()
        self.restore_maximized = False

    def show_transfers(self):
        if self.transfers_dialog is None:
            self.transfers_dialog = TransfersDialog(self.bus, self)
        self.transfers_dialog.show()
        self.transfers_dialog.raise_()
        self.transfers_dialog.activateWindow()

    def show_trust_settings(self):
        if self.trust_dialog is None:
            self.trust_dialog = TrustSettingsDialog(self.bus, self)
        self.trust_dialog.show()
        self.trust_dialog.raise_()
        self.trust_dialog.activateWindow()

    def show_cache_settings(self):
        if self.cache_dialog is None:
            self.cache_dialog = CacheSettingsDialog(self.bus, self)
        self.cache_dialog.show()
        self.cache_dialog.raise_()
        self.cache_dialog.activateWindow()
        self.cache_dialog.refresh()

    def show_issues(self):
        if self.issues_dialog is None:
            self.issues_dialog = IssuesDialog(self.bus, self, configure=self.configure_account)
            self.issues_dialog.resolved.connect(self.schedule_refresh)
            self.issues_dialog.pause_intent.connect(self.pause_intent.emit)
            self.issues_dialog.setWindowFlags(Qt.Widget)
            self.issues_dialog.setMinimumSize(0, 0)
            for button in self.issues_dialog.findChildren(QPushButton):
                if button.text() == "关闭":
                    button.clicked.connect(lambda: self.tabs.setCurrentWidget(self.overview_page))
            self.issues_page.layout().insertWidget(0, self.issues_dialog, 1)
            for index in range(1, self.issues_page.layout().count()):
                widget = self.issues_page.layout().itemAt(index).widget()
                if widget: widget.hide()
        self.issues_dialog.update_context(self.overview)
        self.tabs.setCurrentWidget(self.issues_page)
        self.issues_dialog.show()
        self.issues_dialog.refresh()

    def card(self, layout, label, value):
        frame = QFrame()
        frame.setObjectName("card")
        inner = QVBoxLayout(frame)
        inner.setContentsMargins(18, 14, 18, 14)
        name = QLabel(label)
        name.setObjectName("muted")
        result = QLabel(value)
        result.setObjectName("metric")
        inner.addWidget(name)
        inner.addWidget(result)
        layout.addWidget(frame, 1)
        return result

    def apply_theme(self):
        apply_window_theme(self)

    def start(self):
        try:
            self.profile = load_profile(self.instance)
            if hasattr(self.bus, "shutdown_changed"): self.shutdown_watcher.refresh()
        except Exception:
            self.show_error("无法读取同步配置，请检查配置文件权限。")
        self.bus.call("GetOverview", callback=self.initial_overview)

    def initial_overview(self, data, error):
        if not error:
            self.received_overview(data, None)
            return
        if self.bus.owner:
            # A busy Store does not mean the already running service is absent.
            self.received_overview(None, error)
            return
        if self.profile:
            self.start_service()
        else:
            self.offline_ui()

    def start_service(self, *, enable=False):
        self.connection_value.setText("正在启动")
        self.launcher.start(enable=enable)

    def start_mount(self, slot=None):
        """Start this page's mount task for the history retry action.

        The page only owns its own instance; a history row bound to another
        mount must be started from that mount's page instead of pretending the
        start succeeded here.
        """
        if slot is not None and (slot or "") != (self.instance or ""):
            self.show_error("只能从当前挂载页面启动该挂载，请在对应挂载页面中启动。")
            return False
        if not self.profile:
            self.show_error("无法启动挂载：当前挂载配置尚未载入。")
            return False
        self.start_service(enable=True)
        return True

    def service_started(self, error):
        if error:
            self.show_error(error)
        self.schedule_refresh()

    def offline_ui(self):
        self.overview = {}
        self.upload_block_label.hide()
        self.upload_block_action.hide()
        if self.issues_dialog is not None:
            self.issues_dialog.update_context({})
        self.progress_timer.stop()
        self.progress_label.hide()
        self.connection_value.setText("尚未连接")
        self.sync_value.setText("尚未核对")
        self.availability_value.setText("尚未核对")
        self.preview_label.hide()
        self.pause.setEnabled(False)
        self.open_folder.setEnabled(False)
        self.up.setEnabled(False)
        self.ready.emit()

    def configure_account(self):
        if self.profile:
            password, ok = QInputDialog.getText(
                self, "解锁 Drive", "输入 NAS 账号密码", QLineEdit.Password
            )
            if ok and password:
                self.unlock_if_matching(password)
            return
        wizard = SetupWizard(self, instance=self.instance, new_name=self.new_name)
        wizard.configured.connect(self.configured)
        wizard.open()
        self.wizard = wizard

    def configured(self, profile, password):
        if self.profile_transition is not None:
            self.catalog_replacement = (self.profile_transition, profile["task_id"])
        self.profile_transition = None
        self.profile = profile
        self.folder = ""
        self.auto_unlock_attempted = False
        self.pending_password = password
        self.start_service(enable=True)

    def owns_catalog_transition(self, old_task, new_profile):
        if new_profile is None:
            return self.profile_transition == old_task and old_task is not None
        if (self.catalog_replacement == (old_task, new_profile.get("task_id"))
                and self.profile and self.profile.get("task_id") == new_profile.get("task_id")):
            self.catalog_replacement = None
            return True
        return False

    def choose_other_folder(self):
        if not self.profile:
            self.configure_account()
            return
        if not self.overview or not same_task(self.profile, self.overview):
            self.show_error("请先连接当前后台服务，核对任务状态后再更换目录。")
            return
        if self.overview.get("pending_operations", 1):
            self.show_error("当前仍有待同步操作，请先完成上传或处理冲突，再更换目录。")
            return
        if not self.launcher.managed:
            self.show_error("开发隔离会话请先停止挂载，并使用独立配置测试目录选择。")
            return
        if QMessageBox.question(self, "更换同步目录", "将停止当前挂载，再从 Drive 顶层重新选择。原配置会备份，缓存和个人文件保留。请先关闭正在编辑的文件。") != QMessageBox.Yes:
            return
        self.change_folder.setEnabled(False)
        previous = dict(self.profile)
        process = QProcess(self)
        self.folder_switch_process = process
        timer = QTimer(process)
        timer.setSingleShot(True)
        timer.timeout.connect(process.kill)

        def stopped(code, status):
            timer.stop()
            self.change_folder.setEnabled(True)
            if code != 0 or status != QProcess.NormalExit:
                self.show_error("后台服务未确认停止，原配置保持不变。")
                return
            try:
                backup = archive_profile(previous["task_id"], self.instance)
            except (OSError, ValueError) as error:
                self.show_error(str(error))
                self.start_service()
                return
            # This page owns the temporary absence between archive and setup.
            self.profile_transition = previous["task_id"]
            self.new_name = Path(previous["mount"]).name
            self.profile = None
            wizard = SetupWizard(self, instance=self.instance, new_name=self.new_name)
            wizard.host.setText(previous["host"])
            wizard.tls.setText(previous["tls_name"])
            wizard.username.setText(previous["username"])
            wizard.trusted_ca_pem = list(previous.get("trusted_ca_pem", []))
            wizard.mount.setText(previous["mount"])
            wizard.configured.connect(self.configured)

            def cancelled():
                try:
                    restore_profile(backup, self.instance)
                    self.profile = previous
                    self.profile_transition = None
                    self.catalog_replacement = None
                    self.start_service()
                except (OSError, ValueError):
                    self.show_error(f"原配置保留在 {backup}；当前存在其他配置或无法恢复，请检查。")

            wizard.rejected.connect(cancelled)
            self.wizard = wizard
            wizard.open()

        process.finished.connect(stopped)
        process.errorOccurred.connect(lambda _error: self.change_folder.setEnabled(True))
        process.start("/usr/bin/systemctl", ["--user", "stop", self.launcher.unit])
        timer.start(30000)

    def service_availability(self, available):
        self.loading = False
        if not available:
            self.offline_ui()
            self.connection_value.setText("后台已停止")
        self.schedule_refresh()

    def schedule_refresh(self):
        if not self.isVisible():
            return
        if self.loading:
            self.refresh_again = True
            return
        if self.refresh_timer.isActive():
            return
        # Coalesce signal bursts into at most one GetOverview per second while
        # keeping the first signal prompt (issue #29). Direct refresh() calls
        # (buttons, first show, tests) are intentionally not throttled.
        elapsed = time.monotonic() - self.last_overview_at
        delay = max(0, int((OVERVIEW_SIGNAL_INTERVAL - elapsed) * 1000))
        self.refresh_timer.start(delay)

    def refresh(self):
        if not self.isVisible():
            return
        if self.loading:
            self.refresh_again = True
            return
        self.loading = True
        self.last_overview_at = time.monotonic()
        self.overview_queries += 1
        self.refresh_progress()
        self.bus.call("GetOverview", callback=self.received_overview)

    def _legacy_status_changed(self):
        # A daemon without StatusChanged2 only has the unversioned signal; the
        # only correct interpretation is a full refresh. Once a StatusChanged2
        # has been seen the legacy signal is a duplicate of the same revision.
        if self.status2_seen:
            return
        self.schedule_refresh()

    def received_status2(self, event):
        """Apply one revisioned StatusChanged2 change set (issue #31).

        Reset/gap/invalid/oversized payloads fall back to the existing full
        refresh path. A plain status change is patched through a single
        GetStates batch, so a write storm does not re-run GetOverview or reload
        the directory page.
        """
        self.status_events += 1
        if event.epoch:
            self.status2_seen = True
        else:
            # Owner-change sentinel: a replacement daemon may speak the old
            # protocol, so let the legacy signal drive refreshes again.
            self.status2_seen = False
        full = event.discontinuous(self.status_epoch, self.status_revision)
        self.status_epoch = event.epoch or None
        self.status_revision = event.to_revision
        if full or event.overview_changed:
            self.schedule_refresh()
            return
        if (not self.isVisible() or not self.files_button.isChecked()
                or self.loading or self._directory_loading or not self.rows
                or not event.changes):
            return
        if self._membership_affected(event.changes):
            self._queue_membership_refresh()
        paths = self._changed_row_paths(event.changes)
        if paths:
            self._request_status_patch(paths)

    @staticmethod
    def _beneath(path, root):
        if not path or not root:
            return False
        root = root.rstrip("/") or "/"
        if root == "/":
            return path.startswith("/") and path != "/"
        return path != root and path.startswith(root + "/")

    def _changed_row_paths(self, changes):
        wanted = set()
        for row in self.rows:
            row_path = row.get("path")
            if not row_path:
                continue
            for change in changes:
                changed_path = change["path"]
                if row_path == changed_path:
                    wanted.add(row_path)
                    break
                kind = change["kind"]
                if kind in ("listing", "subtree"):
                    if (self._beneath(row_path, changed_path)
                            or self._beneath(changed_path, row_path)):
                        wanted.add(row_path)
                        break
                elif kind == "status" and self._beneath(changed_path, row_path):
                    # A changed child updates the aggregate on its ancestor rows.
                    wanted.add(row_path)
                    break
        return wanted

    def _membership_affected(self, changes):
        folder = self.folder or (self.overview or {}).get("mount") or ""
        if not folder:
            return False
        for change in changes:
            path = change["path"]
            if change["kind"] == "listing" and path == folder:
                return True
            if change["kind"] == "subtree" and (
                    path == folder or self._beneath(folder, path)):
                return True
        return False

    def _queue_membership_refresh(self):
        if self._membership_again or self.membership_timer.isActive():
            return
        self._membership_again = True
        self.membership_timer.start()

    def _refresh_membership(self):
        self._membership_again = False
        if (not self.isVisible() or not self.files_button.isChecked()
                or self._directory_loading or self.loading):
            return
        self.membership_refreshes += 1
        self.request_page(self.folder, self.view_generation, 0)

    def _request_status_patch(self, paths):
        if self._status_loading:
            self._status_pending.update(paths)
            return
        self._status_loading = True
        self._status_generation = self.view_generation
        self.status_getstates += 1
        self.bus.call("GetStates", [sorted(paths)[:256]],
                      callback=self._received_status_states)

    def _received_status_states(self, data, error):
        self._status_loading = False
        current = (self._status_generation == self.view_generation
                   and not error and self.isVisible()
                   and self.files_button.isChecked()
                   and isinstance(data, dict))
        if current:
            states = {entry["path"]: entry for entry in data.get("entries", [])
                      if isinstance(entry, dict) and entry.get("path")}
            patched = 0
            dropped = 0
            page_rows = []
            for row in self.page_rows:
                state = states.get(row.get("path"))
                if state is None:
                    page_rows.append(row)
                elif state.get("error"):
                    dropped += 1
                else:
                    row.update({key: value for key, value in state.items()
                                if key != "path"})
                    page_rows.append(row)
                    patched += 1
            self.page_rows = page_rows
            self.status_patched_rows += patched
            self.status_dropped_rows += dropped
            if patched or dropped:
                self.rows = list({
                    row.get("id", row.get("path")): row
                    for row in self.page_rows}.values())
                self.render_rows()
            if dropped:
                self._queue_membership_refresh()
        if self._status_pending:
            pending = self._status_pending
            self._status_pending = set()
            self._request_status_patch(pending)

    def refresh_progress(self):
        if not self.isVisible():
            return
        if self.progress_loading:
            self.progress_again = True
            return
        self.progress_loading = True

        def received(data, error):
            self.progress_loading = False
            if (not error and self.isVisible() and self.profile
                    and data.get("task_id") == self.profile.get("task_id")):
                self.render_progress(data.get("initialization"))
            if self.progress_again:
                self.progress_again = False
                QTimer.singleShot(100, self.refresh_progress)

        self.bus.call("GetInitialization", callback=received)

    def render_shutdown(self, value, mounted):
        text = shutdown_text(value)
        self.shutdown_label.setText(text)
        self.shutdown_label.setVisible(bool(text))
        if isinstance(value, dict) and value.get('phase') not in (None, 'idle'):
            self.pause.setEnabled(False)
            self.open_folder.setEnabled(mounted is True)

    def render_progress(self, progress):
        title, detail = progress_text(progress)
        self.progress_label.setText(f"{title}\n{detail}" if title else "")
        self.progress_label.setVisible(bool(title))
        if title and progress.get("phase") != "waiting" and self.isVisible():
            if not self.progress_timer.isActive():
                self.progress_timer.start()
        else:
            self.progress_timer.stop()

    def edit_permissions(self):
        if "write_policy_revision" in self.overview:
            PermissionSettings(self.bus, self.overview, self).exec()
            self.refresh()

    def count(self, value):
        return int(value) if isinstance(value, int) and not isinstance(value, bool) and value > 0 else 0

    def update_sync_card(self, data):
        """Sync activity card; connection and cache state stay separate."""
        pending = self.count(data.get("pending_operations")) + self.count(
            data.get("uncertain_operations"))
        initialization = data.get("initialization")
        if not isinstance(initialization, dict):
            initialization = {}
        phase = initialization.get("phase")
        sync_state = data.get("sync_state")
        derived = sync_state not in SYNC or sync_state == "unknown"
        if not derived:
            value = SYNC[sync_state]
            if sync_state == "transferring":
                active = data.get("active_transfers")
                value = (value.format(n=self.count(active))
                         if isinstance(active, int) and not isinstance(active, bool) else "正在传输")
            elif sync_state == "pending":
                value = value.format(n=pending)
        else:
            state = data.get("connection")
            if data.get("error") or state == "needs_attention":
                value = SYNC["needs_attention"]
            elif state == "paused":
                value = SYNC["paused"]
            elif data.get("upload_block"):
                value = "上传暂缓"
            elif initialization.get("active") is True and phase != "waiting":
                value = SYNC["checking"]
            elif pending:
                value = SYNC["pending"].format(n=pending)
            elif phase == "waiting":
                value = SYNC["idle"]
            else:
                value = "尚未核对"
        self.sync_value.setText(value)
        tooltip = []
        if initialization:
            _title, detail = progress_text(initialization)
            if detail:
                tooltip.append(detail)
        tooltip.append(f"待处理 {self.count(data.get('pending_operations'))} 项")
        if self.count(data.get("uncertain_operations")):
            tooltip.append(f"待确认 {self.count(data.get('uncertain_operations'))} 项")
        if self.count(data.get("paused_uploads")):
            tooltip.append(f"{self.count(data.get('paused_uploads'))} 项上传已暂停自动重试")
        if derived:
            tooltip.append("按旧后台推导")
        self.sync_value.setToolTip("\n".join(tooltip))

    def update_availability_card(self, data):
        """Local availability card; owner of the cache aggregate is the backend."""
        cache = data.get("cache_availability")
        if isinstance(cache, dict) and cache.get("state") in CACHE:
            self.cache_from_backend = True
            self.availability_value.setText(CACHE[cache["state"]])
            total = self.count(cache.get("total"))
            offline = self.count(cache.get("offline"))
            pinned = self.count(cache.get("pinned"))
            self.availability_value.setToolTip(f"可离线打开 {offline}/{total}；已固定 {pinned}")
        else:
            self.cache_from_backend = False
            self.availability_value.setToolTip("后端不支持缓存汇总")

    def mark_cards_stale(self):
        for card in (self.connection_value, self.sync_value, self.availability_value):
            if card.text():
                card.setToolTip("状态可能过期（上次确认）")

    def received_overview(self, data, error):
        if error:
            self.loading = False
            if not self.bus.owner:
                self.offline_ui()
            else:
                # NoReply/Timeout can be a slow local transaction. Preserve the
                # last confirmed mount/control state; only owner loss is offline.
                self.show_error(f"暂时无法读取状态详情（{error}）。保留上次确认的挂载状态。")
                self.mark_cards_stale()
                self.refresh_progress()
                self.ready.emit()
            self.followup_refresh()
            return
        self.overview = data
        self.permission_label.setText(permission_text(data))
        self.permission_settings.setEnabled("write_policy_revision" in data)
        self.permission_recheck.setEnabled("write_policy_revision" in data)
        if "write_policy_revision" in data:
            folder, task = self.folder, data.get("task_id")
            def received_permissions(value, error):
                if not error and value and self.folder == folder and self.overview.get("task_id") == task:
                    self.permission_label.setText(permission_text(self.overview, value))
            self.bus.call("GetDirectoryPermissions", (json.dumps({"path": folder}),), callback=received_permissions)
        block = data.get("upload_block")
        self.upload_block_label.setText("上传受阻：" + str(block)[:2000] if block else "")
        self.upload_block_label.setVisible(bool(block))
        self.upload_block_action.setVisible(bool(block))
        if self.issues_dialog is not None:
            self.issues_dialog.update_context(data)
        self.preview_label.setVisible(data.get("preview_only") is True)
        self.overview_hint.setText(
            "已确认的目录可以从文件管理器浏览。\n初始化预览中的文件暂不能打开或编辑。"
            if data.get("preview_only") is True else
            "文件可直接从文件管理器打开。\n选择“文件”页，按需载入当前目录的状态列表。")
        state = data["connection"]
        self.connection_value.setText(CHANNEL.get(state, "需要检查"))
        self.connection_value.setToolTip("")
        self.render_progress(data.get("initialization"))
        self.update_sync_card(data)
        self.update_availability_card(data)
        self.account.setText(f"{data.get('username','')}\n{data.get('host','')}")
        self.configure.setText("解锁 / 重新登录")
        self.configure.setEnabled(self.profile is not None)
        scope_name = Path(data.get("scope_path", "/")).name or "同步根目录"
        self.files_title.setText(scope_name)
        self.subtitle.setText(f"{scope_name}  ·  本地文件夹 {Path(data['mount']).name}")
        self.subtitle.setToolTip(f"{data.get('scope_path','')} → {data['mount']}")
        self.setWindowTitle(Path(data["mount"]).name + " · Drive Native")
        self.open_folder.setEnabled(data["mounted"])
        self.pause.setEnabled(state not in ("locked", "stopping"))
        self.pause.setText("恢复同步" if state == "paused" else "暂停同步")
        if self.shutdown_watcher.value is None:
            self.render_shutdown(data.get("shutdown"), data.get("mounted"))
        else:
            self.render_shutdown(self.shutdown_watcher.value, self.shutdown_watcher.mounted)
        if data.get("error"):
            self.show_error(ERRORS.get(data["error"], "同步需要检查，已保存的文件会保留。"))
        elif data.get("background_error"):
            self.show_error("部分传输未完成。已有完整内容已保留，请检查连接后重试。")
        elif data.get("download_errors"):
            self.show_error("部分下载未完成。已有完整内容会保留，可查看文件状态或右键重试下载。")
        elif queue_notice(data):
            self.show_error(queue_notice(data))
        elif state in ("offline", "paused", "locked"):
            self.show_error("当前未与 NAS 同步，已确认的目录仍可预览，文件正文暂不可用。"
                            if data.get("preview_only") is True else
                            "当前未与 NAS 同步。本地完整文件仍可使用；远端变化将在连接恢复后核对。")
        else:
            self.banner.hide()
        if state == "locked" and self.pending_password:
            password = self.pending_password
            self.pending_password = None
            self.unlock_if_matching(password)
        elif state == "locked" and self.profile and not self.auto_unlock_attempted:
            self.auto_unlock_attempted = True
            self.keyring_job = secret_job(self.profile["task_id"], self.keyring_loaded)
        if data["mounted"] and self.isVisible() and self.files_button.isChecked():
            # Keep the current page accumulator until the replacement offset-0
            # reply arrives; clearing it here dropped rows that the first page
            # of a multi-page directory had already appended (issue #29).
            if not self._directory_loading:
                self.request_page(self.folder, self.view_generation, 0)
        else:
            self.loading = False
            self.ready.emit()
            self.followup_refresh()

    def unlock_if_matching(self, password):
        owner = self.bus.owner

        def checked(data, error):
            if error or owner != self.bus.owner or not same_task(self.profile, data):
                self.show_error("当前后台任务与配置不一致，未发送登录凭据。")
                return
            self.bus.call("Unlock", [password], self.action_done)

        self.bus.call("GetOverview", callback=checked)

    def keyring_loaded(self, password, error):
        if password:
            self.unlock_if_matching(password)
        elif error:
            self.show_error(error)

    def request_page(self, path, generation, offset):
        self.directory_queries += 1
        self._directory_loading = True
        self.bus.call(
            "GetDirectory",
            [path, offset],
            lambda data, error: self.received_directory(path, generation, data, error),
        )

    def received_directory(self, path, generation, data, error):
        # This callback closes one page; a multi-page traversal re-arms the
        # flag in request_page. While it is set, received_overview must not
        # start a duplicate offset-0 request (issue #29).
        self._directory_loading = False
        if not self.isVisible() or not self.files_button.isChecked():
            self.loading = False
            self.followup_refresh()
            return
        if path != self.folder or generation != self.view_generation:
            self.loading = False
            self.schedule_refresh()
            return
        if error:
            self.loading = False
            self.show_error("无法读取本地文件列表。")
            self.followup_refresh()
            return
        if data.get("preview_only") is True and data.get("directory_state") != "received":
            # An incomplete listing is not an empty directory. Do not retain
            # names from a preview whose membership/permission was revoked.
            self.loading = False
            self.page_rows = []
            self.rows = []
            self.render_rows()
            self.more.hide()
            self.detail.setText("该目录暂时没有访问权限。" if data.get("directory_state") == "denied"
                                else "目录列表尚未就绪，完成核对后会自动刷新。")
            if not self.cache_from_backend:
                self.availability_value.setText("尚未确认")
            self.up.setEnabled(bool(self.folder))
            self.ready.emit()
            self.followup_refresh()
            return
        token = (data.get("preview_only") is True, data.get("preview_generation"))
        if data["offset"] == 0:
            # Offset 0 starts a new listing: replace the accumulator here
            # rather than in received_overview, so a reply that races an
            # overview refresh cannot append the same first page twice.
            self.page_view_token = token
            self.page_rows = list(data["entries"])
        elif token != getattr(self, "page_view_token", None):
            self.loading = False
            self.page_rows = []
            self.rows = []
            self.render_rows()
            self.more.hide()
            self.detail.setText("目录内容已变化，等待下一次状态刷新。")
            if not self.cache_from_backend:
                self.availability_value.setText("尚未确认")
            self.ready.emit()
            self.followup_refresh()
            return
        else:
            self.page_rows.extend(data["entries"])
        next_offset = data["offset"] + len(data["entries"])
        if data["entries"] and next_offset < min(self.display_limit, data["total"]):
            self.request_page(path, generation, next_offset)
            return
        self.loading = False
        self.rows = list({row.get("id", row["path"]): row for row in self.page_rows}.values())
        self.render_rows()
        self.more.setVisible(next_offset < data["total"])
        skipped = (self.overview or {}).get("skipped_permission_directories", 0)
        suffix = f" · 已跳过 {skipped} 个无权限目录；权限恢复后重新登录可重新核对" if skipped else ""
        links = (self.overview or {}).get("skipped_symlinks", 0)
        if links: suffix += f" · 已跳过 {links} 个符号链接（暂不支持，不读取链接目标）"
        qualifier = "初始化预览，正文尚未可用" if data.get("preview_only") is True else "状态按最后已知远端版本显示"
        self.detail.setText(f"{len(self.rows)} / {data['total']} 项 · {qualifier}" + suffix)
        if not self.cache_from_backend:
            self.availability_value.setText(
                f"{sum(bool(r.get('offline_available')) for r in self.rows)} / {len(self.rows)}"
            )
        self.breadcrumb.setText(self.files_title.text() + ("/" + self.folder if self.folder else ""))
        self.up.setEnabled(bool(self.folder))
        self.ready.emit()
        self.followup_refresh()

    def followup_refresh(self):
        if self.refresh_again:
            self.refresh_again = False
            self.schedule_refresh()

    def render_rows(self):
        # Keep item identities while status signals arrive. Rebuilding the tree
        # between press/release/double-click loses Qt's activation target and
        # also disrupts keyboard focus, selection and scrolling.
        changed = False
        existing = {}
        for index in range(self.table.topLevelItemCount()):
            item = self.table.topLevelItem(index)
            old = item.data(0, Qt.UserRole)
            existing[old.get("id", old["path"])] = item
        ordered = sorted(
            self.rows, key=lambda r: (not r.get("directory"), Path(r["path"]).name.casefold())
        )
        wanted = {row.get("id", row["path"]) for row in ordered}
        for key, item in existing.items():
            if key not in wanted:
                self.table.takeTopLevelItem(self.table.indexOfTopLevelItem(item))
                changed = True
        for index, row in enumerate(ordered):
            key = row.get("id", row["path"])
            item = existing.get(key)
            if item is None:
                item = QTreeWidgetItem()
                self.table.insertTopLevelItem(index, item)
                changed = True
            elif self.table.indexOfTopLevelItem(item) != index:
                self.table.takeTopLevelItem(self.table.indexOfTopLevelItem(item))
                self.table.insertTopLevelItem(index, item)
            status = row.get("status")
            for column, text in enumerate(
                [
                    Path(row["path"]).name,
                    sync_row_text(row),
                    cache_row_text(row),
                ]
            ):
                if item.text(column) != text:
                    item.setText(column, text)
                    changed = True
            failure = row.get("sync_error")
            detail = ""
            if failure:
                detail = f"下载尝试 {failure['attempts']} 次"
                if failure.get("server_code") is not None:
                    detail += f" · 服务器错误 {failure['server_code']}"
                if row.get("offline_available"):
                    detail += " · 仍可使用已有完整版本"
            item.setToolTip(1, detail)
            item.setData(0, Qt.UserRole, row)
            if item.data(1, Qt.UserRole) != status:
                item.setIcon(1, status_icon(status))
                item.setData(1, Qt.UserRole, status)
                self.icon_sets += 1
            directory = bool(row.get("directory"))
            if item.data(0, Qt.UserRole + 1) != directory:
                item.setIcon(0, self._standard_icon(directory))
                item.setData(0, Qt.UserRole + 1, directory)
                self.icon_sets += 1
        if changed:
            self.apply_filter()

    def _standard_icon(self, directory):
        # QStyle returns the same standard icon every call, but resolving it
        # per row per repaint is avoidable state lookup (issue #29).
        if directory:
            if self._dir_icon is None:
                self._dir_icon = self.style().standardIcon(QStyle.SP_DirIcon)
            return self._dir_icon
        if self._file_icon is None:
            self._file_icon = self.style().standardIcon(QStyle.SP_FileIcon)
        return self._file_icon

    def apply_filter(self):
        self.filter_passes += 1
        text = self.filter.text().casefold()
        for index in range(self.table.topLevelItemCount()):
            item = self.table.topLevelItem(index)
            hidden = text not in item.text(0).casefold()
            if item.isHidden() != hidden:
                item.setHidden(hidden)
                self.filter_row_updates += 1

    def navigate(self, path):
        self.folder = path
        self.display_limit = 256
        self.view_generation += 1
        self.schedule_refresh()

    def parent_folder(self):
        self.navigate(self.folder.rsplit("/", 1)[0] if "/" in self.folder else "")

    def load_more(self):
        self.display_limit += 256
        self.schedule_refresh()

    def activate(self, item, _column):
        row = item.data(0, Qt.UserRole)
        if row.get("directory"):
            self.navigate(row["path"])
        elif self.overview.get("preview_only") is True or row.get("status") == "initializing":
            self.show_error("该文件仍处于初始化预览，正文尚未可用。")
        else:
            QDesktopServices.openUrl(
                QUrl.fromLocalFile(str(Path(self.overview["mount"]) / row["path"]))
            )

    def open_in_files(self):
        if self.overview.get("mounted"):
            QDesktopServices.openUrl(
                QUrl.fromLocalFile(str(Path(self.overview["mount"]) / self.folder))
            )

    def apply_backend_update(self):
        if not same_task(self.profile, self.overview):
            self.show_error("任务身份尚未确认，请等待状态刷新。")
            return
        answer = QMessageBox.question(self, "应用后台更新", "请先保存并关闭此挂载中的文件。将暂停同步、正常停止旧挂载，再启动已安装的后台；缓存和未上传修改会保留。", QMessageBox.Yes | QMessageBox.No, QMessageBox.No)
        if answer == QMessageBox.Yes:
            if self.backend_upgrade.begin(self.profile):
                self.update_backend.setEnabled(False)

    def backend_updated(self, error):
        self.update_backend.setEnabled(True)
        if error:
            self.show_error(error)
        else:
            self.show_error("后台已切换。请检查暂停状态后继续同步。")
        self.schedule_refresh()

    def toggle_pause(self):
        self.pause_intent.emit()
        self.bus.call(
            "Resume" if self.overview.get("connection") == "paused" else "Pause",
            callback=self.action_done,
        )

    def stop_and_unmount(self):
        """Confirm, then ask the unified window to stop and unmount this mount.

        This page never stops services itself: the window owns stage polling and
        diagnostics. Pausing keeps the mount; stopping ends the mount and its
        background service without deleting local cache, drafts or remote files.
        """
        answer = QMessageBox.question(
            self, "停止此挂载并卸载",
            "暂停同步只暂停此挂载并保留挂载点，可稍后恢复。\n"
            "停止并卸载会结束此挂载及其后台服务，不会删除本地缓存、草稿或远端文件。\n"
            "确定要停止并卸载此挂载吗？",
            QMessageBox.Yes | QMessageBox.No, QMessageBox.No)
        if answer != QMessageBox.Yes:
            return
        self.set_stopping(True)
        self.show_error("已请求停止并卸载此挂载，正在等待后台确认停止阶段。暂停同步只会暂停并保留挂载。")
        self.stop_requested.emit()

    def set_stopping(self, stopping, message=""):
        """Allow the owner window to restore the button when stopping fails."""
        self.stop_mount.setEnabled(not stopping)
        self.stop_mount.setText("正在停止并卸载…" if stopping else "停止此挂载并卸载…")
        if message:
            self.show_error(message)

    def show_error(self, text):
        self.banner.setText(text)
        self.banner.show()

    def begin_release(self, rows):
        """Start one cancellable background job for every selected path."""
        paths = [row["path"] for row in rows]
        if not paths:
            return
        self.bus.call("BeginReleaseSpace", [paths], self.release_started)

    def release_started(self, data, error):
        total = data.get("total") if isinstance(data, dict) else None
        job_id = data.get("job_id") if isinstance(data, dict) else None
        if (error or not isinstance(data, dict) or data.get("schema") != 1
                or not isinstance(job_id, str) or not job_id
                or not isinstance(total, int) or total < 0):
            self.show_error("操作未完成，文件仍由后台服务保护。")
            return
        self.release_job_id = job_id
        self.release_total = total
        self.release_label.setText(
            f"正在释放 0/{total} · 已回收 {byte_text(0)}")
        self.release_label.show()
        self.release_cancel.setEnabled(True)
        self.release_cancel.setText("取消释放")
        self.release_cancel.show()
        self.release_timer.start()

    def poll_release_job(self):
        if not self.release_job_id:
            self.release_timer.stop()
            return
        self.bus.call("GetReleaseJob", [self.release_job_id],
                      self.release_progress)

    def release_progress(self, data, error):
        job_id = self.release_job_id
        if not job_id:
            return
        # A failed poll is retried; the last known progress stays visible.
        if error or not isinstance(data, dict) or data.get("schema") != 1:
            return
        if data.get("available") is False:
            # The job table only lives in service memory: after a restart the
            # id is unknown and the backend reports available:false (see
            # docs/CACHE_RELEASE.md). Stop polling and return the window to the
            # idle state instead of leaving a fake progress row behind.
            self.release_timer.stop()
            self.release_job_id = None
            self.release_total = 0
            self.release_label.hide()
            self.release_cancel.hide()
            self.show_error(
                "后台服务已重启，上一次释放任务已结束；文件仍受保护，可重新发起释放。")
            self.schedule_refresh()
            return
        if data.get("job_id") != job_id:
            return
        done = data.get("done")
        total = data.get("total", self.release_total)
        reclaimed = data.get("reclaimed_bytes", 0)
        if not isinstance(done, int) or not isinstance(total, int):
            return
        if not data.get("finished"):
            self.release_label.setText(
                f"正在释放 {done}/{total} · 已回收 {byte_text(reclaimed)}")
            return
        self.release_timer.stop()
        self.release_job_id = None
        self.release_label.hide()
        self.release_cancel.hide()
        if data.get("cancelled"):
            self.show_error(
                f"已取消释放：完成 {done}/{total}，已回收 {byte_text(reclaimed)}。"
                "已完成的文件不会回滚。")
        else:
            protected = data.get("protected_count", 0)
            message = (f"释放完成：已释放 {data.get('released_count', 0)} 个文件，"
                       f"跳过 {data.get('skipped_count', 0)} 个，"
                       f"回收 {byte_text(reclaimed)}。")
            if isinstance(protected, int) and protected > 0:
                message += f"另有 {protected} 个文件受保护暂未释放。"
            self.show_error(message)
        self.schedule_refresh()

    def cancel_release(self):
        if not self.release_job_id:
            return
        self.release_cancel.setEnabled(False)
        self.release_cancel.setText("正在取消…")
        self.show_error("正在取消释放，已完成的文件不会回滚。")
        self.bus.call("CancelReleaseSpace", [self.release_job_id],
                      self.release_cancel_done)

    def release_cancel_done(self, data, error):
        if error or not isinstance(data, dict):
            self.show_error("操作未完成，文件仍由后台服务保护。")

    def action_done(self, data, error):
        if error:
            self.show_error("操作未完成，文件仍由后台服务保护。")
        elif isinstance(data, dict) and data.get("protected_or_failed"):
            self.show_error("部分文件仍在使用、已固定或有未同步修改，暂未释放。")
        self.schedule_refresh()

    def context_menu(self, point):
        if (self.overview or {}).get("preview_only") is True:
            return
        selected = [item.data(0, Qt.UserRole) for item in self.table.selectedItems()]
        if not selected:
            return
        menu = QMenu(self)
        for title, method, value in [
            ("固定到本地", "SetPinned", True),
            ("取消固定", "SetPinned", False),
            ("释放空间", "ReleaseSpace", None),
            ("重试下载", "RetryDownloads", None),
        ]:
            action = menu.addAction(title)
            action.triggered.connect(
                lambda _checked=False, m=method, v=value: self.apply_action(selected, m, v)
            )
        menu.exec(self.table.viewport().mapToGlobal(point))

    def apply_action(self, rows, method, value):
        if (self.overview or {}).get("preview_only") is True or any(
                row.get("status") == "initializing" for row in rows):
            self.show_error("初始化预览期间暂不能固定、释放或修改文件。")
            return
        if method == "ReleaseSpace":
            self.begin_release(rows)
            return
        for row in rows:
            args = [row["path"]] if value is None else [row["path"], value]
            self.bus.call(method, args, self.action_done)
