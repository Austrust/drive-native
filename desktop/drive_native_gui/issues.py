"""Local conflict centre. Decisions carry a version token checked by the fsd."""
from pathlib import Path, PurePosixPath
from uuid import uuid4
from PySide6.QtCore import Qt, QTimer, Signal, QUrl
from PySide6.QtWidgets import (
    QApplication, QDialog, QVBoxLayout, QHBoxLayout, QLabel, QTreeWidget,
    QTreeWidgetItem, QHeaderView, QLineEdit, QPushButton,
)

from PySide6.QtGui import QDesktopServices
from .issue_actions import next_step, parent_candidates, diagnostic_text, recheck_message, TASK_STEPS

REASONS = {
    "content_conflict": "本地和远端都有修改",
    "remote_deleted": "远端已删除，本地修改仍保留",
    "commit_uncertain": "服务器执行结果尚未确认",
    "namespace_conflict": "目录、改名或删除操作需要处理",
    "directory_creation_conflict": "新建文件夹与远端同名条目冲突",
    "creation_conflict": "新建文件与远端同名条目冲突",
    "creation_identity_mismatch": "新建文件身份需要进一步核对",
    "parent_unavailable": "原文件夹已不可用",
    "local_content_unavailable": "需要恢复本地内容",
    "file_in_use": "请先在其他应用中关闭文件",
}


class IssuesDialog(QDialog):
    resolved = Signal()

    def __init__(self, bus, parent=None, *, configure=None):
        super().__init__(parent)
        self.bus = bus
        self.configure = configure
        self.overview = {}
        self.context_loading = False
        self.action_generation = 0
        self.busy = False
        self.loading = False
        self.refresh_again = False
        self.generation = 0
        self.selected_id = None
        self.rows = {}
        self.cursor = ""
        self.display_limit = 256
        self.setWindowTitle("同步问题 · Drive Native")
        self.resize(860, 560)
        self.setMinimumSize(640, 440)
        layout = QVBoxLayout(self)
        title = QLabel("处理同步问题")
        title.setObjectName("heading")
        layout.addWidget(title)
        self.message = QLabel("正在读取本地状态…")
        self.message.setWordWrap(True)
        layout.addWidget(self.message)
        self.context_notice = QLabel()
        self.context_notice.setWordWrap(True)
        self.context_notice.hide()
        layout.addWidget(self.context_notice)
        self.table = QTreeWidget()
        self.table.setRootIsDecorated(False)
        self.table.setHeaderLabels(["文件", "需要处理的原因"])
        self.table.header().setSectionResizeMode(0, QHeaderView.Stretch)
        self.table.header().setSectionResizeMode(1, QHeaderView.ResizeToContents)
        self.table.itemSelectionChanged.connect(self.selection_changed)
        layout.addWidget(self.table, 1)
        self.explanation = QLabel("选择一个文件查看处理方式。")
        self.explanation.setWordWrap(True)
        layout.addWidget(self.explanation)
        self.name = QLineEdit()
        self.name.setPlaceholderText("本地副本的新文件名")
        self.name.setAccessibleName("本地冲突副本文件名")
        layout.addWidget(self.name)
        recovery = QHBoxLayout()
        self.recheck = QPushButton("重新核对 / 重连")
        self.recheck.clicked.connect(self.request_recheck)
        recovery.addWidget(self.recheck)
        self.open_parent = QPushButton("打开可用父目录")
        self.open_parent.clicked.connect(self.open_available_parent)
        recovery.addWidget(self.open_parent)
        self.copy_details = QPushButton("复制诊断")
        self.copy_details.clicked.connect(self.copy_diagnostic)
        recovery.addWidget(self.copy_details)
        self.pause_sync = QPushButton("暂停并保留数据")
        self.pause_sync.clicked.connect(self.pause_task)
        recovery.addWidget(self.pause_sync)
        layout.addLayout(recovery)
        self.login = QPushButton("解锁 / 重新登录")
        self.login.clicked.connect(lambda: self.configure() if self.configure else None)
        self.login.setVisible(configure is not None)
        layout.addWidget(self.login)
        actions = QHBoxLayout()
        self.more = QPushButton("加载更多")
        self.more.clicked.connect(self.load_more)
        self.more.hide()
        actions.addWidget(self.more)
        self.refresh_button = QPushButton("刷新状态")
        self.refresh_button.clicked.connect(self.refresh)
        actions.addWidget(self.refresh_button)
        actions.addStretch()
        self.keep = QPushButton("保留双方版本")
        self.keep.setObjectName("primary")
        self.keep.clicked.connect(self.keep_both)
        actions.addWidget(self.keep)
        close = QPushButton("关闭")
        close.clicked.connect(self.close)
        actions.addWidget(close)
        layout.addLayout(actions)
        for label in self.findChildren(QLabel):
            label.setTextFormat(Qt.PlainText)
        self.timer = QTimer(self)
        self.timer.setSingleShot(True)
        self.timer.setInterval(150)
        self.timer.timeout.connect(self.refresh)
        self.bus.changed.connect(self.schedule_refresh)
        self.selection_changed()

    def showEvent(self, event):
        super().showEvent(event)
        self.refresh()

    def hideEvent(self, event):
        self.timer.stop()
        self.action_generation += 1
        super().hideEvent(event)

    def schedule_refresh(self):
        if self.isVisible():
            self.timer.start()

    def refresh(self):
        if not self.isVisible():
            return
        if self.busy or self.loading:
            self.refresh_again = True
            return
        self.generation += 1
        self.request_page("", self.generation)
        self.refresh_context()

    def load_more(self):
        if self.cursor and not self.loading and not self.busy:
            self.display_limit += 256
            self.request_page(self.cursor, self.generation)

    def request_page(self, after, generation):
        self.loading = True
        self.bus.call("GetIssues", [after], lambda data, error: self.received(after, generation, data, error))

    def received(self, after, generation, data, error):
        if generation != self.generation:
            return
        self.loading = False
        if error or not isinstance(data, dict) or not isinstance(data.get("entries"), list):
            self.message.setText("暂时无法读取状态，请在服务恢复后刷新。")
            self.rows.clear()
            self.table.clear()
            self.selection_changed()
            return
        if not after:
            self.rows = {}
        self.rows.update({row["id"]: row for row in data["entries"]})
        self.cursor = data.get("next") or ""
        if self.isVisible() and self.cursor and len(self.rows) < self.display_limit:
            self.request_page(self.cursor, generation)
            return
        selected = self.selected_id
        self.table.blockSignals(True)
        self.table.clear()
        for row in sorted(self.rows.values(), key=lambda r: r["path"].casefold()):
            item = QTreeWidgetItem([row["path"], REASONS.get(row["reason"], "需要检查")])
            item.setData(0, Qt.UserRole, row)
            self.table.addTopLevelItem(item)
            if row["id"] == selected:
                self.table.setCurrentItem(item)
        self.table.blockSignals(False)
        self.cursor = data.get("next") or ""
        self.more.setVisible(bool(self.cursor))
        self.message.setText(f"{len(self.rows)} 项待处理 · 本地修改会保留" if self.rows else "目前没有需要处理的同步问题。")
        self.selection_changed()
        if self.refresh_again:
            self.refresh_again = False
            self.schedule_refresh()

    def selection_changed(self):
        item = self.table.currentItem()
        row = item.data(0, Qt.UserRole) if item else None
        self.update_actions(row)
        enabled = bool(row and (row.get("can_keep_both") or row.get("can_keep_directory")) and not self.busy)
        self.keep.setEnabled(enabled)
        self.name.setEnabled(enabled)
        if not row:
            self.selected_id = None
            self.explanation.setText("选择一个文件查看处理方式。若没有列出上传阻塞项，可先请求重新核对或复制诊断。")
            return
        directory = bool(row.get("directory"))
        self.name.setPlaceholderText("本地文件夹的新名称" if directory else "本地副本的新文件名")
        self.name.setAccessibleName("本地冲突文件夹名称" if directory else "本地冲突副本文件名")
        if self.selected_id != row["id"]:
            path = PurePosixPath(row["path"])
            # Leave ample UTF-8 room for marker and extension in a Linux leaf name.
            stem = path.name if directory else path.stem
            while len(stem.encode("utf-8")) > 140:
                stem = stem[:-1]
            suffix = path.suffix if not directory and len(path.suffix.encode("utf-8")) <= 32 else ""
            self.name.setText(f"{stem} (本地冲突 {uuid4().hex[:8]}){suffix}")
        self.selected_id = row["id"]
        if row.get("can_keep_directory"):
            self.explanation.setText("整个本地文件夹将改用新名称，保留所有子文件，并在联网后上传。同名远端条目及其内容保持不变。")
        elif row.get("can_keep_both") and row.get("reason") == "creation_conflict":
            self.explanation.setText("本地文件将以新名称保留，并在联网后上传。同名远端条目保持不变，取得其元数据后会在原位置显示。")
        else:
            self.explanation.setText(
                "本地修改将成为同目录下的新文件，并在联网后上传。原文件保留远端版本；远端已删除时不会恢复原路径。"
                if row.get("can_keep_both") else REASONS.get(row["reason"], "需要进一步核对") + "。" + next_step(row)
            )

    def selected_row(self):
        item = self.table.currentItem()
        return item.data(0, Qt.UserRole) if item else None

    def update_context(self, overview):
        self.overview = overview if isinstance(overview, dict) else {}
        block = self.overview.get("upload_block")
        state = self.overview.get("connection")
        if block:
            self.context_notice.setText("上传受阻：" + str(block)[:2000] + "\n本地修改已保留。先重新核对依赖操作；若仍未恢复，请复制诊断。")
        elif self.overview.get("error") in TASK_STEPS:
            self.context_notice.setText(TASK_STEPS[self.overview["error"]])
        elif state in ("paused", "locked", "offline", "needs_attention", "stopping"):
            self.context_notice.setText({"paused": "此任务已暂停；不会自动恢复同步。", "locked": "请先解锁 / 重新登录。", "offline": "当前离线。可请求重连，已有完整本地文件仍保留。", "stopping": "挂载正在停止，请等待正常退出。", "needs_attention": "任务需要处理。请核对证书、连接设置与账号权限，再重新登录。"}[state])
        elif not isinstance(self.overview.get("recovery_actions"), dict) or self.overview["recovery_actions"].get("recheck_issues") is not True:
            self.context_notice.setText("当前后台尚未提供安全核对能力。可先刷新状态；需要更新后台时保留现有缓存和草稿，勿重新建任务。")
        else:
            self.context_notice.setText("")
        self.context_notice.setVisible(bool(self.context_notice.text()))
        self.update_actions(self.selected_row())

    def update_actions(self, row):
        actions = self.overview.get("recovery_actions")
        supported = isinstance(actions, dict) and actions.get("recheck_issues") is True
        self.recheck.setEnabled(supported and not self.busy)
        self.recheck.setToolTip("请求安全核对；不会清除缓存或盲目重复未知提交。" if supported else
                               "当前后台未提供安全核对接口。可刷新本地状态，或更新后台后再试。")
        self.open_parent.setEnabled(bool(row and parent_candidates(row.get("path")) and self.overview.get("mounted") is True and not self.busy))
        self.pause_sync.setEnabled(not self.busy and self.overview.get("connection") in ("online", "offline", "connecting", "needs_attention"))
        self.login.setEnabled(not self.busy and self.configure is not None)

    def refresh_context(self):
        if not self.isVisible() or self.context_loading:
            return
        self.context_loading = True
        def done(data, error):
            self.context_loading = False
            if not error and isinstance(data, dict):
                self.update_context(data)
            elif error:
                # A stale capability is not permission to command a replacement service.
                self.update_context({})
        self.bus.call("GetOverview", callback=done)

    def request_recheck(self):
        if not self.recheck.isEnabled() or self.busy:
            return
        self.busy = True
        self.selection_changed()
        self.message.setText("正在请求后台核对…")
        def done(data, error):
            self.busy = False
            self.selection_changed()
            if error:
                self.message.setText("未确认核对请求。请刷新状态；后台较旧时请先更新后台。缓存与草稿保持不变。")
            else:
                self.message.setText(recheck_message(data))
                self.resolved.emit()
            self.refresh_context()
        self.bus.call("RecheckIssues", callback=done)

    def pause_task(self):
        if not self.pause_sync.isEnabled() or self.busy:
            return
        self.busy = True
        self.selection_changed()
        self.message.setText("正在请求暂停此任务…")
        def done(_data, error):
            self.busy = False
            if error:
                self.message.setText("未确认暂停，仍按后台实际状态显示。请刷新后检查。")
            else:
                self.update_context(dict(self.overview, connection="paused"))
                self.message.setText("此任务已暂停，挂载、缓存和未上传修改保留。需要继续时在任务页恢复同步。")
                self.resolved.emit()
            self.selection_changed()
            self.refresh_context()
        self.bus.call("Pause", callback=done)

    def copy_diagnostic(self):
        QApplication.clipboard().setText(diagnostic_text(self.overview, self.selected_row()))
        self.message.setText("已复制诊断摘要（不含路径、账号、凭据、文件正文或版本令牌），可粘贴到问题反馈。")

    def open_available_parent(self):
        row = self.selected_row()
        if not row or not self.open_parent.isEnabled() or self.busy:
            return
        candidates = parent_candidates(row.get("path"))
        mount = self.overview.get("mount")
        if not isinstance(mount, str) or not Path(mount).is_absolute():
            self.message.setText("挂载位置尚未确认，请刷新状态。")
            return
        self.action_generation += 1
        generation = self.action_generation
        task = self.overview.get("task_id")
        self.busy = True
        self.selection_changed()
        self.message.setText("正在核对可访问的父目录…")
        def attempt():
            if generation != self.action_generation or not self.isVisible():
                self.busy = False
                self.selection_changed()
                return
            if not candidates or self.overview.get("mounted") is not True or self.overview.get("task_id") != task:
                self.busy = False
                self.selection_changed()
                self.message.setText("未确认可访问的父目录。请先恢复挂载或权限，数据保持不变。")
                return
            path = candidates.pop(0)
            def checked(data, error):
                if generation != self.action_generation or not self.isVisible():
                    self.busy = False
                    self.selection_changed()
                    return
                if error in ("service_changed", "service_unavailable") or self.overview.get("task_id") != task or self.overview.get("mounted") is not True:
                    candidates.clear()
                    attempt()
                    return
                if error and any(word in str(error).casefold() for word in ("timeout", "timedout", "noreply")):
                    self.busy = False
                    self.selection_changed()
                    self.message.setText("本地目录核对超时。未打开失效路径，请稍后重试或复制诊断。")
                    return
                valid = not error and isinstance(data, dict) and data.get("path") == path and isinstance(data.get("entries"), list)
                if valid and data.get("preview_only") is True:
                    valid = data.get("directory_state") == "received"
                if not valid:
                    QTimer.singleShot(0, attempt)
                    return
                self.busy = False
                self.selection_changed()
                ok = QDesktopServices.openUrl(QUrl.fromLocalFile(str(Path(mount) / path)))
                self.message.setText("已请求文件管理器打开父目录。" if ok else "文件管理器未接受打开请求，请检查桌面应用关联。")
            self.bus.call("GetDirectory", [path, 0], checked)
        attempt()

    def keep_both(self):
        item = self.table.currentItem()
        row = item.data(0, Qt.UserRole) if item else None
        if not row or not (row.get("can_keep_both") or row.get("can_keep_directory")) or self.busy:
            return
        name = self.name.text()
        if not name or name in (".", "..") or "/" in name or "\0" in name or len(name.encode("utf-8")) > 255:
            self.message.setText("请输入有效的新文件名（不含路径）。")
            return
        self.busy = True
        self.selection_changed()
        self.message.setText("正在保存本地副本…")
        method = "KeepDirectoryBoth" if row.get("can_keep_directory") else "KeepBoth"
        self.bus.call(method, [row["id"], row["token"], name], self.completed)

    def completed(self, data, error):
        self.busy = False
        self.selection_changed()
        if error:
            self.message.setText("未更改冲突状态。文件可能仍被占用、版本已变化或新文件名已存在，请刷新后重试。")
        else:
            self.message.setText("双方版本已保留，新副本等待同步。")
            self.resolved.emit()
            self.refresh()

    def closeEvent(self, event):
        self.timer.stop()
        super().closeEvent(event)
