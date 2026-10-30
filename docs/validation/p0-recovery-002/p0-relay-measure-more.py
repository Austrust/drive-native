import hashlib,json,math,os,pathlib,select,signal,socket,sqlite3,subprocess,time,sys
from gi.repository import Gio,GLib
root=pathlib.Path(sys.argv[1]);cfg=json.loads((root/'config.json').read_text())
out=root/'measurement-001';out.mkdir(mode=0o700,exist_ok=False)
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
def overview():
 v=bus.call_sync(cfg['bus'],'/org/drivenative/Client1','org.drivenative.Client1','GetOverview',None,GLib.VariantType.new('(s)'),Gio.DBusCallFlags.NONE,3000,None)
 return json.loads(v.unpack()[0])
def control(command):
 with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as s:
  s.settimeout(3);s.connect(str(root/'relay.sock'));s.sendall((command+'\n').encode());data=b''
  while not data.endswith(b'\n'):data+=s.recv(4096)
  return json.loads(data)
def workers(pid):
 children=set(int(v) for f in pathlib.Path(f'/proc/{pid}/task').glob('*/children') for v in f.read_text().split())
 return sorted(n for n in children if pathlib.Path(f'/proc/{n}/exe').resolve().name=='drive-native-worker')
deadline=time.monotonic()+180
while True:
 try:
  if overview()['connection']=='online':break
 except GLib.Error as error:
  with (out/'initialization.jsonl').open('a') as f:f.write(json.dumps({'time':time.time(),'error_type':type(error).__name__})+'\n')
 if time.monotonic()>deadline:raise RuntimeError('authentication or initial pull timeout; preserve state')
 time.sleep(.5)
pid=int(subprocess.check_output(['systemctl','--user','show',cfg['unit'],'-p','MainPID','--value'],text=True))
assert hashlib.sha256(pathlib.Path(f'/proc/{pid}/exe').read_bytes()).hexdigest()==cfg['runtime_sha256']
state=pathlib.Path(cfg['state']);db=sqlite3.connect(state.joinpath('state.sqlite').as_uri()+'?mode=ro',uri=True)
node=db.execute('SELECT id FROM nodes WHERE deleted=0 AND directory=0 AND blob IS NOT NULL LIMIT 1').fetchone()[0]
parts=[]
while node:
 parent,name=db.execute('SELECT parent,name FROM nodes WHERE id=?',(node,)).fetchone()
 if parent is None:break
 parts.append(name);node=parent
path=str(pathlib.Path(cfg['mount']).joinpath(*reversed(parts)));db.close()
manifest={'registered_at':time.time(),'planned_trials':30,'deadline_s':15,'poll_interval_s':.05,'scope':cfg['source_scope'],
 'fault':'abort both sides of owned loopback byte relay; no TLS termination, no payload capture; keep listener and fsd alive',
 'recovery':'different worker PID plus online after authenticated initial resume; relay accepted count must increase',
 'timing':'first drop request to verified recovery, includes local control/D-Bus overhead','cache':'read already opened immutable complete-cache handle during reconnect; compare hash to pre-fault bytes',
 'runtime_sha256':cfg['runtime_sha256'],'runtime_source_commit':'62be13c9f9f4a2457a3040829f478bf43b1821df','interference':'original six mounts, two probes and other idle readonly clone stay running; no mixed synthetic load',
 'stop_rule':'stop on first failed recovery; unrun trials remain unrun, retain failed sample','source_identity':'docs/validation/p0-baseline-audit.json'}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
planned=root.parent/'planned-loads.jsonl'
with planned.open('a') as f:f.write(json.dumps({'start':time.time(),'run':root.name+'/measurement-001','kind':'30 preregistered isolated transport resets'})+'\n')
helper='''import hashlib,json,os,sys,time
f=open(sys.argv[1],'rb');h=hashlib.sha256(f.read()).hexdigest();f.seek(0)
print('ready',flush=True);sys.stdin.readline();t=time.monotonic();body=f.read();r={'read_s':time.monotonic()-t,'bytes':len(body),'hash_matches':hashlib.sha256(body).hexdigest()==h};t=time.monotonic();r['directory_entries']=len(os.listdir(sys.argv[2]));r['directory_s']=time.monotonic()-t;f.close();print(json.dumps(r),flush=True)
'''
rows=[]
for trial in range(30):
 assert overview()['connection']=='online'
 old=workers(pid);assert len(old)==1
 assert hashlib.sha256(pathlib.Path(f'/proc/{old[0]}/exe').read_bytes()).hexdigest()=='c9b7bae2a75a9bdf83a17a29fe54f2111c03cd7c6ece9435a7c963834bcafa9d'
 reader=subprocess.Popen(['python3','-c',helper,path,cfg['mount']],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True,start_new_session=True)
 assert select.select([reader.stdout],[],[],15)[0] and reader.stdout.readline().strip()=='ready'
 row={'trial':trial,'time':time.time(),'old_worker_pid':old[0]};start=time.monotonic()
 try:
  before=control('status');drop=control('drop');assert drop['aborted_streams']>=2
  row['aborted_streams']=drop['aborted_streams'];reader.stdin.write('read\n');reader.stdin.flush()
  states=[];recovered=False
  while time.monotonic()-start<15:
   d=overview();current=workers(pid);states.append({'elapsed_s':time.monotonic()-start,'connection':d['connection'],'mounted':d['mounted'],'error':d.get('error')})
   if current and current!=old and d['connection']=='online' and control('status')['accepted']>before['accepted']:recovered=True;break
   time.sleep(.05)
  row.update(recovered=recovered,seconds=time.monotonic()-start,states=states)
  raw,_=reader.communicate(timeout=5);row['cache']=json.loads(raw)
  row['ok']=recovered and row['cache']['hash_matches'] and all(s['mounted'] for s in states)
 except Exception as e:row.update(ok=False,error=type(e).__name__,seconds=time.monotonic()-start)
 finally:
  if reader.poll() is None:os.killpg(reader.pid,signal.SIGKILL);reader.wait()
 rows.append(row)
 with (out/'samples.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
 print(json.dumps({k:row.get(k) for k in ['trial','ok','seconds','error']}),flush=True)
 if not row['ok']:break
 time.sleep(.5)
summary={'executed':len(rows),'planned':30,'errors':sum(not r['ok'] for r in rows)}
for key,values in [('recovery',[r['seconds'] for r in rows]),('cached_read',[r['cache']['read_s'] for r in rows if 'cache' in r]),('directory',[r['cache']['directory_s'] for r in rows if 'cache' in r])]:
 if values:
  values.sort();summary[key]={'samples':len(values),**{f'p{q}_s':values[math.ceil(len(values)*q/100)-1] for q in [50,95,99]},'max_s':max(values)}
(out/'summary.json').write_text(json.dumps(summary,indent=2))
with planned.open('a') as f:f.write(json.dumps({'end':time.time(),'run':root.name+'/measurement-001'})+'\n')
print(json.dumps(summary),flush=True)

