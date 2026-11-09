"""Local desktop regressions: no NAS and no real user profile."""

import json, os, sys, tempfile, unittest, resource
from pathlib import Path

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
from PySide6.QtWidgets import QApplication
from PySide6.QtCore import Qt
from PySide6.QtDBus import QDBusArgument
from drive_native_gui import config
from drive_native_gui.window import MainWindow

app = QApplication([])


class DesktopTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="drive-native-gui-unit-")
        self.old = os.environ.get("XDG_CONFIG_HOME")
        os.environ["XDG_CONFIG_HOME"] = self.scratch.name

    def tearDown(self):
        if self.old is None:
            os.environ.pop("XDG_CONFIG_HOME", None)
        else:
            os.environ["XDG_CONFIG_HOME"] = self.old
        self.scratch.cleanup()

    def test_profile_is_private_exclusive_and_no_secret_field(self):
        value = {"schema": 1, "task_id": "synthetic-test", "mount": str(Path(self.scratch.name) / "mount")}
        path = config.save_profile(value)
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(config.load_profile(), value)
        with self.assertRaises(ValueError):
            config.save_profile(value)
        with self.assertRaises(ValueError):
            config.save_profile(dict(value, password="synthetic-test"))
        self.assertEqual(json.loads(path.read_text()), value)

    def test_profile_rejects_symlink_shared_mode_fifo_and_bad_schema(self):
        path = config.config_path()
        path.parent.mkdir(mode=0o700)
        target = Path(self.scratch.name) / "target"
        target.write_text('{"schema":1}')
        target.chmod(0o600)
        path.symlink_to(target)
        with self.assertRaises(OSError):
            config.load_profile()
        path.unlink()
        os.mkfifo(path, 0o600)
        with self.assertRaises(ValueError):
            config.load_profile()
        path.unlink()
        path.write_text('{"schema":1}')
        path.chmod(0o644)
        with self.assertRaises(ValueError):
            config.load_profile()
        path.chmod(0o600)
        path.write_text('{"schema":2}')
        with self.assertRaises(ValueError):
            config.load_profile()

    def test_string_array_signature_including_empty_and_unicode(self):
        for paths in [[], ["A.doc", "Pinned"], ["中文/文件"]]:
            value = QDBusArgument()
            value << paths
            self.assertEqual(value.currentSignature(), "as")

    def test_unlock_checks_task_and_owner_before_sending_credential(self):
        window = MainWindow()
        window.refresh_timer.stop()
        profile = {"task_id": "test-task", "host": "test.invalid", "username": "test"}
        window.profile = profile

        class Bus:
            owner = ":1.123"

            def __init__(self, overview, switch=False):
                self.overview, self.switch, self.calls = overview, switch, []

            def call(self, method, args=(), callback=None):
                self.calls.append(method)
                if method == "GetOverview":
                    if self.switch:
                        self.owner = ":1.124"
                    callback(self.overview, None)

        try:
            for overview, switch in [
                (dict(profile, task_id="different"), False),
                (dict(profile, host="other.invalid"), False),
                (profile, True),
            ]:
                window.bus = Bus(overview, switch)
                window.unlock_if_matching("synthetic-secret")
                self.assertEqual(window.bus.calls, ["GetOverview"])
            window.bus = Bus(profile)
            window.unlock_if_matching("synthetic-secret")
            self.assertEqual(window.bus.calls, ["GetOverview", "Unlock"])
        finally:
            window.close()
            window.deleteLater()

    def test_status_refresh_preserves_item_focus_and_selection(self):
        window = MainWindow()
        window.refresh_timer.stop()
        try:
            row = {
                "id": "stable-id",
                "path": "folder",
                "directory": True,
                "status": "online_only",
                "offline_available": False,
            }
            window.rows = [row]
            window.render_rows()
            item = window.table.topLevelItem(0)
            item.setSelected(True)
            window.table.setCurrentItem(item)
            window.rows = [dict(row, status="pinned_complete", offline_available=True, pinned=True)]
            window.render_rows()
            self.assertIs(window.table.topLevelItem(0), item)
            self.assertIs(window.table.currentItem(), item)
            self.assertTrue(item.isSelected())
            # Column 2 is local availability; pinned files report 已固定 there.
            self.assertEqual(item.text(2), "已固定")
            window.rows = [
                dict(
                    row, status="download_failed", sync_error={"attempts": 5, "server_code": 32772}
                )
            ]
            window.render_rows()
            # Column 1 is the sync-status axis; a failed download needs checking.
            self.assertEqual(item.text(1), "需要检查")
            self.assertIn("32772", item.toolTip(1))
            window.rows = [dict(row, path="renamed")]
            window.render_rows()
            self.assertIs(window.table.topLevelItem(0), item)
            self.assertEqual(item.text(0), "renamed")
            window.activate(item, 0)
            self.assertEqual(window.folder, "renamed")
        finally:
            window.close()
            window.deleteLater()


class StatusChanged2ProtocolTests(unittest.TestCase):
    """Issue #31: parse the revisioned change-set payload before consumers use it."""

    def test_valid_payload_is_normalised(self):
        from drive_native_gui.bus import parse_status_event
        event = parse_status_event(
            4, 5, 'epoch-1', False, True,
            '{"schema":1,"changes":[{"path":"/mnt/d/./Lab/x.txt","directory":false,'
            '"kind":"status"}],"truncated":false}')
        self.assertFalse(event.invalid)
        self.assertEqual((event.from_revision, event.to_revision), (4, 5))
        self.assertEqual(event.epoch, 'epoch-1')
        self.assertFalse(event.reset)
        self.assertTrue(event.overview_changed)
        self.assertFalse(event.truncated)
        self.assertEqual(len(event.changes), 1)
        self.assertEqual(event.changes[0]['path'], '/mnt/d/Lab/x.txt')
        self.assertEqual(event.changes[0]['kind'], 'status')
        self.assertFalse(event.changes[0]['directory'])

    def test_truncated_and_all_change_kinds_are_preserved(self):
        from drive_native_gui.bus import parse_status_event
        event = parse_status_event(
            1, 2, 'e', False, False,
            '{"schema":1,"changes":[{"path":"/mnt/d/dir","directory":true,'
            '"kind":"listing"},{"path":"/mnt/d/sub","directory":true,'
            '"kind":"subtree"}],"truncated":true}')
        self.assertFalse(event.invalid)
        self.assertTrue(event.truncated)
        self.assertEqual([change['kind'] for change in event.changes],
                         ['listing', 'subtree'])

    def test_invalid_payloads_are_marked_for_full_fallback(self):
        from drive_native_gui.bus import parse_status_event
        payloads = [
            '',
            'not-json',
            '{"schema":2,"changes":[]}',
            '{"schema":1,"changes":[{"path":"relative","directory":false,'
            '"kind":"status"}]}',
            '{"schema":1,"changes":[{"path":"/mnt/d/x","directory":false,'
            '"kind":"unknown"}]}',
        ]
        for payload in payloads:
            event = parse_status_event(1, 2, 'e', False, False, payload)
            self.assertTrue(event.invalid, payload)
            self.assertEqual(event.changes, [])


if __name__ == "__main__":
    unittest.main()
