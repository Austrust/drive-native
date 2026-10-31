"""Local conflict centre. Decisions carry a version token checked by the fsd."""
from pathlib import PurePosixPath
from uuid import uuid4
from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel, QTreeWidget,
    QTreeWidgetItem, QHeaderView, QLineEdit, QPushButton,
)

REASONS = {
    "content_conflict": "本地和远端都有修改",
    "remote_deleted": "远端已删除，本地修改仍保留",
    "commit_uncertain": "正在核对服务器是否已提交",
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

    def __init__(self, bus, parent=None):
        super().__init__(parent)
        self.bus = bus
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
        enabled = bool(row and (row.get("can_keep_both") or row.get("can_keep_directory")) and not self.busy)
        self.keep.setEnabled(enabled)
        self.name.setEnabled(enabled)
        if not row:
            self.selected_id = None
            self.explanation.setText("选择一个文件查看处理方式。")
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
                if row.get("can_keep_both") else REASONS.get(row["reason"], "需要进一步核对") + "。本地内容继续保留，当前不能直接解决。"
            )

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
