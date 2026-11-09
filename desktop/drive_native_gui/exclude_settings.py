"""Task-scoped exclusion-rule editor backed by the local profile file.

Issue #50 phase 4: the worker already understands ``exclude_patterns`` in the
profile; this dialog lets the user read and edit that array without touching a
mount.  Rule changes are picked up by the next mount start, so the UI keeps the
restart instruction visible instead of pretending they apply immediately.
"""
from PySide6.QtWidgets import (
    QAbstractItemView,
    QDialog,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QListWidget,
    QPushButton,
    QVBoxLayout,
)

from . import config


class ExcludeSettingsDialog(QDialog):
    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.instance = instance
        self.task_id = None
        self.loaded = False
        self.saving = False
        self.dirty = False
        self.setWindowTitle("排除规则 · Drive Native")
        self.resize(640, 560)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(26, 24, 26, 24)
        layout.setSpacing(16)
        title = QLabel("按任务排除路径")
        title.setObjectName("heading")
        layout.addWidget(title)
        explanation = QLabel(
            "规则是区分大小写的正则表达式，匹配相对同步目录根的 POSIX 路径"
            "（没有前导 /）。命中的文件、目录及其整个子树不会出现在挂载视图中；"
            "空列表表示不排除任何路径。"
        )
        explanation.setWordWrap(True)
        layout.addWidget(explanation)
        self.patterns_list = QListWidget()
        self.patterns_list.setAccessibleName("排除规则列表")
        self.patterns_list.setSelectionMode(
            QAbstractItemView.SelectionMode.SingleSelection
        )
        layout.addWidget(self.patterns_list, 1)

        editor = QHBoxLayout()
        self.pattern_edit = QLineEdit()
        self.pattern_edit.setPlaceholderText(r"例如 (^|/)node_modules/ 或 \.iso$")
        self.pattern_edit.setAccessibleName("排除规则文本")
        self.pattern_edit.returnPressed.connect(self.add_pattern)
        editor.addWidget(self.pattern_edit, 1)
        self.add_button = QPushButton("添加")
        self.add_button.setAccessibleName("添加排除规则")
        self.add_button.clicked.connect(self.add_pattern)
        editor.addWidget(self.add_button)
        self.update_button = QPushButton("更新所选")
        self.update_button.setAccessibleName("更新所选排除规则")
        self.update_button.clicked.connect(self.update_selected)
        editor.addWidget(self.update_button)
        self.delete_button = QPushButton("删除所选")
        self.delete_button.setAccessibleName("删除所选排除规则")
        self.delete_button.clicked.connect(self.delete_selected)
        editor.addWidget(self.delete_button)
        layout.addLayout(editor)

        self.note = QLabel(
            "保存后需要停止并重新启动该挂载才会完全生效；"
            "已被排除的本地数据由后台清理规则处理。"
        )
        self.note.setWordWrap(True)
        layout.addWidget(self.note)
        self.message = QLabel()
        self.message.setAccessibleName("排除规则提示")
        self.message.setWordWrap(True)
        layout.addWidget(self.message)

        buttons = QHBoxLayout()
        self.save_button = QPushButton("保存规则")
        self.save_button.setAccessibleName("保存排除规则")
        self.save_button.clicked.connect(self.save)
        buttons.addStretch()
        buttons.addWidget(self.save_button)
        self.close_button = QPushButton("关闭")
        self.close_button.setAccessibleName("关闭排除规则设置")
        self.close_button.clicked.connect(self.close)
        buttons.addWidget(self.close_button)
        layout.addLayout(buttons)
        self.update_controls()

    # -- profile state -----------------------------------------------------
    def reload(self):
        if self.saving:
            return
        try:
            profile = config.load_profile(self.instance)
        except (OSError, ValueError) as error:
            profile = None
            self.message.setText(f"无法读取同步配置：{error}")
        if profile is None:
            self.task_id = None
            self.patterns_list.clear()
            self.loaded = False
            self.dirty = False
            if not self.message.text():
                self.message.setText("未找到同步配置，请先完成连接设置。")
            self.update_controls()
            return
        self.task_id = profile.get("task_id")
        patterns = profile.get("exclude_patterns")
        if not isinstance(patterns, list):
            patterns = []
        self.patterns_list.clear()
        for pattern in patterns:
            if isinstance(pattern, str) and pattern:
                self.patterns_list.addItem(pattern)
        self.loaded = True
        self.dirty = False
        self.message.setText(
            "当前规则会在下次启动挂载时生效。保存后请停止并重新启动该挂载。"
            if patterns else "当前没有排除规则（不排除任何路径）。"
        )
        self.update_controls()

    def showEvent(self, event):
        super().showEvent(event)
        self.reload()

    def current_patterns(self):
        return [
            self.patterns_list.item(index).text()
            for index in range(self.patterns_list.count())
        ]

    # -- editing -----------------------------------------------------------
    def _validated(self, text):
        try:
            return config.normalize_exclude_patterns([text])[0]
        except ValueError as error:
            self.message.setText(f"规则未添加：{error}")
            return None

    def add_pattern(self):
        if not self.loaded or self.saving:
            return False
        pattern = self._validated(self.pattern_edit.text())
        if pattern is None:
            return False
        self.patterns_list.addItem(pattern)
        self.pattern_edit.clear()
        self.dirty = True
        self.message.setText("规则已添加，尚未保存；保存后需要重新启动挂载。")
        self.update_controls()
        return True

    def update_selected(self):
        if not self.loaded or self.saving:
            return False
        row = self.patterns_list.currentRow()
        if row < 0:
            self.message.setText("请先选择要更新的规则。")
            return False
        pattern = self._validated(self.pattern_edit.text())
        if pattern is None:
            return False
        self.patterns_list.item(row).setText(pattern)
        self.pattern_edit.clear()
        self.dirty = True
        self.message.setText("规则已更新，尚未保存；保存后需要重新启动挂载。")
        self.update_controls()
        return True

    def delete_selected(self):
        if not self.loaded or self.saving:
            return False
        row = self.patterns_list.currentRow()
        if row < 0:
            self.message.setText("请先选择要删除的规则。")
            return False
        self.patterns_list.takeItem(row)
        self.dirty = True
        self.message.setText("规则已删除，尚未保存；保存后需要重新启动挂载。")
        self.update_controls()
        return True

    def update_controls(self):
        editable = self.loaded and not self.saving
        for widget in (
            self.patterns_list,
            self.pattern_edit,
            self.add_button,
            self.update_button,
            self.delete_button,
        ):
            widget.setEnabled(editable)
        self.save_button.setEnabled(editable and self.dirty)
        self.close_button.setEnabled(not self.saving)

    # -- persistence -------------------------------------------------------
    def save(self):
        if not self.loaded or self.saving:
            return False
        patterns = self.current_patterns()
        self.saving = True
        self.update_controls()
        try:
            saved = config.update_exclude_patterns(
                patterns, self.instance, expected_task=self.task_id
            )
        except (OSError, ValueError) as error:
            self.message.setText(f"规则未保存：{error}")
            return False
        finally:
            self.saving = False
        self.dirty = False
        if saved:
            self.message.setText(
                f"已保存 {len(saved)} 条排除规则；停止并重新启动该挂载后完全生效。"
            )
        else:
            self.message.setText(
                "已保存 0 条排除规则：不排除任何路径；"
                "重新启动该挂载后规则变更生效。"
            )
        self.update_controls()
        return True
