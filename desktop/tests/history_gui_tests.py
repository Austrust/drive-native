"""Synthetic history contracts; helper tests read isolated temporary files only."""
import json
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject, QCoreApplication, QEvent, Qt
from PySide6.QtWidgets import QApplication, QMessageBox
from drive_native_gui.history import HistoryPage, HistorySource, HistoryModel, valid_record
from drive_native_gui.history_actions import HistoryActions, safe_path
from drive_native_gui.bus import ClientBus

APP = QApplication.instance() or QApplication([])


class Bus(QObject):
    def __init__(self):
        super().__init__(); self.calls = []
    def call(self, method, args=(), callback=None):
        self.calls.append((method, args, callback))
    def last(self, method):
        return [entry for entry in self.calls if entry[0] == method][-1]


def record(id=1, **values):
    return dict(id=id, seq=id, completed_at=1793750400, name='file.txt',
                path_at_event='old/file.txt', action='upload', result='ok', attempts=1, **values)


class HistoryTests(unittest.TestCase):
    def setUp(self):
        self.a, self.b = Bus(), Bus()
        self.sources = {key: HistorySource(key, key.upper(), bus, 'task-' + key, '/mnt/' + key)
                        for key, bus in [('a', self.a), ('b', self.b)]}
        self.page = HistoryPage(lambda: self.sources); self.page.show(); APP.processEvents()
    def tearDown(self):
        self.page.hide(); self.page.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def reply(self, bus, rows, cursor=None):
        bus.last('GetSyncHistory')[2]({'records': rows, 'next_cursor': cursor}, None)

    def test_service_ids_are_scoped_and_busy_task_does_not_block_other(self):
        self.reply(self.a, [record()], '1')
        self.reply(self.b, [record()])
        self.assertEqual(self.page.model.rowCount(), 2)
        self.assertEqual({source.slot for source, _ in self.page.model.records}, {'a', 'b'})
        self.page.load_more()
        self.assertEqual(json.loads(self.a.last('GetSyncHistory')[1][0])['cursor'], '1')
        self.assertEqual(len(self.b.calls), 1)
        self.a.last('GetSyncHistory')[2]({'error': 'busy'}, None)
        self.assertIn('稍后刷新', self.page.message.text())
        self.assertEqual(self.page.model.rowCount(), 2)

    def test_filters_start_new_cursor_and_stale_reply_is_ignored(self):
        old = self.a.last('GetSyncHistory')[2]
        self.page.result_filter.setCurrentIndex(1)
        query = json.loads(self.a.last('GetSyncHistory')[1][0])
        self.assertEqual(query['result'], 'failed'); self.assertNotIn('cursor', query)
        old({'records': [record(9)], 'next_cursor': None}, None)
        self.assertEqual(self.page.model.rowCount(), 0)
        self.page.name_filter.setText('literal%_')
        self.page.refresh()
        self.assertEqual(json.loads(self.a.last('GetSyncHistory')[1][0])['name'], 'literal%_')

    def test_mount_filter_only_calls_selected_service(self):
        a, b = len(self.a.calls), len(self.b.calls)
        self.page.mount_filter.setCurrentIndex(self.page.mount_filter.findData('b'))
        self.assertEqual(len(self.a.calls), a)
        self.assertEqual(len(self.b.calls), b + 1)

    def test_hidden_page_rejects_inflight_response_and_makes_no_new_query(self):
        reply = self.a.last('GetSyncHistory')[2]
        self.page.hide()
        reply({'records': [record()], 'next_cursor': '1'}, None)
        self.page.load_more()
        self.assertEqual(self.page.model.rowCount(), 0)
        self.assertEqual(len(self.a.calls), 1)

    def test_replaced_profile_rejects_old_service_record(self):
        reply = self.a.last('GetSyncHistory')[2]
        replacement_bus = Bus()
        self.sources['a'] = HistorySource('a', 'A', replacement_bus, 'replacement', '/mnt/a')
        reply({'records': [record()], 'next_cursor': None}, None)
        self.assertEqual(self.page.model.rowCount(), 0)
        self.assertEqual(len(replacement_bus.calls), 1)
        self.reply(replacement_bus, [record(2)])
        self.assertNotIn('a', self.page.pending)
        self.assertEqual([row['id'] for _, row in self.page.model.records], [2])

    def test_clear_all_has_no_days_and_no_file_mutation_calls(self):
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.page.clear_history('all')
        for bus in (self.a, self.b):
            method, args, callback = bus.last('ClearSyncHistory')
            self.assertEqual(json.loads(args[0]), {'mode': 'all'})
            callback({'removed': 1}, None)
            self.assertTrue(all(method in ('GetSyncHistory', 'ClearSyncHistory') for method, _, _ in bus.calls))

    def test_record_and_model_keep_control_text_plain_and_stable_order(self):
        self.assertFalse(valid_record({'id': True}))
        self.reply(self.a, [record(2), record(1)])
        self.assertEqual([row['id'] for _, row in self.page.model.records], [2, 1])
        self.assertEqual(self.page.model.data(self.page.model.index(0, 4)), '已完成')

    def test_wrapper_clear_all_omits_days(self):
        bus = Bus()
        ClientBus.clear_sync_history(bus, 'all', None, None)
        self.assertEqual(json.loads(bus.calls[-1][1][0]), {'mode': 'all'})


class HistoryActionTests(unittest.TestCase):
    def setUp(self):
        self.bus = Bus()
        self.source = HistorySource('a', 'A', self.bus, 'task-a', '/mnt/a')
        self.sources = {'a': self.source}
        self.actions = HistoryActions(lambda: self.sources)
        self.messages = []; self.actions.message.connect(self.messages.append)
    def tearDown(self):
        self.actions.deleteLater(); QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def resolve(self, **fields):
        data = dict(path='/mnt/a/current/file.txt', parent_path='/mnt/a/current', exists=True,
                    offline_available=True, mount_online=True, node='node-a', reason=None)
        data.update(fields)
        self.bus.last('ResolveHistoryTarget')[2](data, None)

    def test_cached_file_opens_resolved_current_path(self):
        with patch.object(self.actions, 'open_url') as opened:
            self.actions.open(self.source, record())
            self.resolve()
            opened.assert_called_once_with('/mnt/a/current/file.txt')
        self.assertEqual(json.loads(self.bus.last('ResolveHistoryTarget')[1][0]), {'id': 1})

    def test_online_file_hydrates_before_opening(self):
        with patch.object(self.actions, 'open_url') as opened, patch.object(self.actions, 'hydrate') as hydrate:
            self.actions.open(self.source, record())
            self.resolve(offline_available=False, reason='online_only')
            opened.assert_not_called(); hydrate.assert_called_once()

    def test_offline_without_content_and_deleted_do_not_open(self):
        for fields in (dict(offline_available=False, mount_online=False, reason='offline'),
                       dict(path=None, exists=False, reason='deleted')):
            with patch.object(self.actions, 'open_url') as opened:
                self.actions.open(self.source, record()); self.resolve(**fields)
                opened.assert_not_called()
        self.assertIn('已删除', self.messages[-1])

    def test_deleted_record_can_locate_validated_parent(self):
        with patch.object(self.actions, 'open_url') as opened:
            self.actions.open(self.source, record(), locate=True)
            self.resolve(path=None, exists=False, reason='deleted')
            opened.assert_called_once_with('/mnt/a/current')

    def test_path_escape_or_replaced_service_never_opens(self):
        self.assertIsNone(safe_path('/mnt/a', '/mnt/a/../b/file'))
        self.assertIsNone(safe_path('/mnt/a', '/mnt/another/file'))
        with patch.object(self.actions, 'open_url') as opened:
            self.actions.open(self.source, record()); self.resolve(path='/outside/file')
            self.actions.open(self.source, record())
            self.sources.clear(); self.resolve()
            opened.assert_not_called()


class HydrationProcessTests(unittest.TestCase):
    """Exercise the actual QProcess and reader script, without a FUSE or NAS."""
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='history-helper-')
        self.path = Path(self.tmp.name) / 'nonempty.bin'
        self.body = b'isolated-history-content\0' * 100000
        self.path.write_bytes(self.body)
        self.bus = Bus()
        self.source = HistorySource('a', 'A', self.bus, 'task-a', self.tmp.name)
        self.sources = {'a': self.source}
        self.actions = HistoryActions(lambda: self.sources)
        self.messages = []; self.actions.message.connect(self.messages.append)

    def tearDown(self):
        for process in list(self.actions.jobs):
            process.kill(); process.waitForFinished(3000)
        self.actions.deleteLater(); QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        self.tmp.cleanup()

    def wait_helper(self):
        deadline = time.monotonic() + 10
        while self.actions.jobs and time.monotonic() < deadline:
            APP.processEvents(); time.sleep(.005)
        self.assertFalse(self.actions.jobs, 'history read helper did not finish')

    def current_response(self, **changes):
        data = dict(path=str(self.path), exists=True, offline_available=True, node='node-a')
        data.update(changes)
        self.bus.last('ResolveHistoryTarget')[2](data, None)

    def test_actual_reader_success_revalidates_before_open_and_preserves_content(self):
        with patch.object(self.actions, 'open_url') as opened:
            self.actions.hydrate(self.source, record(), str(self.path), 'node-a')
            opened.assert_not_called()
            self.wait_helper()
            self.assertEqual([entry[0] for entry in self.bus.calls], ['ResolveHistoryTarget'])
            opened.assert_not_called()
            self.current_response()
            opened.assert_called_once_with(str(self.path))
        self.assertEqual(self.path.read_bytes(), self.body)
        self.assertEqual(list(Path(self.tmp.name).iterdir()), [self.path])

    def test_actual_reader_failure_never_opens_or_claims_completion(self):
        with patch.object(self.actions, 'open_url') as opened:
            self.actions.hydrate(self.source, record(), str(self.path.with_name('missing')), 'node-a')
            self.wait_helper(); opened.assert_not_called()
        self.assertEqual(self.bus.calls, [])
        self.assertIn('下载尚未完成', self.messages[-1])

    def test_task_replacement_during_real_read_discards_completion(self):
        with patch.object(self.actions, 'open_url') as opened:
            self.actions.hydrate(self.source, record(), str(self.path), 'node-a')
            self.sources['a'] = HistorySource('a', 'A', self.bus, 'replacement', self.tmp.name)
            self.wait_helper(); opened.assert_not_called()
        self.assertEqual(self.bus.calls, [])

    def test_node_replacement_after_real_read_never_opens_replacement(self):
        with patch.object(self.actions, 'open_url') as opened:
            self.actions.hydrate(self.source, record(), str(self.path), 'node-a')
            self.wait_helper()
            self.current_response(node='replacement-node')
            opened.assert_not_called()
        self.assertIn('状态或位置已变化', self.messages[-1])


if __name__ == '__main__': unittest.main()
