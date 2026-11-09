"""Bulk release runs as a background job with visible progress and cancel."""
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication
from drive_native_gui.window import MainWindow

app = QApplication.instance() or QApplication([])

MOUNT = '/tmp/drive-native-test-mount'


class Bus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'

    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.calls = []
        self.replies = {}

    def call(self, method, args=(), callback=None):
        self.calls.append((method, list(args), callback))
        reply = self.replies.get(method)
        if callable(reply):
            reply = reply()
        if isinstance(reply, list):
            reply = reply.pop(0) if reply else None
        if callback is not None:
            callback(reply, None)


JOB = {"schema": 1, "job_id": "7", "total": 3}
PROGRESS = {"schema": 1, "job_id": "7", "total": 3, "done": 1,
            "released_count": 1, "skipped_count": 0, "protected_count": 0,
            "reclaimed_bytes": 1024, "finished": False, "cancelled": False}
FINAL = {"schema": 1, "job_id": "7", "total": 3, "done": 3,
         "released_count": 2, "skipped_count": 1, "protected_count": 0,
         "reclaimed_bytes": 2048, "finished": True, "cancelled": False,
         "released": ["one", "two"], "skipped": ["three"],
         "protected_or_failed": []}
PROTECTED = dict(FINAL, protected_count=1, protected_or_failed=["three"])
CANCELLED = dict(FINAL, done=1, released_count=1, skipped_count=0,
                 reclaimed_bytes=1024, cancelled=True,
                 released=["one"], skipped=[], protected_or_failed=[])


class ReleaseGuiTests(unittest.TestCase):
    def setUp(self):
        self.start_patch = patch.object(MainWindow, 'start', lambda _: None)
        self.start_patch.start()
        self.bus_patch = patch('drive_native_gui.window.ClientBus', Bus)
        self.bus_patch.start()
        self.window = MainWindow()
        self.window.show()
        app.processEvents()
        self.window.refresh_timer.stop()
        self.bus = Bus()
        self.window.bus = self.bus
        self.window.overview = {"connection": "online", "mounted": True,
                                "preview_only": False}

    def tearDown(self):
        self.bus_patch.stop()
        self.start_patch.stop()
        self.window.close()
        self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def rows(self):
        return [{"path": f"{MOUNT}/folder", "directory": True,
                 "status": "local_complete"}]

    def begin(self, reply=None):
        self.bus.replies["BeginReleaseSpace"] = reply or JOB
        self.window.apply_action(self.rows(), "ReleaseSpace", None)

    def test_release_starts_a_background_job_and_shows_progress(self):
        self.begin()
        self.assertEqual(self.bus.calls[-1][0], "BeginReleaseSpace")
        self.assertEqual(self.bus.calls[-1][1], [[f"{MOUNT}/folder"]])
        self.assertFalse(self.window.release_label.isHidden())
        self.assertIn("正在释放 0/3", self.window.release_label.text())
        self.assertTrue(self.window.release_timer.isActive())

        self.bus.replies["GetReleaseJob"] = [PROGRESS, FINAL]
        self.window.poll_release_job()
        self.assertEqual(self.bus.calls[-1][0], "GetReleaseJob")
        self.assertEqual(self.bus.calls[-1][1], ["7"])
        self.assertIn("正在释放 1/3", self.window.release_label.text())
        self.assertIn("1.00 KiB", self.window.release_label.text())

        self.window.poll_release_job()
        self.assertTrue(self.window.release_label.isHidden())
        self.assertTrue(self.window.release_cancel.isHidden())
        self.assertFalse(self.window.release_timer.isActive())
        polls = [call for call in self.bus.calls if call[0] == "GetReleaseJob"]
        self.assertEqual(len(polls), 2)
        self.assertFalse(self.window.banner.isHidden())
        self.assertIn("已释放 2 个文件", self.window.banner.text())
        self.assertIn("跳过 1", self.window.banner.text())
        self.assertIn("2.00 KiB", self.window.banner.text())

    def test_release_result_reports_protected_files(self):
        self.begin()
        self.bus.replies["GetReleaseJob"] = [PROTECTED]
        self.window.poll_release_job()
        self.assertIn("已释放 2 个文件", self.window.banner.text())
        self.assertIn("暂未释放", self.window.banner.text())

    def test_cancel_requests_backend_cancel_and_reports_partial_result(self):
        self.begin()
        self.bus.replies["GetReleaseJob"] = [PROGRESS, CANCELLED]
        self.window.poll_release_job()
        self.assertIn("正在释放 1/3", self.window.release_label.text())

        self.bus.replies["CancelReleaseSpace"] = {"schema": 1, "job_id": "7",
                                                  "cancel_requested": True,
                                                  "finished": False}
        self.window.cancel_release()
        self.assertEqual(self.bus.calls[-1][0], "CancelReleaseSpace")
        self.assertEqual(self.bus.calls[-1][1], ["7"])
        self.assertFalse(self.window.release_cancel.isEnabled())
        self.assertIn("正在取消", self.window.banner.text())

        self.window.poll_release_job()
        self.assertFalse(self.window.release_timer.isActive())
        self.assertIn("已取消释放", self.window.banner.text())
        self.assertIn("1/3", self.window.banner.text())
        self.assertIn("1.00 KiB", self.window.banner.text())

    def test_stale_job_snapshot_is_ignored(self):
        self.begin()
        self.bus.replies["GetReleaseJob"] = [dict(PROGRESS, job_id="999")]
        self.window.poll_release_job()
        self.assertIn("正在释放 0/3", self.window.release_label.text())
        self.assertTrue(self.window.release_timer.isActive())

    def test_release_start_failure_is_reported_without_a_job(self):
        self.window.release_started(None, "boom")
        self.assertTrue(self.window.release_label.isHidden())
        self.assertFalse(self.window.banner.isHidden())
        self.assertIn("操作未完成", self.window.banner.text())
        self.assertFalse(self.window.release_timer.isActive())

    def test_preview_only_blocks_release_requests(self):
        self.window.overview = {"connection": "online", "mounted": True,
                                "preview_only": True}
        self.window.apply_action(self.rows(), "ReleaseSpace", None)
        self.assertFalse(any(call[0] == "BeginReleaseSpace"
                             for call in self.bus.calls))
        self.assertIn("初始化预览", self.window.banner.text())
        self.assertTrue(self.window.release_label.isHidden())


if __name__ == '__main__':
    unittest.main()
