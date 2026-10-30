import json, tempfile, unittest
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject
from PySide6.QtWidgets import QApplication
from drive_native_gui.trust import TrustEditor, TrustSettingsDialog
from drive_native_gui.wizard import SetupWizard

app = QApplication([])
fixtures = Path(__file__).resolve().parents[2] / "crates/sproto/tests/fixtures/trust"
CA = (fixtures / "ca.pem").read_text()
KEY = (fixtures / "ca-key.pem").read_bytes()


class Bus(QObject):
    def __init__(self):
        super().__init__()
        self.calls = []

    def call(self, method, args=(), callback=None):
        self.calls.append((method, list(args), callback))


class Tests(unittest.TestCase):
    def test_import_duplicate_private_key_and_limit_leave_existing_values(self):
        editor = TrustEditor()
        editor.set_certificates([CA])
        before = editor.values()
        for data in [KEY, b"not a cert", b"x" * 32769]:
            with self.assertRaises(ValueError):
                editor.decode(data)
        with self.assertRaises(ValueError):
            editor.set_certificates([CA, CA])
        self.assertEqual(editor.values(), before)
        editor.close()

    def test_der_import_and_selected_removal(self):
        editor = TrustEditor()
        cert = editor.decode(CA.encode())[0]
        editor.replace(editor.decode(bytes(cert.toDer())))
        self.assertEqual(editor.values(), [CA])
        editor.table.setCurrentItem(editor.table.topLevelItem(0))
        editor.remove_selected()
        self.assertEqual(editor.values(), [])
        editor.close()

    def test_save_carries_task_and_version_and_failure_keeps_user_selection(self):
        bus = Bus()
        dialog = TrustSettingsDialog(bus)
        data = {
            "schema": 1,
            "settings": {"schema": 1, "task_id": "task", "trusted_ca_pem": []},
            "version": "revision",
            "host": "127.0.0.1",
            "tls_name": "localhost",
        }
        bus.calls[-1][2](data, None)
        dialog.editor.set_certificates([CA])
        dialog.apply()
        method, args, callback = bus.calls[-1]
        self.assertEqual(method, "SetTrustSettings")
        body = json.loads(args[0])
        self.assertEqual(body["expected_version"], "revision")
        self.assertEqual(body["settings"]["task_id"], "task")
        self.assertEqual(body["settings"]["trusted_ca_pem"], [CA])
        self.assertFalse(dialog.save.isEnabled())
        callback(None, "changed")
        self.assertEqual(dialog.editor.values(), [CA])
        self.assertTrue(dialog.save.isEnabled())
        dialog.close()

    def test_reload_after_uncertain_save_uses_visible_settings_and_fresh_version(self):
        bus = Bus()
        dialog = TrustSettingsDialog(bus)
        initial = {
            "schema": 1,
            "settings": {"schema": 1, "task_id": "task", "trusted_ca_pem": [CA]},
            "version": "before-removal",
            "host": "127.0.0.1",
            "tls_name": "localhost",
        }
        bus.calls[-1][2](initial, None)
        dialog.editor.set_certificates([])
        dialog.apply()
        bus.calls[-1][2](None, "durability not confirmed")
        self.assertIn("重新读取", dialog.message.text())
        dialog.refresh.click()
        self.assertEqual(bus.calls[-1][0], "GetTrustSettings")
        visible = dict(
            initial,
            version="visible-removal",
            settings={
                "schema": 1,
                "task_id": "task",
                "trusted_ca_pem": [],
            },
        )
        bus.calls[-1][2](visible, None)
        self.assertEqual(dialog.editor.values(), [])
        dialog.save.click()
        body = json.loads(bus.calls[-1][1][0])
        self.assertEqual(body["expected_version"], "visible-removal")
        self.assertEqual(body["settings"]["trusted_ca_pem"], [])
        bus.calls[-1][2](visible, None)
        self.assertIn("已保存", dialog.message.text())
        dialog.close()

    def test_failed_load_disables_mutation(self):
        bus = Bus()
        dialog = TrustSettingsDialog(bus)
        bus.calls[-1][2](None, "offline")
        self.assertFalse(dialog.save.isEnabled())
        dialog.apply()
        self.assertEqual(len(bus.calls), 1)
        dialog.close()

    def test_wizard_persists_only_connection_that_was_authenticated(self):
        wizard = SetupWizard()
        wizard.host.setText("original.invalid")
        wizard.tls.setText("cert.original.invalid")
        wizard.username.setText("test")
        wizard.password.setText("synthetic-password")
        wizard.trusted_ca_pem = [CA]
        calls = []
        with patch.object(
            wizard.client, "request", lambda payload, callback: calls.append((payload, callback))
        ):
            wizard.advance()
        self.assertEqual(calls[0][0]["trusted_ca_pem"], [CA])
        wizard.host.setText("changed.invalid")
        wizard.tls.setText("changed.invalid")
        wizard.trusted_ca_pem = []
        calls[0][1](
            {
                "spaces": [{"name":"我的文件","view_id":1,"path":"/","personal":True,
                    "available":True,"writable":True,
                    "scope":{"view_id":1,"node_id":1,"file_id":"1","path":"/"}}],
                "warnings": [],
            },
            None,
        )
        wizard.tree.setCurrentItem(wizard.tree.topLevelItem(0).child(0))
        with tempfile.TemporaryDirectory() as directory:
            wizard.mount.setText(str(Path(directory) / "mount"))
            wizard.remember.setChecked(False)
            saved = []
            with patch(
                "drive_native_gui.wizard.save_profile", lambda profile: saved.append(profile)
            ):
                wizard.finish_setup()
            self.assertEqual(saved[0]["host"], "original.invalid")
            self.assertEqual(saved[0]["tls_name"], "cert.original.invalid")
            self.assertEqual(saved[0]["trusted_ca_pem"], [CA])
            self.assertNotIn("password", saved[0])
        wizard.close()


if __name__ == "__main__":
    unittest.main()
