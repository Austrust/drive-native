import hashlib,json,pathlib,shutil,subprocess
w=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/work')
root=pathlib.Path.home()/'.local/state/drive-native-p0/wait-diagnostic-001'
out=w.parent/'outputs/drive-native-github/docs/validation/p0-wait-deadline-001'
result=json.loads((root/'result.json').read_text())
assert len(result['results'])==2 and not result['runtime_product_changed']
out.mkdir(exist_ok=False)
manifest=json.loads((root/'manifest.json').read_text())
# Do not export profiles, databases, credentials or content.
safe_manifest={k:v for k,v in manifest.items() if k!='variants'}
safe_manifest['variants']=[{k:v for k,v in x.items() if k in ['outer_seconds','inner_seconds','readonly','nodes','unit']} for x in manifest['variants']]
(out/'manifest.json').write_text(json.dumps(safe_manifest,indent=2))
shutil.copyfile(root/'result.json',out/'result.json')
summary=[]
for r in result['results']:
    n=str(r['outer_seconds']);dest=out/n;dest.mkdir()
    for name in ['trace.jsonl','final-trace.jsonl','result.json']:shutil.copyfile(root/n/name,dest/name)
    rows=[json.loads(s) for s in (dest/'trace.jsonl').read_text().splitlines()]
    assert all(x['realtime_us']/1e6<r['stop_requested_at'] for x in rows)
    assert r['initial_overview']['connection']=='online' and r['initial_overview']['mounted']
    terminal=[x for x in rows if x['phase'] in ['reply','remote_failure'] and x.get('outcome','').startswith('io_')]
    native_end=[x for x in rows if x['layer']=='native' and x['phase']=='exchange_end']
    summary.append({'outer_seconds':int(n),'terminal_errors':terminal,'native_exchanges_completed':native_end,'history_rows_before_observation':sum(x['realtime_us']/1e6<r['observed_from'] for x in rows),'observation_window_rows':sum(x['realtime_us']/1e6>=r['observed_from'] for x in rows),'stop':r['stop']})
for source,dest in [('p0-instrument-wait.py','instrument.py'),('p0-build-wait-diag.sh','build.sh'),('p0-setup-wait-diag.py','setup.py'),('p0-observe-wait-diag.py','observe.py'),('p0-export-wait-diag.py','export.py'),('p0-wait-diag-build.log','build.log'),('p0-wait-diag-protocol-tests.log','protocol-tests.log')]:
    shutil.copyfile(w/source,out/dest)
original=[]
catalog=json.loads((root.parent.parent/'drive-native-soak/catalog.json').read_text())
for task in catalog:
    unit='drive-native-soak@'+task['instance']+'.service'
    raw=subprocess.check_output(['systemctl','--user','show',unit,'-p','MainPID','-p','ActiveState'],text=True)
    props=dict(line.split('=',1) for line in raw.splitlines())
    pid=int(props['MainPID'])
    digest=hashlib.sha256(pathlib.Path('/proc',str(pid),'exe').read_bytes()).hexdigest() if pid else None
    original.append({'view_id':task['view_id'],'unit':unit,'properties':props,'fsd_sha256':digest})
checks={'variants':summary,'original_tasks':original,'baseline_status':json.loads((root.parent/'status.json').read_text()),'runtime_product_changed':False,'experiment_NAS_writes':False,'published_package':False}
(out/'checks.json').write_text(json.dumps(checks,indent=2))
assert all(x['properties']['ActiveState']=='active' and x['fsd_sha256']=='3082a758f32b7c0ecfda5058e2dc629b8d41172dacafdd798ca613c0aaf61678' for x in original)
(out/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.relative_to(out).as_posix()+'\n' for p in sorted(out.rglob('*')) if p.is_file() and p.name!='SHA256SUMS'))
print(json.dumps(checks,indent=2))
