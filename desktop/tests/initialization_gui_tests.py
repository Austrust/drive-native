"""Progress must remain responsive while the overview/Store is unavailable."""
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent, Qt
from PySide6.QtWidgets import QApplication
from drive_native_gui.window import MainWindow
from drive_native_gui.initialization import progress_text
from drive_native_gui.manager import MountManager

app = QApplication.instance() or QApplication([])


class Bus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'

    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.calls = []
        self.progress = {'active': True, 'phase': 'enumerating', 'units': 7, 'items': 88}

    def call(self, method, args=(), callback=None):
        self.calls.append(method)
        if method == 'GetInitialization':
            callback({'schema': 1, 'task_id': 'synthetic', 'initialization': self.progress}, None)
        # GetOverview deliberately stays pending, as during a long Store transaction.


class ProgressTests(unittest.TestCase):
    def test_preview_mode_blocks_body_actions_then_promotes_without_closing_window(self):
        with patch.object(MainWindow, 'start', lambda _: None), patch('drive_native_gui.window.ClientBus', Bus):
            window = MainWindow()
            try:
                window.show(); app.processEvents(); window.refresh_timer.stop()
                data = {'connection': 'connecting', 'mounted': True, 'pending_operations': 0,
                        'mount': '/tmp/synthetic', 'initialization': window.bus.progress, 'preview_only': True}
                window.received_overview(data, None)
                self.assertTrue(window.preview_label.isVisible())
                self.assertTrue(window.open_folder.isEnabled())
                row = {'id': 'file', 'path': 'A.doc', 'directory': False,
                       'status': 'initializing', 'offline_available': False}
                window.rows = [row]; window.render_rows()
                item = window.table.topLevelItem(0)
                self.assertEqual(item.text(2), '尚未可用')
                window.bus.calls.clear()
                with patch('drive_native_gui.window.QDesktopServices.openUrl') as opened:
                    window.activate(item, 0)
                    window.apply_action([row], 'SetPinned', True)
                    opened.assert_not_called()
                    self.assertNotIn('SetPinned', window.bus.calls)
                    window.received_overview(dict(data, connection='online', preview_only=False), None)
                    self.assertFalse(window.preview_label.isVisible())
                    window.rows = [dict(row, status='online_only')]; window.render_rows()
                    window.activate(window.table.topLevelItem(0), 0)
                    opened.assert_called_once()
                window.offline_ui()
                self.assertFalse(window.preview_label.isVisible())
            finally:
                window.close(); window.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_unready_and_changed_preview_pages_are_not_rendered_as_empty_or_complete(self):
        with patch.object(MainWindow, 'start', lambda _: None), patch('drive_native_gui.window.ClientBus', Bus):
            window = MainWindow()
            try:
                window.show(); app.processEvents(); window.refresh_timer.stop()
                window.files_button.setChecked(True)
                window.overview = {'preview_only': True}
                window.rows = [{'id': 'old', 'path': 'Old', 'directory': True, 'status': 'initializing'}]
                window.render_rows()
                for state in ('pending', 'denied'):
                    window.received_directory(window.folder, window.view_generation,
                        {'preview_only': True, 'directory_state': state, 'offset': 0, 'total': None, 'entries': []}, None)
                    self.assertEqual(window.table.topLevelItemCount(), 0)
                    self.assertEqual(window.availability_value.text(), '尚未确认')
                    self.assertNotIn('0 / 0', window.detail.text())
                window.page_rows = [{'id': 'old', 'path': 'Old'}]
                window.page_view_token = (True, 'previous-generation')
                window.received_directory(window.folder, window.view_generation,
                    {'preview_only': True, 'directory_state': 'received', 'preview_generation': 'new-generation',
                     'offset': 1, 'total': 2, 'entries': [{'id': 'new', 'path': 'New'}]}, None)
                self.assertFalse(window.page_rows)
                self.assertIn('已变化', window.detail.text())
            finally:
                window.close(); window.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_retry_reason_remains_visible_during_next_scan(self):
        title, detail = progress_text({'active': True, 'phase': 'enumerating',
            'last_retry_reason': 'parent_unavailable', 'retry_count': 1})
        self.assertIn('枚举', title)
        self.assertIn('1 次', detail)
        self.assertIn('父目录', detail)
        self.assertNotIn('已同步', detail)

    def test_watermark_and_reconciliation_are_not_history_or_completed_sync(self):
        title, detail = progress_text({'active': True, 'phase': 'watermark', 'units': 2})
        self.assertIn('当前事件水位', title)
        self.assertNotIn('历史', title)
        self.assertIn('2 次', detail)
        title, detail = progress_text({'active': True, 'phase': 'reconciling', 'units': 3})
        self.assertIn('协调', title)
        self.assertIn('3 个', detail)
        self.assertIn('尚未提交', detail)

    def test_idle_wait_is_not_reported_as_stalled_or_estimated_percent(self):
        title, detail = progress_text({'active': True, 'phase': 'waiting', 'since_progress_ms': 480000})
        self.assertIn('等待远端变更', title)
        self.assertNotIn('距上次', detail)
        self.assertNotIn('%', title + detail)
        self.assertEqual(progress_text({'active': False}), ('', ''))
        _, detail = progress_text({'active': True, 'phase': 'authenticating',
                                   'phase_elapsed_ms': 45000, 'since_progress_ms': None})
        self.assertIn('45 秒', detail)
        self.assertNotIn('上次完成', detail)

    def test_actual_counts_and_changed_without_advance_are_visible(self):
        title, detail = progress_text({'active': True, 'phase': 'catching_up',
                                      'cursor_advances': 500, 'events_applied': 1182,
                                      'consecutive_changed_without_advance': 2,
                                      'since_progress_ms': 32000})
        self.assertIn('追赶', title)
        self.assertIn('1182', detail)
        self.assertIn('连续 2 次', detail)
        self.assertIn('32 秒', detail)

    def test_progress_signal_does_not_reload_directory_or_wait_for_overview(self):
        with patch.object(MainWindow, 'start', lambda _: None), patch('drive_native_gui.window.ClientBus', Bus):
            window = MainWindow()
            window.profile = {'task_id': 'synthetic'}
            try:
                window.show(); app.processEvents(); window.refresh_timer.stop()
                window.refresh()
                self.assertTrue(window.loading)
                window.bus.calls.clear()
                window.bus.progress_changed.emit()
                self.assertEqual(window.bus.calls, ['GetInitialization'])
                self.assertIn('88', window.progress_label.text())
                self.assertTrue(window.loading)
                window.hide(); window.bus.calls.clear(); window.bus.progress_changed.emit()
                self.assertFalse(window.bus.calls)
            finally:
                window.close(); window.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_manager_uses_lightweight_progress_without_overriding_paused(self):
        profiles = [('demo', {'task_id': 'synthetic', 'mount': '/tmp/synthetic', 'scope': {'path': '/synthetic'}})]
        with patch('drive_native_gui.manager.ClientBus', Bus), patch('drive_native_gui.manager.profile_records', return_value=profiles):
            manager = MountManager()
            try:
                manager.show(); app.processEvents()
                bus = manager.buses['demo']; bus.calls.clear()
                bus.progress_changed.emit()
                self.assertEqual(bus.calls, ['GetInitialization'])
                self.assertIn('枚举', manager.items['demo'].text(1))
                manager.items['demo'].setText(1, '已暂停')
                manager.items['demo'].setData(1, Qt.UserRole, 'paused')
                bus.progress_changed.emit()
                self.assertEqual(manager.items['demo'].text(1), '已暂停')
                manager.hide(); bus.calls.clear(); bus.progress_changed.emit()
                self.assertFalse(bus.calls)
            finally:
                manager.close(); manager.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_overview_failure_keeps_confirmed_controls_and_reports_error(self):
        with patch.object(MainWindow, 'start', lambda _: None), patch('drive_native_gui.window.ClientBus', Bus):
            window = MainWindow()
            window.profile = {'task_id': 'synthetic'}
            try:
                window.show(); app.processEvents(); window.refresh_timer.stop()
                window.overview = {'connection': 'online', 'mounted': True}
                window.connection_value.setText('已连接')
                window.open_folder.setEnabled(True)
                window.pause.setEnabled(True)
                window.up.setEnabled(True)
                for error in ('org.freedesktop.DBus.Error.NoReply',
                              'org.freedesktop.DBus.Error.Timeout',
                              'org.freedesktop.DBus.Error.Failed',
                              'invalid_service_response'):
                    window.received_overview(None, error)
                    self.assertIn(error, window.banner.text())
                    self.assertTrue(window.banner.isVisible())
                    self.assertTrue(window.open_folder.isEnabled())
                    self.assertTrue(window.pause.isEnabled())
                    self.assertTrue(window.up.isEnabled())
                    self.assertTrue(window.overview['mounted'])
                    # Progress is visible, but it owns the progress area only; the
                    # confirmed connection card must stay untouched (DEF-01).
                    self.assertIn('枚举', window.progress_label.text())
                    self.assertEqual(window.connection_value.text(), '已连接')
                # A progress response cannot override a confirmed paused state.
                window.overview['connection'] = 'paused'
                window.connection_value.setText('已暂停')
                window.received_overview(None, 'org.freedesktop.DBus.Error.NoReply')
                self.assertEqual(window.connection_value.text(), '已暂停')
                # Owner loss, unlike a failed detail query, really is offline.
                window.bus.owner = ''
                window.bus.availability.emit(False)
                self.assertFalse(window.open_folder.isEnabled())
                self.assertFalse(window.pause.isEnabled())
                self.assertFalse(window.up.isEnabled())
                self.assertFalse(window.progress_label.isVisible())
                self.assertEqual(window.connection_value.text(), '后台已停止')
            finally:
                window.close(); window.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_first_overview_timeout_uses_live_progress_without_starting_service(self):
        with patch.object(MainWindow, 'start', lambda _: None), patch('drive_native_gui.window.ClientBus', Bus):
            window = MainWindow()
            window.profile = {'task_id': 'synthetic'}
            try:
                window.show(); app.processEvents(); window.refresh_timer.stop()
                window.open_folder.setEnabled(False)
                window.pause.setEnabled(False)
                with patch.object(window, 'start_service') as start:
                    window.initial_overview(None, 'org.freedesktop.DBus.Error.NoReply')
                    start.assert_not_called()
                # Live progress is shown without a successful overview, in the
                # progress area rather than by overwriting the connection card.
                self.assertIn('枚举', window.progress_label.text())
                self.assertEqual(window.connection_value.text(), '尚未启动')
                self.assertFalse(window.overview)
                self.assertFalse(window.open_folder.isEnabled())
                self.assertFalse(window.pause.isEnabled())
                self.assertIn('NoReply', window.banner.text())
                # A later successful overview clears the query error normally.
                window.received_overview({
                    'connection': 'online', 'mounted': True, 'pending_operations': 0,
                    'mount': '/tmp/synthetic', 'initialization': window.bus.progress,
                }, None)
                self.assertFalse(window.banner.isVisible())
                self.assertTrue(window.open_folder.isEnabled())
                self.assertTrue(window.pause.isEnabled())
                window.bus.owner = ''
                window.received_overview(None, 'org.freedesktop.DBus.Error.NameHasNoOwner')
                self.assertEqual(window.connection_value.text(), '尚未连接')
                self.assertFalse(window.open_folder.isEnabled())
            finally:
                window.close(); window.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)


if __name__ == '__main__':
    unittest.main()
