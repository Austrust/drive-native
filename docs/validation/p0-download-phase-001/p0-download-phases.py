"""Explicit cold-body reads in an isolated read-only task; no NAS mutations."""
import hashlib,json,math,pathlib,sqlite3,subprocess,time
from gi.repository import Gio,GLib
root=pathlib.Path.home()/'.local/state/drive-native-p0/relay-phase-001'
cfg=json.loads((root/'config.json').read_text());assert cfg['readonly']
out=root/'phases-001';out.mkdir(mode=0o700,exist_ok=False)
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
state=pathlib.Path(cfg['state']);db=sqlite3.connect(state.joinpath('state.sqlite').as_uri()+'?mode=ro',uri=True)
original=json.loads((root/'cache-preparation-result.json').read_text())
node,version,blob=db.execute('SELECT n.id,n.remote_version,n.blob FROM nodes n JOIN node_metadata m ON m.node=n.id WHERE n.deleted=0 AND n.directory=0 AND m.size=? AND n.blob IS NOT NULL LIMIT 1',(original['size'],)).fetchone()
expected=original['sha256'];assert hashlib.sha256((state/'blobs'/blob).read_bytes()).hexdigest()==expected
cursor=node;parts=[]
while cursor:
    parent,name=db.execute('SELECT parent,name FROM nodes WHERE id=?',(cursor,)).fetchone()
    if parent is None:break
    parts.append(name);cursor=parent
path=str(pathlib.Path(cfg['mount']).joinpath(*reversed(parts)))
manifest={'registered_at':time.time(),'planned_samples':10,'warmup':2,'timeout_s':15,'scope':'isolated readonly training view 11','bytes':original['size'],'expected_sha256':expected,'release':'D-Bus ReleaseSpace before every sample; require SQLite blob NULL and prior blob absent; never mutate database','clock':'open/first byte/complete exclude child process startup; include complete native download verification and publication','cache':'body cold locally, metadata/authentication warm; NAS/server caches uncontrolled','transport':'loopback opaque relay with real NAS TLS; no imposed rate/loss/delay','interference':'original six tasks and two probes; 20ms GetTransfers sampling adds observation load','runtime_source_commit':'62be13c9f9f4a2457a3040829f478bf43b1821df','runtime_sha256':cfg['runtime_sha256'],'hash_limits':'compares with prior complete downloaded version, not independent server content oracle','stop_rule':'stop on first failure, preserve and report unrun samples'}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
helper="import hashlib,json,sys,time;print('ready',flush=True);sys.stdin.readline();t=time.monotonic();f=open(sys.argv[1],'rb');opened=time.monotonic();b=f.read(1);first=time.monotonic();rest=f.read();end=time.monotonic();f.close();print(json.dumps({'open_s':opened-t,'first_byte_s':first-t,'complete_s':end-t,'bytes':len(b)+len(rest),'hash_matches':hashlib.sha256(b+rest).hexdigest()==sys.argv[2]}))"
with (root.parent/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'start':time.time(),'run':root.name+'/phases-001','kind':'12 cold-body reads with 20ms GetTransfers phase sampling'})+'\n')
rows=[];warmup=[]
for index in range(-2,10):
    row={'index':index,'time':time.time()};proc=None
    try:
        current=db.execute('SELECT remote_version,blob FROM nodes WHERE id=?',(node,)).fetchone();assert current[0]==version
        prior_blob=current[1]
        v=bus.call_sync(cfg['bus'],'/org/drivenative/Client1','org.drivenative.Client1','ReleaseSpace',GLib.Variant('(s)',(path,)),GLib.VariantType.new('(s)'),Gio.DBusCallFlags.NONE,5000,None)
        released=json.loads(v.unpack()[0]);assert released['released']==[node] and not released['protected_or_failed']
        assert db.execute('SELECT blob FROM nodes WHERE id=?',(node,)).fetchone()[0] is None
        assert prior_blob is None or not (state/'blobs'/prior_blob).exists()
        row['local_body_absent_before_read']=True
        start=time.monotonic()
        proc=subprocess.Popen(['python3','-c',helper,path,expected],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True)
        import select
        assert select.select([proc.stdout],[],[],5)[0] and proc.stdout.readline().strip()=='ready'
        row['startup_s']=time.monotonic()-start
        proc.stdin.write('read\n');proc.stdin.flush()
        phase_start=time.monotonic();observations=[]
        while proc.poll() is None:
            if time.monotonic()-phase_start>15:raise TimeoutError('read deadline')
            query_start=time.monotonic()
            try:
                reply=bus.call_sync(cfg['bus'],'/org/drivenative/Client1','org.drivenative.Client1','GetTransfers',None,GLib.VariantType.new('(s)'),Gio.DBusCallFlags.NONE,1000,None)
                entries=json.loads(reply.unpack()[0])['entries']
                selected=[{k:e.get(k) for k in ['request','progress','elapsed_ms']} for e in entries if e['node']==node]
                observations.append({'elapsed_s':time.monotonic()-phase_start,'query_s':time.monotonic()-query_start,'entries':selected})
            except GLib.Error as error:observations.append({'elapsed_s':time.monotonic()-phase_start,'error':type(error).__name__})
            time.sleep(.02)
        row['phase_samples']=observations
        raw,_=proc.communicate(timeout=1)
        assert proc.returncode==0
        row.update(json.loads(raw));row['ok']=row['hash_matches'] and row['bytes']==original['size']
        assert db.execute('SELECT remote_version FROM nodes WHERE id=?',(node,)).fetchone()[0]==version
    except Exception as error:row.update(ok=False,error_type=type(error).__name__)
    finally:
        if proc is not None and proc.poll() is None:proc.kill();proc.wait()
    target=warmup if index<0 else rows;target.append(row)
    with (out/('warmup.jsonl' if index<0 else 'samples.jsonl')).open('a') as f:f.write(json.dumps(row)+'\n')
    print(json.dumps({k:row.get(k) for k in ['index','ok','error_type','first_byte_s','complete_s']}),flush=True)
    if not row['ok']:break
    time.sleep(.2)
summary={'planned':10,'executed':len(rows),'errors':sum(not r['ok'] for r in rows),'warmup_errors':sum(not r['ok'] for r in warmup),'over_2s':[r['index'] for r in rows if r.get('complete_s',float('inf'))>2]}
for field in ['startup_s','open_s','first_byte_s','complete_s']:
    values=sorted(r[field] for r in rows if field in r)
    if values:summary[field]={'n':len(values),**{f'p{p}':values[math.ceil(len(values)*p/100)-1] for p in [50,95,99]},'max':max(values)}
(out/'summary.json').write_text(json.dumps(summary,indent=2));db.close()
with (root.parent/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'end':time.time(),'run':root.name+'/phases-001'})+'\n')
print(json.dumps(summary))

