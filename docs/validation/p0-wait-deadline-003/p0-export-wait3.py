import hashlib,json,pathlib,shutil,subprocess
w=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/work')
root=pathlib.Path.home()/'.local/state/drive-native-p0/wait-diagnostic-003'
result=json.loads((root/'result.json').read_text());assert len(result['results'])==1
out=w.parent/'outputs/drive-native-github/docs/validation/p0-wait-deadline-003';out.mkdir(exist_ok=False)
manifest=json.loads((root/'manifest.json').read_text())
manifest['variants']=[{k:v for k,v in x.items() if k in ['outer_seconds','inner_seconds','readonly','nodes','unit']} for x in manifest['variants']]
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
shutil.copyfile(root/'result.json',out/'result.json')
for name in ['initial.jsonl','trace.jsonl','final-trace.jsonl']:
    p=root/'470'/name
    if p.exists():shutil.copyfile(p,out/name)
for name in ['p0-instrument-wait.py','p0-instrument-wait2.py','p0-instrument-wait3.py','p0-build-wait3-diag.sh','p0-setup-wait3-diag.py','p0-observe-wait3-diag.py','p0-export-wait3.py','p0-wait3-diag-build.log','p0-wait3-diag-protocol-tests.log']:
    shutil.copyfile(w/name,out/name)
r=result['results'][0];rows=[json.loads(x) for x in (out/'final-trace.jsonl').read_text().splitlines()]
pre=[x for x in rows if x['realtime_us']/1e6<r['stop_requested_at']]
phases={x['phase']:x for x in pre if x['phase'].startswith('snapshot_')}
timings={label:(phases[end]['monotonic_us']-phases[start]['monotonic_us'])/1e6 for label,start,end in [('snapshot_rpc_s','snapshot_rpc_begin','snapshot_rpc_end'),('snapshot_install_s','snapshot_install_begin','snapshot_install_end')] if start in phases and end in phases}
checks={'experiment_end_reason':r['end_reason'],'initial_online_verified':r.get('initial_overview',{}).get('connection')=='online','timings':timings,
 'native_endings':[x for x in pre if x['layer']=='native' and x['phase']=='exchange_end'],
 'keepalive_frames':sum(x.get('kind')=='keep_alive' for x in pre),
 'stop':r['stop'],'original_tasks':[],'baseline_status':json.loads((root.parent/'status.json').read_text())}
catalog=json.loads((root.parent.parent/'drive-native-soak/catalog.json').read_text())
for task in catalog:
    unit='drive-native-soak@'+task['instance']+'.service'
    raw=subprocess.check_output(['systemctl','--user','show',unit,'-p','MainPID','-p','ActiveState'],text=True)
    props=dict(line.split('=',1) for line in raw.splitlines());pid=int(props['MainPID'])
    sha=hashlib.sha256(pathlib.Path('/proc',str(pid),'exe').read_bytes()).hexdigest() if pid else None
    checks['original_tasks'].append({'unit':unit,'properties':props,'fsd_sha256':sha})
assert len(checks['original_tasks'])==6 and all(t['properties']['ActiveState']=='active' and t['fsd_sha256']=='3082a758f32b7c0ecfda5058e2dc629b8d41172dacafdd798ca613c0aaf61678' for t in checks['original_tasks'])
(out/'checks.json').write_text(json.dumps(checks,indent=2))
print(json.dumps({k:v for k,v in checks.items() if k!='original_tasks'},indent=2))
