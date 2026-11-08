"""Online does not mean local operations have confirmed server results."""
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication
from drive_native_gui.queue_health import uncertain_operations, queue_notice, queue_caption
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


class QueueTests(unittest.TestCase):
    def test_counts_mean_operations_not_connectivity_or_file_count(self):
        value = {'connection': 'online', 'queue_health': {
            'namespace_uncertain': 1, 'upload_uncertain': 2,
            'namespace_unfinished': 27, 'upload_unfinished': 4}}
        self.assertEqual(uncertain_operations(value), 3)
        self.assertEqual(queue_caption(value), '3 项提交待确认')
        self.assertIn('本地修改已保留', queue_notice(value))
        self.assertNotIn('正在', queue_notice(value))

    def test_legacy_absent_and_invalid_counts_do_not_invent_uncertainty(self):
        for health in (None, [], 'invalid', {}, {'namespace_uncertain': True},
                       {'namespace_uncertain': -1}, {'upload_uncertain': 1.5},
                       {'namespace_uncertain': 1 << 64}):
            self.assertEqual(uncertain_operations({'queue_health': health}), 0)
        self.assertEqual(queue_notice({'pending_operations': 27}), '')
        self.assertEqual(uncertain_operations(None), 0)

    def test_confirmation_clears_notice_without_assuming_other_queue_work_done(self):
        value = {'pending_operations': 26, 'queue_health': {
            'namespace_uncertain': 0, 'upload_uncertain': 0}}
        self.assertEqual(queue_notice(value), '')
        self.assertEqual(value['pending_operations'], 26)

    def test_online_window_exposes_uncertainty_and_progress_cannot_mask_it(self):
        with patch.object(MainWindow, 'start', lambda _: None), patch(
                'drive_native_gui.window.ClientBus', Bus):
            window = MainWindow()
            try:
                window.show(); app.processEvents(); window.refresh_timer.stop()
                value = dict(connection='online', mounted=True, pending_operations=27,
                             mount='/tmp/synthetic-mount', scope_path='/Lab',
                             initialization={'active': True, 'phase': 'waiting'},
                             queue_health={'namespace_uncertain': 1, 'upload_uncertain': 0})
                window.received_overview(value, None)
                self.assertEqual(window.connection_value.text(), '已连接')
                self.assertIn('尚未确认', window.banner.text())
                # The sync card now carries the pending count as a sentence; the
                # raw 27 must stay visible instead of only in the old bare number.
                self.assertEqual(window.sync_value.text(), '27 项等待同步')
                self.assertIn('待处理 27 项', window.sync_value.toolTip())
                window.render_progress({'active': True, 'phase': 'waiting'})
                self.assertEqual(window.connection_value.text(), '已连接')
                window.received_overview(dict(value, queue_health={
                    'namespace_uncertain': 0, 'upload_uncertain': 0}), None)
                self.assertTrue(window.banner.isHidden())
                self.assertFalse(any(method in ('KeepBoth', 'KeepDirectoryBoth')
                                     for method in window.bus.calls))
            finally:
                window.close(); window.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)


if __name__ == '__main__':
    unittest.main()
