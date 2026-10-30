import json, unittest
from PySide6.QtCore import QObject, Signal
from PySide6.QtWidgets import QApplication
from drive_native_gui.transfers import TransfersDialog

app = QApplication([])


class Bus(QObject):
    changed = Signal()
    availability = Signal(bool)

    def __init__(self):
        super().__init__()
        self.calls = []

    def call(self, method, args=(), callback=None):
        self.calls.append((method, args, callback))


class Tests(unittest.TestCase):
    def setUp(self):
        self.bus = Bus()
        self.d = TransfersDialog(self.bus)
        self.settings()

    def settings(self, task="a", version="v1"):
        self.d.settings_received(
            {
                "schema": 1,
                "task_id": task,
                "version": version,
                "limits": {"download_bps": 65536, "upload_bps": 0},
            },
            None,
        )

    def tearDown(self):
        self.d.close()
        self.d.deleteLater()
        app.processEvents()

    def progress(self, done, total, phase="transferring", task="a"):
        self.d.received(
            {
                "schema": 1,
                "task_id": task,
                "entries": [
                    {
                        "request": 1,
                        "path": "A.doc",
                        "elapsed_ms": 1000,
                        "progress": {
                            "bytes": done,
                            "total": total,
                            "direction": "upload",
                            "phase": phase,
                        },
                    }
                ],
            },
            None,
        )

    def test_full_body_does_not_mean_synchronized(self):
        self.progress(5, 5, "awaiting_confirmation")
        row = self.d.table.topLevelItem(0)
        self.assertEqual(row.text(3), "等待服务器确认")
        self.assertEqual(self.d.table.itemWidget(row, 2).value(), 1000)

    def test_empty_payload_is_visible_and_awaits_receipt(self):
        self.progress(0, 0, "awaiting_confirmation")
        self.assertEqual(self.d.table.topLevelItem(0).text(3), "等待服务器确认")

    def test_settings_keep_edits_and_bind_task_version_units(self):
        self.d.download.setValue(37)
        self.progress(1, 5)
        self.d.apply()
        name, args, _ = self.bus.calls[-1]
        self.assertEqual(name, "SetTransferSettings")
        self.assertEqual(
            json.loads(args[0]),
            {
                "schema": 1,
                "task_id": "a",
                "expected_version": "v1",
                "limits": {"download_bps": 37 * 1024, "upload_bps": 0},
            },
        )

    def test_task_change_disables_stale_save(self):
        self.progress(1, 5, task="b")
        self.assertIsNone(self.d.current)
        self.assertFalse(self.d.save.isEnabled())
        self.assertEqual(self.d.table.topLevelItemCount(), 0)

    def test_uncertain_save_requires_reload_and_service_change_disables(self):
        self.d.apply()
        self.d.saved(None, "lost_reply")
        self.assertFalse(self.d.save.isEnabled())
        self.settings()
        self.bus.availability.emit(False)
        self.assertIsNone(self.d.current)

    def test_malformed_settings_and_stale_progress_fail_closed(self):
        self.d.settings_received(
            {"schema": 1, "limits": {"download_bps": 0, "upload_bps": 0}}, None
        )
        self.assertIsNone(self.d.current)
        self.d.received(None, "disconnected")
        self.assertFalse(self.d.table.isEnabled())


if __name__ == "__main__":
    unittest.main()
