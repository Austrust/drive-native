"""Conflict UI regressions with no NAS or filesystem mutations."""
import unittest
from PySide6.QtCore import QObject, Signal
from PySide6.QtWidgets import QApplication
from drive_native_gui.issues import IssuesDialog
app=QApplication([])
class Bus(QObject):
    changed=Signal()
    def __init__(self):super().__init__();self.calls=[]
    def call(self,method,args=(),callback=None):self.calls.append((method,list(args),callback))
class Tests(unittest.TestCase):
    def setUp(self):
        self.bus=Bus();self.dialog=IssuesDialog(self.bus)
        self.row={'id':'a','path':'A.doc','reason':'content_conflict','can_keep_both':True,'token':'one','directory':False}
        self.update(self.row)
        self.dialog.table.setCurrentItem(self.dialog.table.topLevelItem(0))
    def update(self,row):self.dialog.received('',self.dialog.generation,{'schema':1,'entries':[row],'next':None},None)
    def tearDown(self):self.dialog.close();self.dialog.deleteLater();app.processEvents()
    def test_refresh_preserves_edited_name_and_uses_new_version_token(self):
        self.dialog.name.setText('chosen.doc');self.update(dict(self.row,token='two'))
        self.assertEqual(self.dialog.name.text(),'chosen.doc');self.dialog.keep_both()
        self.assertEqual(self.bus.calls[-1][:2],('KeepBoth',['a','two','chosen.doc']))
        self.assertFalse(self.dialog.keep.isEnabled())
    def test_unknown_commit_never_offers_resolution(self):
        self.update(dict(self.row,reason='commit_uncertain',can_keep_both=False))
        self.assertFalse(self.dialog.keep.isEnabled());self.dialog.keep_both()
        self.assertFalse(any(c[0]=='KeepBoth' for c in self.bus.calls))
    def test_invalid_name_and_service_failure_keep_original_issue(self):
        self.dialog.name.setText('../escape');self.dialog.keep_both()
        self.assertFalse(any(c[0]=='KeepBoth' for c in self.bus.calls))
        self.dialog.name.setText('copy.doc');self.dialog.keep_both();self.dialog.completed(None,'service_changed')
        self.assertIn('a',self.dialog.rows);self.assertFalse(self.dialog.busy)
        self.assertIn('未更改',self.dialog.message.text())
    def test_creation_resolution_explains_remote_entry_and_uses_current_token(self):
        self.update(dict(self.row,reason='creation_conflict',token='creation-token'))
        self.assertTrue(self.dialog.keep.isEnabled())
        self.assertIn('同名远端条目保持不变',self.dialog.explanation.text())
        self.dialog.name.setText('new-copy.doc');self.dialog.keep_both()
        self.assertEqual(self.bus.calls[-1][:2],('KeepBoth',['a','creation-token','new-copy.doc']))
    def test_creation_identity_mismatch_is_gated(self):
        self.update(dict(self.row,reason='creation_identity_mismatch',can_keep_both=False))
        self.assertFalse(self.dialog.keep.isEnabled());self.dialog.keep_both()
        self.assertFalse(any(c[0]=='KeepBoth' for c in self.bus.calls))
        self.assertIn('身份需要进一步核对',self.dialog.explanation.text())
    def test_directory_conflict_uses_tree_operation_and_explains_scope(self):
        self.update(dict(self.row,directory=True,reason='directory_creation_conflict',can_keep_both=False,can_keep_directory=True))
        self.assertTrue(self.dialog.keep.isEnabled());self.assertIn('整个本地文件夹',self.dialog.explanation.text())
        self.dialog.name.setText('Local folder');self.dialog.keep_both()
        self.assertEqual(self.bus.calls[-1][:2],('KeepDirectoryBoth',['a','one','Local folder']))
    def test_unsupported_directory_never_sends_an_action(self):
        self.update(dict(self.row,directory=True,reason='namespace_conflict',can_keep_both=False,can_keep_directory=False))
        self.assertFalse(self.dialog.keep.isEnabled());self.dialog.keep_both()
        self.assertFalse(any(c[0] in ('KeepDirectoryBoth','KeepBoth') for c in self.bus.calls))
    def test_directory_name_with_dot_keeps_full_name_as_stem(self):
        self.update(dict(self.row,id='folder',path='folder.v1',directory=True,reason='directory_creation_conflict',can_keep_both=False,can_keep_directory=True))
        self.dialog.table.setCurrentItem(self.dialog.table.topLevelItem(0))
        self.assertTrue(self.dialog.name.text().startswith('folder.v1 ('))
        self.assertEqual(self.dialog.name.placeholderText(),'本地文件夹的新名称')
if __name__=='__main__':unittest.main()
