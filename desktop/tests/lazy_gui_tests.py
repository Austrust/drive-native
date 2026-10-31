"""Lazy desktop queries with synthetic local state; no profile, NAS or service."""
import unittest
from unittest.mock import patch

from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication
from drive_native_gui.window import MainWindow

app = QApplication.instance() or QApplication([])


class Bus(QObject):
    changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'

    def __init__(self, parent=None):
        super().__init__(parent)
        self.calls = []
        self.directory_callbacks = []

    def call(self, method, args=(), callback=None):
        self.calls.append(method)
        if method == 'GetOverview':
            callback({'connection': 'online', 'pending_operations': 0,
                      'mount': '/tmp/synthetic-lazy-mount', 'mounted': True}, None)
        elif method == 'GetDirectory':
            self.directory_callbacks.append(callback)


class LazyGuiTests(unittest.TestCase):
    def setUp(self):
        self.start = patch.object(MainWindow, 'start', lambda _: None)
        self.bus_patch = patch('drive_native_gui.window.ClientBus', Bus)
        self.start.start()
        self.bus_patch.start()
        self.window = MainWindow()
        self.window.show()
        app.processEvents()
        self.window.refresh_timer.stop()
        self.window.bus.calls.clear()

    def tearDown(self):
        self.window.close()
        self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        self.bus_patch.stop()
        self.start.stop()

    def test_start_shows_overview_without_querying_directory_or_settings(self):
        self.window.refresh()
        self.assertEqual(self.window.bus.calls, ['GetOverview'])
        for name in ('transfers_dialog', 'issues_dialog', 'cache_dialog', 'trust_dialog'):
            self.assertIsNone(getattr(self.window, name))
        self.assertFalse(self.window.file_details.isVisible())

    def test_expanding_queries_and_collapsing_stops_inflight_page_chain(self):
        self.window.files_button.setChecked(True)
        self.window.refresh_timer.stop()
        self.window.refresh()
        self.assertEqual(self.window.bus.calls, ['GetOverview', 'GetDirectory'])
        self.assertTrue(self.window.file_details.isVisible())
        self.window.files_button.setChecked(False)
        self.window.bus.directory_callbacks[0](
            {'entries': [{'id': 'one', 'path': 'one'}], 'offset': 0, 'total': 1000}, None)
        self.assertEqual(self.window.bus.calls.count('GetDirectory'), 1)
        self.assertFalse(self.window.loading)
        self.assertFalse(self.window.rows)

    def test_hidden_window_does_not_query_and_reopens_with_refresh(self):
        self.window.hide()
        self.window.bus.changed.emit()
        self.window.refresh()
        self.assertEqual(self.window.bus.calls, [])
        self.assertFalse(self.window.refresh_timer.isActive())
        self.window.show()
        self.assertTrue(self.window.refresh_timer.isActive())
        self.window.refresh_timer.stop()
        self.window.refresh()
        self.assertEqual(self.window.bus.calls, ['GetOverview'])

    def test_no_query_after_close_and_no_stop_service_action(self):
        self.window.close()
        self.window.bus.changed.emit()
        app.processEvents()
        self.assertEqual(self.window.bus.calls, [])
        self.assertFalse(self.window.refresh_timer.isActive())


if __name__ == '__main__':
    unittest.main()
