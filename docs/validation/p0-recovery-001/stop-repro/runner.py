import hashlib,json,os,pathlib,subprocess,time
from gi.repository import Gio,GLib
root=pathlib.Path.home()/'.local/state/drive-native-p0/stop-repro-001'
scripts=pathlib.Path('/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/outputs/drive-native-github/scripts/validation')
subprocess.run(['python3',str(scripts/'prepare_benchmark.py'),'--builder',str(pathlib.Path.home()/'.cache/drive-native-build/target/release/examples/performance_fixture'),
 '--fsd',str(pathlib.Path.home()/'.local/lib/drive-native-soak/perf/drive-native-fsd'),'--runtime-commit','62be13c9f9f4a2457a3040829f478bf43b1821df','--output',str(root),'--start'],check=True)
cfg=json.loads((root/'config.json').read_text());bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
deadline=time.monotonic()+30
while True:
 try:
  r=bus.call_sync(cfg['bus'],'/org/drivenative/Client1','org.drivenative.Client1','GetOverview',None,GLib.VariantType.new('(s)'),Gio.DBusCallFlags.NONE,2000,None)
  if json.loads(r.unpack()[0])['mounted']:break
 except GLib.Error:pass
 if time.monotonic()>deadline:raise RuntimeError('mount deadline')
 time.sleep(.2)
with (root.parent/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'start':time.time(),'run':'stop-repro-001','kind':'synthetic mixed-held load then explicit stop; no NAS'})+'\n')
(root/'manifest.json').write_text(json.dumps({'preregistered_at':time.time(),'phase':'mixed_held','rounds':5,'lanes':48,'stop_watch_after_seconds':5,'systemd_timeout_unchanged':True,'purpose':'reproduce shutdown failure; subset of previous 7-phase workload, no product change'},indent=2))
subprocess.run(['python3',str(scripts/'benchmark_local.py'),'--config',str(root/'config.json'),'--output',str(root/'mixed-held'),'--phase','mixed_held'],check=True)
pid=int(subprocess.check_output(['systemctl','--user','show',cfg['unit'],'-p','MainPID','--value'],text=True))
assert hashlib.sha256(pathlib.Path(f'/proc/{pid}/exe').read_bytes()).hexdigest()==cfg['binary_sha256']
started=time.monotonic()
stop=subprocess.Popen(['systemctl','--user','stop',cfg['unit']])
try:stop.wait(timeout=5)
except subprocess.TimeoutExpired:
 marker={'pid':pid,'unit':cfg['unit'],'profile':str(root/'profile.json'),'binary_sha256':cfg['binary_sha256'],'time':time.time()}
 (root/'await-stack.json').write_text(json.dumps(marker));print(json.dumps({'needs_stack':True,'pid':pid}),flush=True)
 stop.wait(timeout=100)
props=subprocess.check_output(['systemctl','--user','show',cfg['unit'],'-p','Result','-p','ExecMainStatus','-p','MainPID'],text=True)
result={'seconds':time.monotonic()-started,'properties':dict(line.split('=',1) for line in props.splitlines()),'stop_command_returncode':stop.returncode}
(root/'stop-result.json').write_text(json.dumps(result,indent=2))
with (root.parent/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'end':time.time(),'run':'stop-repro-001'})+'\n')
print(json.dumps(result),flush=True)
