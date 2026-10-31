import os
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
)
from .issues import IssuesDialog
from .trust import TrustSettingsDialog
from .transfers import TransfersDialog
from .cache_settings import CacheSettingsDialog
from .bus import ClientBus
from .config import load_profile, config_path, binary, archive_profile, restore_profile
from .wizard import SetupWizard
from .credentials import secret_job
from .lifecycle import ServiceLauncher, same_task
from .upgrade import BackendUpgrade

STATUS = {
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
    "native_index_capacity": "目录历史或索引超过当前处理预算；本地内容已保留。请导出诊断，重新登录不会解决此问题。",
    "native_response_capacity": "服务器元数据响应超过当前解码预算；本地内容已保留。请导出诊断。",
    "native_state_recovery_required": "远端状态需要恢复，已保存的本地内容会保留。",
    "network_unavailable": "暂时无法连接 NAS，已有缓存仍可使用。",
    "local_or_protocol_error": "同步遇到问题，已保存的本地内容会保留。",
}


def status_icon(status):
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
    return QIcon(pix)


class MainWindow(QMainWindow):
    ready = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Drive Native")
        self.resize(1120, 740)
        self.setMinimumSize(820, 580)
        self.bus = ClientBus(self)
        self.launcher = ServiceLauncher(self)
        self.launcher.completed.connect(self.service_started)
        self.profile = None
        self.overview = {}
        self.folder = ""
        self.display_limit = 256
        self.view_generation = 0
        self.rows = []
        self.loading = False
        self.refresh_again = False
        self.pending_password = None
        self.auto_unlock_attempted = False
        self.keyring_job = None
        root = QWidget()
        self.setCentralWidget(root)
        layout = QHBoxLayout(root)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        side = QFrame()
        side.setObjectName("sidebar")
        side.setFixedWidth(214)
        left = QVBoxLayout(side)
        left.setContentsMargins(24, 30, 24, 22)
        left.setSpacing(12)
        brand = QLabel("Drive Native")
        brand.setObjectName("brand")
        left.addWidget(brand)
        caption = QLabel("你的文件，随手可用")
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
        layout.addWidget(side)
        content = QVBoxLayout()
        content.setContentsMargins(30, 28, 30, 24)
        content.setSpacing(18)
        layout.addLayout(content, 1)
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
        self.pending_value = self.card(cards, "待处理操作", "—")
        self.availability_value = self.card(cards, "当前目录本地可用", "按需查询")
        content.addLayout(cards)
        self.banner = QLabel()
        self.banner.setWordWrap(True)
        self.banner.setObjectName("message")
        self.banner.hide()
        content.addWidget(self.banner)
        self.overview_hint = QLabel("文件可直接从文件管理器打开。\n点击左侧“查看文件状态”，按需载入当前目录的状态列表。")
        self.overview_hint.setObjectName("muted")
        self.overview_hint.setAlignment(Qt.AlignCenter)
        self.overview_hint.setWordWrap(True)
        content.addWidget(self.overview_hint, 1)
        self.file_details = QWidget()
        details = QVBoxLayout(self.file_details)
        details.setContentsMargins(0, 0, 0, 0)
        content.addWidget(self.file_details, 1)
        self.file_details.hide()
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
        self.table.setHeaderLabels(["名称", "同步状态", "离线使用"])
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
        self.refresh_timer = QTimer(self)
        self.refresh_timer.setSingleShot(True)
        self.refresh_timer.setInterval(100)
        self.refresh_timer.timeout.connect(self.refresh)
        self.bus.changed.connect(self.schedule_refresh)
        self.bus.availability.connect(self.service_availability)
        for label in self.findChildren(QLabel):
            label.setTextFormat(Qt.PlainText)
        self.apply_theme()
        QTimer.singleShot(0, self.start)

    def toggle_files(self, visible):
        self.file_details.setVisible(visible)
        self.overview_hint.setVisible(not visible)
        self.files_button.setText("收起文件状态" if visible else "查看文件状态")
        self.view_generation += 1
        if visible:
            self.schedule_refresh()
        else:
            self.availability_value.setText("按需查询")

    def showEvent(self, event):
        super().showEvent(event)
        self.schedule_refresh()

    def hideEvent(self, event):
        self.refresh_timer.stop()
        self.refresh_again = False
        self.view_generation += 1
        super().hideEvent(event)

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
            self.issues_dialog = IssuesDialog(self.bus, self)
            self.issues_dialog.resolved.connect(self.schedule_refresh)
        self.issues_dialog.show()
        self.issues_dialog.raise_()
        self.issues_dialog.activateWindow()
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
        muted = "#a8b2c2" if self.palette().color(QPalette.Window).lightness() < 128 else "#626d7c"
        stylesheet = """QWidget{font-size:14px;} QMainWindow{background:palette(window);}
            QFrame#sidebar{background:palette(base);border-right:1px solid palette(midlight);}
            QLabel#brand{font-size:22px;font-weight:700;} QLabel#heading{font-size:27px;font-weight:650;}
            QLabel#metric{font-size:24px;font-weight:600;} QLabel#muted{color:MUTED_COLOR;font-size:12px;}
            QFrame#card{background:palette(base);border:1px solid palette(midlight);border-radius:12px;}
            QPushButton{color:palette(button-text);padding:9px 14px;border:1px solid palette(midlight);border-radius:7px;background:palette(base);}
            QPushButton:hover{background:palette(alternate-base);} QPushButton:disabled{color:palette(mid);}
            QPushButton#primary,QPushButton#navigation{background:#446fd1;color:white;border:none;}
            QLineEdit{color:palette(text);selection-background-color:palette(highlight);selection-color:palette(highlighted-text);padding:9px;border:1px solid palette(midlight);border-radius:6px;background:palette(base);}
            QTreeWidget{border:1px solid palette(midlight);border-radius:9px;background:palette(base);color:palette(text);selection-background-color:palette(highlight);selection-color:palette(highlighted-text);outline:none;}
            QTreeWidget::item{padding:10px 6px;} QHeaderView::section{padding:10px;background:palette(alternate-base);border:none;text-align:left;}
            QLabel#message{padding:12px;border-radius:8px;background:palette(alternate-base);}
        """
        self.setStyleSheet(stylesheet.replace("MUTED_COLOR", muted))

    def start(self):
        try:
            self.profile = load_profile()
        except Exception:
            self.show_error("无法读取同步配置，请检查配置文件权限。")
        self.bus.call("GetOverview", callback=self.initial_overview)

    def initial_overview(self, data, error):
        if not error:
            self.received_overview(data, None)
            return
        if self.profile:
            self.start_service()
        else:
            self.offline_ui()

    def start_service(self, *, enable=False):
        self.connection_value.setText("正在启动")
        self.launcher.start(enable=enable)

    def service_started(self, error):
        if error:
            self.show_error(error)
        self.schedule_refresh()

    def offline_ui(self):
        self.connection_value.setText("尚未连接")
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
        wizard = SetupWizard(self)
        wizard.configured.connect(self.configured)
        wizard.open()
        self.wizard = wizard

    def configured(self, profile, password):
        self.profile = profile
        self.folder = ""
        self.auto_unlock_attempted = False
        self.pending_password = password
        self.start_service(enable=True)

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
                backup = archive_profile(previous["task_id"])
            except (OSError, ValueError) as error:
                self.show_error(str(error))
                self.start_service()
                return
            self.profile = None
            wizard = SetupWizard(self)
            wizard.host.setText(previous["host"])
            wizard.tls.setText(previous["tls_name"])
            wizard.username.setText(previous["username"])
            wizard.trusted_ca_pem = list(previous.get("trusted_ca_pem", []))
            wizard.mount.setText(previous["mount"])
            wizard.configured.connect(self.configured)

            def cancelled():
                try:
                    restore_profile(backup)
                    self.profile = previous
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
            self.connection_value.setText("后台已停止")
            self.pause.setEnabled(False)
        self.schedule_refresh()

    def schedule_refresh(self):
        if not self.isVisible():
            return
        if self.loading:
            self.refresh_again = True
        elif not self.refresh_timer.isActive():
            self.refresh_timer.start()

    def refresh(self):
        if not self.isVisible():
            return
        if self.loading:
            self.refresh_again = True
            return
        self.loading = True
        self.bus.call("GetOverview", callback=self.received_overview)

    def received_overview(self, data, error):
        if error:
            self.loading = False
            self.offline_ui()
            return
        self.overview = data
        state = data["connection"]
        self.connection_value.setText(CONNECTION.get(state, "需要检查"))
        self.pending_value.setText(str(data["pending_operations"]))
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
        if data.get("error"):
            self.show_error(ERRORS.get(data["error"], "同步需要检查，已保存的文件会保留。"))
        elif data.get("background_error"):
            self.show_error("部分传输未完成。已有完整内容已保留，请检查连接后重试。")
        elif data.get("download_errors"):
            self.show_error("部分下载未完成。已有完整内容会保留，可查看文件状态或右键重试下载。")
        elif state in ("offline", "paused", "locked"):
            self.show_error("当前未与 NAS 同步。本地完整文件仍可使用；远端变化将在连接恢复后核对。")
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
            self.page_rows = []
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
        self.bus.call(
            "GetDirectory",
            [path, offset],
            lambda data, error: self.received_directory(path, generation, data, error),
        )

    def received_directory(self, path, generation, data, error):
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
        self.detail.setText(f"{len(self.rows)} / {data['total']} 项 · 状态按最后已知远端版本显示" + suffix)
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
        for index, row in enumerate(ordered):
            key = row.get("id", row["path"])
            item = existing.get(key)
            if item is None:
                item = QTreeWidgetItem()
                self.table.insertTopLevelItem(index, item)
            elif self.table.indexOfTopLevelItem(item) != index:
                self.table.takeTopLevelItem(self.table.indexOfTopLevelItem(item))
                self.table.insertTopLevelItem(index, item)
            status = row.get("status")
            label = STATUS.get(status, ("状态不可用", ""))[0]
            if row.get("error"):
                label = "状态读取失败"
            for column, text in enumerate(
                [
                    Path(row["path"]).name,
                    label,
                    "可用" if row.get("offline_available") else "需要联网",
                ]
            ):
                if item.text(column) != text:
                    item.setText(column, text)
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
            item.setIcon(1, status_icon(status))
            item.setIcon(
                0,
                self.style().standardIcon(
                    QStyle.SP_DirIcon if row.get("directory") else QStyle.SP_FileIcon
                ),
            )
        self.apply_filter()

    def apply_filter(self):
        text = self.filter.text().casefold()
        for index in range(self.table.topLevelItemCount()):
            item = self.table.topLevelItem(index)
            item.setHidden(text not in item.text(0).casefold())

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
        self.bus.call(
            "Resume" if self.overview.get("connection") == "paused" else "Pause",
            callback=self.action_done,
        )

    def show_error(self, text):
        self.banner.setText(text)
        self.banner.show()

    def action_done(self, data, error):
        if error:
            self.show_error("操作未完成，文件仍由后台服务保护。")
        elif isinstance(data, dict) and data.get("protected_or_failed"):
            self.show_error("部分文件仍在使用、已固定或有未同步修改，暂未释放。")
        self.schedule_refresh()

    def context_menu(self, point):
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
        for row in rows:
            args = [row["path"]] if value is None else [row["path"], value]
            self.bus.call(method, args, self.action_done)
