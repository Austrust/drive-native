import os,tempfile,unittest,uuid
from pathlib import Path
from unittest.mock import patch
from PySide6.QtWidgets import QApplication
from drive_native_gui import config
from drive_native_gui.wizard import SetupWizard

APP=QApplication.instance() or QApplication([])
class MultiMountTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory()
        self.env=patch.dict(os.environ,{'XDG_CONFIG_HOME':self.tmp.name,'DRIVE_NATIVE_INSTANCE':''})
        self.env.start()
    def tearDown(self):self.env.stop();self.tmp.cleanup()
    def profile(self,folder):
        return {'schema':1,'task_id':str(uuid.uuid4()),'mount':str(Path(self.tmp.name)/folder),'scope':{'path':'/'}}
    def test_legacy_and_two_instances_coexist_with_private_profiles(self):
        a,b=uuid.uuid4().hex,uuid.uuid4().hex
        config.save_profile(self.profile('old'))
        for slot,name in [(a,'one'),(b,'two')]:
            os.environ['DRIVE_NATIVE_INSTANCE']=slot
            config.save_profile(self.profile(name))
        self.assertEqual(len(config.profile_records()),3)
        self.assertEqual(len({config.bus_name(s) for s in ['',a,b]}),3)
        self.assertEqual(len({config.service_unit(s) for s in ['',a,b]}),3)
        self.assertEqual(config.config_path(a).stat().st_mode&0o777,0o600)
    def test_overlapping_mounts_and_invalid_instance_are_rejected(self):
        config.save_profile(self.profile('shared'))
        os.environ['DRIVE_NATIVE_INSTANCE']=uuid.uuid4().hex
        for name in ['shared','shared/child']:
            with self.assertRaises(ValueError):config.save_profile(self.profile(name))
        with self.assertRaises(ValueError):config.config_path('../escape')
    def test_changing_one_task_preserves_other_task_and_legacy(self):
        old=self.profile('old');config.save_profile(old)
        slot=uuid.uuid4().hex;os.environ['DRIVE_NATIVE_INSTANCE']=slot
        new=self.profile('new');config.save_profile(new)
        backup=config.archive_profile(new['task_id'])
        self.assertEqual(config.load_profile(''),old)
        config.restore_profile(backup)
        self.assertEqual(config.load_profile(slot),new)
    def test_restore_rejects_mount_claimed_by_another_task(self):
        old=self.profile('same');config.save_profile(old)
        backup=config.archive_profile(old['task_id'])
        os.environ['DRIVE_NATIVE_INSTANCE']=uuid.uuid4().hex
        config.save_profile(self.profile('same'))
        os.environ['DRIVE_NATIVE_INSTANCE']=''
        with self.assertRaises(ValueError):config.restore_profile(backup)
        self.assertTrue(backup.exists())
        self.assertIsNone(config.load_profile())
    def test_mount_name_controls_local_name_without_changing_remote_scope(self):
        w=SetupWizard();w.mount.setText(str(Path(self.tmp.name)/'old'))
        w.rename_mount('我的报销盘')
        self.assertEqual(Path(w.mount.text()).name,'我的报销盘')
        self.assertEqual(w.mount_name.text(),'我的报销盘')
        before=w.mount.text();w.rename_mount('../escape');self.assertEqual(w.mount.text(),before)
        w.reject()

if __name__=='__main__':unittest.main()
