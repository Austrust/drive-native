"""Lazy desktop queries with synthetic local state; no profile, NAS or service."""
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

    def __init__(self, parent=None):
        super().__init__(parent)
        self.calls = []
        self.directory_callbacks = []

    def call(self, method, args=(), callback=None):
        self.calls.append(method)
        if method == 'GetOverview':
            callback({'connection': 'online', 'pending_operations': 0,
                      'mount': '/tmp/synthetic-lazy-mount', 'mounted': True}, None)
        elif method == 'GetInitialization':
            callback({'task_id': 'synthetic', 'initialization': {'active': False}}, None)
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
        self.assertEqual(self.window.bus.calls, ['GetInitialization', 'GetOverview'])
        for name in ('transfers_dialog', 'issues_dialog', 'cache_dialog', 'trust_dialog'):
            self.assertIsNone(getattr(self.window, name))
        self.assertFalse(self.window.file_details.isVisible())

    def test_expanding_queries_and_collapsing_stops_inflight_page_chain(self):
        self.window.files_button.setChecked(True)
        self.window.refresh_timer.stop()
        self.window.refresh()
        self.assertEqual(self.window.bus.calls, ['GetInitialization', 'GetOverview', 'GetDirectory'])
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
        self.window.bus.calls.clear()
        self.window.refresh_timer.stop()
        self.window.refresh()
        self.assertEqual(self.window.bus.calls, ['GetInitialization', 'GetOverview'])

    def test_no_query_after_close_and_no_stop_service_action(self):
        self.window.close()
        self.window.bus.changed.emit()
        app.processEvents()
        self.assertEqual(self.window.bus.calls, [])
        self.assertFalse(self.window.refresh_timer.isActive())


class PagedBus(QObject):
    """In-memory bus with a 2560-entry static directory and holdable pages."""
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'
    generation = 0

    def __init__(self, parent=None):
        super().__init__(parent)
        self.calls = []       # (method, args)
        self.waiting = []     # (method, args, callback) for GetDirectory
        self.total = 2560
        self.cookie = 'ns1:0000000000000001:0'
        self.preview_only = False
        self.preview_generation = None
        self.directory_state = 'received'

    def call(self, method, args=(), callback=None):
        self.calls.append((method, list(args)))
        if method == 'GetOverview':
            callback({'connection': 'online', 'pending_operations': 0,
                      'task_id': 'task-a', 'mount': '/tmp/synthetic-page-mount',
                      'mounted': True}, None)
        elif method == 'GetInitialization':
            callback({'task_id': 'task-a', 'initialization': {'active': False}}, None)
        elif method == 'GetDirectory':
            self.waiting.append((method, list(args), callback))
        elif callback:
            callback(None, None)

    def directory_offsets(self):
        return [args[1] for method, args in self.calls if method == 'GetDirectory']

    def pending_directory(self):
        return [entry for entry in self.waiting if entry[0] == 'GetDirectory']

    def page(self, offset, path='/Lab'):
        tag = (self.cookie or 'nocookie').replace(':', '-')
        count = min(256, self.total - offset)
        return {
            'path': path, 'offset': offset, 'total': self.total,
            'entries': [{'id': '%s-%d' % (tag, offset + index),
                         'path': '%s/%s/%05d.txt' % (path, tag, offset + index),
                         'directory': False, 'status': 'local_complete',
                         'offline_available': True}
                        for index in range(count)],
            'preview_only': self.preview_only,
            'preview_generation': self.preview_generation,
            'directory_state': self.directory_state,
            'listing_cookie': self.cookie,
        }


class ListingPaginationTests(unittest.TestCase):
    """Issue #93: 加载更多 appends the next page of an unchanged listing.

    The issue derives 55 GetDirectory requests from ten screens (each click
    re-reads every prefix from offset 0, 1+2+...+10).  With a stable
    ``listing_cookie`` the same interaction must issue exactly ten requests
    with offsets 0,256,...,2304 and never fetch a page twice.
    """

    def setUp(self):
        self.start = patch.object(MainWindow, 'start', lambda _: None)
        self.bus_patch = patch('drive_native_gui.window.ClientBus', PagedBus)
        self.start.start()
        self.bus_patch.start()
        self.window = MainWindow()
        self.window.profile = {'task_id': 'task-a',
                               'mount': '/tmp/synthetic-page-mount'}
        self.window.show()
        app.processEvents()
        for timer in (self.window.refresh_timer, self.window.transfer_timer,
                      self.window.progress_timer, self.window.release_timer):
            timer.stop()
        self.window.last_overview_at = 0.0
        self.entries_served = 0
        self.renders = 0
        original = self.window.render_rows

        def counted_render():
            self.renders += 1
            original()

        self.window.render_rows = counted_render

    def tearDown(self):
        self.window.close()
        self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        self.bus_patch.stop()
        self.start.stop()

    @property
    def bus(self):
        return self.window.bus

    def settle(self):
        """Answer queued directory pages until the request chain is idle."""
        app.processEvents()
        guard = 0
        while self.bus.waiting:
            guard += 1
            if guard > 200:
                self.fail('directory request chain did not settle: %r'
                          % (self.bus.directory_offsets(),))
            method, args, callback = self.bus.waiting.pop(0)
            if method == 'GetDirectory':
                page = self.bus.page(args[1], args[0])
                self.entries_served += len(page['entries'])
                callback(page, None)
            else:
                callback(None, None)
            app.processEvents()

    def open_files(self, path='/Lab'):
        self.window.files_button.setChecked(True)
        self.window.refresh_timer.stop()   # drive the first refresh explicitly below
        self.window.navigate(path)
        self.window.refresh_timer.stop()
        self.window.last_overview_at = 0.0
        self.window.refresh()
        self.settle()

    def take_pending_directory(self):
        entry = self.bus.pending_directory()[0]
        entry = self.bus.waiting.pop(self.bus.waiting.index(entry))
        return entry

    def click_load_more(self):
        self.window.last_overview_at = 0.0
        self.window.load_more()
        app.processEvents()
        self.settle()

    # --- the issue #93 trajectory --------------------------------------
    def test_static_directory_ten_screens_use_ten_requests(self):
        self.open_files()
        self.assertEqual(self.bus.directory_offsets(), [0])
        self.assertFalse(self.window.more.isHidden())
        overview_before = sum(1 for method, _args in self.bus.calls
                              if method == 'GetOverview')
        for _screen in range(9):
            self.window.last_overview_at = 0.0
            self.window.load_more()
            app.processEvents()
            self.settle()
        expected = [0, 256, 512, 768, 1024, 1280, 1536, 1792, 2048, 2304]
        self.assertEqual(
            self.bus.directory_offsets(), expected,
            '静态目录十屏应恰好 10 次顺序追加；实际 %r，累计返回条目 %d（红测基线为 55 次/14080 项）'
            % (self.bus.directory_offsets(), self.entries_served))
        self.assertEqual(
            sum(1 for method, _args in self.bus.calls if method == 'GetOverview'),
            overview_before,
            '加载更多 must not trigger a full overview/directory refresh')
        self.assertEqual(len(self.window.rows), 2560)
        self.assertTrue(self.window.more.isHidden())
        self.assertEqual(self.window.directory_queries, 10)
        self.assertEqual(self.entries_served, 2560)
        self.assertEqual(self.renders, 10,
                         'one render per completed screen: %d' % self.renders)
        self.assertFalse(self.window._directory_loading)
        self.assertFalse(self.window.loading)

    def test_listing_cookie_change_discards_snapshot_and_rereads(self):
        self.open_files()
        self.bus.cookie = 'ns1:0000000000000002:0'
        self.window.last_overview_at = 0.0
        self.window.load_more()
        pending = self.take_pending_directory()
        self.assertEqual(pending[1][1], 256)
        pending[2](self.bus.page(256), None)   # reply reports a new listing version
        app.processEvents()
        self.assertEqual(self.bus.directory_offsets(), [0, 256, 0])
        self.settle()
        self.assertEqual(self.bus.directory_offsets(), [0, 256, 0, 256])
        self.assertEqual(len(self.window.rows), 512)
        self.assertTrue(all('0000000000000002' in row['path'] for row in self.window.rows),
                        'rows from the replaced listing must not be mixed in')
        self.assertFalse(self.window._directory_loading)

    def test_missing_listing_cookie_never_appends(self):
        self.bus.cookie = None
        self.open_files()
        self.assertIsNone(self.window.page_snapshot,
                          'an old backend without listing_cookie must not be appended to')
        self.window.last_overview_at = 0.0
        self.window.load_more()
        self.settle()
        self.assertEqual(self.bus.directory_offsets(), [0, 0, 256])
        self.assertTrue(all('nocookie' in row['path'] for row in self.window.rows))
        self.assertEqual(len(self.window.rows), 512)

    def test_repeated_clicks_merge_into_sequential_pages(self):
        self.open_files()
        for _click in range(3):
            self.window.last_overview_at = 0.0
            self.window.load_more()
        self.assertEqual(len(self.bus.pending_directory()), 1,
                         'only one request may advance the cursor at a time')
        self.assertEqual(self.bus.directory_offsets(), [0, 256])
        while self.bus.pending_directory():
            pending = self.bus.pending_directory()
            self.assertEqual(len(pending), 1,
                             'duplicate cursor requests: %r'
                             % (self.bus.directory_offsets(),))
            entry = pending[0]
            callback = self.bus.waiting.pop(self.bus.waiting.index(entry))[2]
            page = self.bus.page(entry[1][1], entry[1][0])
            self.entries_served += len(page['entries'])
            callback(page, None)
            app.processEvents()
        self.assertEqual(self.bus.directory_offsets(), [0, 256, 512, 768])
        self.assertEqual(len(self.window.rows), 1024)
        self.assertFalse(self.window._directory_loading)

    def test_switching_directory_discards_snapshot(self):
        self.open_files()
        self.click_load_more()
        self.assertEqual(self.bus.directory_offsets(), [0, 256])
        self.window.last_overview_at = 0.0
        self.window.navigate('/Other')
        self.settle()
        self.assertEqual(self.bus.directory_offsets(), [0, 256, 0])
        self.assertEqual(len(self.window.rows), 256)
        self.assertTrue(all(row['path'].startswith('/Other/') for row in self.window.rows))

    def test_owner_change_discards_snapshot(self):
        self.open_files()
        self.click_load_more()
        self.assertEqual(self.bus.directory_offsets(), [0, 256])
        self.bus.owner = ':1.replacement'
        self.bus.generation += 1
        self.click_load_more()
        self.assertEqual(self.bus.directory_offsets(), [0, 256, 0, 256, 512])
        self.assertEqual(len(self.window.rows), 768)

    def test_late_append_reply_cannot_mix_into_new_directory(self):
        self.open_files()
        self.window.last_overview_at = 0.0
        self.window.load_more()
        stale = self.take_pending_directory()[2]
        self.window.last_overview_at = 0.0
        self.window.navigate('/Other')
        app.processEvents()
        # The append is still in flight, so the overview must not start a second
        # page request for the new directory yet.
        self.assertEqual(self.bus.directory_offsets(), [0, 256])
        self.window.last_overview_at = 0.0   # let the invalidation refresh run now
        stale(self.bus.page(256), None)   # superseded reply arrives late
        app.processEvents()
        self.assertEqual(self.bus.pending_directory()[0][1], ['/Other', 0])
        self.assertTrue(self.window._directory_loading)
        # A duplicate late copy must not clear the new request's loading flag.
        stale(self.bus.page(256), None)
        app.processEvents()
        self.assertTrue(self.window._directory_loading,
                        'the late reply cleared the new request loading flag')
        self.assertEqual(self.bus.pending_directory()[0][1], ['/Other', 0])
        self.settle()
        self.assertEqual(len(self.window.rows), 256)
        self.assertTrue(all(row['path'].startswith('/Other/') for row in self.window.rows))
        self.assertFalse(self.window._directory_loading)

    def test_revoked_preview_clears_rows_and_snapshot(self):
        self.bus.preview_only = True
        self.bus.preview_generation = 'pv-gen-1'
        self.bus.cookie = 'pv1:preview-one'
        self.open_files()
        self.assertEqual(self.window.page_snapshot['cookie'], 'pv1:preview-one')
        self.window.last_overview_at = 0.0
        self.window.load_more()
        pending = self.take_pending_directory()
        self.assertEqual(pending[1][1], 256)
        self.bus.directory_state = 'denied'
        self.bus.cookie = 'pv1:denied'
        pending[2](self.bus.page(256), None)
        app.processEvents()
        self.assertEqual(self.window.rows, [])
        self.assertEqual(self.window.table.topLevelItemCount(), 0)
        self.assertIn('没有访问权限', self.window.detail.text())
        self.assertIsNone(self.window.page_snapshot)
        self.assertTrue(self.window.more.isHidden())
        # The revoked preview must not make the next click append silently.
        self.window.last_overview_at = 0.0
        self.window.load_more()
        app.processEvents()
        self.assertEqual(self.bus.pending_directory()[0][1], ['/Lab', 0])

    def test_preview_page_cannot_be_upgraded_by_append(self):
        self.bus.preview_only = True
        self.bus.preview_generation = 'pv-gen-1'
        self.bus.cookie = 'pv1:preview-one'
        self.open_files()
        self.window.last_overview_at = 0.0
        self.window.load_more()
        pending = self.take_pending_directory()
        self.bus.preview_only = False
        self.bus.preview_generation = None
        self.bus.cookie = 'ns1:0000000000000009:0'
        pending[2](self.bus.page(256), None)   # claims an authoritative view
        app.processEvents()
        self.assertEqual(self.bus.directory_offsets(), [0, 256, 0])
        self.settle()
        self.assertEqual(self.window.page_snapshot['cookie'], 'ns1:0000000000000009:0')
        self.assertTrue(all('0000000000000009' in row['path'] for row in self.window.rows))

    def test_status_patch_survives_append(self):
        self.open_files()
        first = self.window.page_rows[0]
        self.window._status_generation = self.window.view_generation
        self.window._received_status_states(
            {'entries': [{'path': first['path'],
                          'status': 'needs_attention'}]}, None)
        self.assertEqual(first['status'], 'needs_attention')
        self.assertIs(self.window.page_snapshot['rows'], self.window.page_rows)
        self.click_load_more()
        self.assertEqual(len(self.window.rows), 512)
        patched = [row for row in self.window.rows
                   if row['path'] == first['path']][0]
        self.assertEqual(patched['status'], 'needs_attention')


if __name__ == '__main__':
    unittest.main()
