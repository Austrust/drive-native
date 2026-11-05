import datetime,json,os,pathlib,signal
matches=[]
for path in pathlib.Path('/proc').iterdir():
    if not path.name.isdigit():continue
    try:argv=(path/'cmdline').read_bytes().split(b'\0')
    except (FileNotFoundError,PermissionError):continue
    if any(a.endswith(b'/issue7-validation/start-g4-008.sh') for a in argv):matches.append(int(path.name))
assert len(matches)==1,matches
parent=matches[0];children=[]
for path in pathlib.Path('/proc').iterdir():
    if not path.name.isdigit():continue
    try:status=(path/'stat').read_text().split(') ',1)[1].split()
    except (FileNotFoundError,PermissionError):continue
    if int(status[1])==parent:children.append(int(path.name))
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-008')
assert not run.exists(),'must not stop an already started final matrix with this queue-only tool'
record={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'candidate':'99aa587e83fc409883ecd3e0d3f60fe8640b2e06',
 'started':False,'tests_run':0,'reason':'new trigger broke v6 downgrade fixture in ca86 fresh workspace; waiting for coordinator fix before full matrix',
 'queue_pid':parent,'waiting_children':children}
output=pathlib.Path(__file__).parent/'prepared-g4-008/not-run.json';assert not output.exists();output.write_text(json.dumps(record,indent=2)+'\n')
for pid in children:os.kill(pid,signal.SIGTERM)
try:os.kill(parent,signal.SIGTERM)
except ProcessLookupError:pass
print(json.dumps(record))
