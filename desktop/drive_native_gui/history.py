"""Persistent history UI. An id is meaningful only together with its service."""
import json
from dataclasses import dataclass
from datetime import datetime
from pathlib import PurePosixPath
from PySide6.QtCore import Qt, QAbstractTableModel, QModelIndex, QTimer
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QComboBox, QLineEdit,
    QPushButton, QTableView, QHeaderView, QAbstractItemView, QMessageBox, QSpinBox,
)
from .history_actions import HistoryActions

ACTIONS = {"upload": "上传", "download": "下载", "remote_change": "远端变更",
           "rename": "改名", "move": "移动", "delete": "删除", "mkdir": "创建文件夹",
           "replace": "替换文件", "error": "错误"}
RESULTS = {"ok": "已完成", "failed": "失败", "cancelled": "已取消", "pending": "待处理"}


@dataclass(frozen=True)
class HistorySource:
    slot: str
    title: str
    bus: object
    task_id: str
    mount: str


def service_error(data, error):
    return error or (data.get("error") if isinstance(data, dict) else None)


def error_text(error):
    if error == "busy":
        return "后台正在提交本地状态，请稍后刷新。"
    if "UnknownMethod" in str(error):
        return "当前后台版本尚不支持同步历史，请应用后台更新。"
    return "历史暂不可用，请检查连接后刷新；其他挂载仍可使用。"


def valid_record(row):
    return (isinstance(row, dict) and type(row.get("id")) is int and row["id"] >= 0
            and type(row.get("seq")) is int and row["seq"] >= 0
            and type(row.get("completed_at")) is int
            and all(isinstance(row.get(key), str) for key in ("name", "action", "result", "path_at_event")))


class HistoryModel(QAbstractTableModel):
    COLUMNS = ("完成时间", "挂载", "名称", "操作", "结果", "尝试次数")

    def __init__(self, parent=None):
        super().__init__(parent)
        self.records = []

    def rowCount(self, parent=QModelIndex()):
        return 0 if parent.isValid() else len(self.records)

    def columnCount(self, parent=QModelIndex()):
        return 0 if parent.isValid() else len(self.COLUMNS)

    def headerData(self, section, orientation, role=Qt.DisplayRole):
        if role == Qt.DisplayRole and orientation == Qt.Horizontal and 0 <= section < len(self.COLUMNS):
            return self.COLUMNS[section]
        return None

    def data(self, index, role=Qt.DisplayRole):
        if not index.isValid() or not 0 <= index.row() < len(self.records):
            return None
        source, row = self.records[index.row()]
        if role == Qt.UserRole:
            return source, row
        if role == Qt.ToolTipRole:
            return row.get("path_at_event", "") + "\n" + str(row.get("error_summary") or "")
        if role != Qt.DisplayRole:
            return None
        try:
            completed = datetime.fromtimestamp(row["completed_at"]).strftime("%Y-%m-%d %H:%M:%S")
        except (ValueError, OverflowError, OSError):
            completed = "时间不可用"
        return (completed, source.title, row["name"], ACTIONS.get(row["action"], row["action"]),
                RESULTS.get(row["result"], row["result"]), str(row.get("attempts", 1)))[index.column()]

    def replace(self, records):
        self.beginResetModel()
        self.records = sorted(records, key=lambda entry: (
            entry[1]["completed_at"], entry[1]["seq"], entry[0].slot, entry[1]["id"]), reverse=True)
        self.endResetModel()


class HistoryPage(QWidget):
    """Independent requests, generation-guarded callbacks and per-source cursors."""
    def __init__(self, sources, parent=None, *, recent=False):
        super().__init__(parent)
        self.sources = sources
        self.recent = recent
        self.generation = 0
        self.rows = {}
        self.cursors = {}
        self.started = set()
        self.pending = set()
        self.errors = {}
        layout = QVBoxLayout(self)
        filters = QHBoxLayout()
        self.mount_filter = QComboBox(); self.mount_filter.addItem("所有挂载", None)
        self.mount_filter.setAccessibleName("按挂载筛选历史")
        self.result_filter = QComboBox()
        for label, value in (("全部结果", None), ("仅失败", "failed"), ("已完成", "ok"),
                             ("待处理", "pending"), ("已取消", "cancelled")):
            self.result_filter.addItem(label, value)
        self.name_filter = QLineEdit(); self.name_filter.setPlaceholderText("按文件名筛选历史")
        self.name_filter.setClearButtonEnabled(True)
        self.refresh_button = QPushButton("刷新")
        for widget in (self.mount_filter, self.result_filter, self.name_filter, self.refresh_button):
            filters.addWidget(widget)
        layout.addLayout(filters)
        self.model = HistoryModel(self)
        self.table = QTableView(); self.table.setModel(self.model)
        self.table.setSelectionBehavior(QAbstractItemView.SelectRows)
        self.table.setSelectionMode(QAbstractItemView.SingleSelection)
        self.table.setEditTriggers(QAbstractItemView.NoEditTriggers)
        self.table.setAlternatingRowColors(True)
        self.table.horizontalHeader().setSectionResizeMode(2, QHeaderView.Stretch)
        self.table.setColumnWidth(0, 165)
        self.table.doubleClicked.connect(lambda _index: self.open_selected())
        self.table.selectionModel().selectionChanged.connect(self.selection_changed)
        layout.addWidget(self.table, 1)
        self.detail = QLabel("选择记录查看操作、结果和尝试次数。")
        self.detail.setTextFormat(Qt.PlainText); self.detail.setWordWrap(True)
        layout.addWidget(self.detail)
        actions = QHBoxLayout()
        self.open_button = QPushButton("打开文件 / 下载并打开")
        self.locate_button = QPushButton("在文件管理器中定位")
        self.more = QPushButton("加载更多")
        self.open_button.clicked.connect(lambda: self.open_selected())
        self.locate_button.clicked.connect(lambda: self.open_selected(locate=True))
        self.more.clicked.connect(self.load_more)
        actions.addWidget(self.open_button); actions.addWidget(self.locate_button)
        actions.addStretch(); actions.addWidget(self.more)
        layout.addLayout(actions)
        self.message = QLabel(); self.message.setTextFormat(Qt.PlainText); self.message.setWordWrap(True)
        layout.addWidget(self.message)
        self.actions = HistoryActions(self.sources, self)
        self.actions.message.connect(self.message.setText)
        self.debounce = QTimer(self); self.debounce.setSingleShot(True); self.debounce.setInterval(300)
        self.debounce.timeout.connect(self.refresh)
        self.mount_filter.currentIndexChanged.connect(self.refresh)
        self.result_filter.currentIndexChanged.connect(self.refresh)
        self.name_filter.textChanged.connect(lambda _text: self.debounce.start())
        self.refresh_button.clicked.connect(self.refresh)
        if recent:
            self.result_filter.hide(); self.name_filter.hide(); self.more.hide()
        else:
            retention = QHBoxLayout()
            retention.addWidget(QLabel("历史默认保留 30 天 / 10,000 条；清理只删除历史记录。"))
            self.clear_days = QSpinBox(); self.clear_days.setRange(1, 3650); self.clear_days.setValue(30)
            self.clear_days.setSuffix(" 天前")
            older = QPushButton("清理较早历史")
            older.clicked.connect(lambda: self.clear_history("older_than_days"))
            clear_all = QPushButton("清空历史")
            clear_all.clicked.connect(lambda: self.clear_history("all"))
            for widget in (self.clear_days, older, clear_all): retention.addWidget(widget)
            layout.addLayout(retention)
        self.selection_changed()

    def current_sources(self):
        selected = self.mount_filter.currentData()
        return {key: source for key, source in self.sources().items()
                if selected is None or selected == key}

    def source_current(self, source):
        current = self.sources().get(source.slot)
        return current is not None and current.bus is source.bus and current.task_id == source.task_id

    def update_sources(self):
        selected = self.mount_filter.currentData()
        sources = self.sources()
        self.mount_filter.blockSignals(True)
        self.mount_filter.clear(); self.mount_filter.addItem("所有挂载", None)
        for key, source in sources.items(): self.mount_filter.addItem(source.title, key)
        index = self.mount_filter.findData(selected)
        self.mount_filter.setCurrentIndex(max(0, index))
        self.mount_filter.blockSignals(False)

    def showEvent(self, event):
        super().showEvent(event)
        self.refresh()

    def hideEvent(self, event):
        self.debounce.stop(); self.generation += 1; self.pending.clear()
        super().hideEvent(event)

    def refresh(self, *_):
        self.debounce.stop()
        self.generation += 1
        self.rows.clear(); self.cursors.clear(); self.started.clear(); self.pending.clear(); self.errors.clear()
        self.model.replace([])
        self.update_sources()
        if self.isVisible(): self.load_more()

    def load_more(self):
        if not self.isVisible(): return
        for source in self.current_sources().values():
            if source.slot in self.pending or (source.slot in self.started and not self.cursors.get(source.slot)):
                continue
            self.request(source)
        self.render_status()

    def request(self, source):
        generation = self.generation
        self.pending.add(source.slot)
        query = {"limit": 12 if self.recent else 100}
        if not self.recent:
            if self.result_filter.currentData(): query["result"] = self.result_filter.currentData()
            if self.name_filter.text(): query["name"] = self.name_filter.text()
            if self.cursors.get(source.slot): query["cursor"] = self.cursors[source.slot]
        def received(data, error):
            if generation != self.generation: return
            if not self.source_current(source):
                # A task can be replaced while a page is open. Invalidate every
                # cursor from the old generation and request current services;
                # never leave its slot permanently pending.
                self.refresh()
                return
            self.pending.discard(source.slot)
            error = service_error(data, error)
            if error or not isinstance(data, dict) or not isinstance(data.get("records"), list):
                self.errors[source.slot] = error_text(error)
            elif not all(valid_record(row) for row in data["records"]):
                self.errors[source.slot] = "后台返回的历史格式不可识别，请更新后台后重试。"
            else:
                self.errors.pop(source.slot, None)
                self.started.add(source.slot)
                self.cursors[source.slot] = None if self.recent else data.get("next_cursor")
                for row in data["records"]:
                    self.rows[(source.slot, row["id"])] = source, row
                self.model.replace(list(self.rows.values()))
            self.render_status()
        source.bus.call("GetRecentActivity" if self.recent else "GetSyncHistory", [json.dumps(query)], received)

    def render_status(self):
        text = f"{len(self.rows)} 条历史"
        if self.pending: text += f" · 正在读取 {len(self.pending)} 个挂载"
        if self.errors: text += "\n" + "\n".join(self.errors.values())
        self.message.setText(text)
        sources = self.current_sources()
        self.more.setEnabled(any(key not in self.pending and (key not in self.started or self.cursors.get(key)) for key in sources))

    def selected(self):
        index = self.table.currentIndex()
        return self.model.data(index, Qt.UserRole) if index.isValid() else None

    def selection_changed(self, *_):
        selected = self.selected()
        self.open_button.setEnabled(bool(selected)); self.locate_button.setEnabled(bool(selected))
        if selected:
            source, row = selected
            self.detail.setText(f"{source.title} · {row['path_at_event']}\n"
                f"{ACTIONS.get(row['action'], row['action'])} · {RESULTS.get(row['result'], row['result'])} · "
                f"尝试 {row.get('attempts', 1)} 次" + (f" · {row['error_summary']}" if row.get("error_summary") else ""))

    def open_selected(self, *, locate=False):
        selected = self.selected()
        if selected: self.actions.open(*selected, locate=locate)

    def clear_history(self, mode):
        sources = list(self.current_sources().values())
        if not sources: return
        title = "所选挂载" if self.mount_filter.currentData() is not None else "所有挂载"
        if QMessageBox.question(self, "清理同步历史", f"清理{title}的{'全部' if mode == 'all' else '较早'}历史记录？文件、草稿和缓存会保留。",
                                QMessageBox.Yes | QMessageBox.No, QMessageBox.No) != QMessageBox.Yes:
            return
        generation = self.generation
        pending = {source.slot for source in sources}; failed = []; removed = [0]
        for source in sources:
            def done(data, error, source=source):
                if generation != self.generation or not self.source_current(source): return
                pending.discard(source.slot)
                error = service_error(data, error)
                if error: failed.append(source.title + "：" + error_text(error))
                elif isinstance(data, dict) and type(data.get("removed")) is int: removed[0] += data["removed"]
                else: failed.append(source.title + "：清理结果尚未确认。")
                if not pending:
                    self.refresh()
                    self.message.setText(f"已清理 {removed[0]} 条历史，文件、草稿和缓存保留。" + ("\n" + "\n".join(failed) if failed else ""))
            query = {"mode": mode}
            if mode == "older_than_days": query["days"] = self.clear_days.value()
            source.bus.call("ClearSyncHistory", [json.dumps(query)], done)
