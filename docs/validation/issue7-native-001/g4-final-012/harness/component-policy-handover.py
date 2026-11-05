"""Private real-component lifecycle probe. This is NOT a Debian install test.

Run only after coordinator authorization, under a private dbus-run-session.
All file/IPC inputs are synthetic. The sole TCP endpoint is owned loopback.
"""
import argparse, errno, hashlib, json, os, pathlib, shutil, socket, sqlite3
import subprocess, threading, time, traceback
from gi.repository import Gio, GLib

p=argparse.ArgumentParser()
p.add_argument('--root',type=pathlib.Path,required=True)
p.add_argument('--fixture',type=pathlib.Path,required=True)
p.add_argument('--old-binaries',type=pathlib.Path,required=True)
p.add_argument('--new-binaries',type=pathlib.Path,required=True)
p.add_argument('--old-commit',required=True)
p.add_argument('--new-commit',required=True)
a=p.parse_args();root=a.root.resolve()
base=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
assert os.getuid()!=0 and root.is_relative_to(base) and not root.exists()
assert os.environ.get('DBUS_SESSION_BUS_ADDRESS'),'private session bus required'
for path in (a.fixture,a.old_binaries,a.new_binaries):assert path.resolve().is_relative_to(base)
os.umask(0o077);root.mkdir();(root/'logs').mkdir()
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
result={'passed':False,'scope':'real IPC21 component pair, private files/D-Bus/FUSE; not a Debian package lifecycle',
    'old_commit':a.old_commit,'new_commit':a.new_commit,'cases':[], 'commands':[], 'nas_accessed':False,
    'expected_ipc_version':21,'native_login_success_claimed':False,'binary_identity':{},'cleanup':[]}
proc=None;streams=[];installed=root/'installed';installed.mkdir()
def publish(): (root/'result.json').write_text(json.dumps(result,indent=2)+'\n')
def command(argv,env=None):
    index=len(result['commands']);stem=f'command-{index:03}'
    row={'argv':argv,'stdout':f'logs/{stem}.stdout.log','stderr':f'logs/{stem}.stderr.log'}
    result['commands'].append(row);publish();started=time.monotonic()
    try:
        process=subprocess.run(argv,env=env,capture_output=True,timeout=30)
        (root/row['stdout']).write_bytes(process.stdout);(root/row['stderr']).write_bytes(process.stderr)
        row.update(exit_code=process.returncode,elapsed_seconds=time.monotonic()-started,
            stdout_sha256=sha(root/row['stdout']),stderr_sha256=sha(root/row['stderr']))
        publish();process.check_returncode();return process
    except BaseException as error:
        if not (root/row['stdout']).exists():(root/row['stdout']).write_bytes(getattr(error,'stdout',None) or b'')
        if not (root/row['stderr']).exists():(root/row['stderr']).write_bytes(getattr(error,'stderr',None) or b'')
        row.update(exception_type=type(error).__name__,elapsed_seconds=time.monotonic()-started,
            stdout_sha256=sha(root/row['stdout']),stderr_sha256=sha(root/row['stderr']))
        publish();raise
command([str(a.fixture),str(root/'fixture')])
fixture=root/'fixture';config=fixture/'config/drive-native/profile.json'
profile=json.loads(config.read_text())
assert profile['username']=='fixture' and profile['host']=='example.invalid'
profile.update(host='127.0.0.1',tls_name='localhost',writable=False)
profile.pop('local_policy',None);profile.pop('writable_source',None)
config.write_text(json.dumps(profile)+'\n');legacy_config_sha=sha(config)
mount=pathlib.Path(profile['mount']);assert mount.is_relative_to(fixture)
state=fixture/'data/drive-native'/profile['task_id']
env=dict(os.environ,XDG_CONFIG_HOME=str(fixture/'config'),XDG_DATA_HOME=str(fixture/'data'))
for key,name in [('XDG_CACHE_HOME','cache'),('XDG_RUNTIME_DIR','runtime')]:
    folder=fixture/name;folder.mkdir(mode=0o700);env[key]=str(folder)
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
def call(method,params=None):
    value=bus.call_sync('org.drivenative.Client1','/org/drivenative/Client1','org.drivenative.Client1',method,
        params,None,Gio.DBusCallFlags.NO_AUTO_START,5000,None).unpack()
    return json.loads(value[0]) if value else None
def wait(check,label,seconds=20):
    end=time.monotonic()+seconds
    while time.monotonic()<end:
        try:
            value=check()
            if value:return value
        except GLib.Error:pass
        time.sleep(.02)
    raise AssertionError(label)
def identity(path):
    st=path.stat();return {'device':st.st_dev,'inode':st.st_ino}
def replace_pair(directory,label):
    records={}
    for name in ('drive-native-fsd','drive-native-worker'):
        origin=directory/name;assert origin.is_file()
        temp=installed/(name+'.new');shutil.copy2(origin,temp);temp.replace(installed/name)
        records[name]={'source':str(origin),'sha256':sha(origin),'installed_identity':identity(installed/name)}
        assert sha(installed/name)==records[name]['sha256']
    result['binary_identity'][label]=records;publish();return records
def start(label):
    global proc
    log=(root/'logs'/f'{label}.log').open('w');streams.append(log)
    argv=[str(installed/'drive-native-fsd'),'--config',str(config)]
    result['commands'].append(argv);publish()
    proc=subprocess.Popen(argv,env=env,stdout=log,stderr=subprocess.STDOUT)
    def ready():
        assert proc.poll() is None,'fsd exited before mount'
        value=call('GetOverview')
        return value if value['mounted'] and mount.is_mount() else None
    return wait(ready,'FUSE mount ready')
def stop():
    global proc
    call('StopMount');proc.wait(timeout=30);assert proc.returncode==0
    proc=None;wait(lambda:not mount.is_mount(),'normal unmount')
def snapshot():
    with sqlite3.connect((state/'state.sqlite').as_uri()+'?mode=ro',uri=True) as db:
        return {'user_version':db.execute('PRAGMA user_version').fetchone()[0],
            'cursor':db.execute("SELECT value FROM metadata WHERE key='committed_cursor'").fetchone()[0],
            'operations':db.execute('SELECT id,node,kind,state FROM operations ORDER BY id').fetchall(),
            'drafts':db.execute('SELECT node,dirty FROM drafts ORDER BY node').fetchall(),
            'history':db.execute('SELECT id,seq,action,result,operation_key FROM sync_history ORDER BY id').fetchall(),
            'policy':db.execute('SELECT root,local_policy,local_policy_source,server_capability,server_capability_source FROM write_policy ORDER BY root').fetchall()}
def check_preserved(label,before,body):
    overview=call('GetOverview');assert overview['local_policy']=='follow_remote'
    assert overview['writable_source']=='user';assert sha(config)==legacy_config_sha
    assert (mount/'A.txt').read_bytes()==body;assert snapshot()==before
    states=call('GetStates',GLib.Variant('(as)',(['A.txt'],)))
    assert states['entries'][0]['pinned'] is True
    result['cases'].append({'name':label,'passed':True,'policy':overview['local_policy'],
        'source':overview['writable_source'],'revision':overview['write_policy_revision'],'body_sha256':hashlib.sha256(body).hexdigest()})
    publish()
def worker_pair_probe(label,expected_worker):
    # Claim the endpoint first; if occupied, fail without connecting to its owner.
    server=socket.socket();server.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
    server.bind(('127.0.0.1',6690));server.listen(1);server.settimeout(12)
    accepted=threading.Event();release=threading.Event();observed={};errors=[]
    def endpoint():
        try:
            conn,peer=server.accept()
            with conn:
                conn.settimeout(5);data=conn.recv(256)
                assert data,'worker must send native initialization bytes'
                observed.update(peer_is_loopback=peer[0]=='127.0.0.1',native_initial_bytes=len(data))
                accepted.set();assert release.wait(8),'owned worker inspection timed out'
                # Deliberately fail before TLS/auth. No payload or credential is logged.
        except BaseException as error:errors.append(type(error).__name__+': '+str(error));accepted.set()
        finally:server.close()
    thread=threading.Thread(target=endpoint);thread.start()
    try:
        before_revision=call('GetOverview')['write_policy_revision']
        call('Unlock',GLib.Variant('(s)',('synthetic-unused',)));call('Resume')
        assert accepted.wait(15) and not errors,errors
        def child():
            for task in (pathlib.Path('/proc')/str(proc.pid)/'task').iterdir():
                try:children=(task/'children').read_text().split()
                except FileNotFoundError:continue
                for pid in children:
                    try:
                        executable=pathlib.Path('/proc')/pid/'exe'
                        if identity(executable)==expected_worker:
                            return {'pid':int(pid),'executable_identity':identity(executable)}
                    except FileNotFoundError:pass
            return None
        observed.update(wait(child,'actual worker image identity',5))
        assert observed['peer_is_loopback'];release.set();thread.join(timeout=10);assert not thread.is_alive()
        call('Pause');wait(lambda:call('GetOverview')['connection']=='paused','pause after local probe')
        after_revision=call('GetOverview')['write_policy_revision']
        assert after_revision>before_revision,'explicit Resume must invalidate observed evidence'
        result['cases'].append({'name':label,'passed':True,**observed,
            'revision_before_resume':before_revision,'revision_after_resume':after_revision,
            'scope':'fsd-owned actual worker initiated native traffic after strict IPC handshake; TLS/auth deliberately not completed'})
        publish()
    finally:
        release.set();server.close();thread.join(timeout=15)
        assert not thread.is_alive(),'loopback endpoint remains'
try:
    old=replace_pair(a.old_binaries,'old-pair')
    initial=start('legacy-ro')
    assert initial['local_policy']=='explicit_read_only' and initial['writable_source']=='legacy'
    assert (mount/'A.txt').read_bytes()==b'legacy'
    try:
        with (mount/'A.txt').open('r+b'):pass
    except OSError as error:assert error.errno==errno.EROFS,error
    else:raise AssertionError('legacy read-only mount accepted write open')
    query={'task_id':profile['task_id'],'expected_revision':initial['write_policy_revision'],'local_policy':'follow_remote'}
    saved=call('SetWritePolicy',GLib.Variant('(s)',(json.dumps(query),)))
    assert saved=={'saved':True,'remount_required':True},saved
    stop();after=start('legacy-follow-remount')
    assert after['local_policy']=='follow_remote' and after['writable_source']=='user'
    assert after['write_policy_revision']>initial['write_policy_revision']
    body=b'private issue7 unsent draft survives component replacement\n'
    with (mount/'A.txt').open('r+b') as stream:
        stream.write(body);stream.truncate();stream.flush();os.fsync(stream.fileno())
    call('SetPinned',GLib.Variant('(sb)',('A.txt',True)))
    call('Pause');wait(lambda:call('GetOverview')['connection']=='paused','paused')
    before=snapshot();assert before['user_version']==25
    assert any(row[3]=='pending' for row in before['operations']) and any(row[1] for row in before['drafts'])
    assert before['history'],'actual pending history must exist'
    check_preserved('legacy_false_explicit_follow_remote_and_actual_remount',before,body)
    current=replace_pair(a.new_binaries,'new-pair')
    assert proc.poll() is None
    check_preserved('running_old_fsd_keeps_policy_and_pending_data_after_pair_replace',before,body)
    worker_pair_probe('old_fsd_pins_old_worker_inode_across_replacement',old['drive-native-worker']['installed_identity'])
    check_preserved('old_pair_after_local_worker_probe',before,body)
    stop();start('new-pair-remount')
    check_preserved('new_pair_remount_keeps_policy_pending_body_history_cursor',before,body)
    worker_pair_probe('new_fsd_uses_new_worker_inode',current['drive-native-worker']['installed_identity'])
    check_preserved('new_pair_after_local_worker_probe',before,body)
    stop()
    # Retain removed payload for evidence. This is file lifecycle, not dpkg remove.
    retired=root/'removed-component-payload';retired.mkdir()
    for name in ('drive-native-fsd','drive-native-worker'):(installed/name).rename(retired/name)
    assert snapshot()==before and sha(config)==legacy_config_sha
    replace_pair(a.new_binaries,'restored-pair');start('restored-pair-remount')
    check_preserved('payload_remove_restore_and_remount_preserves_user_state',before,body)
    stop();result['passed']=True
except BaseException:
    result['failure_traceback']=traceback.format_exc();raise
finally:
    if proc and proc.poll() is None:
        try:stop();result['cleanup'].append('normal StopMount after failure')
        except BaseException:
            proc.terminate()
            try:proc.wait(timeout=30)
            except subprocess.TimeoutExpired:proc.kill();proc.wait(timeout=15)
            result['cleanup'].append('forced fsd termination after failure')
    if mount.is_mount():
        cleanup=subprocess.run(['fusermount3','-u','--',str(mount)],capture_output=True,text=True,timeout=30)
        result['cleanup'].append({'forced_unmount':str(mount),'exit_code':cleanup.returncode,'stderr':cleanup.stderr})
        result['passed']=False
    for stream in streams:stream.close()
    result['residual_mount']=mount.is_mount()
    result['counts']={'passed':sum(row['passed'] for row in result['cases']),'failed':0 if result['passed'] else 1,'ignored':0}
    result['passed']=result['passed'] and not result['residual_mount']
    publish();print(json.dumps(result,indent=2))
