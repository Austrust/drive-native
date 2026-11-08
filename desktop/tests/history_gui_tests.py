"""Synthetic history contracts; helper tests read isolated temporary files only."""
import json
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent, Qt
from PySide6.QtWidgets import QApplication, QMessageBox
from drive_native_gui.history import (HistoryPage, HistorySource, HistoryModel, valid_record,
                                      CLEANUP_KEEPS_TEXT, detail_text, diagnostics_text)
from drive_native_gui.history_actions import (HistoryActions, safe_path, record_error_text,
                                              error_text_for_token)
from drive_native_gui.bus import ClientBus
from drive_native_gui.window import MainWindow

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

    def calls(self, bus, method):
        return [entry for entry in bus.calls if entry[0] == method]

    def count(self, bus, method):
        return len(self.calls(bus, method))

    def reply(self, bus, rows, cursor=None):
        bus.last('GetSyncHistory')[2]({'records': rows, 'next_cursor': cursor}, None)

    def retention_reply(self, bus, days=30, records=10_000, error=None, data=None):
        if data is None:
            data = {'schema': 1, 'task_id': 'task-a', 'days': days, 'records': records}
        bus.last('GetSyncHistoryRetention')[2](None if error else data, error)

    def save_reply(self, bus, days=30, records=10_000, error=None, data=None):
        if data is None:
            data = {'saved': True, 'days': days, 'records': records}
        bus.last('SetSyncHistoryRetention')[2](data, error)

    # -- history rows -------------------------------------------------------

    def test_service_ids_are_scoped_and_busy_task_does_not_block_other(self):
        self.reply(self.a, [record()], '1')
        self.reply(self.b, [record()])
        self.assertEqual(self.page.model.rowCount(), 2)
        self.assertEqual({source.slot for source, _ in self.page.model.records}, {'a', 'b'})
        self.page.load_more()
        self.assertEqual(json.loads(self.a.last('GetSyncHistory')[1][0])['cursor'], '1')
        self.assertEqual(self.count(self.b, 'GetSyncHistory'), 1)
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
        self.assertEqual(len(self.b.calls), b + 2)
        self.assertEqual(self.count(self.a, 'GetSyncHistoryRetention'), 1)
        self.assertEqual(self.count(self.b, 'GetSyncHistoryRetention'), 2)

    def test_hidden_page_rejects_inflight_response_and_makes_no_new_query(self):
        before = len(self.a.calls)
        reply = self.a.last('GetSyncHistory')[2]
        self.page.hide()
        reply({'records': [record()], 'next_cursor': '1'}, None)
        self.page.load_more()
        self.assertEqual(self.page.model.rowCount(), 0)
        self.assertEqual(len(self.a.calls), before)

    def test_replaced_profile_rejects_old_service_record(self):
        reply = self.a.last('GetSyncHistory')[2]
        replacement_bus = Bus()
        self.sources['a'] = HistorySource('a', 'A', replacement_bus, 'replacement', '/mnt/a')
        reply({'records': [record()], 'next_cursor': None}, None)
        self.assertEqual(self.page.model.rowCount(), 0)
        self.assertEqual(self.count(replacement_bus, 'GetSyncHistory'), 1)
        self.assertEqual(self.count(replacement_bus, 'GetSyncHistoryRetention'), 1)
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
            self.assertTrue(all(method in ('GetSyncHistory', 'ClearSyncHistory', 'GetSyncHistoryRetention')
                                for method, _, _ in bus.calls))

    def test_record_and_model_keep_control_text_plain_and_stable_order(self):
        self.assertFalse(valid_record({'id': True}))
        self.reply(self.a, [record(2), record(1)])
        self.assertEqual([row['id'] for _, row in self.page.model.records], [2, 1])
        self.assertEqual(self.page.model.data(self.page.model.index(0, 4)), '已完成')

    def test_wrapper_clear_all_omits_days(self):
        bus = Bus()
        ClientBus.clear_sync_history(bus, 'all', None, None)
        self.assertEqual(json.loads(bus.calls[-1][1][0]), {'mode': 'all'})

    # -- retention ----------------------------------------------------------

    def test_retention_reads_each_mount_and_shows_current_value(self):
        self.assertEqual(self.page.retention_days.value(), 30)
        self.assertEqual(self.page.retention_records.value(), 10_000)
        self.assertEqual(self.page.retention_days.maximum(), 3650)
        self.assertEqual(self.page.retention_records.maximum(), 1_000_000)
        for bus in (self.a, self.b):
            self.assertEqual(bus.last('GetSyncHistoryRetention')[1], [])
        self.assertIn('正在读取', self.page.retention_message.text())
        self.assertFalse(self.page.retention_save.isEnabled())
        self.retention_reply(self.a, 7, 500); self.retention_reply(self.b, 7, 500)
        self.assertEqual(self.page.retention_days.value(), 7)
        self.assertEqual(self.page.retention_records.value(), 500)
        self.assertIn('当前保留：最近 7 天，最多 500 条', self.page.retention_message.text())
        self.assertTrue(self.page.retention_save.isEnabled())

    def test_retention_read_failure_keeps_defaults_and_offers_retry(self):
        self.retention_reply(self.a, error='busy')
        self.retention_reply(self.b, error='org.freedesktop.DBus.Error.UnknownMethod')
        text = self.page.retention_message.text()
        self.assertIn('历史默认保留 30 天 / 10,000 条', text)
        self.assertIn('稍后重试保留期限', text)
        self.assertIn('尚不支持保留期限设置，仍可清理历史', text)
        self.assertTrue(self.page.retention_save.isEnabled())

    def test_retention_differs_per_mount_and_single_filter_only_queries_selected(self):
        self.retention_reply(self.a, 7, 500); self.retention_reply(self.b, 30, 10_000)
        text = self.page.retention_message.text()
        self.assertIn('各挂载保留期限不同', text)
        self.assertIn('A：最近 7 天 / 500 条', text)
        self.assertIn('B：最近 30 天 / 10,000 条', text)
        self.page.mount_filter.setCurrentIndex(self.page.mount_filter.findData('b'))
        self.assertEqual(self.count(self.a, 'GetSyncHistoryRetention'), 1)
        self.assertEqual(self.count(self.b, 'GetSyncHistoryRetention'), 2)
        self.retention_reply(self.b, 30, 10_000)
        self.assertIn('当前保留', self.page.retention_message.text())

    def test_retention_save_writes_each_mount_once_and_reports_success(self):
        self.retention_reply(self.a); self.retention_reply(self.b)
        self.page.retention_days.setValue(14); self.page.retention_records.setValue(2000)
        self.page.save_retention()
        self.assertEqual(self.count(self.a, 'SetSyncHistoryRetention'), 1)
        self.assertEqual(self.count(self.b, 'SetSyncHistoryRetention'), 1)
        self.assertEqual(json.loads(self.a.last('SetSyncHistoryRetention')[1][0]),
                         {'schema': 1, 'task_id': 'task-a', 'days': 14, 'records': 2000})
        self.assertEqual(json.loads(self.b.last('SetSyncHistoryRetention')[1][0]),
                         {'schema': 1, 'task_id': 'task-b', 'days': 14, 'records': 2000})
        self.assertFalse(self.page.retention_save.isEnabled())
        self.save_reply(self.a, 14, 2000); self.save_reply(self.b, 14, 2000)
        text = self.page.retention_message.text()
        self.assertIn('已保存保留期限：最近 14 天，最多 2,000 条（2 个挂载）', text)
        self.assertIn(CLEANUP_KEEPS_TEXT, text)
        self.assertTrue(self.page.retention_save.isEnabled())

    def test_retention_partial_failure_reports_each_mount_without_false_success(self):
        self.retention_reply(self.a); self.retention_reply(self.b)
        self.page.save_retention()
        self.save_reply(self.a, 30, 10_000)
        self.save_reply(self.b, error='busy')
        text = self.page.retention_message.text()
        self.assertIn('已在 1 个挂载保存保留期限；1 个未保存', text)
        self.assertIn('稍后重试保留期限', text)
        self.assertIn(CLEANUP_KEEPS_TEXT, text)
        self.assertNotIn('已保存保留期限', text)

    def test_retention_busy_on_every_mount_never_claims_success(self):
        self.retention_reply(self.a); self.retention_reply(self.b)
        self.page.save_retention()
        self.save_reply(self.a, error='busy'); self.save_reply(self.b, error='busy')
        text = self.page.retention_message.text()
        self.assertIn('保留期限未保存', text)
        self.assertNotIn('已保存', text)

    def test_retention_unsupported_backend_still_allows_cleanup(self):
        self.retention_reply(self.a, error='org.freedesktop.DBus.Error.UnknownMethod')
        self.retention_reply(self.b, error='org.freedesktop.DBus.Error.UnknownMethod')
        self.assertIn('当前后台版本尚不支持保留期限设置，仍可清理历史',
                      self.page.retention_message.text())
        self.assertFalse(self.page.retention_save.isEnabled())
        self.assertEqual(self.count(self.a, 'SetSyncHistoryRetention'), 0)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.page.clear_history('all')
        for bus in (self.a, self.b):
            self.assertEqual(len(self.calls(bus, 'ClearSyncHistory')), 1)
            bus.last('ClearSyncHistory')[2]({'removed': 1}, None)
        self.assertIn('已清理 2 条历史', self.page.message.text())
        self.assertIn('文件、草稿和缓存保留', self.page.message.text())

    def test_retention_late_reply_is_discarded_after_refresh(self):
        stale = self.a.last('GetSyncHistoryRetention')[2]
        self.page.refresh()
        stale({'schema': 1, 'task_id': 'task-a', 'days': 1, 'records': 1}, None)
        self.assertNotIn('a', self.page.retention)
        self.assertIn('正在读取', self.page.retention_message.text())
        self.assertEqual(self.page.retention_days.value(), 30)

    def test_retention_reply_from_replaced_source_is_discarded(self):
        stale = self.a.last('GetSyncHistoryRetention')[2]
        self.sources['a'] = HistorySource('a', 'A', Bus(), 'replacement', '/mnt/a')
        stale({'schema': 1, 'task_id': 'task-a', 'days': 1, 'records': 1}, None)
        self.assertNotIn('a', self.page.retention)
        self.assertIn('挂载身份已变化', self.page.retention_message.text())

    def test_retention_invalid_values_are_rejected_not_displayed(self):
        self.retention_reply(self.a, days=0, records=500)
        self.retention_reply(self.b, days=30, records=True)
        self.assertIn('格式不可识别', self.page.retention_message.text())
        self.assertEqual(self.page.retention, {})
        self.assertTrue(self.page.retention_save.isEnabled())


class WindowBus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    shutdown_changed = Signal()
    owner = ':1.synthetic'
    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent); self.calls = []
    def call(self, method, args=(), callback=None):
        self.calls.append((method, args, callback))


class StopMountButtonTests(unittest.TestCase):
    """The mount page only confirms and delegates; it never stops services itself."""
    def setUp(self):
        from unittest.mock import patch as _patch
        self.start_patch = _patch.object(MainWindow, 'start', lambda _: None)
        self.start_patch.start()
        self.bus_patch = _patch('drive_native_gui.window.ClientBus', WindowBus)
        self.bus_patch.start()
        self.window = MainWindow()
        self.window.show(); APP.processEvents(); self.window.refresh_timer.stop()
        self.window.bus.calls.clear()
        self.emitted = []
        self.window.stop_requested.connect(lambda: self.emitted.append(True))

    def tearDown(self):
        self.bus_patch.stop(); self.start_patch.stop()
        self.window.close(); self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_stop_button_is_available_with_accessible_name(self):
        self.assertEqual(self.window.stop_mount.text(), '停止此挂载并卸载…')
        self.assertEqual(self.window.stop_mount.accessibleName(), '停止此挂载并卸载')
        self.assertTrue(self.window.stop_mount.isEnabled())

    def test_cancel_explains_pause_versus_stop_and_emits_nothing(self):
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.No) as question:
            self.window.stop_mount.click()
        args = question.call_args.args
        self.assertEqual(args[1], '停止此挂载并卸载')
        self.assertIn('暂停同步只暂停此挂载并保留挂载点，可稍后恢复', args[2])
        self.assertIn('停止并卸载会结束此挂载及其后台服务', args[2])
        self.assertIn('不会删除本地缓存、草稿或远端文件', args[2])
        self.assertEqual(args[3], QMessageBox.Yes | QMessageBox.No)
        self.assertEqual(args[4], QMessageBox.No)
        self.assertEqual(self.emitted, [])
        self.assertTrue(self.window.stop_mount.isEnabled())
        self.assertNotIn('StopMount', [method for method, _, _ in self.window.bus.calls])

    def test_confirmed_stop_emits_once_disables_button_and_leaves_stop_to_owner(self):
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.window.stop_mount.click()
        self.assertEqual(len(self.emitted), 1)
        self.assertFalse(self.window.stop_mount.isEnabled())
        self.assertEqual(self.window.stop_mount.text(), '正在停止并卸载…')
        self.assertIn('已请求停止并卸载此挂载', self.window.banner.text())
        self.assertFalse(self.window.banner.isHidden())
        self.assertNotIn('StopMount', [method for method, _, _ in self.window.bus.calls])

    def test_owner_can_restore_button_after_failed_stop(self):
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.window.stop_mount.click()
        self.window.set_stopping(False, '停止未完成，挂载仍在，可重试。')
        self.assertTrue(self.window.stop_mount.isEnabled())
        self.assertEqual(self.window.stop_mount.text(), '停止此挂载并卸载…')
        self.assertIn('停止未完成', self.window.banner.text())


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


class MountActions:
    """Duck-typed stand-in for the manager's start hook."""

    def __init__(self):
        self.started = []

    def start_mount(self, slot):
        self.started.append(slot)


class HistoryRecoveryTests(unittest.TestCase):
    """Wave2 GUI recovery: readable errors, disabled unsafe opens, retry, detail."""
    def setUp(self):
        self.bus = Bus()
        self.source = HistorySource('a', 'A', self.bus, 'task-a', '/mnt/a')
        self.sources = {'a': self.source}
        self.mounts = MountActions()
        self.page = HistoryPage(lambda: self.sources, mount_actions=self.mounts)
        self.page.show(); APP.processEvents(); self.page.auto_refresh.stop()
        self.bus.last('GetSyncHistory')[2]({'records': [record(2), record(1)],
                                           'next_cursor': '1'}, None)

    def tearDown(self):
        self.page.hide(); self.page.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def select_first(self):
        self.page.table.selectRow(0)
        self.assertEqual(self.page.selected()[1]['id'], 2)
        return self.bus.last('ResolveHistoryTarget')[2]

    def resolve_reply(self, callback, **fields):
        data = dict(path='/mnt/a/current/file.txt', parent_path='/mnt/a/current', exists=True,
                    offline_available=True, mount_online=True, node='node-a', reason=None)
        data.update(fields)
        callback(data, None)
        return data

    def count(self, bus, method):
        return len([entry for entry in bus.calls if entry[0] == method])

    def test_raw_controlled_tokens_are_shown_as_chinese_only(self):
        self.assertEqual(error_text_for_token('transport'), '网络传输失败，请检查网络连接后重试。')
        self.assertEqual(record_error_text(record(error_kind='uncertain_commit')),
                         '尚未确认服务器是否完成该操作，请稍后刷新核对。')
        self.assertEqual(record_error_text(record(error_kind=None,
                                                  error_summary='服务器返回权限相关响应，提交状态待核对')),
                         '服务器返回权限相关响应，提交状态待核对')
        unknown = record_error_text(record(error_kind='weird_code'))
        self.assertIn('weird_code', unknown)
        self.assertIn('同步失败', unknown)
        self.bus.last('GetSyncHistory')[2]({'records': [
            record(4, error_kind='transport',
                   error_summary='transport')], 'next_cursor': None}, None)
        tooltip = self.page.model.data(self.page.model.index(0, 0), Qt.ToolTipRole)
        self.assertIn('网络传输失败', tooltip)
        self.assertNotIn('transport', tooltip)

    def test_deleted_record_disables_open_but_keeps_parent_folder(self):
        callback = self.select_first()
        self.resolve_reply(callback, path=None, exists=False, reason='deleted',
                           offline_available=False, mount_online=False)
        self.assertFalse(self.page.open_button.isEnabled())
        self.assertTrue(self.page.parent_button.isEnabled())
        self.assertIn('已删除', self.page.detail.text())
        before = self.count(self.bus, 'ResolveHistoryTarget')
        self.page.open_selected()
        self.assertIn('已删除', self.page.message.text())
        self.assertEqual(self.count(self.bus, 'ResolveHistoryTarget'), before)
        with patch.object(self.page.actions, 'open_url') as opened:
            self.page.open_parent_selected()
            opened.assert_called_once_with('/mnt/a/current')

    def test_offline_without_cache_offers_start_mount_then_retry(self):
        callback = self.select_first()
        self.resolve_reply(callback, path=None, exists=True, reason='offline',
                           offline_available=False, cached=False, mount_online=False,
                           mount_slot='a', suggested_action='start_mount')
        self.assertFalse(self.page.open_button.isEnabled())
        self.assertEqual(self.page.retry_button.text(), '启动挂载后重试')
        self.page.retry_selected()
        self.assertEqual(self.mounts.started, ['a'])
        self.assertIn('正在启动挂载', self.page.message.text())
        self.page.actions.mount_actions = None
        self.page.retry_selected()
        self.assertIn('请在挂载管理中启动该挂载', self.page.message.text())

    def test_detail_and_diagnostics_include_context_and_redact_secrets(self):
        row = record(7, error_kind='transport', node='node-7')
        row.update(attempts=3, action='upload', result='failed',
                   detail={'permission_response': {'action': 'upload', 'code': 5},
                           'reconcile_attempts': 2, 'token': 'topsecret',
                           'nested': [{'cookie': 'sessionid'}]})
        resolved = dict(path=None, exists=True, reason='offline', parent_path='/mnt/a/current',
                        offline_available=False, cached=False, mount_online=False,
                        mount_slot='a', suggested_action='start_mount')
        text = detail_text(self.source, row, resolved)
        self.assertIn('尝试次数：3', text)
        self.assertIn('网络传输失败', text)
        self.assertIn('错误代码：transport', text)
        self.assertIn('permission_response', text)
        self.assertIn('***', text)
        self.assertNotIn('topsecret', text)
        self.assertNotIn('sessionid', text)
        diag = diagnostics_text(self.source, row, resolved)
        self.assertIn('error_kind=transport', diag)
        self.assertIn('suggested_action=start_mount', diag)
        self.assertIn('mount_slot=a', diag)
        self.assertNotIn('topsecret', diag)

    def test_auto_refresh_merges_newest_page_without_losing_loaded_pages(self):
        self.page.load_more()
        second = self.bus.last('GetSyncHistory')
        self.assertEqual(json.loads(second[1][0])['cursor'], '1')
        second[2]({'records': [record(0)], 'next_cursor': None}, None)
        self.assertEqual(self.page.model.rowCount(), 3)
        before = self.count(self.bus, 'GetSyncHistory')
        self.page.refresh_newest()
        self.assertEqual(self.count(self.bus, 'GetSyncHistory'), before + 1)
        newest = self.bus.last('GetSyncHistory')
        self.assertNotIn('cursor', json.loads(newest[1][0]))
        self.page.refresh_newest()
        self.assertEqual(self.count(self.bus, 'GetSyncHistory'), before + 1)
        newest[2]({'records': [record(9)], 'next_cursor': '9'}, None)
        self.assertEqual(self.page.model.rowCount(), 4)
        self.assertEqual([row['id'] for _, row in self.page.model.records][0], 9)
        self.page.hide()
        self.assertFalse(self.page.auto_refresh.isActive())

    def test_old_resolve_payload_still_opens_cached_file(self):
        callback = self.select_first()
        old = dict(path='/mnt/a/file.txt', parent_path='/mnt/a', exists=True,
                   offline_available=True, mount_online=True, reason=None, node='node-a')
        callback(old, None)
        self.assertTrue(self.page.open_button.isEnabled())
        with patch.object(self.page.actions, 'open_url') as opened:
            self.page.open_selected()
            self.bus.last('ResolveHistoryTarget')[2](old, None)
            opened.assert_called_once_with('/mnt/a/file.txt')

    def test_recent_page_hides_recovery_actions_and_loads_recent_rows(self):
        bus = Bus()
        source = HistorySource('a', 'A', bus, 'task-a', '/mnt/a')
        page = HistoryPage(lambda: {'a': source}, recent=True)
        page.show(); APP.processEvents(); page.auto_refresh.stop()
        try:
            for button in (page.retry_button, page.parent_button, page.diag_button,
                           page.detail_button):
                self.assertTrue(button.isHidden())
            bus.last('GetRecentActivity')[2]({'records': [record()]}, None)
            self.assertEqual(page.model.rowCount(), 1)
        finally:
            page.hide(); page.deleteLater()
            QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)


if __name__ == '__main__': unittest.main()
