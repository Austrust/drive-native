"""Coordinator-requested scope stop; retain original command failure/interruption log."""
import datetime,json,os,pathlib,signal
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-007')
summary=json.loads((run/'summary.json').read_text())
assert not summary['complete']
matches=[]
for path in pathlib.Path('/proc').iterdir():
    if not path.name.isdigit():continue
    try:argv=(path/'cmdline').read_bytes().split(b'\0')
    except (FileNotFoundError,PermissionError):continue
    if b'--run' in argv and str(run).encode() in argv and any(a.endswith(b'/issue7-validation/run-g4.py') for a in argv):matches.append(int(path.name))
assert len(matches)==1,matches
record={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'pid':matches[0],
 'reason':'coordinator scope changed; cached older drive-core may have survived git archive mtime; stop before repeated FUSE/release groups and rebuild from touched source content unchanged',
 'current_command':summary['commands'][-1]['name'],'signal':'SIGINT'}
output=run/'coordinator-stop.json';assert not output.exists();output.write_text(json.dumps(record,indent=2)+'\n')
os.kill(matches[0],signal.SIGINT);print(json.dumps(record))
