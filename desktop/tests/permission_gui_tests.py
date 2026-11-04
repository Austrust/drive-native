"""Offline ACL UI fixtures. No service, network, or credential access."""
import json
import os
import unittest
from pathlib import Path
from unittest.mock import patch
from PySide6.QtWidgets import QApplication, QLabel, QDialogButtonBox
from drive_native_gui.permissions import permission_text, PermissionSettings
from drive_native_gui.wizard import SetupWizard

APP = QApplication.instance() or QApplication([])


class Bus:
    def __init__(self): self.calls = []
    def call(self, method, args=(), callback=None): self.calls.append((method, args, callback))


class PermissionTests(unittest.TestCase):
    def test_four_states_and_priority(self):
        unknown = {"local_policy": "follow_remote", "server_capability": "unknown"}
        denied = {"actions": {"create_dir": {"state": "denied"}}}
        cases = [("user", {**unknown, "local_policy": "explicit_read_only"}, denied, "用户设为只读"),
                 ("space", {**unknown, "server_capability": "space_forbidden"}, denied, "服务禁止"),
                 ("directory", unknown, denied, "当前目录权限不足"),
                 ("unknown", unknown, None, "权限待验证")]
        for name, overview, directory, expected in cases:
            text = permission_text(overview, directory)
            self.assertIn(expected, text)
            if os.environ.get("DRIVE_NATIVE_TEST_SCREENSHOTS"):
                root = Path(os.environ["DRIVE_NATIVE_TEST_SCREENSHOTS"]); root.mkdir(parents=True, exist_ok=True)
                label = QLabel(text); label.resize(680, 100); label.show(); APP.processEvents()
                self.assertTrue(label.grab().save(str(root / ("acl-" + name + ".png")))); label.close()

    def test_mixed_unknown_is_not_presented_as_verified(self):
        self.assertIn("待验证", permission_text({}, {"actions": {"create_dir": {"state": "allowed"}, "create_file": {"state": "unknown"}}}))

    def test_save_sends_explicit_choice_and_revision(self):
        bus = Bus(); dialog = PermissionSettings(bus, {"task_id": "task", "write_policy_revision": 9, "local_policy": "explicit_read_only"})
        dialog.choice.setCurrentIndex(0); dialog.save()
        method, args, callback = bus.calls[-1]
        self.assertEqual(method, "SetWritePolicy")
        self.assertEqual(json.loads(args[0]), {"task_id": "task", "expected_revision": 9, "local_policy": "follow_remote"})
        callback({"saved": True, "remount_required": True}, None)
        self.assertIn("重新启动", dialog.message.text())
        self.assertFalse(dialog.buttons.button(QDialogButtonBox.Save).isEnabled()); dialog.close()

    def test_failed_save_stays_editable(self):
        bus = Bus(); dialog = PermissionSettings(bus, {"task_id": "task", "write_policy_revision": 1})
        dialog.save(); bus.calls[-1][2](None, "revision changed")
        self.assertTrue(dialog.buttons.isEnabled()); self.assertIn("未能保存", dialog.message.text()); dialog.close()

    def test_new_wizard_requires_user_choice_for_read_only(self):
        with patch("drive_native_gui.wizard.SetupClient"):
            dialog = SetupWizard(instance="isolated")
            self.assertFalse(dialog.read_only.isChecked())
            dialog.read_only.setChecked(True); self.assertTrue(dialog.read_only.isChecked()); dialog.close()


if __name__ == "__main__": unittest.main()
