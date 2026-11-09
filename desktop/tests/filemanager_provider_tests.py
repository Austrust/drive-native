"""Exercise the real Nautilus 4 API with synthetic paths and a stub D-Bus transport."""
import importlib.util, unittest, os
from pathlib import Path
from unittest.mock import patch
from gi.repository import Gio
host = os.environ.get('DRIVE_NATIVE_TEST_FILEMANAGER', 'nemo')
assert host in {'nemo', 'nautilus'}
spec=importlib.util.spec_from_file_location('drive_native_' + host,Path(__file__).parents[1]/host/('drive_native_' + host + '.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class File:
    def __init__(self,path):self.path=path;self.attributes={};self.emblems=[];self.invalidations=0
    def get_uri_scheme(self):return 'file'
    def get_uri(self):return Gio.File.new_for_path(self.path).get_uri()
    def add_string_attribute(self,key,value):self.attributes[key]=value
    def add_emblem(self,value):self.emblems.append(value)
    def invalidate_extension_info(self):self.invalidations+=1

class Tests(unittest.TestCase):
    def setUp(self):
        with patch.object(Gio.DBusProxy,'new_for_bus'):
            self.task=m._TaskExtension()
        self.task.root='/tmp/synthetic-drive-mount';self.task.connection='online'
        self.task._schedule=lambda: None  # Flush deterministically; no idle GLib sources.
    def test_preview_does_not_offer_body_actions_or_offline_emblems(self):
        file = File(self.task.root + '/A.doc')
        self.task.preview_only = True
        self.task.cache[file.path] = {'status': 'initializing', 'offline_available': False}
        self.task.update_file_info(file)
        self.assertIn('正文尚未可用', file.attributes['drive_native_status'])
        self.assertEqual(file.emblems, ['drive-native-pending'])
        self.assertEqual(self.task.get_file_items([file]), ())
        with patch.object(self.task, '_call') as call:
            self.task._activate(None, [file.path], 'SetPinned', True)
            call.assert_not_called()

    def test_preview_transition_invalidates_previous_emblems_and_requeries(self):
        file = File(self.task.root + '/A.doc')
        self.task._track(file.path, file)
        self.task.cache[file.path] = {'status': 'local_complete'}
        self.task._call = lambda method, args, cb: cb(
            {'mount': self.task.root, 'connection': 'connecting', 'preview_only': True}, False)
        self.task._changed()
        self.assertTrue(self.task.preview_only)
        self.assertNotIn(file.path, self.task.cache)
        self.assertIn(file.path, self.task.pending)
        self.assertEqual(file.invalidations, 1)

    def test_nautilus4_menus_accept_removed_window_argument_and_keep_paths_scoped(self):
        inside=File('/tmp/synthetic-drive-mount/folder');outside=File('/tmp/synthetic-drive-mount-other/file')
        self.assertEqual(len(self.task.get_file_items([inside])),1)
        self.assertEqual(len(self.task.get_background_items(inside)),1)
        self.assertEqual(self.task.get_file_items([inside,outside]),())
        self.assertEqual(self.task.get_background_items(outside),())
        # Nemo's older signature also remains valid.
        self.assertEqual(len(self.task.get_file_items(None,[inside])),1)
    def test_status_uses_local_cache_without_file_body_or_remote_request(self):
        file=File('/tmp/synthetic-drive-mount/file')
        self.task.cache[file.path]={'status':'pinned_complete'}
        with patch.object(self.task,'_call',side_effect=AssertionError('unexpected D-Bus request')):
            self.task.update_file_info(file)
        self.assertEqual(file.attributes['drive_native_status'],'已固定')
        self.assertEqual(file.emblems,['drive-native-pinned'])
    def test_bounded_directory_summary_is_neutral(self):
        file=File('/tmp/synthetic-drive-mount/large')
        self.task.cache[file.path]={'status':'not_summarized'};self.task.update_file_info(file)
        self.assertEqual(file.attributes['drive_native_status'],'状态尚未汇总')
        self.assertEqual(file.emblems,['drive-native-pending'])
        self.assertNotIn('drive-native-error',file.emblems)
    def test_unchanged_status_signal_does_not_invalidate_other_extensions(self):
        file=File('/tmp/synthetic-drive-mount/file')
        state={'path':file.path,'status':'local_complete'}
        self.task._track(file.path,file);self.task.cache[file.path]=state
        def reply(method,args,callback):
            callback({'mount':self.task.root,'connection':'online'} if method=='GetOverview'
                     else {'entries':[dict(state)]},False)
        self.task._call=reply
        self.task._changed();self.task._flush()
        self.assertEqual(file.invalidations,0)
        state=dict(state,status='pinned_complete')
        self.task._changed();self.task._flush()
        self.assertEqual(file.invalidations,1)

    def test_mount_change_invalidates_old_paths_and_discards_pending(self):
        file = File(self.task.root + '/file')
        self.task._track(file.path, file)
        self.task.cache[file.path] = {'status': 'local_complete'}
        self.task.pending.add(file.path)
        self.task._call = lambda method, args, cb: cb(
            {'mount': '/tmp/another-mount', 'connection': 'online'}, False)
        self.task._changed()
        self.assertEqual(file.invalidations, 1)
        self.assertEqual(self.task.cache, {})
        self.assertEqual(self.task.pending, set())

    def test_reconnect_requeries_visible_paths_and_clears_stale_reply(self):
        file = File(self.task.root + '/file')
        self.task._track(file.path, file)
        self.task.cache[file.path] = {'path': file.path, 'status': 'local_complete'}
        replies = []
        self.task._call = lambda method, args, cb: replies.append((method, cb))
        self.task.pending.add(file.path)
        self.task._flush()
        self.task._changed()  # owner loss while GetStates is in flight
        replies[1][1](None, True)
        replies[0][1]({'entries': [{'path': file.path, 'status': 'pinned_complete'}]}, False)
        self.assertFalse(self.task.busy)
        self.assertNotIn(file.path, self.task.cache)
        self.assertFalse(self.task.pending)
        self.task._changed()  # owner returns
        replies[2][1]({'mount': '/tmp/synthetic-drive-mount', 'connection': 'online'}, False)
        self.assertIn(file.path, self.task.pending)
        self.task._flush()
        replies[3][1]({'entries': [{'path': file.path, 'status': 'conflict'}]}, False)
        self.assertEqual(self.task.cache[file.path]['status'], 'conflict')

    def test_tasks_do_not_share_caches_or_accept_other_mount_commands(self):
        with patch.object(Gio.DBusProxy, 'new_for_bus'):
            second = m._TaskExtension(m.NAME + '.i' + 'a' * 32)
        second.root = '/tmp/second-mount'
        second.connection = 'online'
        file = File(self.task.root + '/file')
        self.task.cache[file.path] = {'status': 'conflict'}
        self.assertIsNot(second.cache, self.task.cache)
        self.assertEqual(second.get_file_items([file]), ())
        second.update_file_info(file)
        self.assertFalse(second.files)
        self.assertFalse(second.pending)

    def test_repeated_identical_status_signals_do_not_invalidate_icons(self):
        file = File(self.task.root + '/file')
        self.task._track(file.path, file)
        state = {'path': file.path, 'status': 'local_complete'}
        self.task.cache[file.path] = state
        self.task._call = lambda method, args, cb: cb(
            {'mount': self.task.root, 'connection': 'online'} if method == 'GetOverview'
            else {'entries': [dict(state)]}, False)
        for _ in range(100):
            self.task._signal(None, None, 'StatusChanged', None)
            self.task._flush()
        self.assertEqual(file.invalidations, 0)
        self.task.connection = 'offline'
        self.task._changed()
        self.assertEqual(file.invalidations, 1)

    def test_release_uses_background_job_and_polls_progress(self):
        calls = []
        notifications = []
        replies = {
            'BeginReleaseSpace': {'schema': 1, 'job_id': 'release-2', 'total': 2},
            'GetReleaseJob': {'schema': 1, 'job_id': 'release-2', 'available': True,
                              'total': 2, 'done': 2, 'released_count': 1,
                              'skipped_count': 1, 'protected_count': 0,
                              'reclaimed_bytes': 1024, 'finished': True,
                              'cancelled': False, 'released': ['a'],
                              'skipped': ['b'], 'protected_or_failed': []},
        }

        def call(method, args, callback):
            calls.append((method, args))
            callback(replies[method], False)

        self.task._call = call
        self.task._notify = notifications.append
        self.task._release([self.task.root + '/folder'])
        # One begin plus one finished poll: never a blocking synchronous release.
        self.assertEqual([method for method, _ in calls],
                         ['BeginReleaseSpace', 'GetReleaseJob'])
        self.assertEqual(calls[0][1].unpack(),
                         ([self.task.root + '/folder'],))
        self.assertEqual(calls[1][1].unpack(), ('release-2',))
        self.assertIn('正在释放 2 个文件', notifications[0])
        self.assertIn('已释放 1 个文件', notifications[-1])
        self.assertIn('跳过 1', notifications[-1])
        self.assertIn('1.00 KiB', notifications[-1])

    def test_release_notifies_failure_when_begin_is_rejected(self):
        notifications = []
        calls = []

        def call(method, args, callback):
            calls.append(method)
            callback(None, True)

        self.task._call = call
        self.task._notify = notifications.append
        self.task._release([self.task.root + '/folder'])
        self.assertEqual(calls, ['BeginReleaseSpace'])
        self.assertIn('操作未完成', notifications[0])

    def test_release_stops_polling_when_job_disappears_after_restart(self):
        notifications = []
        calls = []
        replies = {
            'BeginReleaseSpace': {'schema': 1, 'job_id': 'release-3', 'total': 2},
            'GetReleaseJob': {'schema': 1, 'job_id': 'release-3',
                              'available': False},
        }

        def call(method, args, callback):
            calls.append(method)
            callback(replies[method], False)

        self.task._call = call
        self.task._notify = notifications.append
        self.task._release([self.task.root + '/folder'])
        self.assertEqual(calls, ['BeginReleaseSpace', 'GetReleaseJob'])
        self.assertIn('释放未完成', notifications[-1])

    def test_status_signal_storm_issues_single_overview(self):
        calls = []

        def reply(method, args, cb):
            calls.append(method)
            cb({'mount': self.task.root, 'connection': 'online'} if method == 'GetOverview'
               else {'entries': []}, False)

        self.task._call = reply
        for _ in range(100):
            self.task._signal(None, None, 'StatusChanged', None)
        self.assertEqual(calls.count('GetOverview'), 1)
        self.assertGreaterEqual(self.task.coalesced_signals, 99)


if __name__=='__main__':unittest.main()
