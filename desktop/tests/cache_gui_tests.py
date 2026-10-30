import json,unittest
from PySide6.QtCore import QObject,Signal
from PySide6.QtWidgets import QApplication
from drive_native_gui.cache_settings import CacheSettingsDialog,MIB,GIB
app=QApplication([])
class Bus(QObject):
    changed=Signal();availability=Signal(bool)
    def __init__(self):super().__init__();self.calls=[]
    def call(self,method,args=(),callback=None):self.calls.append((method,list(args),callback))
class Tests(unittest.TestCase):
    def setUp(self):self.bus=Bus();self.dialog=CacheSettingsDialog(self.bus);self.received()
    def received(self,task='task-a',limit=0):
        self.dialog.received({'schema':1,'task_id':task,'connection':'paused','usage':{'limit_bytes':limit,'content_bytes':3*MIB,'reclaimable_bytes':MIB,'protected_bytes':2*MIB,'over_target_bytes':0}},None)
    def tearDown(self):self.dialog.close();self.dialog.deleteLater();app.processEvents()
    def test_user_limit_survives_usage_refresh_and_targets_loaded_task(self):
        d=self.dialog;d.enabled.setChecked(True);d.amount.setValue(37);self.received()
        self.assertEqual(d.amount.value(),37);d.save();method,args,_=self.bus.calls[-1];self.assertEqual(method,'SetCacheSettings');self.assertEqual(json.loads(args[0]),{'schema':1,'task_id':'task-a','limit_bytes':37*GIB});self.assertFalse(d.save_button.isEnabled())
    def test_task_change_reloads_policy_and_does_not_apply_old_edits(self):
        d=self.dialog;d.enabled.setChecked(True);d.amount.setValue(37);self.received('task-b',3*MIB)
        self.assertEqual(d.amount.value(),3);self.assertEqual(d.unit.currentText(),'MiB');self.assertFalse(d.dirty);d.amount.setValue(4);d.save();self.assertEqual(json.loads(self.bus.calls[-1][1][0])['task_id'],'task-b')
    def test_disconnect_and_invalid_reply_disable_save_without_request(self):
        d=self.dialog;d.availability(False);d.save();self.assertFalse(any(c[0]=='SetCacheSettings' for c in self.bus.calls));d.received({'usage':{}},None);self.assertFalse(d.loaded)
    def test_failed_save_retains_edits_and_disabling_policy_uses_zero(self):
        d=self.dialog;self.received(limit=2*GIB);d.enabled.setChecked(False);d.save();self.assertEqual(json.loads(self.bus.calls[-1][1][0])['limit_bytes'],0);d.saved(None,'failure');self.assertTrue(d.dirty);self.assertTrue(d.save_button.isEnabled())
    def test_large_mib_limit_roundtrips_without_silent_clamping(self):
        d=self.dialog;limit=1024**4+MIB;self.received(limit=limit)
        self.assertEqual(d.unit.currentText(),'MiB');self.assertEqual(d.amount.value()*MIB,limit)
        d.amount.setValue(d.amount.value()+1);d.save()
        self.assertEqual(json.loads(self.bus.calls[-1][1][0])['limit_bytes'],limit+MIB)
        d.saved(None,'failure');d.unit.setCurrentText('GiB')
        self.assertLessEqual(d.amount.value()*GIB,1024**5)
if __name__=='__main__':unittest.main()
