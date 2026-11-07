"""Persistent history UI. An id is meaningful only together with its service."""
import json
from dataclasses import dataclass
from datetime import datetime
from pathlib import PurePosixPath
from PySide6.QtCore import Qt, QAbstractTableModel, QModelIndex, QTimer
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QComboBox, QLineEdit,
    QPushButton, QTableView, QHeaderView, QAbstractItemView, QMessageBox, QSpinBox,
    QApplication,
)
from .history_actions import (
    HistoryActions, record_error_code, record_error_text, reason_text, safe_path,
)

ACTIONS = {"upload": "上传", "download": "下载", "remote_change": "远端变更",
           "rename": "改名", "move": "移动", "delete": "删除", "mkdir": "创建文件夹",
           "replace": "替换文件", "error": "错误"}
RESULTS = {"ok": "已完成", "failed": "失败", "cancelled": "已取消", "pending": "待处理"}
DEFAULT_RETENTION_DAYS = 30
DEFAULT_RETENTION_RECORDS = 10_000
RETENTION_DAY_RANGE = (1, 3650)
RETENTION_RECORD_RANGE = (1, 1_000_000)
CLEANUP_KEEPS_TEXT = "清理只删除历史记录，文件、草稿和缓存保留。"
AUTO_REFRESH_MS = 2000
DETAIL_HINT = "选择记录查看操作、结果和尝试次数。"
_SENSITIVE_DETAIL_KEYS = frozenset({
    "password", "passwd", "token", "secret", "credential", "credentials",
    "authorization", "cookie", "session", "server", "server_address",
    "address", "url", "account", "username", "user",
})


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


def retention_values(data):
    """Return (days, records) only for an in-range retention response."""
    if not isinstance(data, dict):
        return None
    days, records = data.get("days"), data.get("records")
    if type(days) is not int or type(records) is not int:
        return None
    if not RETENTION_DAY_RANGE[0] <= days <= RETENTION_DAY_RANGE[1]:
        return None
    if not RETENTION_RECORD_RANGE[0] <= records <= RETENTION_RECORD_RANGE[1]:
        return None
    return days, records


def retention_error_text(error):
    if error == "busy":
        return "后台正在提交本地状态，请稍后重试保留期限。"
    if "UnknownMethod" in str(error):
        return "当前后台版本尚不支持保留期限设置，仍可清理历史。"
    return "保留期限暂不可用，请检查连接后刷新；清理历史仍可使用。"


def timestamp_text(value):
    try:
        return datetime.fromtimestamp(value).strftime("%Y-%m-%d %H:%M:%S")
    except (ValueError, OverflowError, OSError):
        return "时间不可用"


def _clean_detail(value):
    """Drop credential-like fields from persisted detail before showing it."""
    if isinstance(value, dict):
        return {key: ("***" if str(key).lower() in _SENSITIVE_DETAIL_KEYS
                      else _clean_detail(item)) for key, item in value.items()}
    if isinstance(value, list):
        return [_clean_detail(item) for item in value]
    return value


def detail_text(source, row, resolved=None):
    """Human detail view. Raw backend tokens stay here (diagnostics), not in the table."""
    lines = [
        f"名称：{row.get('name', '')}",
        f"挂载：{source.title}",
        f"操作：{ACTIONS.get(row.get('action'), row.get('action', ''))}",
        f"结果：{RESULTS.get(row.get('result'), row.get('result', ''))}",
        f"尝试次数：{row.get('attempts', 1)}",
        f"事件路径：{row.get('path_at_event', '')}",
        f"开始时间：{timestamp_text(row.get('started_at')) if row.get('started_at') is not None else '未记录'}",
        f"完成时间：{timestamp_text(row.get('completed_at'))}",
    ]
    error = record_error_text(row)
    token = record_error_code(row)
    if error:
        lines.append(f"失败原因：{error}")
    if token:
        lines.append(f"错误代码：{token}")
    detail = row.get("detail")
    if detail is not None:
        cleaned = _clean_detail(detail)
        try:
            lines.append("详情：" + json.dumps(cleaned, ensure_ascii=False, sort_keys=True))
        except (TypeError, ValueError):
            lines.append("详情：格式不可识别")
    lines.append(f"记录：id={row.get('id')} seq={row.get('seq')}")
    lines.append(f"节点身份：{row.get('node') or '未记录'}")
    if isinstance(resolved, dict):
        path = safe_path(source.mount, resolved.get("path"))
        parent = safe_path(source.mount, resolved.get("parent_path"))
        cached = resolved.get("cached")
        if not isinstance(cached, bool):
            cached = resolved.get("offline_available") is True
        lines.extend([
            "当前位置：解析结果",
            f"· 路径：{path or '不可用'}",
            f"· 父文件夹：{parent or '不可用'}",
            f"· 存在：{'是' if resolved.get('exists') is True else '否'}",
            f"· 本地可用：{'是' if cached else '否'}",
            f"· 挂载在线：{'是' if resolved.get('mount_online') is True else '否'}",
            f"· 状态：{reason_text(resolved.get('reason')) if resolved.get('reason') else '可访问'}",
        ])
    return "\n".join(lines)


def diagnostics_text(source, row, resolved=None):
    """Copyable diagnostics; keeps raw controlled tokens, no credentials."""
    lines = [
        f"mount={source.slot} title={source.title}",
        f"record id={row.get('id')} seq={row.get('seq')} action={row.get('action')} "
        f"result={row.get('result')} attempts={row.get('attempts', 1)}",
        f"error_kind={row.get('error_kind') or ''} error_summary={row.get('error_summary') or ''}",
        f"node={row.get('node') or ''} path_at_event={row.get('path_at_event', '')}",
    ]
    if isinstance(resolved, dict):
        lines.extend([
            f"resolved exists={resolved.get('exists')} mount_online={resolved.get('mount_online')} "
            f"offline_available={resolved.get('offline_available')} cached={resolved.get('cached')} "
            f"reason={resolved.get('reason')} node={resolved.get('node')}",
            f"resolved path={resolved.get('path')} parent_path={resolved.get('parent_path')} "
            f"mount_slot={resolved.get('mount_slot')} suggested_action={resolved.get('suggested_action')}",
        ])
    return "\n".join(lines)


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
            error = record_error_text(row)
            return row.get("path_at_event", "") + ("\n" + error if error else "")
        if role != Qt.DisplayRole:
            return None
        return (timestamp_text(row["completed_at"]), source.title, row["name"],
                ACTIONS.get(row["action"], row["action"]), RESULTS.get(row["result"], row["result"]),
                str(row.get("attempts", 1)))[index.column()]

    def replace(self, records):
        self.beginResetModel()
        self.records = sorted(records, key=lambda entry: (
            entry[1]["completed_at"], entry[1]["seq"], entry[0].slot, entry[1]["id"]), reverse=True)
        self.endResetModel()


class HistoryPage(QWidget):
    """Independent requests, generation-guarded callbacks and per-source cursors."""
    def __init__(self, sources, parent=None, *, recent=False, mount_actions=None):
        super().__init__(parent)
        self.sources = sources
        self.recent = recent
        self.mount_actions = mount_actions
        self.generation = 0
        self.rows = {}
        self.cursors = {}
        self.started = set()
        self.pending = set()
        self.errors = {}
        self.retention = {}
        self.retention_errors = {}
        self.retention_unsupported = set()
        self.retention_pending = set()
        self.retention_saving = False
        self.retention_notice = ""
        self.resolve_serial = 0
        self.resolved = None
        self.action_flags = {}
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
        self.detail = QLabel(DETAIL_HINT)
        self.detail.setTextFormat(Qt.PlainText); self.detail.setWordWrap(True)
        layout.addWidget(self.detail)
        actions = QHBoxLayout()
        self.open_button = QPushButton("打开文件")
        self.open_button.setAccessibleName("打开历史记录对应的文件")
        self.locate_button = QPushButton("在文件管理器中定位")
        self.retry_button = QPushButton("重试")
        self.retry_button.setAccessibleName("重新核对文件当前位置")
        self.parent_button = QPushButton("打开父文件夹")
        self.parent_button.setAccessibleName("打开仍存在的父文件夹")
        self.diag_button = QPushButton("复制诊断")
        self.diag_button.setAccessibleName("复制历史诊断信息")
        self.detail_button = QPushButton("查看详情")
        self.detail_button.setAccessibleName("查看历史记录的详情")
        self.more = QPushButton("加载更多")
        self.open_button.clicked.connect(lambda: self.open_selected())
        self.locate_button.clicked.connect(lambda: self.open_selected(locate=True))
        self.retry_button.clicked.connect(self.retry_selected)
        self.parent_button.clicked.connect(self.open_parent_selected)
        self.diag_button.clicked.connect(self.copy_diagnostics)
        self.detail_button.clicked.connect(self.show_detail)
        self.more.clicked.connect(self.load_more)
        for widget in (self.open_button, self.locate_button, self.retry_button, self.parent_button):
            actions.addWidget(widget)
        actions.addStretch()
        for widget in (self.diag_button, self.detail_button, self.more):
            actions.addWidget(widget)
        layout.addLayout(actions)
        self.message = QLabel(); self.message.setTextFormat(Qt.PlainText); self.message.setWordWrap(True)
        layout.addWidget(self.message)
        self.actions = HistoryActions(self.sources, self, mount_actions=mount_actions)
        self.actions.message.connect(self.message.setText)
        self.debounce = QTimer(self); self.debounce.setSingleShot(True); self.debounce.setInterval(300)
        self.debounce.timeout.connect(self.refresh)
        self.auto_refresh = QTimer(self); self.auto_refresh.setInterval(AUTO_REFRESH_MS)
        self.auto_refresh.timeout.connect(self.refresh_newest)
        self.mount_filter.currentIndexChanged.connect(self.refresh)
        self.result_filter.currentIndexChanged.connect(self.refresh)
        self.name_filter.textChanged.connect(lambda _text: self.debounce.start())
        self.refresh_button.clicked.connect(self.refresh)
        if recent:
            self.result_filter.hide(); self.name_filter.hide(); self.more.hide()
            for widget in (self.retry_button, self.parent_button, self.diag_button, self.detail_button):
                widget.hide()
        else:
            retention = QHBoxLayout()
            retention.addWidget(QLabel("历史保留：最近"))
            self.retention_days = QSpinBox()
            self.retention_days.setRange(*RETENTION_DAY_RANGE)
            self.retention_days.setValue(DEFAULT_RETENTION_DAYS)
            self.retention_days.setSuffix(" 天")
            self.retention_days.setAccessibleName("历史保留天数")
            retention.addWidget(self.retention_days)
            retention.addWidget(QLabel("最多"))
            self.retention_records = QSpinBox()
            self.retention_records.setRange(*RETENTION_RECORD_RANGE)
            self.retention_records.setValue(DEFAULT_RETENTION_RECORDS)
            self.retention_records.setGroupSeparatorShown(True)
            self.retention_records.setSuffix(" 条")
            self.retention_records.setAccessibleName("历史最多保留条数")
            retention.addWidget(self.retention_records)
            self.retention_save = QPushButton("保存保留期限")
            self.retention_save.setAccessibleName("保存历史保留期限")
            self.retention_save.clicked.connect(self.save_retention)
            retention.addWidget(self.retention_save)
            retention.addStretch()
            layout.addLayout(retention)
            self.retention_message = QLabel("正在读取保留期限…")
            self.retention_message.setTextFormat(Qt.PlainText)
            self.retention_message.setWordWrap(True)
            layout.addWidget(self.retention_message)
            cleanup = QHBoxLayout()
            cleanup.addWidget(QLabel(CLEANUP_KEEPS_TEXT))
            self.clear_days = QSpinBox(); self.clear_days.setRange(1, 3650); self.clear_days.setValue(30)
            self.clear_days.setSuffix(" 天前")
            older = QPushButton("清理较早历史")
            older.clicked.connect(lambda: self.clear_history("older_than_days"))
            clear_all = QPushButton("清空历史")
            clear_all.clicked.connect(lambda: self.clear_history("all"))
            for widget in (self.clear_days, older, clear_all): cleanup.addWidget(widget)
            cleanup.addStretch()
            layout.addLayout(cleanup)
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

    def load_retention(self):
        """Read each visible mount's retention; hidden pages only clear state."""
        if self.recent: return
        self.retention.clear(); self.retention_errors.clear()
        self.retention_unsupported.clear(); self.retention_pending.clear()
        self.retention_saving = False; self.retention_notice = ""
        if self.isVisible():
            generation = self.generation
            for source in self.current_sources().values():
                self.retention_pending.add(source.slot)
                def received(data, error, source=source, generation=generation):
                    if generation != self.generation: return
                    if not self.source_current(source):
                        self.retention_pending.discard(source.slot)
                        self.retention_errors[source.slot] = "挂载身份已变化，请刷新后重试。"
                        self.render_retention(); return
                    self.retention_pending.discard(source.slot)
                    error = service_error(data, error)
                    values = retention_values(data) if not error else None
                    if error:
                        self.retention_errors[source.slot] = retention_error_text(error)
                        if "UnknownMethod" in str(error): self.retention_unsupported.add(source.slot)
                    elif values is None:
                        self.retention_errors[source.slot] = "后台返回的保留期限格式不可识别，请更新后台后重试。"
                    else:
                        self.retention[source.slot] = values
                    self.render_retention()
                source.bus.call("GetSyncHistoryRetention", [], received)
        self.render_retention()

    def render_retention(self):
        if self.recent: return
        sources = self.current_sources()
        confirmed = {slot: self.retention[slot] for slot in sources if slot in self.retention}
        errors = [self.retention_errors[slot] for slot in sources if slot in self.retention_errors]
        pending = sources.keys() & self.retention_pending
        unsupported = bool(sources) and all(slot in self.retention_unsupported for slot in sources)
        for widget in (self.retention_days, self.retention_records):
            widget.setEnabled(bool(sources) and not self.retention_saving)
        self.retention_save.setEnabled(bool(sources) and not pending
                                       and not self.retention_saving and not unsupported)
        self.retention_save.setText("正在保存…" if self.retention_saving else "保存保留期限")
        if not sources:
            self.retention_message.setText("当前没有可用挂载，无法读取或保存保留期限。")
            return
        if self.retention_saving:
            self.retention_message.setText("正在保存保留期限…")
            return
        if self.retention_notice:
            self.retention_message.setText(self.retention_notice)
            return
        values = list(confirmed.values())
        lines = []
        if pending:
            lines.append("正在读取各挂载的保留期限…")
        elif not confirmed:
            self.retention_days.setValue(DEFAULT_RETENTION_DAYS)
            self.retention_records.setValue(DEFAULT_RETENTION_RECORDS)
            lines.append("历史默认保留 30 天 / 10,000 条。")
        elif len(confirmed) < len(sources):
            if len(set(values)) == 1:
                days, records = values[0]
                lines.append(f"部分挂载读取失败；已读取的保留期限为最近 {days} 天 / {records:,} 条。")
            else:
                lines.append("各挂载保留期限不同；保存会将所选挂载统一设置为下面的数值。")
        elif len(set(values)) > 1:
            lines.append("各挂载保留期限不同；保存会将所选挂载统一设置为下面的数值。")
            for slot, (days, records) in confirmed.items():
                lines.append(f"· {sources[slot].title}：最近 {days} 天 / {records:,} 条")
        else:
            days, records = values[0]
            self.retention_days.setValue(days); self.retention_records.setValue(records)
            lines.append(f"当前保留：最近 {days} 天，最多 {records:,} 条。")
        lines.extend(errors)
        self.retention_message.setText("\n".join(lines))

    def save_retention(self):
        if self.recent or self.retention_saving: return
        sources = list(self.current_sources().values())
        if not sources: return
        days, records = self.retention_days.value(), self.retention_records.value()
        generation = self.generation
        self.retention_saving = True
        self.retention_notice = ""
        self.retention_save.setEnabled(False)
        self.retention_message.setText("正在保存保留期限…")
        pending = {source.slot for source in sources}
        saved = []; failed = []
        def finished():
            if generation != self.generation: return
            self.retention_saving = False
            if not failed:
                self.retention_notice = (f"已保存保留期限：最近 {days} 天，最多 {records:,} 条"
                                         f"（{len(saved)} 个挂载）。" + CLEANUP_KEEPS_TEXT)
            elif saved:
                self.retention_notice = (f"已在 {len(saved)} 个挂载保存保留期限；{len(failed)} 个未保存：\n"
                                         + "\n".join(failed) + "\n" + CLEANUP_KEEPS_TEXT)
            else:
                self.retention_notice = "保留期限未保存：\n" + "\n".join(failed)
            self.render_retention()
        for source in sources:
            def done(data, error, source=source):
                if generation != self.generation: return
                pending.discard(source.slot)
                if not self.source_current(source):
                    failed.append(source.title + "：挂载身份已变化，结果未确认。")
                else:
                    error = service_error(data, error)
                    values = retention_values(data) if not error else None
                    if error:
                        failed.append(source.title + "：" + retention_error_text(error))
                        if "UnknownMethod" in str(error): self.retention_unsupported.add(source.slot)
                    elif not isinstance(data, dict) or data.get("saved") is not True or values is None:
                        failed.append(source.title + "：后台未确认保存。")
                    else:
                        self.retention[source.slot] = values
                        self.retention_errors.pop(source.slot, None)
                        saved.append(source.slot)
                if not pending: finished()
            body = {"schema": 1, "task_id": source.task_id, "days": days, "records": records}
            source.bus.call("SetSyncHistoryRetention", [json.dumps(body)], done)

    def showEvent(self, event):
        super().showEvent(event)
        self.refresh()
        if not self.recent and not self.auto_refresh.isActive():
            self.auto_refresh.start()

    def hideEvent(self, event):
        self.debounce.stop(); self.generation += 1; self.pending.clear()
        self.retention_pending.clear(); self.retention_saving = False
        self.auto_refresh.stop()
        self.resolve_serial += 1; self.resolved = None
        super().hideEvent(event)

    def refresh(self, *_):
        self.debounce.stop()
        self.generation += 1
        self.resolve_serial += 1; self.resolved = None
        self.rows.clear(); self.cursors.clear(); self.started.clear(); self.pending.clear(); self.errors.clear()
        self.model.replace([])
        self.update_sources()
        self._set_buttons(False, False, False, False)
        if not self.recent: self.load_retention()
        if self.isVisible(): self.load_more()

    def refresh_newest(self, *_):
        """Refresh the newest page only; loaded pages and pagination stay intact."""
        if self.recent or not self.isVisible(): return
        for source in self.current_sources().values():
            if source.slot in self.pending: continue
            self.request(source, first_page=True)

    def load_more(self):
        if not self.isVisible(): return
        for source in self.current_sources().values():
            if source.slot in self.pending or (source.slot in self.started and not self.cursors.get(source.slot)):
                continue
            self.request(source)
        self.render_status()

    def request(self, source, first_page=False):
        generation = self.generation
        self.pending.add(source.slot)
        query = {"limit": 12 if self.recent else 100}
        if not self.recent:
            if self.result_filter.currentData(): query["result"] = self.result_filter.currentData()
            if self.name_filter.text(): query["name"] = self.name_filter.text()
            if not first_page and self.cursors.get(source.slot): query["cursor"] = self.cursors[source.slot]
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
                for row in data["records"]:
                    self.rows[(source.slot, row["id"])] = source, row
                if first_page:
                    if source.slot not in self.started:
                        self.started.add(source.slot)
                        self.cursors[source.slot] = None if self.recent else data.get("next_cursor")
                else:
                    self.started.add(source.slot)
                    self.cursors[source.slot] = None if self.recent else data.get("next_cursor")
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

    def _resolved_data(self, source, row):
        if self.resolved is None: return None
        slot, record_id, data = self.resolved
        return data if slot == source.slot and record_id == row["id"] else None

    def _set_buttons(self, open_ok, locate_ok, retry_ok, parent_ok):
        self.open_button.setEnabled(open_ok)
        self.locate_button.setEnabled(locate_ok)
        self.retry_button.setEnabled(retry_ok)
        self.parent_button.setEnabled(parent_ok)

    def _update_actions(self):
        selected = self.selected()
        if not selected:
            self.action_flags = {}
            self._set_buttons(False, False, False, False)
            return
        source, row = selected
        data = self._resolved_data(source, row)
        open_ok, locate_ok, retry_ok, parent_ok = True, True, True, False
        label, start = "打开文件", False
        if data is not None:
            path = safe_path(source.mount, data.get("path"))
            parent = safe_path(source.mount, data.get("parent_path"))
            cached = data.get("cached")
            if not isinstance(cached, bool):
                cached = data.get("offline_available") is True
            online = data.get("mount_online") is True
            exists = data.get("exists") is True
            reason = data.get("reason")
            suggested = data.get("suggested_action")
            start = reason in ("offline", "unavailable") or suggested == "start_mount"
            open_ok = bool(exists and path and (cached or online) and reason != "deleted")
            locate_ok = bool((path and cached) or parent) and reason != "permission_denied"
            parent_ok = bool(parent) and reason != "permission_denied"
            retry_ok = True
            if reason == "online_only":
                label = "下载并打开"
            elif cached:
                label = "打开文件"
        self._set_buttons(open_ok, locate_ok, retry_ok, parent_ok)
        self.open_button.setText(label)
        self.retry_button.setText("启动挂载后重试" if start else "重试")
        self.action_flags = {"open": open_ok, "locate": locate_ok, "retry": retry_ok,
                             "parent": parent_ok, "start": start}

    def _render_detail_line(self, source, row, resolved=None):
        error = record_error_text(row)
        text = (f"{source.title} · {row['path_at_event']}\n"
                f"{ACTIONS.get(row['action'], row['action'])} · {RESULTS.get(row['result'], row['result'])} · "
                f"尝试 {row.get('attempts', 1)} 次" + (f" · {error}" if error else ""))
        if isinstance(resolved, dict) and resolved.get("reason"):
            text += "\n" + reason_text(resolved.get("reason"))
        self.detail.setText(text)

    def _request_preflight(self, source, row):
        serial = self.resolve_serial
        def received(data, error):
            self._preflight_received(serial, source, row, data, error)
        self.actions.resolve_target(source, row, received)

    def _preflight_received(self, serial, source, row, data, error):
        if serial != self.resolve_serial: return
        selected = self.selected()
        if not selected or selected[0].slot != source.slot or selected[1]["id"] != row["id"]: return
        if not self.source_current(source): return
        payload = data if not error and isinstance(data, dict) and not data.get("error") else None
        self.resolved = (source.slot, row["id"], payload)
        self._update_actions()
        self._render_detail_line(source, row, payload)

    def selection_changed(self, *_):
        self.resolve_serial += 1
        self.resolved = None
        selected = self.selected()
        self.detail_button.setEnabled(bool(selected))
        self.diag_button.setEnabled(bool(selected))
        if not selected:
            self._set_buttons(False, False, False, False)
            self.detail.setText(DETAIL_HINT)
            self.action_flags = {}
            return
        source, row = selected
        self._render_detail_line(source, row)
        self._set_buttons(True, True, True, False)
        if self.isVisible() and self.source_current(source):
            self._request_preflight(source, row)

    def open_selected(self, *, locate=False):
        selected = self.selected()
        if not selected: return
        source, row = selected
        data = self._resolved_data(source, row)
        if data is not None and not locate and not self.action_flags.get("open", True):
            hint = " 可选择“打开父文件夹”查看仍可用的位置。" if self.action_flags.get("parent") else ""
            self.message.setText(reason_text(data.get("reason")) + hint)
            return
        self.actions.open(*selected, locate=locate)

    def retry_selected(self):
        selected = self.selected()
        if not selected: return
        source, row = selected
        data = self._resolved_data(source, row)
        if self.actions.start_mount(source, data):
            return
        if data is None:
            self.message.setText("正在重新核对文件当前位置…")
        self._request_preflight(source, row)

    def open_parent_selected(self):
        selected = self.selected()
        if not selected: return
        source, row = selected
        data = self._resolved_data(source, row)
        parent = safe_path(source.mount, data.get("parent_path")) if isinstance(data, dict) else None
        if parent:
            self.actions.open_url(parent)
            self.message.setText("已打开当前可用的父目录。")
        else:
            self.message.setText("该记录没有仍可用的父文件夹。")

    def copy_diagnostics(self):
        selected = self.selected()
        if not selected: return
        source, row = selected
        text = diagnostics_text(source, row, self._resolved_data(source, row))
        clipboard = QApplication.clipboard()
        if clipboard is not None:
            clipboard.setText(text)
            self.message.setText("诊断信息已复制。")
        else:
            self.message.setText("无法访问剪贴板，诊断信息未复制。")

    def show_detail(self):
        selected = self.selected()
        if not selected: return
        source, row = selected
        QMessageBox.information(self, "历史详情", detail_text(source, row, self._resolved_data(source, row)))

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
