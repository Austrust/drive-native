"""A hidden settings page must not continue automatic queries or pagination."""
import unittest
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication
from drive_native_gui.issues import IssuesDialog
from drive_native_gui.cache_settings import CacheSettingsDialog
from drive_native_gui.transfers import TransfersDialog

app = QApplication.instance() or QApplication([])


class Bus(QObject):
    changed = Signal()
    availability = Signal(bool)

    def __init__(self):
        super().__init__()
        self.calls = []

    def call(self, method, args=(), callback=None):
        self.calls.append((method, callback))


class HiddenDialogTests(unittest.TestCase):
    def check_hidden(self, cls):
        bus = Bus()
        panel = cls(bus)
        try:
            app.processEvents()
            self.assertEqual(bus.calls, [])
            panel.show()
            app.processEvents()
            self.assertTrue(bus.calls)
            panel.hide()
            bus.calls.clear()
            bus.changed.emit()
            bus.availability.emit(True)
            panel.timer.timeout.emit()  # A queued timeout may arrive after hide.
            app.processEvents()
            self.assertFalse(panel.timer.isActive())
            self.assertEqual(bus.calls, [])
            return panel, bus
        except BaseException:
            panel.deleteLater()
            raise

    def dispose(self, panel):
        panel.close()
        panel.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_hidden_cache_settings_ignore_signals_and_queued_timeout(self):
        panel, _ = self.check_hidden(CacheSettingsDialog)
        self.dispose(panel)

    def test_hidden_transfers_do_not_query_on_service_reconnect(self):
        panel, _ = self.check_hidden(TransfersDialog)
        self.dispose(panel)

    def test_hidden_issues_stop_automatic_next_page(self):
        panel, bus = self.check_hidden(IssuesDialog)
        try:
            panel.refresh_again = True
            panel.received('', panel.generation, {'entries': [], 'next': 'second-page'}, None)
            self.assertEqual(bus.calls, [])
            self.assertFalse(panel.timer.isActive())
            panel.show()
            self.assertEqual(bus.calls[0][0], 'GetIssues')
        finally:
            self.dispose(panel)


if __name__ == '__main__':
    unittest.main()
