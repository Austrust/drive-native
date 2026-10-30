import hashlib,json,pathlib,shutil,sqlite3,subprocess,time,uuid
home=pathlib.Path.home();soak=home/'.local/state/drive-native-soak';base=home/'.local/state/drive-native-p0'
catalog=json.loads((soak/'catalog.json').read_text())
repo=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/outputs/drive-native-github')
for label,match,address in [('personal',lambda t:t['name']=='Personal_Sync','127.0.0.3'),('training',lambda t:t['view_id']==11,'127.0.0.4')]:
    root=base/('relay-'+label+'-001');root.mkdir(mode=0o700,exist_ok=False)
    task=next(t for t in catalog if match(t));assert not task['writable']
    profile=json.loads((soak/'profiles'/(task['instance']+'.json')).read_text());assert not profile['writable']
    assert not any(k.lower() in ['password','token','secret'] for k in profile)
    upstream=profile['host'];new_id=str(uuid.uuid4());instance=new_id.replace('-','')
    mount=root/'mount';mount.mkdir(mode=0o700)
    state=root/'data/drive-native'/new_id;state.mkdir(parents=True,mode=0o700)
    old=soak/'data/drive-native'/task['task_id']
    source=sqlite3.connect(old.joinpath('state.sqlite').as_uri()+'?mode=ro',uri=True);dest=sqlite3.connect(state/'state.sqlite')
    source.backup(dest);source.close()
    assert dest.execute("SELECT count(*) FROM operations WHERE state NOT IN ('committed','superseded')").fetchone()[0]==0
    assert dest.execute('SELECT count(*) FROM drafts WHERE dirty!=0').fetchone()[0]==0
    assert dest.execute('SELECT count(*) FROM nodes WHERE dirty_blob IS NOT NULL').fetchone()[0]==0
    (state/'blobs').mkdir(mode=0o700)
    blobs=list(dest.execute('SELECT DISTINCT blob FROM nodes WHERE blob IS NOT NULL'))
    for (blob,) in blobs:
        assert pathlib.PurePath(blob).name==blob
        shutil.copyfile(old/'blobs'/blob,state/'blobs'/blob)
    nodes=dest.execute('SELECT count(*) FROM nodes WHERE deleted=0').fetchone()[0];dest.close()
    profile.update(host=address,task_id=new_id,device_id=str(uuid.uuid4()),mount=str(mount))
    config=root/'profile.json';config.write_text(json.dumps(profile));config.chmod(0o600)
    binary=home/'.local/lib/drive-native-soak/perf/drive-native-fsd';digest=hashlib.sha256(binary.read_bytes()).hexdigest()
    assert digest=='3082a758f32b7c0ecfda5058e2dc629b8d41172dacafdd798ca613c0aaf61678'
    relay_unit='drive-native-p0-relay-'+label;client_unit=relay_unit+'-client'
    metadata={'bus':'org.drivenative.Client1.i'+instance,'unit':client_unit+'.service','relay_unit':relay_unit+'.service','task_id':new_id,'state':str(state),'mount':str(mount),'readonly':True,'nodes_before':nodes,'source_scope':label,'tls_name_unchanged':True,'runtime_sha256':digest}
    (root/'config.json').write_text(json.dumps(metadata,indent=2))
    with (base/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'start':time.time(),'run':root.name,'kind':'additional isolated readonly clone initialization'})+'\n')
    subprocess.run(['systemd-run','--user','--unit='+relay_unit,'--property=UMask=0077','/usr/bin/python3',str(repo/'scripts/validation/byte_relay.py'),'--state',str(root),'--listen-host',address,'--upstream-host',upstream],check=True)
    subprocess.run(['systemd-run','--user','--unit='+client_unit,'--property=UMask=0077','--setenv=XDG_DATA_HOME='+str(root/'data'),'--setenv=DRIVE_NATIVE_INSTANCE='+instance,str(binary),'--config',str(config)],check=True)
    print(json.dumps({'scope':label,'nodes':nodes,'complete_blobs':len(blobs),'auth':'locked; TLS check not yet executed'}))
