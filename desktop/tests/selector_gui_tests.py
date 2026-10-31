"""Drive top-level selection: no real account, service or NAS."""
import os,tempfile,unittest
from unittest.mock import patch
from PySide6.QtWidgets import QApplication
from PySide6.QtCore import Qt
from drive_native_gui.wizard import SetupWizard
from drive_native_gui import config

app=QApplication([])
class SelectorTests(unittest.TestCase):
    def setUp(self):
        self.w=SetupWizard()
        self.w.logged_in({'spaces':[
            {'name':'Team','view_id':70,'path':'/','available':True,'writable':False,'scope':None},
            {'name':'我的文件','view_id':54,'path':'/','available':True,'writable':True,'scope':None,'personal':True},
            {'name':'Denied','view_id':71,'path':'/','available':False,'writable':False,'scope':None},
        ],'warnings':[]},None)
    def tearDown(self):
        self.w.reject()
    def team(self):
        root=self.w.tree.topLevelItem(0)
        return next(root.child(i) for i in range(root.childCount()) if root.child(i).text(0)=='Team')
    def test_catalog_personal_first_and_top_level_not_selectable(self):
        root=self.w.tree.topLevelItem(0)
        self.assertEqual(root.text(0),'Synology Drive')
        self.assertEqual(root.child(0).text(0),'我的文件')
        self.w.finish_setup()
        self.assertIsNone(self.w.profile)
        self.assertTrue(next(root.child(i) for i in range(root.childCount()) if root.child(i).text(0)=='Denied').isDisabled())
    def test_expansion_uses_selected_view_and_keeps_readonly(self):
        calls=[]
        def request(message,callback):
            calls.append(message)
            callback({'view_id':70,'path':'/','root_scope':{'view_id':70,'node_id':2,'file_id':'team-root','path':'/'},
                'folders':[{'name':'Lab','view_id':70,'path':'/Lab','writable':False,'scope':{'view_id':70,'node_id':3,'file_id':'lab','path':'/Lab'}}]},None)
        self.w.client.request=request
        team=self.team();self.w.expand(team)
        self.assertEqual(calls,[{'command':'list','view_id':70,'path':'/'}])
        self.assertEqual(team.data(0,Qt.UserRole)['scope']['file_id'],'team-root')
        self.assertFalse(team.child(0).data(0,Qt.UserRole)['writable'])
        self.w.expand(team);self.assertEqual(len(calls),1)
    def test_wrong_view_response_not_attached(self):
        self.w.client.request=lambda _,cb:cb({'view_id':54,'path':'/','folders':[]},None)
        team=self.team();self.w.expand(team)
        self.assertFalse(team.data(0,Qt.UserRole)['loaded'])
        self.assertIn('不一致',self.w.message.text())
    def test_team_selection_saves_team_scope_and_permissions(self):
        team=self.team();record=team.data(0,Qt.UserRole)
        record['scope']={'view_id':70,'node_id':2,'file_id':'team-root','path':'/'}
        team.setData(0,Qt.UserRole,record);self.w.tree.setCurrentItem(team)
        self.w.connection_snapshot={'host':'test.invalid','tls_name':'test.invalid','username':'test'}
        self.w.remember.setChecked(False)
        with tempfile.TemporaryDirectory() as tmp,patch('drive_native_gui.wizard.save_profile') as save:
            self.w.mount.setText(tmp);self.w.finish_setup()
            profile=save.call_args.args[0]
            self.assertEqual(profile['scope']['view_id'],70)
            self.assertFalse(profile['writable'])
    def test_profile_backup_restore_preserves_original(self):
        with tempfile.TemporaryDirectory() as tmp,patch.dict(os.environ,{'XDG_CONFIG_HOME':tmp}):
            original={'schema':1,'task_id':'original','mount':'/tmp/example'}
            config.save_profile(original)
            with self.assertRaises(ValueError):config.archive_profile('other')
            backup=config.archive_profile('original')
            self.assertIsNone(config.load_profile())
            config.restore_profile(backup)
            self.assertEqual(config.load_profile(),original)
            with self.assertRaises(FileExistsError):config.restore_profile(backup)

    def test_setup_error_transport_keeps_categories_distinct_without_raw_reason(self):
        import json
        from drive_native_gui.setup_client import SetupClient
        client = SetupClient()
        results = []
        class Response:
            data = b""
            def readAllStandardOutput(self):
                data, self.data = self.data, b""
                return data
        fake = Response()
        client.process = fake
        categories = ['login_rejected', 'download_permission', 'certificate', 'network', 'protocol', 'session_expired']
        try:
            for category in categories:
                fake.data = (json.dumps({'schema':1,'ok':False,'error':category,'reason':'untrusted-sensitive-reason'})+'\n').encode()
                client.callbacks.append(lambda data,error: results.append((data,error)))
                client._read()
            self.assertEqual(len(results), len(categories))
            self.assertEqual(len({error for _,error in results}), len(categories))
            self.assertTrue(all(data is None and 'untrusted-sensitive-reason' not in error for data,error in results))
            self.assertFalse(client.timeout.isActive())
            self.assertFalse(client.callbacks)
        finally:
            client.deleteLater()

if __name__=='__main__':unittest.main()
