import faulthandler
faulthandler.enable()
import importlib.util,json,os,time
from pathlib import Path
from gi.repository import GLib
spec=importlib.util.spec_from_file_location('drive_native_nemo',Path(__file__).parents[1]/'nemo/drive_native_nemo.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
print("nemo imported",flush=True)
e=m.DriveNativeExtension();context=GLib.MainContext.default()
end=time.monotonic()+8
while time.monotonic()<end:
 while context.pending():context.iteration(False)
 if len(e.tasks)==3 and all(t.root for t in e.tasks.values()):break
 time.sleep(.02)
print("nemo discovered",len(e.tasks),flush=True)
assert len(e.tasks)==3 and all(t.root for t in e.tasks.values())
roots={t.root for t in e.tasks.values()};assert len(roots)==3
assert e.tasks[m.NAME+'.i'+'a'*32].connection=='paused'
assert e.tasks[m.NAME+'.i'+'b'*32].connection=='locked'
# Ubuntu libnemo-extension Column finalizer crashes even in a standalone baseline.
# Do not construct it here; this regression covers multi-task discovery/routing.
print(json.dumps({'nemo_discovery':3,'distinct_roots':True,'state_routing':True}))
