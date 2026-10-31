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

if __name__=='__main__':unittest.main()
