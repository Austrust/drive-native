import json,os,pathlib,subprocess,time
root=pathlib.Path.home()/'.local/state/drive-native-p0/fixed-001'
out=root/'idle-001';out.mkdir(exist_ok=False)
cfg=json.loads((root/'config.json').read_text())
manifest={'registered_at':time.time(),'settle_seconds':90,'window_seconds':300,'sample_interval_seconds':5,
 'unit':cfg['unit'],'binary_sha256':cfg['binary_sha256'],'nodes':210000,
 'scope':'synthetic locked/offline fsd and direct children; no GUI, no network worker, no requests during measured window; other NAS mounts/probes remain running',
 'cpu':'sum CPU-seconds differences for identical PID set / monotonic wall interval, one core=100%; PID change invalidates interval',
 'memory':'sum RSS duplicates shared pages; also record summed PSS; no allocator trimming'}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
time.sleep(90)
for i in range(61):
 start=time.monotonic();r={'sample':i,'time':time.time(),'monotonic':start}
 try:
  pid=int(subprocess.check_output(['systemctl','--user','show',cfg['unit'],'-p','MainPID','--value'],text=True))
  assert pid>0
  children=set(int(c) for f in pathlib.Path(f'/proc/{pid}/task').glob('*/children') for c in f.read_text().split())
  procs=[]
  for n in [pid]+sorted(children):
   p=pathlib.Path(f'/proc/{n}');status=dict(v.split(':',1) for v in (p/'status').read_text().splitlines() if ':' in v)
   stat=(p/'stat').read_text().rsplit(')',1)[1].split();mem=dict(v.split(':',1) for v in (p/'smaps_rollup').read_text().splitlines() if ':' in v)
   procs.append({'pid':n,'cpu_seconds':(int(stat[11])+int(stat[12]))/os.sysconf('SC_CLK_TCK'),
    'rss_kib':int(status['VmRSS'].split()[0]),'pss_kib':int(mem['Pss'].split()[0]),'threads':int(status['Threads']),
    'fds':len(list((p/'fd').iterdir()))})
  r.update(ok=True,processes=procs)
 except Exception as e:r.update(ok=False,error=type(e).__name__)
 with (out/'samples.jsonl').open('a') as f:f.write(json.dumps(r)+'\n')
 if i<60:time.sleep(max(0,5-(time.monotonic()-start)))
print('idle samples complete; requires review',flush=True)
