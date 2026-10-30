import hashlib,json,pathlib,shutil,subprocess
w=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/work')
root=pathlib.Path.home()/'.local/state/drive-native-p0/wait-diagnostic-002'
out=w.parent/'outputs/drive-native-github/docs/validation/p0-wait-deadline-002'
result=json.loads((root/'result.json').read_text());assert len(result['results'])==1
r=result['results'][0];assert r['end_reason']=='observer_error' and r['observed_rows']==0
assert r['stop']['properties']['Result']=='timeout' and r['stop']['properties']['ExecMainStatus']=='9'
assert not r['stop']['mount_remaining']
out.mkdir(exist_ok=False)
manifest=json.loads((root/'manifest.json').read_text())
manifest['variants']=[{k:v for k,v in x.items() if k in ['outer_seconds','inner_seconds','readonly','nodes','unit']} for x in manifest['variants']]
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
for name in ['result.json','local-state-counts.json','unit-journal-categories.json']:shutil.copyfile(root/name,out/name)
shutil.copyfile(root/'470/final-trace.jsonl',out/'final-trace.jsonl')
for name in ['p0-instrument-wait.py','p0-instrument-wait2.py','p0-build-wait2-diag.sh','p0-setup-wait2-diag.py','p0-observe-wait2-diag.py','p0-read-wait2-stop.py','p0-export-wait2.py','p0-wait2-diag-build.log','p0-wait2-diag-protocol-tests.log']:
    shutil.copyfile(w/name,out/name)
tasks=[]
catalog=json.loads((root.parent.parent/'drive-native-soak/catalog.json').read_text())
for task in catalog:
    unit='drive-native-soak@'+task['instance']+'.service'
    raw=subprocess.check_output(['systemctl','--user','show',unit,'-p','MainPID','-p','ActiveState'],text=True)
    p=dict(line.split('=',1) for line in raw.splitlines());pid=int(p['MainPID'])
    sha=hashlib.sha256(pathlib.Path('/proc',str(pid),'exe').read_bytes()).hexdigest() if pid else None
    tasks.append({'unit':unit,'properties':p,'fsd_sha256':sha})
assert len(tasks)==6 and all(t['properties']['ActiveState']=='active' and t['fsd_sha256']=='3082a758f32b7c0ecfda5058e2dc629b8d41172dacafdd798ca613c0aaf61678' for t in tasks)
(out/'checks.json').write_text(json.dumps({'long_poll_observation_entered':False,'stop_timeout_preserved':True,'mount_remaining':False,'original_tasks':tasks,'baseline_status':json.loads((root.parent/'status.json').read_text())},indent=2))
(out/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n' for p in sorted(out.iterdir()) if p.is_file() and p.name!='SHA256SUMS'))
print(json.dumps({'failure_saved':True,'long_poll_observation_entered':False,'original_six_fsd_hashes_match':True,'output':str(out)}))
