"""Sequential same-candidate supplements after the complete G4-012 matrix."""
import hashlib, importlib.util, json, os, pathlib, re, signal, subprocess, time, traceback

HERE=pathlib.Path(__file__).resolve().parent
BASE=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
RUN=BASE/'g4-012'; SOURCE=RUN/'source'; TARGET=RUN/'target'
CANDIDATE='9c9aabae447d92275f93dae65320788240a03113'
OLD=BASE/'g4-011'; OLD_COMMIT='41365a650eb7d7df696046e21b7671697231f851'
main=json.loads((RUN/'summary.json').read_text())
assert main['candidate_commit']==CANDIDATE and main['complete'] and main['all_commands_passed']
assert json.loads((OLD/'source.json').read_text())['candidate_commit']==OLD_COMMIT
OUT=RUN/'supplemental-001';OUT.mkdir(exist_ok=False);(OUT/'logs').mkdir()
spec=importlib.util.spec_from_file_location('frozen_g4',HERE/'prepared-g4-012/run-g4.py')
helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
env=os.environ.copy()
for key in list(env):
    if key.startswith('DRIVE_NATIVE_') or key in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','DBUS_SESSION_BUS_ADDRESS','DISPLAY','WAYLAND_DISPLAY'):
        env.pop(key,None)
env.update(CARGO_TARGET_DIR=str(TARGET),CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',
    PYTHONDONTWRITEBYTECODE='1',RUST_BACKTRACE='1',QT_QPA_PLATFORM='offscreen',PYTHONPATH=str(SOURCE/'desktop'),
    TMPDIR=str(RUN/'tmp'))
for key,name in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache'),('XDG_RUNTIME_DIR','runtime')]:
    env[key]=str(RUN/name)

def sha(path):
    with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def save(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
result={'candidate_commit':CANDIDATE,'complete':False,'all_passed':False,'commands':[],
    'scope':'210k functional capacity, real worker TLS chain, private component lifecycle; no NAS/deb lifecycle/performance claims',
    'main_summary_sha256':sha(RUN/'summary.json'),'target_reuse':'completed same candidate only, sequential, explicitly authorized',
    'harness_sha256':sha(pathlib.Path(__file__))}
def persist():save(OUT/'summary.json',result)
def interrupted(signum, frame):raise InterruptedError(f'supplement driver received signal {signum}')
signal.signal(signal.SIGTERM,interrupted)
def command(name,argv,kind='prepare',custom_env=None,timeout=3600,cwd=SOURCE):
    row={'name':name,'argv':argv,'cwd':str(cwd),'kind':kind,'log':'logs/'+name+'.log',
        'host_space_before':helper.host_c_space(),'started_utc':helper.stamp()}
    result['commands'].append(row);persist()
    free=row['host_space_before']['free_bytes']
    if free is None or free<10*1024**3:
        row.update(passed=False,not_run=True,reason='host C below 10GiB/unknown; stopped before command');persist()
        raise RuntimeError(row['reason'])
    start=time.monotonic();active_env=env.copy();active_env.update(custom_env or {})
    with (OUT/row['log']).open('wb') as log:
        process=subprocess.Popen(argv,cwd=cwd,env=active_env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
        row['process_group']=process.pid;persist()
        try:row['exit_code']=process.wait(timeout=timeout)
        except BaseException as error:
            row.update(exception_type=type(error).__name__,error=str(error))
            try:os.killpg(process.pid,signal.SIGTERM)
            except ProcessLookupError:pass
            try:process.wait(timeout=15)
            except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=15)
            row.update(exit_code=process.returncode,timed_out=isinstance(error,subprocess.TimeoutExpired))
    row.update(finished_utc=helper.stamp(),elapsed_seconds=time.monotonic()-start,log_sha256=sha(OUT/row['log']))
    row['counts']=helper.counts((OUT/row['log']).read_text(errors='replace'),kind)
    row['passed']=row['exit_code']==0 and 'exception_type' not in row
    persist();print(json.dumps({'name':name,'passed':row['passed'],'counts':row['counts']}),flush=True)
    if 'exception_type' in row or row['exit_code']<0 or row['exit_code']>=128:
        raise RuntimeError('supplement interrupted or timed out; do not enter another command')
    return row['passed']

def executed_elf_witness():
    paths=set()
    for row in main['commands']:
        if not row.get('log') or 'exit_code' not in row:continue
        text=(RUN/row['log']).read_text(errors='replace')
        for item in re.findall(r'^\s*Running .*\(([^)]+)\)\s*$',text,re.M):
            path=pathlib.Path(item)
            if not path.is_absolute():path=SOURCE/path
            paths.add(path.resolve())
    for name in ('drive-native-fsd','drive-native-worker','drive-native-setup','native-loopback-probe'):
        path=TARGET/'debug'/name
        if path.is_file():paths.add(path)
    records=[]
    for path in sorted(paths):
        assert path.is_relative_to(TARGET) and path.is_file(),path
        with path.open('rb') as stream:assert stream.read(4)==b'\x7fELF',path
        records.append({'path':str(path),'sha256':sha(path),'bytes':path.stat().st_size})
    assert records
    witness={'candidate_commit':CANDIDATE,'executed_tests_from':'Cargo Running lines in preserved main logs',
        'observation_boundary':'ELFs still present at main-matrix end, hashed before supplements and compared after; not hashes taken at each execution instant',
        'also_included':'current normal fsd/worker/setup/native-loopback binaries','files':records}
    save(OUT/'executed-elf-before.json',witness)
    return witness

witness=None;persist()
try:
    witness=executed_elf_witness();persist()
    cargo=['cargo','test','--offline','--locked','-p','drive-core','--lib','--all-features']
    assert command('capacity-ignored-list',cargo+['--','--list','--ignored'])
    text=(OUT/'logs/capacity-ignored-list.log').read_text()
    names=re.findall(r'^(\S*default_budget_admits_210k_node_snapshot_without_holding_store): test$',text,re.M)
    assert len(names)==1,names
    command('capacity-210k',cargo+[names[0],'--','--ignored','--exact','--test-threads=1'],'rust')
    chain=HERE/'native-worker-chain-012'
    built=command('tls-harness-build',['python3','-B',str(chain/'build.py'),'--source',str(SOURCE),
        '--output',str(RUN/'native-worker-chain-build-001'),'--target',str(TARGET),'--candidate',CANDIDATE,
        '--host-space-helper',str(HERE/'prepared-g4-012/run-g4.py'),'--reuse-completed-run',str(RUN)],timeout=4200)
    if built:
        build=json.loads((RUN/'native-worker-chain-build-001/build.json').read_text());assert build['passed']
        command('tls-actual-worker-all-ten',['python3','-B',str(chain/'run.py'),'--binary',build['binary']['path'],
            '--worker',str(TARGET/'debug/drive-native-worker'),'--crate',build['crate'],'--candidate',CANDIDATE,
            '--output',str(RUN/'native-worker-chain-001'),'--host-space-helper',str(HERE/'prepared-g4-012/run-g4.py')],timeout=900)
    old_target=OLD/'native-worker-chain-target'
    old_probe=old_target/'debug/native-worker-chain-probe'
    result['old_candidate_supplement_only']={'candidate':OLD_COMMIT,'source':str(OLD/'source'),'target':str(old_target),
        'preserved_probe':{'path':str(old_probe),'sha256':sha(old_probe)},
        'scope':'build old components for real paired upgrade; not acceptance tests on the new candidate'};persist()
    binaries=['cargo','build','--offline','--locked','-p','drive-runtime','--all-features','--bins','--example','multi_mount_fixture']
    old_built=command('old-component-build',binaries,custom_env={'CARGO_TARGET_DIR':str(old_target)},cwd=OLD/'source')
    new_built=command('new-component-build',binaries)
    assert sha(old_probe)==result['old_candidate_supplement_only']['preserved_probe']['sha256']
    if old_built and new_built:
        command('private-component-policy-handover',['dbus-run-session','--','/usr/bin/python3','-B',str(HERE/'component-policy-handover.py'),
            '--root',str(RUN/'component-policy-handover-001'),'--fixture',str(TARGET/'debug/examples/multi_mount_fixture'),
            '--old-binaries',str(old_target/'debug'),'--new-binaries',str(TARGET/'debug'),
            '--old-commit',OLD_COMMIT,'--new-commit',CANDIDATE],timeout=600)
    result['complete']=True
except BaseException as error:
    result.update(exception_type=type(error).__name__,error=str(error))
    (OUT/'failure.txt').write_text(traceback.format_exc())
finally:
    result['audit_errors']=[]
    result['changed_executed_elf']=[];result['changed_source_files']=[];result['residual_owned_mounts']=[]
    try:
        assert witness is not None,'ELF witness was not completed'
        result['changed_executed_elf']=[row['path'] for row in witness['files'] if not pathlib.Path(row['path']).is_file() or sha(pathlib.Path(row['path']))!=row['sha256']]
    except BaseException as error:result['audit_errors'].append({'audit':'executed ELF','type':type(error).__name__,'error':str(error)})
    try:
        source_before=json.loads((RUN/'source-hashes-before.json').read_text())
        result['changed_source_files']=[name for name,expected in source_before.items() if not (SOURCE/name).is_file() or sha(SOURCE/name)!=expected]
    except BaseException as error:result['audit_errors'].append({'audit':'frozen source','type':type(error).__name__,'error':str(error)})
    try:result['residual_owned_mounts']=[str(path) for path in helper.mounts_below(RUN)]
    except BaseException as error:result['audit_errors'].append({'audit':'mountinfo','type':type(error).__name__,'error':str(error)})
    result['all_passed']=result['complete'] and all(row.get('passed') for row in result['commands']) and not result['changed_executed_elf'] and not result['changed_source_files'] and not result['residual_owned_mounts'] and not result['audit_errors']
    persist();print(json.dumps({'summary':str(OUT/'summary.json'),'all_passed':result['all_passed']}),flush=True)
raise SystemExit(0 if result['all_passed'] else 1)
