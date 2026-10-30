import hashlib,json,pathlib,shutil
base=pathlib.Path.home()/'.local/state/drive-native-p0/fixed-001'
src=base/'idle-001'
rows=[json.loads(x) for x in (src/'samples.jsonl').read_text().splitlines()]
assert len(rows)==61 and all(r['ok'] for r in rows)
ids={p['pid'] for p in rows[0]['processes']}
assert all({p['pid'] for p in r['processes']}==ids for r in rows)
cfg=json.loads((base/'config.json').read_text())
for pid in ids:assert hashlib.sha256(pathlib.Path(f'/proc/{pid}/exe').read_bytes()).hexdigest()==cfg['binary_sha256']
window=rows[-1]['monotonic']-rows[0]['monotonic']
cpu=lambda r:sum(p['cpu_seconds'] for p in r['processes'])
rss=[sum(p['rss_kib'] for p in r['processes']) for r in rows]
pss=[sum(p['pss_kib'] for p in r['processes']) for r in rows]
summary={'observed_window_seconds':window,'samples':61,'errors':0,'stable_pid_set':True,
 'single_core_cpu_percent':100*(cpu(rows[-1])-cpu(rows[0]))/window,
 'rss_kib_min':min(rss),'rss_kib_max':max(rss),'rss_kib_first':rss[0],'rss_kib_last':rss[-1],
 'pss_kib_min':min(pss),'pss_kib_max':max(pss),'rss_growth_percent':100*(rss[-1]-rss[0])/rss[0],
 'fds_min':min(sum(p['fds'] for p in r['processes']) for r in rows),'fds_max':max(sum(p['fds'] for p in r['processes']) for r in rows),
 'threads_min':min(sum(p['threads'] for p in r['processes']) for r in rows),'threads_max':max(sum(p['threads'] for p in r['processes']) for r in rows),
 'scope':'offline synthetic 210k-node task, after mixed load; no GUI or network worker; not online idle or 7-day growth proof'}
dest=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/outputs/drive-native-github/docs/validation/p0-fixed-001/idle')
dest.mkdir(exist_ok=False)
for name in ['manifest.json','samples.jsonl']:shutil.copyfile(src/name,dest/name)
(dest/'summary.json').write_text(json.dumps(summary,indent=2))
print(json.dumps(summary))
