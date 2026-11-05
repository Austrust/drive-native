"""Complete the read-only process audit where systemd hides same-user exe links."""
import hashlib,json,os,pathlib,datetime
assert os.geteuid()==0,'root is used only to read process identity links'
base=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation');run=base/'g4-012'
output=run/'closeout-root-assisted.json';assert not output.exists()
oldpath=run/'closeout.json';old=json.loads(oldpath.read_text())
owned=[];compilers=[];errors=[];resolved=[]
old_pids={row['pid'] for row in old['inspection_errors']}
for entry in pathlib.Path('/proc').iterdir():
    if not entry.name.isdecimal():continue
    try:
        comm=(entry/'comm').read_text().strip()
        if comm in ('cargo','rustc'):compilers.append({'pid':int(entry.name),'comm':comm})
        if entry.stat().st_uid!=1000:continue
        executable=os.readlink(entry/'exe').removesuffix(' (deleted)')
        if int(entry.name) in old_pids:resolved.append({'pid':int(entry.name),'comm':comm,'executable':executable})
        if executable.startswith(str(base)+'/'):owned.append({'pid':int(entry.name),'executable':executable})
    except (FileNotFoundError,ProcessLookupError):pass
    except PermissionError:errors.append({'pid':int(entry.name),'error':'permission error'})
mounts=[]
for line in pathlib.Path('/proc/self/mountinfo').read_text().splitlines():
    raw=line.split()[4]
    for escaped,plain in [('\\040',' '),('\\011','\t'),('\\012','\n'),('\\134','\\')]:raw=raw.replace(escaped,plain)
    if raw.startswith(str(base)+'/'):mounts.append(raw)
changed=[name for name,digest in old['input_hashes'].items() if hashlib.sha256((run/name).read_bytes()).hexdigest()!=digest]
result={'candidate_commit':old['candidate_commit'],'observed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'prior_audit_sha256':hashlib.sha256(oldpath.read_bytes()).hexdigest(),'prior_permission_errors_resolved':resolved,
    'owned_validation_elf_processes':owned,'cargo_or_rustc_processes':compilers,'residual_validation_mounts':mounts,
    'inspection_errors':errors,'changed_input_evidence':changed,
    'scope':'root only reads process comm/UID/exe links and mountinfo; no arguments, credentials, service changes or process termination',
    'passed':not owned and not compilers and not mounts and not errors and not changed}
output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));assert result['passed']
