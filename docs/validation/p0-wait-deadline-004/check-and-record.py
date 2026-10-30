import pathlib,json,re,hashlib,shutil
w=pathlib.Path(__file__).resolve().parent;r=w.parent/'outputs/drive-native-github';d=r/'docs/validation/p0-wait-deadline-004'
c=json.loads((d/'checks.json').read_text());result=json.loads((d/'result.json').read_text())['results'][0]
rows=[json.loads(s) for s in (d/'trace.jsonl').read_text().splitlines()]
final=[json.loads(s) for s in (d/'final-trace.jsonl').read_text().splitlines()]
assert final[:len(rows)]==rows and result['end_reason']=='confirmed_reply_after450s'
assert all(t.get('outcome')!='native_timeout' for t in final)
ends=[t for t in rows if t['layer']=='native' and t['phase']=='exchange_end']
assert [t['outcome'] for t in ends]==['reply','reply'] and int(ends[1]['elapsed_us'])==483633068
assert rows[-1]['phase']=='notification'
errors=[t for t in final if t.get('outcome','').startswith('io_')]
assert len(errors)==1 and errors[0]['outcome']=='io_NotConnected' and errors[0]['realtime_us']/1e6>=result['stop_requested_at']
assert c['stop']['properties']=={'MainPID':'0','Result':'timeout','ExecMainStatus':'9','ActiveState':'failed'} and not c['stop']['mount_remaining']
assert len(c['original_tasks'])==6 and all(x['properties']['ActiveState']=='active' for x in c['original_tasks'])
p=r/'docs/P0.md';s=p.read_text(encoding='utf-8')
paragraph=next(line for line in s.splitlines() if line.startswith('诊断004已经启动'))
s=s.replace(paragraph,'[诊断004](validation/p0-wait-deadline-004/REPORT.md)已结束：600/650秒诊断截止收到483.633秒正常通知，直接确认旧450秒截止过短；随后显式停止虽完成shutdown并取消RPC，却在20秒后被SIGKILL。没有残留挂载，原六任务保持。下一步定位任务join及FUSE卸载卡点；本次不算整体修复通过，也未改产品。')
p.write_text(s,encoding='utf-8',newline='\n')
p=r/'DELIVERY_GATES.md';s=p.read_text(encoding='utf-8')
needle='新发布必须来自干净提交'
assert s.count(needle)==1
s=s.replace(needle,'[诊断004](docs/validation/p0-wait-deadline-004/REPORT.md)实测483.633秒正常通知，确认450秒旧截止会提前结束合法等待；但显式停止后20秒超时/SIGKILL，退出缺陷仍开放。原六任务保持，24h归档进度约3.79小时、16个失败采样。\n\n'+needle)
p.write_text(s,encoding='utf-8',newline='\n')
(d/'SHA256SUMS').touch(exist_ok=True)
links=0
for p in [r/'DELIVERY_GATES.md',r/'docs/P0.md',d/'REPORT.md']:
    for link in re.findall(r'\]\(([^)]+)\)',p.read_text(encoding='utf-8')):
        if re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:',link) or link.startswith('#'):continue
        assert (p.parent/link.split('#')[0]).exists(),(p,link);links+=1
shutil.copyfile(__file__,d/'check-and-record.py')
(d/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.relative_to(d).as_posix()+'\n' for p in sorted(d.rglob('*')) if p.is_file() and p.name!='SHA256SUMS'),encoding='utf-8',newline='\n')
print(json.dumps({'long_reply_verified':True,'explicit_stop_failure_preserved':True,'links_checked':links}))
