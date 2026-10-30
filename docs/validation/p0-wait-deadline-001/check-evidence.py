import hashlib,json,pathlib,re,sys
r=pathlib.Path(sys.argv[1]).resolve() if len(sys.argv)>1 else pathlib.Path(__file__).resolve().parents[3]
d=r/'docs/validation/p0-wait-deadline-001'
checks=json.loads((d/'checks.json').read_text())
result=json.loads((d/'result.json').read_text())
assert len(checks['original_tasks'])==6 and all(x['properties']['ActiveState']=='active' for x in checks['original_tasks'])
for run in result['results']:
    assert run['end_reason']=='600s_observation_deadline'
    assert run['stop']['properties']=={'MainPID':'0','Result':'success','ExecMainStatus':'0','ActiveState':'inactive'}
    assert not run['stop']['mount_remaining']
    base=d/str(run['outer_seconds'])
    trace=[json.loads(x) for x in (base/'trace.jsonl').read_text().splitlines()]
    final=[json.loads(x) for x in (base/'final-trace.jsonl').read_text().splitlines()]
    assert final[:len(trace)]==trace and len(final)==len(trace)+1
    assert all(not x.get('outcome','').startswith('io_') for x in trace)
    assert final[-1]['outcome']=='io_NotConnected' and final[-1]['realtime_us']/1e6>=run['stop_requested_at']
    assert len({x['pid'] for x in trace if x['layer']=='native'})==1
    for x in final:
        assert set(x)<=set('layer phase pid outer_s inner_s elapsed_us outcome ok monotonic_us realtime_us journal_pid'.split())
    assert sum(x['realtime_us']/1e6<run['observed_from'] for x in trace)==10
links=0
for p in [r/'DELIVERY_GATES.md',r/'docs/P0.md',d/'REPORT.md']:
    for link in re.findall(r'\]\(([^)]+)\)',p.read_text(encoding='utf-8')):
        if re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:',link) or link.startswith('#'):continue
        assert (p.parent/link.split('#')[0]).exists(),(str(p),link)
        links+=1
(d/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.relative_to(d).as_posix()+'\n' for p in sorted(d.rglob('*')) if p.is_file() and p.name!='SHA256SUMS'),encoding='utf-8',newline='\n')
print(json.dumps({'trace_schema_and_stop_boundaries_verified':True,'links_checked':links,'original_fsd_count':6,'evidence_files':sum(p.is_file() for p in d.rglob('*'))}))
