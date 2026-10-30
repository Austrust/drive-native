import hashlib,json,pathlib,subprocess,time,uuid
home=pathlib.Path.home();base=home/'.local/state/drive-native-p0';root=base/'durability-001';root.mkdir(mode=0o700,exist_ok=False)
builder=home/'.cache/drive-native-build/target/release/examples/durability_fixture'
subprocess.run([str(builder),str(root/'fixture')],check=True)
task=str(uuid.uuid4());instance=task.replace('-','');state=root/'data/drive-native'/task
state.parent.mkdir(parents=True);(root/'fixture/state').rename(state);state.chmod(0o700)
mount=root/'mount';mount.mkdir(mode=0o700)
fsd=home/'.local/lib/drive-native-soak/perf/drive-native-fsd';digest=hashlib.sha256(fsd.read_bytes()).hexdigest()
assert digest=='3082a758f32b7c0ecfda5058e2dc629b8d41172dacafdd798ca613c0aaf61678'
profile={'schema':1,'task_id':task,'host':'127.0.0.1:9','tls_name':'fixture.invalid','trusted_ca_pem':[],'username':'local-fixture','device_id':str(uuid.uuid4()),'scope':{'view_id':1,'node_id':1,'file_id':'root','path':'/'},'mount':str(mount),'writable':True}
(root/'profile.json').write_text(json.dumps(profile));(root/'profile.json').chmod(0o600)
cfg={'unit':'drive-native-p0-durability-001','bus':'org.drivenative.Client1.i'+instance,'instance':instance,'state':str(state),'mount':str(mount),'fsd':str(fsd),'fsd_sha256':digest,'builder_sha256':hashlib.sha256(builder.read_bytes()).hexdigest(),'builder_source':'71006ad','runtime_source':'62be13c9f9f4a2457a3040829f478bf43b1821df'}
(root/'config.json').write_text(json.dumps(cfg,indent=2))
manifest={'registered_at':time.time(),'scope':'synthetic local-only fixture; never Unlock; no NAS','cases':['write without fsync and inspect','fsync while handle remains open and inspect','close and inspect','temp write+fsync+rename replace+directory fsync','mkdir and child write+fsync+directory fsync','normal unmount/restart independent namespace/content verification'],'rounds':1,'limitations':'boundary observation only, not P2 repetitions or P3 failure/power-loss injection','operation_deadline_s':15}
(root/'manifest.json').write_text(json.dumps(manifest,indent=2))
replacement=bytes((i*7)%251 for i in range(8192));child=b'child-fsynced\n'
(root/'expected.json').write_text(json.dumps({'schema':1,'files':{'A.bin':hashlib.sha256(replacement).hexdigest(),'created/B.bin':hashlib.sha256(child).hexdigest()},'directories':['created']},indent=2))
with (base/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'start':time.time(),'run':'durability-001','kind':'small synthetic writable boundary test; no network'})+'\n')
subprocess.run(['systemd-run','--user','--unit='+cfg['unit'],'--property=UMask=0077','--setenv=XDG_DATA_HOME='+str(root/'data'),'--setenv=DRIVE_NATIVE_INSTANCE='+instance,str(fsd),'--config',str(root/'profile.json')],check=True)
print(json.dumps({'started':cfg['unit'],'network':'locked','nodes':3}))
