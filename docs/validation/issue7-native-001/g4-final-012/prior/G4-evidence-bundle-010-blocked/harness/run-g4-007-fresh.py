"""Rebuild immutable candidate bytes after rejecting copied-cache provenance.

This appends a fresh subrun, never changes the original summary/logs. Coordinator
requested no repeated release/FUSE until the next product candidate freezes.
"""
import datetime,hashlib,importlib.util,json,os,pathlib,re,shutil,signal,subprocess,time,traceback
here=pathlib.Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('g4_helpers',here/'run-g4.py');helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-007')
source=run/'source';target=run/'target';out=run/'fresh-001';assert not out.exists()
old=json.loads((run/'summary.json').read_text());assert old.get('failure_type')=='KeyboardInterrupt'
assert old['candidate_commit']=='ca86a85e2be9c83c7a59407fc0737114158ecf7f'
assert not helper.mounts_below(run)
out.mkdir();(out/'logs').mkdir();(out/'old-artifacts').mkdir()
sha=helper.sha;stamp=helper.stamp
before=json.loads((run/'source-hashes-before.json').read_text())
assert all(sha(source/name)==digest for name,digest in before.items())
failed_log=(run/'logs/native-read-stale-preflight.log').read_text()
binary=pathlib.Path(re.search(r'Running unittests .* \(([^)]+)\)',failed_log)[1]);assert binary.is_relative_to(target)
old_elf=out/'old-artifacts'/binary.name;shutil.copy2(binary,old_elf)
blob=old_elf.read_bytes();old_paths={name:blob.count(f'/g4-{name}/source/crates/drive-core'.encode()) for name in ('005','006','007')}
provenance={'candidate_commit':old['candidate_commit'],'old_test_elf_sha256':sha(old_elf),
 'old_test_elf':str(old_elf.relative_to(out)),'old_elf_embedded_core_path_counts':old_paths,
 'source_content_before_matches_archive':True,'timestamp_adjustment_at':stamp(),
 'reason':'git archive commit mtimes may be older than copied Cargo artifacts; force clean only this private target product packages and touch extracted source metadata'}
for path in source.rglob('*'):
    if path.is_file() and not path.is_symlink():os.utime(path,None)
provenance['source_content_after_matches_archive']=all(sha(source/name)==digest for name,digest in before.items())
assert provenance['source_content_after_matches_archive']
(out/'cache-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
env=os.environ.copy()
for key in list(env):
    if key.startswith('DRIVE_NATIVE_') or key in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','DBUS_SESSION_BUS_ADDRESS','DISPLAY','WAYLAND_DISPLAY'}:env.pop(key,None)
env.update(CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',RUST_BACKTRACE='1',
 PYTHONDONTWRITEBYTECODE='1',PYTHONPATH=str(source/'desktop'),QT_QPA_PLATFORM='offscreen',DRIVE_NATIVE_FS_INDEX='1',DRIVE_NATIVE_FS_TTL_MS='1000')
for variable,name in [('TMPDIR','tmp'),('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache'),('XDG_RUNTIME_DIR','runtime')]:
    path=out/name;path.mkdir(mode=0o700);env[variable]=str(path)
summary={'candidate_commit':old['candidate_commit'],'complete':False,'start':stamp(),'commands':[],
 'scope':'fresh product rebuild; native/IPC/history/workspace/default/GUI; coordinator defers complete FUSE/release to next product candidate',
 'harness_sha256':sha(pathlib.Path(__file__))}
def save():helper.save(out/'summary.json',summary)
def execute(name,argv,kind='rust',result=None,timeout=3600,required=()):
    row={'name':name,'command':argv,'kind':kind,'start':stamp(),'log':f'logs/{name}.log'};summary['commands'].append(row);save()
    at=time.monotonic();interrupted=None
    with (out/row['log']).open('w') as log:
        process=subprocess.Popen(argv,cwd=source,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
        try:process.wait(timeout=timeout)
        except BaseException as error:
            row['exception_type']=type(error).__name__;row['timeout']=isinstance(error,subprocess.TimeoutExpired)
            if not row['timeout']:interrupted=error
            if process.poll() is None:
                os.killpg(process.pid,signal.SIGTERM)
                try:process.wait(timeout=15)
                except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=15)
    original=(out/row['log']).read_text(errors='replace')
    row.update(end=stamp(),elapsed_seconds=time.monotonic()-at,exit_code=process.returncode,log_sha256=sha(out/row['log']),counts=helper.counts(original,kind))
    if result and result.is_file():
        data=json.loads(result.read_text());values=data.get('counts',data)
        row.update(result_path=str(result.relative_to(out)),result_sha256=sha(result))
        row['counts']={'reported':True,**{key:values[key] for key in ('passed','failed','ignored')}}
    row['required_tests']={test:bool(re.search(r'\b'+re.escape(test)+r' \.\.\. ok\b',original)) for test in required}
    row['residual_mounts_after_command']=[str(p) for p in helper.mounts_below(run)]
    row['passed']=process.returncode==0 and not row.get('exception_type') and not row['residual_mounts_after_command'] and all(row['required_tests'].values()) and (kind not in {'rust','gui','json'} or row['counts']['reported'] and row['counts']['passed']>0 and row['counts']['failed']==0)
    save();print(json.dumps({k:row[k] for k in ('name','passed','counts')}),flush=True)
    if interrupted:raise interrupted
    assert not helper.mounts_below(run)
cargo=['cargo','test','--offline','--locked','--no-fail-fast']
try:
    execute('clean-private-product-packages',['cargo','clean','-p','drive-core','-p','drive-fs','-p','drive-runtime','-p','sproto'],'prepare')
    assert summary['commands'][-1]['passed']
    native_case='native_read_refusal_learns_only_the_requested_identity_and_recheck_releases_retry'
    execute('runtime-lib-all-features',cargo+['-p','drive-runtime','--lib','--all-features','--','--test-threads=1'],required=(native_case,'profile_only_notification_invalidates_observed_acl_without_file_events','protocol_version_mismatch_disconnects_all_waiters'))
    rebuilt=(out/'logs/runtime-lib-all-features.log').read_text()
    assert 'Compiling drive-core ' in rebuilt and 'Compiling drive-runtime ' in rebuilt,'fresh product dependency compilation must appear in original log'
    new_elf=pathlib.Path(re.search(r'Running unittests .* \(([^)]+)\)',rebuilt)[1]);blob=new_elf.read_bytes()
    provenance.update(new_test_elf_sha256=sha(new_elf),new_elf_embedded_core_path_counts={name:blob.count(f'/g4-{name}/source/crates/drive-core'.encode()) for name in ('005','006','007')})
    (out/'cache-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
    # Rebuild the existing external crate with identical frozen source and lock.
    execute('native-loopback-build',['cargo','build','--offline','--locked','--manifest-path',str(run/'native-loopback-crate/Cargo.toml')],'prepare')
    if summary['commands'][-1]['passed']:
        execute('native-loopback',[str(target/'debug/native-loopback-probe'),'--work-dir',str(out/'native-loopback-scratch'),'--output',str(out/'native-loopback-results.json')],'json',result=out/'native-loopback-results.json')
    execute('core-history-probe',cargo+['--manifest-path',str(run/'core-history-probe/Cargo.toml'),'--','--test-threads=1'])
    execute('workspace-all-features',cargo+['--workspace','--all-features'],required=(native_case,helper.ACL_RUNTIME_TEST)+helper.ACL_PUBLICATION_TESTS)
    execute('core-fs-runtime-default',cargo+['-p','drive-core','-p','drive-fs','-p','drive-runtime'],required=(native_case,))
    execute('worker-elf-versions',['python3',str(here/'verify-worker-versions.py'),'--worker',str(target/'debug/drive-native-worker'),'--output',str(out/'worker-versions.json')],'json',result=out/'worker-versions.json')
    for name in helper.GUI_MODULES:
        execute('gui-'+name,['dbus-run-session','--',str(helper.GUI_PYTHON),'desktop/tests/'+name+'.py','-v'],'gui',timeout=300)
    execute('gui-permission-history-labels',['dbus-run-session','--',str(helper.GUI_PYTHON),str(here/'verify-history-labels.py'),'-v'],'gui',timeout=300)
    summary['complete']=True
except BaseException as error:
    summary['failure_type']=type(error).__name__;(out/'failure.txt').write_text(traceback.format_exc());raise
finally:
    summary.update(end=stamp(),changed_source_files=[name for name,digest in before.items() if not (source/name).is_file() or sha(source/name)!=digest],residual_owned_mounts=[str(p) for p in helper.mounts_below(run)])
    summary['all_commands_passed']=summary['complete'] and all(r.get('passed') for r in summary['commands']) and not summary['changed_source_files'] and not summary['residual_owned_mounts']
    save()
