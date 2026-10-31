"""Isolated real session-bus test; no Drive server, FUSE or credentials."""
import argparse,hashlib,json,os,pathlib,signal,subprocess,threading,time,uuid
from gi.repository import Gio,GLib
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--probe',type=pathlib.Path,required=True)
parser.add_argument('--output-dir',type=pathlib.Path,required=True)
options=parser.parse_args()
root=options.output_dir;root.mkdir(mode=0o700,exist_ok=False)
probe=options.probe.resolve()
instance='probe_test_'+uuid.uuid4().hex;busname='org.drivenative.ProbeTest.i'+uuid.uuid4().hex
path='/org/drivenative/Client1';iface='org.drivenative.Client1'
(root/'catalog.json').write_text(json.dumps([{'instance':instance,'bus':busname,'name':'synthetic probe test','mount':str(root/'not-mounted')}]))
(root/'manifest.json').write_text(json.dumps({'registered_at':time.time(),'scope':'mock D-Bus only','probe_sha256':hashlib.sha256(probe.read_bytes()).hexdigest(),'coalesce_ms':200,'burst_signals':1000,'expected_signals':1002,'cases':['initial ownership','offline then online signal','1000-signal burst','ignore unknown signal/body and extra overview fields'],'max_signal_queries':12},indent=2))
state={'connection':'online','error':None,'background_error':None,'mounted':False,'credential':'DO_NOT_LOG_SYNTHETIC_CANARY'}
calls=0
connection=Gio.bus_get_sync(Gio.BusType.SESSION,None)
xml='<node><interface name="org.drivenative.Client1"><method name="GetOverview"><arg type="s" direction="out"/></method><signal name="StatusChanged"><arg type="t"/></signal><signal name="Ignored"><arg type="s"/></signal></interface></node>'
def method(conn,sender,obj,interface,name,parameters,invocation):
 global calls
 calls+=1;invocation.return_value(GLib.Variant('(s)',(json.dumps(state),)))
node=Gio.DBusNodeInfo.new_for_xml(xml);reg=connection.register_object(path,node.interfaces[0],method,None,None)
owner=Gio.bus_own_name_on_connection(connection,busname,Gio.BusNameOwnerFlags.NONE,None,None)
loop=GLib.MainLoop();thread=threading.Thread(target=loop.run);thread.start()
def rows():
 p=root/'watch-probe.jsonl'
 if not p.exists():return []
 result=[]
 for line in p.read_text().splitlines():
  try:result.append(json.loads(line))
  except ValueError:pass
 return result
def until(fn,seconds=8):
 end=time.monotonic()+seconds
 while not fn():
  if time.monotonic()>end:raise TimeoutError('test condition not observed')
  time.sleep(.02)
def emit(revision):connection.emit_signal(None,path,iface,'StatusChanged',GLib.Variant('(t)',(revision,)))
p=None;result={'passed':False}
try:
 p=subprocess.Popen(['/usr/bin/python3',str(probe),'--state-dir',str(root)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
 until(lambda:any(r.get('kind')=='bus_owner_change' and r.get('owned') for r in rows()))
 state.update(connection='offline',error='network_unavailable');started=time.monotonic();emit(1)
 until(lambda:any(r.get('kind')=='status_event' and r.get('connection')=='offline' and r.get('error')=='network_unavailable' for r in rows()))
 result['offline_detection_ms']=(time.monotonic()-started)*1000
 state.update(connection='online',error=None);emit(2)
 until(lambda:any(r.get('kind')=='status_event' and r.get('signal_revision')==2 and r.get('connection')=='online' for r in rows()))
 connection.emit_signal(None,path,iface,'Ignored',GLib.Variant('(s)',('DO_NOT_LOG_SYNTHETIC_CANARY',)))
 for i in range(3,1003):emit(i)
 until(lambda:sum(r.get('signals',0) for r in rows() if r.get('kind')=='status_event')==1002)
 events=[r for r in rows() if r.get('kind')=='status_event']
 assert len(events)<=12,len(events)
 assert events[-1]['signal_revision']==1002
 assert not any(r.get('probe_error') for r in events)
 assert 'DO_NOT_LOG_SYNTHETIC_CANARY' not in (root/'watch-probe.jsonl').read_text()
 result.update(passed=True,status_event_rows=len(events),received_signals=sum(r['signals'] for r in events),overview_calls=calls,last_revision=events[-1]['signal_revision'],raw_sensitive_canary_logged=False)
except Exception as e:result['error']=type(e).__name__+': '+str(e)
finally:
 if p is not None:
  os.killpg(p.pid,signal.SIGTERM)
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait(timeout=5);result['forced_cleanup']=True
 Gio.bus_unown_name(owner);connection.unregister_object(reg);loop.quit();thread.join(timeout=5)
 result['mock_loop_stopped']=not thread.is_alive()
 (root/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
raise SystemExit(0 if result['passed'] else 1)
