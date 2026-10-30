import hashlib,json,pathlib,re,sys
r=pathlib.Path(sys.argv[1]).resolve()
d=r/'docs/validation/p0-wait-deadline-003'
c=json.loads((d/'checks.json').read_text())
result=json.loads((d/'result.json').read_text())['results'][0]
trace=[json.loads(s) for s in (d/'trace.jsonl').read_text().splitlines()]
final=[json.loads(s) for s in (d/'final-trace.jsonl').read_text().splitlines()]
assert final[:len(trace)]==trace
assert result['end_reason']=='bridge_terminal_error'
assert c['keepalive_frames']==0
ends=[t for t in trace if t['layer']=='native' and t['phase']=='exchange_end']
assert [t['outcome'] for t in ends]==['reply','native_timeout']
assert int(ends[-1]['elapsed_us'])==450000874
assert trace[-1]['phase']=='remote_failure' and trace[-1]['outcome']=='io_NotConnected'
assert trace[-1]['realtime_us']/1e6<result['stop_requested_at']
assert all(t['layer']=='service' and t['realtime_us']/1e6>=result['stop_requested_at'] for t in final[len(trace):])
assert c['stop']['properties']=={'MainPID':'0','Result':'success','ExecMainStatus':'0','ActiveState':'inactive'}
assert not c['stop']['mount_remaining']
assert len(c['original_tasks'])==6
assert all(t['properties']['ActiveState']=='active' and t['fsd_sha256']=='3082a758f32b7c0ecfda5058e2dc629b8d41172dacafdd798ca613c0aaf61678' for t in c['original_tasks'])
initial=[json.loads(s) for s in (d/'initial.jsonl').read_text().splitlines()]
assert initial==result['initial_samples'] and initial[-1]['connection']=='online'
manifest=json.loads((d/'server/manifest.json').read_text())
for f in manifest['functions']:
    rows=(d/'server'/f['disassembly']).read_text().splitlines()
    assert rows[0].startswith('_Z')
    addresses=[int(s.split(':',1)[0],16) for s in rows[1:] if s]
    start=int(f['address'],16)
    assert addresses[0]==start and addresses==sorted(set(addresses))
    assert all(start<=a<start+f['size'] for a in addresses)
(d/'SHA256SUMS').touch(exist_ok=True)
links=0
for p in [r/'DELIVERY_GATES.md',r/'docs/P0.md',r/'docs/REFERENCES.md',d/'REPORT.md']:
    for link in re.findall(r'\]\(([^)]+)\)',p.read_text(encoding='utf-8')):
        if re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:',link) or link.startswith('#'):continue
        assert (p.parent/link.split('#')[0]).exists(),(p,link)
        links+=1
(d/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.relative_to(d).as_posix()+'\n' for p in sorted(d.rglob('*')) if p.is_file() and p.name!='SHA256SUMS'),encoding='utf-8',newline='\n')
print(json.dumps({'failure_before_explicit_stop':True,'server_function_bounds_checked':len(manifest['functions']),'links_checked':links,'initial_samples':len(initial),'initial_error_samples':sum('error_type' in v for v in initial),'original_fsd_count':6}))
