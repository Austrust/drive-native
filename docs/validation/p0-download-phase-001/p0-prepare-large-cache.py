import hashlib,json,pathlib,sqlite3,subprocess,time
from gi.repository import Gio,GLib
root=pathlib.Path.home()/'.local/state/drive-native-p0/relay-phase-001';cfg=json.loads((root/'config.json').read_text())
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
def overview():
    v=bus.call_sync(cfg['bus'],'/org/drivenative/Client1','org.drivenative.Client1','GetOverview',None,GLib.VariantType.new('(s)'),Gio.DBusCallFlags.NONE,3000,None)
    return json.loads(v.unpack()[0])
d=overview();assert d['connection']=='online',d['connection']
db=sqlite3.connect(pathlib.Path(cfg['state']).joinpath('state.sqlite').as_uri()+'?mode=ro',uri=True)
node,size=db.execute('SELECT n.id,m.size FROM nodes n JOIN node_metadata m ON m.node=n.id WHERE n.deleted=0 AND n.directory=0 AND m.size BETWEEN 67108864 AND 134217728 ORDER BY m.size LIMIT 1').fetchone()
parts=[]
while node:
    parent,name=db.execute('SELECT parent,name FROM nodes WHERE id=?',(node,)).fetchone()
    if parent is None:break
    parts.append(name);node=parent
db.close();path=str(pathlib.Path(cfg['mount']).joinpath(*reversed(parts)))
manifest={'scope':'training','purpose':'prepare one uncached 64-to-128MiB file, smallest eligible, before 3 repeated cold reads','expected_size':size,'registered_at':time.time(),'timeout_s':120,'readonly':True}
(root/'large-preparation-manifest.json').write_text(json.dumps(manifest,indent=2))
helper="import hashlib,json,sys,time;t=time.monotonic();f=open(sys.argv[1],'rb');opened=time.monotonic();b=f.read(1);first=time.monotonic();rest=f.read();end=time.monotonic();f.close();print(json.dumps({'open_s':opened-t,'first_byte_s':first-t,'complete_s':end-t,'size':len(b)+len(rest),'sha256':hashlib.sha256(b+rest).hexdigest()}))"
try:
    p=subprocess.run(['python3','-c',helper,path],capture_output=True,text=True,timeout=120)
    result=json.loads(p.stdout) if p.returncode==0 else {'error':'read_failed','returncode':p.returncode}
except subprocess.TimeoutExpired:result={'error':'timeout','timeout_s':120}
(root/'large-preparation-result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))


