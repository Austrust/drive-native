import hashlib,json,pathlib,shutil,sqlite3,subprocess,time,uuid
w=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/work')
home=pathlib.Path.home();base=home/'.local/state/drive-native-p0';soak=base.parent/'drive-native-soak'
build=pathlib.Path((w/'p0-wait2-build-location.txt').read_text())
binary=build/'diag-bin/drive-native-fsd';assert binary.is_file()
root=base/'wait-diagnostic-002';root.mkdir(mode=0o700,exist_ok=False)
task=next(t for t in json.loads((soak/'catalog.json').read_text()) if t['view_id']==14)
original=json.loads((soak/'profiles'/(task['instance']+'.json')).read_text())
assert not task['writable'] and not original['writable']
assert not any(k.lower() in ['password','token','secret'] for k in original)
with (base/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'start':time.time(),'run':root.name,'kind':'one readonly MFM clone; diagnostic outer470 inner450 with send/receive/keepalive phase logs; no NAS writes'})+'\n')
variants=[]
for seconds in [470]:
    run=root/str(seconds);run.mkdir(mode=0o700)
    task_id=str(uuid.uuid4());instance=task_id.replace('-','')
    mount=run/'mount';mount.mkdir(mode=0o700)
    state=run/'data/drive-native'/task_id;state.mkdir(parents=True,mode=0o700)
    old=soak/'data/drive-native'/task['task_id']
    source=sqlite3.connect((old/'state.sqlite').as_uri()+'?mode=ro',uri=True);dest=sqlite3.connect(state/'state.sqlite')
    source.backup(dest);source.close()
    assert dest.execute("SELECT count(*) FROM operations WHERE state NOT IN ('committed','superseded')").fetchone()[0]==0
    assert dest.execute('SELECT count(*) FROM drafts WHERE dirty!=0').fetchone()[0]==0
    assert dest.execute('SELECT count(*) FROM nodes WHERE dirty_blob IS NOT NULL').fetchone()[0]==0
    (state/'blobs').mkdir(mode=0o700)
    for (blob,) in dest.execute('SELECT DISTINCT blob FROM nodes WHERE blob IS NOT NULL'):
        assert pathlib.PurePath(blob).name==blob;shutil.copyfile(old/'blobs'/blob,state/'blobs'/blob)
    nodes=dest.execute('SELECT count(*) FROM nodes WHERE deleted=0').fetchone()[0];dest.close()
    profile=dict(original,task_id=task_id,device_id=str(uuid.uuid4()),mount=str(mount))
    config=run/'profile.json';config.write_text(json.dumps(profile));config.chmod(0o600)
    unit='drive-native-p0-wait2-'+str(seconds)
    metadata={'outer_seconds':seconds,'inner_seconds':450,'unit':unit+'.service','bus':'org.drivenative.Client1.i'+instance,'mount':str(mount),'state':str(state),'readonly':True,'nodes':nodes}
    (run/'config.json').write_text(json.dumps(metadata,indent=2));variants.append(metadata)
    subprocess.run(['systemd-run','--user','--unit='+unit,'--property=UMask=0077','--property=TimeoutStopSec=20','--setenv=XDG_DATA_HOME='+str(run/'data'),
      '--setenv=DRIVE_NATIVE_INSTANCE='+instance,'--setenv=P0_WAIT_OUTER_SECONDS='+str(seconds),str(binary),'--config',str(config)],check=True)
manifest={'base_commit':(w/'p0-wait2-base-commit.txt').read_text().strip(),'runtime_reference':'0.1.0~dev20261030.6','diagnostic_only':True,'not_a_deb_or_candidate':True,
 'binary_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (build/'diag-bin').iterdir()},'variants':variants,
 'auth_input':'fresh getpass; no saved credential read','payload_logging':False,'experiment_limit_s':900,'stop_after':'first bridge terminal error per variant or900s; initial online verification required',
 'TLS':'original certificate validation and trust profile unchanged','original_six_tasks_restarted':False}
(root/'manifest.json').write_text(json.dumps(manifest,indent=2))
print(json.dumps({'variants':[{'outer_seconds':x['outer_seconds'],'nodes':x['nodes'],'readonly':x['readonly']} for x in variants],'auth':'locked; not yet validated'}))
