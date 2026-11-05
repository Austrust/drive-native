"""Read-only process/mount end audit after all owned G4 commands have exited."""
import hashlib,json,os,pathlib,datetime
base=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
run=base/'g4-012';output=run/'closeout.json';assert not output.exists()
names=['summary.json','supplemental-001/summary.json','native-worker-chain-001/results.json','component-policy-handover-001/result.json','final-integrity.json']
hashes={}
for name in names:
    path=run/name;data=json.loads(path.read_text())
    assert data.get('all_commands_passed') or data.get('all_passed') or data.get('passed'),name
    hashes[name]=hashlib.sha256(path.read_bytes()).hexdigest()
owned=[];compilers=[];errors=[]
for entry in pathlib.Path('/proc').iterdir():
    if not entry.name.isdecimal():continue
    try:
        comm=(entry/'comm').read_text().strip()
        if comm in ('cargo','rustc'):compilers.append({'pid':int(entry.name),'comm':comm})
        if entry.stat().st_uid!=os.getuid():continue
        executable=os.readlink(entry/'exe').removesuffix(' (deleted)')
        if executable.startswith(str(base)+'/'):owned.append({'pid':int(entry.name),'executable':executable})
    except (FileNotFoundError,ProcessLookupError):pass
    except PermissionError:errors.append({'pid':int(entry.name),'error':'permission error; ownership audit incomplete'})
mounts=[]
for line in pathlib.Path('/proc/self/mountinfo').read_text().splitlines():
    raw=line.split()[4]
    for escaped,plain in [('\\040',' '),('\\011','\t'),('\\012','\n'),('\\134','\\')]:raw=raw.replace(escaped,plain)
    if raw.startswith(str(base)+'/'):mounts.append(raw)
result={'candidate_commit':'9c9aabae447d92275f93dae65320788240a03113',
    'observed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'owned_validation_elf_processes':owned,
    'cargo_or_rustc_processes':compilers,'residual_validation_mounts':mounts,'inspection_errors':errors,
    'scope':'process names and own-user executable links only; no process arguments or credentials read',
    'input_hashes':hashes,'passed':not owned and not compilers and not mounts and not errors}
output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));assert result['passed']
