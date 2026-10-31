"""Local watchdog; records bounded diagnostics, never credentials or file bodies."""
import argparse,asyncio,ast,json,pathlib,re,time
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--state-dir',type=pathlib.Path,default=pathlib.Path.home()/'.local/state/drive-native-soak')
options=parser.parse_args()
ROOT=options.state_dir
TASKS=json.loads((ROOT/'catalog.json').read_text())
def emit(row):
 row['time']=time.time()
 log=ROOT/'watch-probe.jsonl'
 if log.exists() and log.stat().st_size>=8*1024*1024:
  for i in range(2,0,-1):
   old=ROOT/f'watch-probe.jsonl.{i}'
   if old.exists():old.replace(ROOT/f'watch-probe.jsonl.{i+1}')
  log.replace(ROOT/'watch-probe.jsonl.1')
 with log.open('a') as f:f.write(json.dumps(row,ensure_ascii=False)+'\n')
async def run(args,timeout=3):
 p=await asyncio.create_subprocess_exec(*args,stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.DEVNULL)
 try:
  out,_=await asyncio.wait_for(p.communicate(),timeout)
  if p.returncode:raise RuntimeError('command_failed')
  return out.decode()
 except asyncio.TimeoutError:
  p.kill();await p.communicate();raise
async def call(t,method,args=()):
 out=await run(['gdbus','call','--session','--dest',t['bus'],'--object-path','/org/drivenative/Client1','--method','org.drivenative.Client1.'+method,*args])
 return json.loads(ast.literal_eval(out)[0])
async def watch(t):
 previous=None;iteration=0
 while True:
  row={'instance':t['instance'],'name':t['name'],'kind':'watchdog'}
  start=time.monotonic()
  try:
   d=await call(t,'GetOverview');row['overview_ms']=round((time.monotonic()-start)*1000,2)
   row.update({k:d.get(k) for k in ['connection','error','background_error','mounted','downloads_started','download_errors','skipped_permission_directories']})
   state=tuple(row.get(k) for k in ['connection','error','background_error','mounted'])
   row['transition']=state!=previous;previous=state
   if d.get('mounted'):
    code='import os,sys,time,json;t=time.monotonic();n=len(os.listdir(sys.argv[1]));print(json.dumps({"entries":n,"readdir_ms":1000*(time.monotonic()-t)}))'
    row.update(json.loads(await run(['/usr/bin/python3','-c',code,t['mount']])))
    if iteration%6==0:
     start=time.monotonic();states=await call(t,'GetStates',[json.dumps([t['mount']])]);row['states_ms']=round((time.monotonic()-start)*1000,2)
     row['states_valid']=all(e.get('error') is None for e in states['entries'])
   unit='drive-native-soak@'+t['instance']+'.service'
   stats=await run(['systemctl','--user','show',unit,'-p','MainPID','-p','NRestarts','-p','MemoryCurrent'])
   row['process']={k:v for k,v in (line.split('=',1) for line in stats.splitlines() if '=' in line)}
  except Exception as e:row['probe_error']=type(e).__name__
  emit(row);iteration+=1;await asyncio.sleep(10)
STATUS_SIGNAL=re.compile(r'org\.drivenative\.Client1\.StatusChanged\s+\((?:uint64\s+)?(\d+),?\)')
async def signal_state(t,wakeup,notice):
 """At most one local query per task, coalesced; never traverse FUSE here."""
 seen=0
 while True:
  await wakeup.wait();await asyncio.sleep(.2);wakeup.clear()
  serial=notice['signals'];revision=notice['revision']
  row={'kind':'status_event','instance':t['instance'],'signals':serial-seen,'signal_revision':revision}
  seen=serial;start=time.monotonic()
  try:
   d=await call(t,'GetOverview')
   row.update({k:d.get(k) for k in ['connection','error','background_error','mounted']})
  except Exception as e:row['probe_error']=type(e).__name__
  row['overview_ms']=round((time.monotonic()-start)*1000,2);emit(row)
async def signals(t,wakeup,notice):
 while True:
  p=await asyncio.create_subprocess_exec('stdbuf','-oL','-eL','gdbus','monitor','--session','--dest',t['bus'],'--object-path','/org/drivenative/Client1',stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.DEVNULL)
  try:
   while line:=await p.stdout.readline():
    text=line.decode(errors='replace')
    if 'is owned by' in text or 'does not have an owner' in text:
     emit({'kind':'bus_owner_change','instance':t['instance'],'owned':'is owned by' in text});wakeup.set()
    match=STATUS_SIGNAL.search(text)
    if match:
     notice['signals']+=1;notice['revision']=int(match.group(1));wakeup.set()
   await p.wait()
   emit({'kind':'monitor_exit','instance':t['instance'],'returncode':p.returncode})
  finally:
   if p.returncode is None:
    p.terminate();await p.wait()
  await asyncio.sleep(5)
async def main():
 emit({'kind':'probe_start','version':2,'tasks':len(TASKS),'status_coalesce_ms':200})
 jobs=[]
 for t in TASKS:
  wakeup=asyncio.Event();notice={'signals':0,'revision':None}
  jobs.extend([watch(t),signals(t,wakeup,notice),signal_state(t,wakeup,notice)])
 await asyncio.gather(*jobs)
asyncio.run(main())
