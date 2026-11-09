"""Window state cards and file columns stay on independent axes."""
import os
import tempfile
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, Qt, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication, QMainWindow
from drive_native_gui.window import MainWindow

app = QApplication.instance() or QApplication([])
TASK = 'a' * 32


class Bus(QObject):
    changed = Signal()
    changed2 = Signal(object)
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'

    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.calls = []
        self.calls_with_args = []
        self.waiting = []
        self.states_reply = None

    def call(self, method, args=(), callback=None):
        self.calls.append(method)
        self.calls_with_args.append((method, args))
        if method == 'GetStates' and self.states_reply is not None and callback:
            callback(self.states_reply, None)
            return
        if callback:
            self.waiting.append((method, callback))


def overview(**changes):
    data = {
        'connection': 'online',
        'mounted': True,
        'pending_operations': 0,
        'uncertain_operations': 0,
        'mount': '/tmp/synthetic',
        'scope_path': '/Lab',
    }
    data.update(changes)
    return data


class WindowStateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name})
        self.env.start()
        self.patches = [
            patch('drive_native_gui.window.ClientBus', Bus),
            patch.object(MainWindow, 'start', lambda _: None),
        ]
        for p in self.patches:
            p.start()
        self.window = MainWindow(instance='')
        self.window.profile = {'task_id': TASK, 'mount': '/tmp/synthetic'}
        self.window.show()
        app.processEvents()
        self.window.refresh_timer.stop()
        self.window.progress_timer.stop()
        self.bus = self.window.bus

    def tearDown(self):
        self.window.transfer_timer.stop()
        self.window.hide()
        self.window.close()
        self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        for p in reversed(self.patches):
            p.stop()
        self.env.stop()
        self.tmp.cleanup()

    def test_three_cards_are_independent_axes(self):
        self.window.received_overview(overview(
            initialization={'active': True, 'phase': 'catching_up'},
            cache_availability={'state': 'partial', 'total': 10, 'offline': 4, 'pinned': 1},
        ), None)
        self.assertEqual(self.window.connection_value.text(), '已连接')
        self.assertEqual(self.window.sync_value.text(), '正在检查变更')
        self.assertEqual(self.window.availability_value.text(), '部分本地可用')
        self.assertIn('可离线打开 4/10', self.window.availability_value.toolTip())
        # A progress update must never rewrite the connection card.
        self.window.render_progress({'active': True, 'phase': 'enumerating', 'units': 3})
        self.assertEqual(self.window.connection_value.text(), '已连接')
        self.assertIn('枚举', self.window.progress_label.text())

    def test_waiting_initialization_means_idle_but_pending_wins(self):
        self.window.received_overview(overview(
            initialization={'active': True, 'phase': 'waiting'},
        ), None)
        self.assertEqual(self.window.sync_value.text(), '暂无待处理变更')
        self.window.received_overview(overview(
            initialization={'active': True, 'phase': 'waiting'},
            pending_operations=27,
        ), None)
        self.assertEqual(self.window.sync_value.text(), '27 项等待同步')

    def test_unknown_initialization_never_claims_idle(self):
        self.window.received_overview(overview(), None)
        self.assertEqual(self.window.sync_value.text(), '尚未核对')
        self.assertIn('按旧后台推导', self.window.sync_value.toolTip())

    def test_backend_sync_state_is_used_when_present(self):
        self.window.received_overview(overview(
            sync_state='transferring', active_transfers=3), None)
        self.assertEqual(self.window.sync_value.text(), '正在传输 3 项')
        self.window.received_overview(overview(
            sync_state='pending', pending_operations=4, uncertain_operations=1), None)
        self.assertEqual(self.window.sync_value.text(), '5 项等待同步')
        self.window.received_overview(overview(sync_state='needs_attention'), None)
        self.assertEqual(self.window.sync_value.text(), '需要处理')

    def test_paused_uploads_are_explained_without_faking_pending_counts(self):
        self.window.received_overview(overview(
            sync_state='needs_attention', paused_uploads=2), None)
        self.assertEqual(self.window.sync_value.text(), '需要处理')
        self.assertIn('2 项上传已暂停自动重试', self.window.sync_value.toolTip())

    def test_upload_block_and_paused_are_conservative(self):
        self.window.received_overview(overview(upload_block={'reason': 'quota'}), None)
        self.assertEqual(self.window.sync_value.text(), '上传暂缓')
        self.window.received_overview(overview(connection='paused'), None)
        self.assertEqual(self.window.connection_value.text(), '未连接（同步已暂停）')
        self.assertEqual(self.window.sync_value.text(), '已暂停同步')

    def test_overview_error_keeps_last_confirmed_cards(self):
        self.window.received_overview(overview(), None)
        self.window.received_overview(
            None, 'org.freedesktop.DBus.Error.NoReply')
        self.assertEqual(self.window.connection_value.text(), '已连接')
        self.window.received_overview(None, 'org.freedesktop.DBus.Error.NoReply')
        self.assertEqual(self.window.connection_value.toolTip(), '状态可能过期（上次确认）')

    def test_file_columns_split_sync_and_availability(self):
        self.window.rows = [
            {'id': 'mixed', 'path': 'Mixed', 'directory': True,
             'status': 'mixed', 'offline_available': True},
            {'id': 'online', 'path': 'Online.doc', 'directory': False,
             'status': 'online_only', 'offline_available': False},
            {'id': 'conflict', 'path': 'Conflict.doc', 'directory': False,
             'status': 'conflict', 'offline_available': False},
            {'id': 'init', 'path': 'Init.doc', 'directory': False,
             'status': 'initializing', 'offline_available': False},
            {'id': 'broken', 'path': 'Broken.doc', 'directory': False,
             'status': 'online_only', 'error': 'boom'},
        ]
        self.window.render_rows()
        items = [self.window.table.topLevelItem(index) for index in range(
            self.window.table.topLevelItemCount())]
        by_name = {item.text(0): item for item in items}
        self.assertEqual(by_name['Mixed'].text(1), '暂无待处理变更')
        self.assertEqual(by_name['Mixed'].text(2), '部分本地可用')
        self.assertEqual(by_name['Online.doc'].text(1), '暂无待处理变更')
        self.assertEqual(by_name['Online.doc'].text(2), '仅在线')
        self.assertEqual(by_name['Conflict.doc'].text(1), '需要处理冲突')
        self.assertEqual(by_name['Init.doc'].text(2), '尚未可用')
        self.assertEqual(by_name['Broken.doc'].text(1), '状态读取失败')

    def test_directory_fallback_only_when_backend_has_no_cache_aggregate(self):
        data = {
            'preview_only': False,
            'offset': 0,
            'total': 1,
            'preview_generation': None,
            'entries': [{'id': 'file', 'path': 'A.doc', 'directory': False,
                         'status': 'local_complete', 'offline_available': True}],
        }
        # A directory reply may arrive before the first overview, before any
        # page accumulator exists: it must not crash and should expose the
        # empty fallback until an overview confirms the availability card.
        self.window.files_button.setChecked(True)
        app.processEvents()
        generation = self.window.view_generation
        empty = dict(data, total=0, entries=[])
        self.window.received_directory('', generation, empty, None)
        self.assertEqual(self.window.table.topLevelItemCount(), 0)
        self.assertIn('0 / 0 项', self.window.detail.text())
        self.assertEqual(self.window.availability_value.text(), '0 / 0')
        self.window.received_overview(overview(
            cache_availability={'state': 'partial', 'total': 10, 'offline': 4, 'pinned': 1},
        ), None)
        generation = self.window.view_generation
        self.window.received_directory('', generation, data, None)
        self.assertEqual(self.window.availability_value.text(), '部分本地可用')
        self.window.cache_from_backend = False
        self.window.received_directory('', generation, data, None)
        self.assertEqual(self.window.availability_value.text(), '1 / 1')


    def test_status_icon_is_memoized(self):
        from drive_native_gui import window as window_module
        self.assertIs(window_module.status_icon('online'), window_module.status_icon('online'))

    def test_signal_storm_coalesces_overview_queries(self):
        from PySide6.QtTest import QTest
        window = self.window
        window.loading = False
        window.refresh_again = False

        def call(method, args=(), callback=None):
            if method == 'GetOverview':
                if callback:
                    window.received_overview(overview(), None)
            elif callback:
                callback(None, 'service_unavailable')

        window.bus.call = call
        baseline = window.overview_queries
        for _ in range(12):
            window.bus.changed.emit()
            QTest.qWait(80)
        QTest.qWait(700)
        self.assertLessEqual(window.overview_queries - baseline, 2,
                             'a 12-signal burst must not issue more than one overview per second')

    def test_render_rows_reuses_status_icons_and_filter_passes(self):
        from drive_native_gui import window as window_module
        original = window_module.status_icon
        calls = []
        window_module.status_icon = lambda status: calls.append(status) or original(status)
        try:
            self.window.rows = [
                {'path': '/Lab/a.txt', 'status': 'local_complete', 'directory': False},
                {'path': '/Lab/b.txt', 'status': 'online_only', 'directory': False},
                {'path': '/Lab/sub', 'status': 'pending_upload', 'directory': True},
            ]
            self.window.render_rows()
            first_render_icons = len(calls)
            self.window.render_rows()
            self.assertEqual(len(calls), first_render_icons,
                             'identical rows must not rebuild status icons')
        finally:
            window_module.status_icon = original
        filters = []
        original_filter = self.window.apply_filter
        self.window.apply_filter = lambda: filters.append(1) or original_filter()
        self.window.render_rows()
        self.assertEqual(filters, [], 'an unchanged render must not re-run the whole-table filter')

    def test_directory_offset_zero_replaces_stale_accumulator(self):
        self.window.files_button.setChecked(True)
        app.processEvents()
        self.window.page_rows = [{'path': '/Lab/stale.txt'}]
        generation = self.window.view_generation
        entry = {'path': '/Lab/new.txt', 'status': 'local_complete', 'directory': False}
        self.window.received_directory('', generation,
                                       {'offset': 0, 'entries': [entry], 'total': 1}, None)
        self.assertEqual([row['path'] for row in self.window.page_rows], ['/Lab/new.txt'])


    # Issue #31: StatusChanged2 is consumed as a revisioned change set. A
    # status-only event patches the visible rows via one GetStates batch and
    # must not fall back to GetOverview/GetDirectory.
    def status_rows(self):
        mount = '/tmp/synthetic'
        lab = mount + '/Lab'
        rows = [
            {'path': lab + '/a.txt', 'status': 'online_only', 'directory': False,
             'offline_available': False},
            {'path': lab + '/b.txt', 'status': 'online_only', 'directory': False,
             'offline_available': False},
        ]
        self.window.folder = lab
        self.window.files_button.setChecked(True)
        app.processEvents()
        # The initial show-time refresh is still waiting for a synthetic reply;
        # clear its in-flight flags so the event path under test is reachable.
        self.window.loading = False
        self.window.progress_loading = False
        self.window._directory_loading = False
        self.window.refresh_again = False
        self.window.refresh_timer.stop()
        self.window.page_rows = [dict(row) for row in rows]
        self.window.rows = [dict(row) for row in rows]
        self.window.status2_seen = True
        self.window.status_epoch = 'epoch-1'
        self.window.status_revision = 7
        return lab

    def test_status_only_event_patches_rows_without_overview_or_directory(self):
        from drive_native_gui.bus import StatusEvent
        from PySide6.QtTest import QTest
        lab = self.status_rows()
        overview_before = self.window.overview_queries
        directory_before = self.window.directory_queries
        states_before = self.window.status_getstates
        self.bus.calls.clear()
        self.bus.calls_with_args.clear()
        self.bus.states_reply = {'schema': 1, 'entries': [{
            'path': lab + '/a.txt', 'id': 'node-a', 'directory': False,
            'status': 'local_complete', 'pinned': False,
            'offline_available': True, 'sync_error': None,
        }]}
        self.bus.changed2.emit(StatusEvent(
            7, 8, 'epoch-1', False, False,
            [{'path': lab + '/a.txt', 'directory': False, 'kind': 'status'}]))
        QTest.qWait(20)
        self.assertEqual(self.window.overview_queries, overview_before)
        self.assertEqual(self.window.directory_queries, directory_before)
        self.assertEqual(self.bus.calls.count('GetStates'), 1)
        states_call = [args for name, args in self.bus.calls_with_args if name == 'GetStates']
        self.assertEqual(list(states_call[0][0]), [lab + '/a.txt'])
        self.assertEqual(self.window.status_getstates, states_before + 1)
        patched = [row for row in self.window.rows if row['path'] == lab + '/a.txt'][0]
        self.assertEqual(patched['status'], 'local_complete')
        self.assertTrue(patched['offline_available'])
        untouched = [row for row in self.window.rows if row['path'] == lab + '/b.txt'][0]
        self.assertEqual(untouched['status'], 'online_only')

    def test_status_only_storm_coalesces_into_one_pending_batch(self):
        from drive_native_gui.bus import StatusEvent
        from PySide6.QtTest import QTest
        lab = self.status_rows()
        overview_before = self.window.overview_queries
        directory_before = self.window.directory_queries
        self.bus.calls.clear()
        self.bus.waiting.clear()
        for revision in range(7, 17):
            self.bus.changed2.emit(StatusEvent(
                revision, revision + 1, 'epoch-1', False, False,
                [{'path': lab + '/a.txt', 'directory': False, 'kind': 'status'}]))
        QTest.qWait(20)
        self.assertEqual(self.window.overview_queries, overview_before)
        self.assertEqual(self.window.directory_queries, directory_before)
        self.assertEqual(self.bus.calls.count('GetStates'), 1)
        self.assertEqual(self.window.status_getstates, 1)
        # While the batch is in flight the other nine events only queue paths;
        # the reply triggers a single follow-up query for all of them.
        in_flight = [entry for entry in self.bus.waiting if entry[0] == 'GetStates']
        self.assertEqual(len(in_flight), 1)
        in_flight[0][1]({'schema': 1, 'entries': []}, None)
        self.assertEqual(self.bus.calls.count('GetStates'), 2)
        self.assertEqual(self.window.status_getstates, 2)

    def test_status_gap_falls_back_to_overview(self):
        from drive_native_gui.bus import StatusEvent
        from PySide6.QtTest import QTest
        self.status_rows()
        self.window.last_overview_at = 0.0
        self.bus.calls.clear()
        self.bus.changed2.emit(StatusEvent(9, 10, 'epoch-1', False, False, []))
        QTest.qWait(30)
        self.assertGreaterEqual(self.bus.calls.count('GetOverview'), 1)
        self.assertEqual(self.bus.calls.count('GetStates'), 0)

    def test_overview_changed_event_uses_coalesced_overview(self):
        from drive_native_gui.bus import StatusEvent
        from PySide6.QtTest import QTest
        self.status_rows()
        self.window.last_overview_at = 0.0
        self.bus.calls.clear()
        self.bus.changed2.emit(StatusEvent(7, 8, 'epoch-1', False, True, []))
        QTest.qWait(30)
        self.assertGreaterEqual(self.bus.calls.count('GetOverview'), 1)
        self.assertEqual(self.bus.calls.count('GetStates'), 0)

    def test_membership_listing_event_reloads_current_directory(self):
        from drive_native_gui.bus import StatusEvent
        from PySide6.QtTest import QTest
        lab = self.status_rows()
        self.bus.calls.clear()
        self.bus.changed2.emit(StatusEvent(
            7, 8, 'epoch-1', False, False,
            [{'path': lab, 'directory': True, 'kind': 'listing'}]))
        QTest.qWait(400)
        self.assertIn('GetDirectory', self.bus.calls)
        self.assertGreaterEqual(self.window.membership_refreshes, 1)

    def test_legacy_changed_signal_is_ignored_once_status2_is_seen(self):
        from PySide6.QtTest import QTest
        self.window.status2_seen = True
        self.window.last_overview_at = 0.0
        self.bus.calls.clear()
        self.bus.changed.emit()
        QTest.qWait(30)
        self.assertNotIn('GetOverview', self.bus.calls)


class MaximizedOwnerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name})
        self.env.start()
        self.patches = [
            patch('drive_native_gui.window.ClientBus', Bus),
            patch.object(MainWindow, 'start', lambda _: None),
        ]
        for p in self.patches:
            p.start()
        self.host = QMainWindow()
        self.window = MainWindow(instance='')
        self.host.setCentralWidget(self.window)
        self.host.resize(800, 600)
        self.host.show()
        app.processEvents()

    def tearDown(self):
        self.window.transfer_timer.stop()
        self.host.hide()
        self.host.close()
        self.host.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        for p in reversed(self.patches):
            p.stop()
        self.env.stop()
        self.tmp.cleanup()

    def test_owner_maximized_state_is_restored_after_hide_show(self):
        self.host.showMaximized()
        app.processEvents()
        if not self.host.isMaximized():
            self.host.setWindowState(self.host.windowState() | Qt.WindowMaximized)
            app.processEvents()
        self.assertTrue(self.host.isMaximized())
        self.window.hide()
        app.processEvents()
        self.assertTrue(self.window.restore_maximized)
        self.host.showNormal()
        app.processEvents()
        self.assertFalse(self.host.isMaximized())
        self.window.show()
        for _ in range(4):
            app.processEvents()
        self.assertFalse(self.window.restore_maximized)
        self.assertTrue(self.host.isMaximized())


if __name__ == '__main__':
    unittest.main()
