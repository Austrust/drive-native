"""Issue #50 phase 4: synthetic coverage for the GUI exclusion-rule editor."""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from PySide6.QtCore import QObject, Signal, Qt
from PySide6.QtTest import QTest
from PySide6.QtWidgets import QApplication, QLabel

from drive_native_gui import config
from drive_native_gui.exclude_settings import ExcludeSettingsDialog
from drive_native_gui.window import MainWindow

APP = QApplication.instance() or QApplication([])


class Bus(QObject):
    changed = Signal()
    changed2 = Signal(object)
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ":1.synthetic"

    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.instance = instance
        self.calls = []

    def call(self, method, args=(), callback=None):
        self.calls.append((method, list(args), callback))


def profile_path():
    return config.config_path()


def read_profile():
    return json.loads(profile_path().read_text(encoding="utf-8"))


class ExcludeGuiTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {
            "XDG_CONFIG_HOME": self.tmp.name,
            "DRIVE_NATIVE_INSTANCE": "",
        })
        self.env.start()
        self.base = {
            "schema": 1,
            "task_id": "task-exclude",
            "mount": str(Path(self.tmp.name) / "mount"),
            "host": "fixture",
            "username": "synthetic",
        }

    def tearDown(self):
        self.env.stop()
        self.tmp.cleanup()

    def write_profile(self, patterns=None):
        profile = dict(self.base)
        if patterns is not None:
            profile["exclude_patterns"] = list(patterns)
        config.save_profile(profile)
        return read_profile()

    def dialog(self):
        widget = ExcludeSettingsDialog()
        widget.reload()
        return widget

    def list_texts(self, dialog):
        return [dialog.patterns_list.item(index).text()
                for index in range(dialog.patterns_list.count())]

    def make_window(self):
        patches = [
            patch.object(MainWindow, "start", lambda _: None),
            patch("drive_native_gui.window.ClientBus", Bus),
        ]
        for item in patches:
            item.start()
            self.addCleanup(item.stop)
        window = MainWindow(instance="")
        window.refresh_timer.stop()
        self.addCleanup(window.deleteLater)
        return window

    def test_reads_existing_patterns_and_a_legacy_profile_without_the_key(self):
        legacy = self.write_profile(["(^|/)node_modules/", r"\.iso$"])
        self.assertEqual(legacy["exclude_patterns"], ["(^|/)node_modules/", r"\.iso$"])
        dialog = self.dialog()
        self.assertTrue(dialog.loaded)
        self.assertEqual(self.list_texts(dialog), ["(^|/)node_modules/", r"\.iso$"])
        self.assertFalse(dialog.save_button.isEnabled())
        self.assertIn("重新启动", dialog.note.text())

        profile_path().unlink()
        legacy = self.write_profile(None)
        self.assertNotIn("exclude_patterns", legacy)
        reloaded = self.dialog()
        self.assertTrue(reloaded.loaded)
        self.assertEqual(self.list_texts(reloaded), [])

    def test_add_and_save_writes_a_rust_compatible_string_array(self):
        self.write_profile(None)
        dialog = self.dialog()
        dialog.pattern_edit.setText("(^|/)node_modules/")
        self.assertTrue(dialog.add_pattern())
        dialog.pattern_edit.setText(r"\.iso$")
        self.assertTrue(dialog.add_pattern())
        self.assertTrue(dialog.dirty)
        self.assertTrue(dialog.save_button.isEnabled())
        self.assertTrue(dialog.save())

        saved = read_profile()
        self.assertEqual(saved["exclude_patterns"], ["(^|/)node_modules/", r"\.iso$"])
        self.assertEqual(saved["task_id"], self.base["task_id"])
        self.assertEqual(saved["mount"], self.base["mount"])
        self.assertEqual(profile_path().stat().st_mode & 0o777, 0o600)
        self.assertIn("重新启动该挂载", dialog.message.text())
        self.assertIn("已保存 2 条", dialog.message.text())
        self.assertFalse(dialog.dirty)
        leftovers = [path.name for path in profile_path().parent.iterdir()
                     if path.name.startswith(".profile-")]
        self.assertEqual(leftovers, [])

    def test_invalid_regex_is_rejected_inline_without_touching_the_profile(self):
        self.write_profile([r"(^|/)\.tmp-"])
        raw = profile_path().read_bytes()
        dialog = self.dialog()
        dialog.pattern_edit.setText("[unclosed")
        self.assertFalse(dialog.add_pattern())
        self.assertIn("[unclosed", dialog.message.text())
        self.assertIn("不是合法正则", dialog.message.text())
        self.assertEqual(self.list_texts(dialog), [r"(^|/)\.tmp-"])
        self.assertEqual(profile_path().read_bytes(), raw)

        dialog.patterns_list.setCurrentRow(0)
        dialog.pattern_edit.setText("[unclosed")
        self.assertFalse(dialog.update_selected())
        self.assertIn("[unclosed", dialog.message.text())
        self.assertEqual(self.list_texts(dialog), [r"(^|/)\.tmp-"])
        self.assertEqual(profile_path().read_bytes(), raw)

    def test_update_selected_rewrites_only_the_chosen_pattern(self):
        self.write_profile([r"a\.txt$", r"b\.txt$"])
        dialog = self.dialog()
        dialog.patterns_list.setCurrentRow(1)
        dialog.pattern_edit.setText(r"(^|/)b\.txt$")
        self.assertTrue(dialog.update_selected())
        self.assertEqual(self.list_texts(dialog), [r"a\.txt$", r"(^|/)b\.txt$"])
        self.assertTrue(dialog.save())
        self.assertEqual(read_profile()["exclude_patterns"], [r"a\.txt$", r"(^|/)b\.txt$"])

    def test_delete_removes_only_the_selected_pattern(self):
        self.write_profile([r"a\.txt$", r"b\.txt$"])
        dialog = self.dialog()
        dialog.patterns_list.setCurrentRow(0)
        self.assertTrue(dialog.delete_selected())
        self.assertEqual(self.list_texts(dialog), [r"b\.txt$"])
        self.assertTrue(dialog.save())
        self.assertEqual(read_profile()["exclude_patterns"], [r"b\.txt$"])

    def test_empty_list_is_saved_and_reported_as_no_exclusions(self):
        self.write_profile([r"^\.cache$"])
        dialog = self.dialog()
        dialog.patterns_list.setCurrentRow(0)
        self.assertTrue(dialog.delete_selected())
        self.assertTrue(dialog.save())
        self.assertEqual(read_profile()["exclude_patterns"], [])
        self.assertIn("不排除任何路径", dialog.message.text())
        reloaded = self.dialog()
        self.assertEqual(self.list_texts(reloaded), [])

    def test_update_requires_an_existing_profile_and_a_matching_task(self):
        with self.assertRaises(ValueError):
            config.update_exclude_patterns(["a"], expected_task="task-exclude")
        self.write_profile([])
        with self.assertRaises(ValueError):
            config.update_exclude_patterns(["a"], expected_task="other-task")
        self.assertEqual(read_profile()["exclude_patterns"], [])
        saved = config.update_exclude_patterns(["a"], expected_task="task-exclude")
        self.assertEqual(saved, ["a"])
        self.assertEqual(read_profile()["exclude_patterns"], ["a"])

    def test_saving_patterns_never_drops_the_exclude_mode_field(self):
        # Issue #56B: the mode is additive profile state. The rule editor only
        # replaces exclude_patterns, so a local_only/hidden choice survives a
        # pattern-only save (the UI selector itself is a documented follow-up).
        self.write_profile([r"\.iso$"])
        prof = read_profile()
        prof["exclude_mode"] = "local_only"
        profile_path().write_text(json.dumps(prof), encoding="utf-8")
        config.update_exclude_patterns([r"\.iso$", r"\.tmp$"], expected_task="task-exclude")
        saved = read_profile()
        self.assertEqual(saved["exclude_mode"], "local_only")
        self.assertEqual(saved["exclude_patterns"], [r"\.iso$", r"\.tmp$"])

    def test_editor_is_keyboard_reachable_and_accessible(self):
        self.write_profile([r"\.iso$"])
        dialog = self.dialog()
        dialog.show()
        APP.processEvents()
        controls = {
            dialog.pattern_edit: "排除规则文本",
            dialog.patterns_list: "排除规则列表",
            dialog.add_button: "添加排除规则",
            dialog.update_button: "更新所选排除规则",
            dialog.delete_button: "删除所选排除规则",
            dialog.save_button: "保存排除规则",
        }
        for widget, name in controls.items():
            self.assertEqual(widget.accessibleName(), name)
            self.assertTrue(widget.focusPolicy() & Qt.FocusPolicy.TabFocus, name)
        self.assertIsInstance(dialog.message, QLabel)
        self.assertEqual(dialog.message.accessibleName(), "排除规则提示")
        self.assertTrue(dialog.message.wordWrap())

        dialog.pattern_edit.setFocus(Qt.TabFocusReason)
        APP.processEvents()
        if QApplication.focusWidget() is not dialog.pattern_edit:
            self.skipTest("offscreen platform did not grant widget focus")
        QTest.keyClick(dialog.pattern_edit, Qt.Key_Tab)
        APP.processEvents()
        self.assertIsNot(QApplication.focusWidget(), dialog.pattern_edit)
        dialog.close()

    def test_sidebar_entry_opens_the_editor_with_the_saved_patterns(self):
        self.write_profile(["(^|/)node_modules/"])
        window = self.make_window()
        button = window.exclude_settings_button
        self.assertEqual(button.text(), "排除规则")
        self.assertEqual(button.accessibleName(), "排除规则设置")
        self.assertTrue(button.focusPolicy() & Qt.FocusPolicy.TabFocus)
        window.show_exclude_settings()
        APP.processEvents()
        self.assertIsNotNone(window.exclude_dialog)
        self.assertTrue(window.exclude_dialog.isVisible())
        self.assertEqual(self.list_texts(window.exclude_dialog), ["(^|/)node_modules/"])
        window.exclude_dialog.close()


if __name__ == "__main__":
    unittest.main()
