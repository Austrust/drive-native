"""P0 preregistered 48-lane read-only benchmark. Uses public FUSE and D-Bus APIs."""
import concurrent.futures,hashlib,json,math,os,pathlib,sqlite3,subprocess,time,ast
ROOT=pathlib.Path.home()/'.local/state/drive-native-soak'
OUT=pathlib.Path.home()/'.local/state/drive-native-p0/mixed-48-001'
TIMEOUT=15;ROUNDS=5;LANES=48
def main():
 OUT.mkdir(parents=True,exist_ok=False)
 task=next(t for t in json.loads((ROOT/'catalog.json').read_text()) if t['name']=='Personal_Sync')
 db=sqlite3.connect('file:'+str(ROOT/'data/drive-native'/task['task_id']/'state.sqlite')+'?mode=ro',uri=True)
 def path_of(node):
  parts=[]
  while node:
   parent,name=db.execute('select parent,name from nodes where id=?',(node,)).fetchone()
   if parent is None:break
   parts.append(name);node=parent
  return str(pathlib.Path(task['mount']).joinpath(*reversed(parts)))
 directory=db.execute('select parent,count(*) n from nodes where deleted=0 and parent is not null and id not in (select id from access_blocked) group by parent order by abs(count(*)-2000),parent limit 1').fetchone()
 file=db.execute('select n.id,n.blob,m.size from nodes n join node_metadata m on n.id=m.node where n.deleted=0 and n.blob is not null and n.directory=0 and n.id not in (select id from access_blocked) order by abs(m.size-1048576),n.id limit 1').fetchone()
 assert file and directory
 dirpath=path_of(directory[0]);filepath=path_of(file[0]);expected=hashlib.sha256((ROOT/'data/drive-native'/task['task_id']/'blobs'/file[1]).read_bytes()).hexdigest()
 nodecount=db.execute('select count(*) from nodes where deleted=0').fetchone()[0]
 manifest={'preregistered_at':time.time(),'lanes':LANES,'mix':{'directory':16,'status':16,'cached_read':16},'rounds':ROUNDS,'warmup_per_kind':2,'timeout_s':TIMEOUT,'nodes':nodecount,'directory_children':directory[1],'directory_target_2000_exact':directory[1]==2000,'file_bytes':file[2],'cache_condition':'existing complete local blob; warmed by two reads; kernel/disk cache not independently controlled','timing':'subprocess launch to complete output; content worker additionally measures first-byte and content-complete; no app startup measured','percentiles':'nearest rank; timed-out samples included at observed timeout duration, right-censored; errors separately counted','interference':'existing six tasks and both diagnostic/24h probes remain active; planned load interval recorded','paths_local_only':True}
 (OUT/'manifest.json').write_text(json.dumps(manifest,indent=2))
 # Private selection enables an exact rerun without publishing names or content.
 (OUT/'selection.private.json').write_text(json.dumps({'task':task,'directory':dirpath,'file':filepath,'expected_hash':expected}));os.chmod(OUT/'selection.private.json',0o600)
 (ROOT.parent/'drive-native-p0/planned-loads.jsonl').open('a').write(json.dumps({'start':time.time(),'run':OUT.name,'kind':'48-lane readonly benchmark'})+'\n')
 local='''import os,sys,time,json,hashlib
t=time.monotonic()
if sys.argv[1]=='directory':print(json.dumps({'entries':len(os.listdir(sys.argv[2]))}))
else:
 h=hashlib.sha256();n=0
 with open(sys.argv[2],'rb') as f:
  b=f.read(65536);first=time.monotonic()-t
  while b:n+=len(b);h.update(b);b=f.read(65536)
 print(json.dumps({'bytes':n,'sha256':h.hexdigest(),'first_byte_s':first,'content_complete_s':time.monotonic()-t}))
'''
 def one(kind,index,warmup=False):
  start=time.monotonic();row={'kind':kind,'index':index,'warmup':warmup,'start':time.time()}
  args=['gdbus','call','--session','--dest',task['bus'],'--object-path','/org/drivenative/Client1','--method','org.drivenative.Client1.GetStates',json.dumps([task['mount']])] if kind=='status' else ['/usr/bin/python3','-c',local,kind,dirpath if kind=='directory' else filepath]
  try:
   p=subprocess.run(args,capture_output=True,text=True,timeout=TIMEOUT,check=True)
   d=json.loads(ast.literal_eval(p.stdout)[0]) if kind=='status' else json.loads(p.stdout)
   row['ok']=all(e.get('error') is None for e in d['entries']) if kind=='status' else d['entries']==directory[1] if kind=='directory' else d['bytes']==file[2] and d['sha256']==expected
   if kind=='cached_read':row.update({k:d[k] for k in ['first_byte_s','content_complete_s']})
  except subprocess.TimeoutExpired:row.update(ok=False,error='timeout',censored=True)
  except Exception as e:row.update(ok=False,error=type(e).__name__)
  row['seconds']=time.monotonic()-start
  return row
 rows=[]
 for kind in ['directory','status','cached_read']:
  for i in range(2):rows.append(one(kind,i,True))
 with concurrent.futures.ThreadPoolExecutor(max_workers=LANES) as pool:
  for rnd in range(ROUNDS):
   futures=[pool.submit(one,kind,rnd*LANES+lane) for lane,kind in enumerate(['directory','status','cached_read']*16)]
   batch=[f.result() for f in futures];rows.extend(batch)
   with (OUT/'samples.jsonl').open('a') as f:
    for row in batch:f.write(json.dumps(row)+'\n')
   print(json.dumps({'round':rnd+1,'failures':sum(not v['ok'] for v in batch),'max_s':max(v['seconds'] for v in batch)}),flush=True)
 summary=[]
 for kind in ['directory','status','cached_read','all']:
  chosen=[v for v in rows if not v['warmup'] and (kind=='all' or v['kind']==kind)];values=sorted(v['seconds'] for v in chosen)
  summary.append({'kind':kind,'n':len(values),'errors':sum(not v['ok'] for v in chosen),'timeouts':sum(v.get('error')=='timeout' for v in chosen),**{f'p{q}_s':values[math.ceil(len(values)*q/100)-1] for q in [50,95,99]},'max_s':max(values)})
 (OUT/'summary.json').write_text(json.dumps(summary,indent=2));(OUT/'warmup.json').write_text(json.dumps(rows[:6],indent=2))
 (ROOT.parent/'drive-native-p0/planned-loads.jsonl').open('a').write(json.dumps({'end':time.time(),'run':OUT.name})+'\n')
 print(json.dumps(summary),flush=True)
if __name__=='__main__':main()
