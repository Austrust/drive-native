"""Real product MainWindow with synthetic local IPC replies, no backend or NAS."""
import argparse
import hashlib
import json
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject,Signal
from PySide6.QtWidgets import QApplication
from drive_native_gui.window import MainWindow
from drive_native_gui.upgrade import BackendUpgrade
from drive_native_gui.theme import install_theme,palette_for

PROFILE={'schema':1,'task_id':'synthetic-permission-render','mount':'/home/demo/DriveMounts/项目资料',
         'host':'demo.invalid','username':'demo','scope':{'path':'/Demo/项目资料'}}
BASE={**PROFILE,'connection':'online','mounted':True,'pending_operations':0,
      'scope_path':PROFILE['scope']['path'],'initialization':{'active':False},
      'write_policy_revision':7,'local_policy':'follow_remote','server_capability':'unknown',
      'writable_source':'user'}
CASES=(
    ('user',{'local_policy':'explicit_read_only'},{'create_dir':{'state':'denied'}},'用户设为只读'),
    ('space',{'server_capability':'space_forbidden'},{'create_dir':{'state':'denied'}},'服务禁止'),
    ('directory',{}, {'create_dir':{'state':'denied'},'create_file':{'state':'unknown'}},'当前目录权限不足'),
    ('unknown',{}, {'create_dir':{'state':'unknown'},'create_file':{'state':'unknown'}},'权限待验证'),
)

class Bus(QObject):
    changed=Signal();progress_changed=Signal();availability=Signal(bool)
    owner=':1.synthetic-render'
    def __init__(self,parent=None,*,instance=None):super().__init__(parent);self.calls=[];self.overview=dict(BASE);self.actions={}
    def call(self,method,args=(),callback=None):
        self.calls.append(method)
        if callback is None:return
        if method=='GetOverview':callback(dict(self.overview),None)
        elif method=='GetDirectoryPermissions':callback({'actions':self.actions},None)
        elif method=='GetInitialization':callback({'task_id':PROFILE['task_id'],'initialization':{'active':False}},None)
        elif method=='GetTransfers':callback({'entries':[]},None)
        elif method=='GetDirectory':callback({'entries':[],'next_offset':None,'path':''},None)
        elif method=='GetShutdown':callback({'task_id':PROFILE['task_id'],'mounted':True,'shutdown':None},None)
        elif method in ('GetRecentActivity','GetSyncHistory'):callback({'records':[],'next_cursor':None},None)
        else:callback(None,'synthetic fixture has no '+method)

def main():
    parser=argparse.ArgumentParser();parser.add_argument('output',type=Path);args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    app=QApplication([]);install_theme(app);evidence=[]
    with patch('drive_native_gui.window.ClientBus',Bus),patch.object(MainWindow,'start',lambda _:None),patch.object(BackendUpgrade,'check',lambda _:None):
        window=MainWindow(instance='a'*32);window.profile=dict(PROFILE);window.resize(1120,740);window.show();app.processEvents()
        for dark in (False,True):
            app.setPalette(palette_for(dark));window.apply_theme();app.processEvents()
            for name,changes,actions,expected in CASES:
                window.bus.overview={**BASE,**changes};window.bus.actions=actions
                window.received_overview(dict(window.bus.overview),None);app.processEvents()
                assert expected in window.permission_label.text()
                assert window.permission_settings.isEnabled() and window.permission_recheck.isEnabled()
                filename=f'permission-{name}-{("dark" if dark else "light")}.png'
                output=args.output/filename;assert window.grab().save(str(output))
                evidence.append({'file':filename,'text':window.permission_label.text(),'sha256':hashlib.sha256(output.read_bytes()).hexdigest(),
                    'settings_enabled':window.permission_settings.isEnabled(),'recheck_enabled':window.permission_recheck.isEnabled()})
        window.hide()
    result={'synthetic_only':True,'network':False,'component':'drive_native_gui.window.MainWindow',
            'scope':'real product component and GetOverview/GetDirectoryPermissions callbacks; mocked IPC; no live backend',
            'cases':evidence}
    (args.output/'evidence.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'images':len(evidence),'all_expected_states_visible':True}))

if __name__=='__main__':main()
