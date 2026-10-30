import json,pathlib,subprocess,time
from gi.repository import Gio,GLib
root=pathlib.Path.home()/'.local/state/drive-native-p0/durability-001';cfg=json.loads((root/'config.json').read_text())
repo=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/outputs/drive-native-github')
def verify(label):
    p=subprocess.run(['python3',str(repo/'scripts/validation/verify_fixture.py'),'--root',str(pathlib.Path(cfg['mount'])/'lab'),'--manifest',str(root/'expected.json'),'--state',cfg['state']],capture_output=True,text=True,timeout=20)
    result=json.loads(p.stdout);(root/(label+'.json')).write_text(json.dumps(result,indent=2));assert p.returncode==0,result
    print(json.dumps({'verification':label,'passed':result['passed']}))
def stop(unit,label):
    start=time.monotonic();subprocess.run(['systemctl','--user','stop','--no-block',unit],check=True)
    while True:
        p=subprocess.check_output(['systemctl','--user','show',unit,'-p','MainPID','-p','Result','-p','ActiveState','-p','ExecMainStatus'],text=True)
        status=dict(line.split('=',1) for line in p.splitlines())
        if status['MainPID']=='0':break
        if time.monotonic()-start>100:raise RuntimeError('stop timeout; preserve experiment')
        time.sleep(.2)
    result={'seconds':time.monotonic()-start,'properties':status,'mount_remaining':subprocess.run(['findmnt','-M',cfg['mount']],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode==0}
    (root/(label+'.json')).write_text(json.dumps(result,indent=2));assert status['Result']=='success' and not result['mount_remaining']
    print(json.dumps({'stop':label,**result}))
verify('oracle-before-stop');stop(cfg['unit'],'first-stop')
unit=cfg['unit']+'-restart'
subprocess.run(['systemd-run','--user','--unit='+unit,'--property=UMask=0077','--setenv=XDG_DATA_HOME='+str(root/'data'),'--setenv=DRIVE_NATIVE_INSTANCE='+cfg['instance'],cfg['fsd'],'--config',str(root/'profile.json')],check=True)
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None);deadline=time.monotonic()+15
while True:
    try:
        v=bus.call_sync(cfg['bus'],'/org/drivenative/Client1','org.drivenative.Client1','GetOverview',None,GLib.VariantType.new('(s)'),Gio.DBusCallFlags.NONE,1000,None)
        d=json.loads(v.unpack()[0])
        if d['mounted']:break
    except GLib.Error:pass
    if time.monotonic()>deadline:raise RuntimeError('restart not mounted; preserve process')
    time.sleep(.1)
overview={k:d.get(k) for k in ['connection','mounted','writable','pending_operations','download_errors']}
(root/'restart-overview.json').write_text(json.dumps(overview,indent=2));assert overview['connection']=='locked' and overview['pending_operations']>0
verify('oracle-after-restart');stop(unit,'final-stop')
with (root.parent/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'end':time.time(),'run':'durability-001'})+'\n')
