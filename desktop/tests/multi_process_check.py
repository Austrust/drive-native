"""Run under dbus-run-session; real local FUSE, no NAS or credentials."""
import ast,json,os,subprocess,sys,tempfile,time
from pathlib import Path
binary=Path(sys.argv[1]);source=Path(sys.argv[2])
with tempfile.TemporaryDirectory(prefix="drive-native-multi-") as scratch:
 base=Path(scratch);env=dict(os.environ,XDG_CONFIG_HOME=str(base/"config"),XDG_DATA_HOME=str(base/"data"),XDG_RUNTIME_DIR=str(base/"runtime"),PYTHONPATH=str(source/"desktop"),QT_QPA_PLATFORM="offscreen")
 (base/"runtime").mkdir(mode=0o700)
 rows=json.loads(subprocess.check_output([str(binary/"examples/multi_mount_fixture"),str(base)],env=env,text=True))
 processes={};logs=[]
 def start(row):
  log=open(base/(row['content']+".log"),'a');logs.append(log)
  processes[row['slot']]=subprocess.Popen([str(binary/"drive-native-fsd"),"--config",row['profile']],env=dict(env,DRIVE_NATIVE_INSTANCE=row['slot']),stdout=log,stderr=log)
 def call(row,method):
  bus="org.drivenative.Client1"+(".i"+row['slot'] if row['slot'] else "")
  r=subprocess.run(["gdbus","call","--session","--dest",bus,"--object-path","/org/drivenative/Client1","--method","org.drivenative.Client1."+method],text=True,capture_output=True,timeout=5)
  if r.returncode: return None
  v=ast.literal_eval(r.stdout)
  return json.loads(v[0]) if v else {}
 def wait_for(fn):
  end=time.monotonic()+15
  while time.monotonic()<end:
   if fn():return
   time.sleep(.05)
  raise AssertionError("multi-process condition timeout")
 try:
  for row in rows:start(row)
  for row in rows:
   wait_for(lambda: bool((d:=call(row,"GetOverview")) and d['mounted']))
   assert call(row,"GetOverview")['task_id']==row['task_id']
   assert (Path(row['mount'])/'A.txt').read_text()==row['content']
  call(rows[1],"Pause");assert call(rows[1],"GetOverview")['connection']=='paused'
  assert call(rows[0],"GetOverview")['connection']!='paused' and call(rows[2],"GetOverview")['connection']!='paused'
  subprocess.run(["/opt/drive-native-build/gui-venv/bin/python",str(source/'desktop/tests/multi_ui_check.py')],env=env,check=True,timeout=20)
  subprocess.run(["/usr/bin/python3",str(source/'desktop/tests/multi_nemo_check.py')],env=env,check=True,timeout=20)
  call(rows[1],"Resume");assert call(rows[1],"GetOverview")['connection']!='paused'
  processes[rows[1]['slot']].terminate();processes[rows[1]['slot']].wait(timeout=15)
  assert (Path(rows[2]['mount'])/'A.txt').read_text()=='beta'
  start(rows[1]);wait_for(lambda: bool((d:=call(rows[1],"GetOverview")) and d['mounted']))
  assert (Path(rows[1]['mount'])/'A.txt').read_text()=='alpha'
  print(json.dumps({'passed':True,'simultaneous_fuse_mounts':3,'legacy_preserved':True,'pause_isolated':True,'restart_isolated':True,'custom_unicode_names':True,'nas_accessed':False}))
 finally:
  for proc in processes.values():
   if proc.poll() is None:proc.terminate()
  for proc in processes.values():
   try:proc.wait(timeout=15)
   except subprocess.TimeoutExpired:proc.kill();proc.wait()
  for log in logs:log.close()
