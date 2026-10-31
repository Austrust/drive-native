"""Offline package handover probe. Run in a private dbus-run-session as non-root.

Uses synthetic profiles and existing packages only. No keyring or NAS access.
The output directory must be new and every file mutation stays underneath it.
"""
import argparse, hashlib, json, os, pathlib, shutil, sqlite3, subprocess, time
from gi.repository import Gio, GLib

parser=argparse.ArgumentParser()
parser.add_argument('root',type=pathlib.Path)
parser.add_argument('old_package',type=pathlib.Path)
parser.add_argument('new_package',type=pathlib.Path)
parser.add_argument('fixture',type=pathlib.Path)
parser.add_argument('--new-binaries',type=pathlib.Path)
parser.add_argument('--fixed-handover',action='store_true')
parser.add_argument('--load-log',type=pathlib.Path)
args=parser.parse_args();root=args.root.resolve()
assert os.getuid()!=0 and not root.exists()
os.umask(0o077);root.mkdir()
def load_marker(kind):
    if args.load_log:
        with args.load_log.open('a') as stream:
            stream.write(json.dumps({'time':time.time(),'kind':kind,'scope':root.name,'activity':'isolated synthetic package extraction and FUSE handover; no performance comparison','baseline_changed':False})+'\n')
load_marker('isolated_development')
for name,package in [('old',args.old_package),('new',args.new_package)]:
    subprocess.run(['dpkg-deb','-x',str(package),str(root/name)],check=True)
if args.new_binaries:
    for binary in ['drive-native-fsd','drive-native-worker']:
        shutil.copy2(args.new_binaries/binary,root/'new/usr/lib/drive-native'/binary)
fixture=root/'fixture'
subprocess.run([str(args.fixture),str(fixture)],check=True,stdout=subprocess.DEVNULL)
profile=fixture/'config/drive-native/profile.json';p=json.loads(profile.read_text())
assert p['username']=='fixture' and p['host']=='example.invalid'
p['host']='127.0.0.1';p['tls_name']='localhost';p['writable']=True
profile.write_text(json.dumps(p)+'\n')
mount=pathlib.Path(p['mount']);state=fixture/'data/drive-native'/p['task_id']
installed=root/'installed';installed.mkdir()
def replace(name):
    for binary in ['drive-native-fsd','drive-native-worker']:
        tmp=installed/(binary+'.new');shutil.copy2(root/name/'usr/lib/drive-native'/binary,tmp);tmp.replace(installed/binary)
replace('old')
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
def call(method,params=None):
    out=bus.call_sync('org.drivenative.Client1','/org/drivenative/Client1','org.drivenative.Client1',method,params,None,Gio.DBusCallFlags.NO_AUTO_START,3000,None).unpack()
    return json.loads(out[0]) if out else None
def wait(predicate,seconds=15):
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        try:
            value=predicate()
            if value:return value
        except GLib.Error:pass
        time.sleep(.1)
    raise AssertionError('bounded condition did not complete')
def db_state():
    db=sqlite3.connect((state/'state.sqlite').as_uri()+'?mode=ro',uri=True)
    result={'cursor':db.execute("SELECT value FROM metadata WHERE key='committed_cursor'").fetchone()[0],
        'pending':db.execute("SELECT count(*) FROM operations WHERE state='pending'").fetchone()[0],
        'dirty':db.execute('SELECT count(*) FROM drafts WHERE dirty=1').fetchone()[0]}
    db.close();return result
def start():
    env=dict(os.environ,XDG_DATA_HOME=str(fixture/'data'),XDG_CONFIG_HOME=str(fixture/'config'))
    process=subprocess.Popen([str(installed/'drive-native-fsd'),'--config',str(profile)],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    def ready():
        if process.poll() is not None:
            raise AssertionError('isolated backend exited before mount became ready')
        return v if (v:=call('GetOverview'))['mounted'] else None
    try:
        wait(ready)
    except BaseException:
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=30)
        raise
    return process
def stop(process):
    call('StopMount');process.wait(timeout=30);assert process.returncode==0
result={'nas_accessed':False,'synthetic':True,'forced_cleanup':False}
process=None
try:
    process=start();old_pid=process.pid
    with (mount/'A.txt').open('r+b') as file:
        file.write(b'pending handover');file.truncate();file.flush();os.fsync(file.fileno())
    call('Pause');wait(lambda:call('GetOverview')['connection']=='paused')
    before=db_state();assert before['pending'] and before['dirty']
    body=(mount/'A.txt').read_bytes();assert body==b'pending handover'
    result['pause_keeps_cached_content']=True
    old_stat=os.stat(f'/proc/{old_pid}/exe');replace('new')
    current_stat=(installed/'drive-native-fsd').stat()
    result['installed_inode_changed']=(old_stat.st_dev,old_stat.st_ino)!=(current_stat.st_dev,current_stat.st_ino)
    result['paused_old_process_survives_install']=process.poll() is None and call('GetOverview')['connection']=='paused'
    assert (mount/'A.txt').read_bytes()==body
    if args.fixed_handover:
        from drive_native_gui.pause_handover import preserve_pause
        previous=os.environ.get('XDG_DATA_HOME')
        os.environ['XDG_DATA_HOME']=str(fixture/'data')
        try:preserve_pause(p)
        finally:
            if previous is None:os.environ.pop('XDG_DATA_HOME',None)
            else:os.environ['XDG_DATA_HOME']=previous
    else:
        # Only a loopback synthetic endpoint is configured for the old-version probe.
        call('Unlock',GLib.Variant('(s)',('synthetic-unused',)));call('Resume')
        failed=wait(lambda:(v if (v:=call('GetOverview'))['connection'] in ['offline','needs_attention'] else None))
        result['old_fsd_new_worker_state']={key:failed.get(key) for key in ['connection','error','mounted']}
        call('Pause')
    stop(process);process=None
    process=start();new=call('GetOverview')
    result['new_process_distinct']=process.pid!=old_pid
    result['state_after_new_start']=new['connection']
    result['pause_survived_handover']=new['connection']=='paused'
    result['pending_and_cursor_retained']=db_state()==before
    result['content_after_handover_correct']=(mount/'A.txt').read_bytes()==body
    stat=os.stat(f'/proc/{process.pid}/exe');installed_stat=(installed/'drive-native-fsd').stat()
    result['new_process_runs_installed_inode']=(stat.st_dev,stat.st_ino)==(installed_stat.st_dev,installed_stat.st_ino)
    if args.fixed_handover:
        assert result['pause_survived_handover']
        assert not (state/'startup-pause-v1.json').exists()
        call('Unlock',GLib.Variant('(s)',('synthetic-unused',)))
        assert call('GetOverview')['connection']=='paused'
        result['unlock_does_not_resume']=True
        replace('new')
        stop(process);process=None
        process=start()
        assert call('GetOverview')['connection']=='paused'
        assert db_state()==before
        assert (mount/'A.txt').read_bytes()==body
        result['second_handover_pause_and_data_retained']=True
    stop(process);process=None
finally:
    if process and process.poll() is None:
        process.terminate()
        try:process.wait(timeout=30)
        except subprocess.TimeoutExpired:process.kill();process.wait();result['forced_cleanup']=True
    result['finished_at']=time.time()
    (root/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    load_marker('isolated_development_completed')
    print(json.dumps(result))
