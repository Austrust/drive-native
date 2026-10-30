"""Local watchdog; records bounded diagnostics, never credentials or file bodies."""
import asyncio,ast,json,pathlib,time
ROOT=pathlib.Path.home()/'.local/state/drive-native-soak'
TASKS=json.loads((ROOT/'catalog.json').read_text())
def emit(row):
 row['time']=time.time()
 with (ROOT/'watch-probe.jsonl').open('a') as f:f.write(json.dumps(row,ensure_ascii=False)+'\n')
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
async def signals(t):
 while True:
  p=await asyncio.create_subprocess_exec('gdbus','monitor','--session','--dest',t['bus'],'--object-path','/org/drivenative/Client1',stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.DEVNULL)
  while line:=await p.stdout.readline():
   text=line.decode(errors='replace')
   if 'is owned by' in text or 'does not have an owner' in text:
    emit({'kind':'bus_owner_change','instance':t['instance'],'owned':'is owned by' in text})
  await p.wait();await asyncio.sleep(5)
async def main():
 emit({'kind':'probe_start','tasks':len(TASKS)})
 await asyncio.gather(*(watch(t) for t in TASKS),*(signals(t) for t in TASKS))
asyncio.run(main())
