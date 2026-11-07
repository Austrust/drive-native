"""Background error banner follows the backend state and hides after recovery."""
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication
from drive_native_gui.window import MainWindow

app = QApplication.instance() or QApplication([])


class Bus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'
    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.calls = []
    def call(self, method, args=(), callback=None):
        self.calls.append(method)


class BackgroundBannerTests(unittest.TestCase):
    def setUp(self):
        self.start_patch = patch.object(MainWindow, 'start', lambda _: None)
        self.start_patch.start()
        self.bus_patch = patch('drive_native_gui.window.ClientBus', Bus)
        self.bus_patch.start()
        self.window = MainWindow()
        self.window.show(); app.processEvents(); self.window.refresh_timer.stop()
        self.value = dict(connection='online', mounted=True, pending_operations=0,
                          mount='/tmp/synthetic-mount', scope_path='/Lab',
                          queue_health={'namespace_uncertain': 0, 'upload_uncertain': 0})

    def tearDown(self):
        self.bus_patch.stop()
        self.start_patch.stop()
        self.window.close(); self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_background_error_shows_banner(self):
        self.window.received_overview(dict(self.value, background_error='upload stopped; local snapshots retained'), None)
        self.assertIn('部分传输未完成', self.window.banner.text())
        self.assertFalse(self.window.banner.isHidden())

    def test_cleared_background_error_hides_banner(self):
        self.window.received_overview(dict(self.value, background_error='upload stopped; local snapshots retained'), None)
        self.assertFalse(self.window.banner.isHidden())
        self.window.received_overview(dict(self.value, background_error=None), None)
        self.assertTrue(self.window.banner.isHidden())

    def test_error_still_precedes_background_error(self):
        self.window.received_overview(dict(self.value, error='stalled', background_error='upload stopped; local snapshots retained'), None)
        self.assertFalse(self.window.banner.isHidden())
        self.assertNotIn('部分传输未完成', self.window.banner.text())

    def test_unknown_background_error_fields_do_not_break_banner(self):
        self.window.received_overview(dict(self.value, background_error='namespace commit result not confirmed; local intent retained',
                                           background_error_kind='namespace_unconfirmed', background_error_since_ms=1234567), None)
        self.assertIn('部分传输未完成', self.window.banner.text())
        self.assertFalse(self.window.banner.isHidden())
        # A kind without a message is not itself a banner trigger.
        self.window.received_overview(dict(self.value, background_error=None,
                                           background_error_kind='namespace_unconfirmed', background_error_since_ms=None), None)
        self.assertTrue(self.window.banner.isHidden())

    def test_upload_block_and_background_error_are_independent(self):
        self.window.received_overview(dict(self.value, upload_block='permission_denied', background_error=None), None)
        self.assertFalse(self.window.upload_block_label.isHidden())
        self.assertTrue(self.window.banner.isHidden())
        self.window.received_overview(dict(self.value, upload_block=None,
                                           background_error='upload stopped; local snapshots retained'), None)
        self.assertTrue(self.window.upload_block_label.isHidden())
        self.assertIn('部分传输未完成', self.window.banner.text())

    def test_download_errors_branch_still_shows_without_background_error(self):
        self.window.received_overview(dict(self.value, background_error=None, download_errors=1), None)
        self.assertIn('部分下载未完成', self.window.banner.text())
        self.assertFalse(self.window.banner.isHidden())


if __name__ == '__main__':
    unittest.main()
