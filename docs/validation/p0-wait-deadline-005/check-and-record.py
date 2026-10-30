import pathlib,json,re,hashlib,shutil
w=pathlib.Path(__file__).resolve().parent;r=w.parent/'outputs/drive-native-github';d=r/'docs/validation/p0-wait-deadline-005'
c=json.loads((d/'checks.json').read_text());result=json.loads((d/'result.json').read_text())['results'][0]
rows=[json.loads(s) for s in (d/'trace.jsonl').read_text().splitlines()]
final=[json.loads(s) for s in (d/'final-trace.jsonl').read_text().splitlines()]
assert final[:len(rows)]==rows and result['end_reason']=='10s_online_stop_boundary'
assert c['stop']['properties']=={'MainPID':'0','Result':'success','ExecMainStatus':'0','ActiveState':'inactive'} and not c['stop']['mount_remaining']
assert all(x['realtime_us']/1e6>=result['stop_requested_at'] for x in final if x.get('outcome','').startswith('io_'))
phases={x['phase']:x for x in final if x['layer'] in ['fsd','tasks']}
expected='tasks_join_begin supervisor_join_begin supervisor_join_end uploads_join_begin uploads_join_end cache_join_begin cache_join_end tasks_join_end fuse_join_begin fuse_join_end'.split()
assert all(x in phases for x in expected)
assert [phases[x]['monotonic_us'] for x in expected]==sorted(phases[x]['monotonic_us'] for x in expected)
native_pid=int(next(x['pid'] for x in final if x['layer']=='native'))
assert native_pid in [x['pid'] for x in result['before_stop_threads']['processes']]
assert len(c['original_tasks'])==6 and all(x['properties']['ActiveState']=='active' for x in c['original_tasks'])
(d/'SHA256SUMS').touch(exist_ok=True)
p=r/'docs/P0.md';s=p.read_text(encoding='utf-8');needle='[诊断004]'
assert s.count(needle)==1
line=next(x for x in s.splitlines() if x.startswith(needle))
s=s.replace(line,line+'\n\n[退出短测005](validation/p0-wait-deadline-005/REPORT.md)新增各任务和FUSE join阶段记录，在线10秒后停止正常完成，耗时0.218秒。它没有覆盖004的完整长等待后停止时序，原退出失败仍开放；下一步用相同阶段记录检查响应后的停止边界。')
p.write_text(s,encoding='utf-8',newline='\n')
links=0
for p in [r/'docs/P0.md',d/'REPORT.md']:
    for link in re.findall(r'\]\(([^)]+)\)',p.read_text(encoding='utf-8')):
        if re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:',link) or link.startswith('#'):continue
        assert (p.parent/link.split('#')[0]).exists(),(p,link);links+=1
shutil.copyfile(__file__,d/'check-and-record.py')
(d/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.relative_to(d).as_posix()+'\n' for p in sorted(d.rglob('*')) if p.is_file() and p.name!='SHA256SUMS'),encoding='utf-8',newline='\n')
print(json.dumps({'join_stages_verified':True,'direct_worker_observed':True,'links_checked':links}))
