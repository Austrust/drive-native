"""Independent G4 execution against an immutable source archive; no product edits.

All subprocesses are offline/local fixtures. An explicit coordinator gate is
required. Every command, original log, count and failed exit stays in a new run.
"""
import argparse
import datetime
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tarfile
import threading
import tomllib
import time
import traceback

BASE=Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
GUI_PYTHON=Path('/opt/drive-native-build/gui-venv/bin/python')
GUI_MODULES=(
    'gui_unit_tests','cache_gui_tests','conflict_gui_tests','hidden_dialog_tests',
    'lazy_gui_tests','multi_mount_tests','trust_gui_tests','transfer_gui_tests',
    'theme_tests','selector_gui_tests','backend_upgrade_test','pause_handover_test',
    'initialization_gui_tests','unified_manager_tests','gui_instance_test',
    'queue_gui_tests','history_gui_tests','permission_gui_tests',
)
ACL_KERNEL_TESTS=(
    'root_a_b_and_action_classes_are_enforced_in_one_mount',
    'live_revocation_restoration_and_list_vs_traverse',
    'target_parent_cover_target_and_independent_submount',
    'setattr_only_persists_mtime_without_granting_content_writes_or_uploads',
    'parent_self_permissions_do_not_deny_independent_child_removal_or_rename',
)
ACL_RUNTIME_TEST='definite_permission_refusal_retains_body_allows_sibling_and_rechecks'
ACL_PUBLICATION_TESTS=(
    'first_denial_publication_waits_for_existing_bounded_reply',
    'cached_reply_lease_does_not_wait_for_ordinary_metadata_store',
)
NSI_ONLY_KERNEL_TESTS=(
    'indexed_plain_continuation_and_seek_do_not_wait_for_store',
    'indexed_plus_continuation_and_seek_do_not_wait_for_store',
)

def sha(path):
    with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()

def stamp():return datetime.datetime.now(datetime.timezone.utc).isoformat()

def save(path,value):
    temporary=path.with_name(path.name+'.next')
    temporary.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')
    temporary.replace(path)

def counts(text,kind):
    if kind=='rust':
        matches=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
        return {'reported':bool(matches),'passed':sum(int(m[0]) for m in matches),
            'failed':sum(int(m[1]) for m in matches),'ignored':sum(int(m[2]) for m in matches),
            'measured':sum(int(m[3]) for m in matches),'filtered':sum(int(m[4]) for m in matches)}
    if kind=='gui':
        runs=re.findall(r'^Ran (\d+) tests? in ',text,re.MULTILINE)
        failed=sum(int(n) for n in re.findall(r'(?:failures|errors)=(\d+)',text))
        ignored=sum(int(n) for n in re.findall(r'skipped=(\d+)',text))
        total=sum(map(int,runs))
        return {'reported':bool(runs),'passed':max(0,total-failed-ignored),'failed':failed,'ignored':ignored}
    return {'reported':False}

def mounts_below(root):
    result=[]
    for line in Path('/proc/self/mountinfo').read_text().splitlines():
        raw=line.split()[4]
        for escaped,plain in [('\\040',' '),('\\011','\t'),('\\012','\n'),('\\134','\\')]:raw=raw.replace(escaped,plain)
        path=Path(raw)
        if path.is_relative_to(root):result.append(path)
    return sorted(result,key=lambda p:len(p.parts),reverse=True)

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--archives',type=Path,required=True)
    parser.add_argument('--run',type=Path,required=True)
    parser.add_argument('--candidate-commit',required=True)
    parser.add_argument('--g4-authorized',action='store_true')
    parser.add_argument('--copy-target',type=Path)
    parser.add_argument('--registry-seed',type=Path)
    parser.add_argument('--nautilus-private-prefix',type=Path)
    parser.add_argument('--preflight-only',action='store_true')
    args=parser.parse_args()
    assert not (args.copy_target and args.registry_seed),'choose one seed strategy'
    assert args.g4_authorized,'G3 completion and explicit G4 authorization are required'
    run=args.run.resolve()
    assert run.is_relative_to(BASE) and run!=BASE and not run.exists(),'use a new isolated G4 child'
    manifest=json.loads((args.archives/'source.json').read_text())
    assert manifest['candidate_commit']==args.candidate_commit
    archive=args.archives/'candidate.tar'
    assert sha(archive)==manifest['candidate_archive_sha256']
    assert GUI_PYTHON.is_file()
    run.mkdir(parents=True,mode=0o700)
    source=run/'source';source.mkdir();(run/'logs').mkdir();(run/'tmp').mkdir(mode=0o700)
    with tarfile.open(archive) as stream:stream.extractall(source,filter='data')
    shutil.copy2(args.archives/'source.json',run/'source.json')
    target=run/'target'
    registry_seed_record=None
    if args.registry_seed:
        # Cargo.lock identifies registry packages; no workspace/path package,
        # test ELF, incremental or hardlink is retained from the old target.
        packages=tomllib.loads((source/'Cargo.lock').read_text())['package']
        names={item['name'] for item in packages if item.get('source','').startswith('registry+')}
        normalized={name.replace('-','_') for name in names}
        copied_files=copied_bytes=0
        for mode in ('debug','release'):
            for folder in ('deps','.fingerprint','build'):
                origin=args.registry_seed.resolve()/mode/folder
                dest=target/mode/folder;dest.mkdir(parents=True,exist_ok=True)
                if not origin.exists():continue
                for item in origin.iterdir():
                    if folder=='deps':
                        if not item.is_file() or item.suffix not in ('.rlib','.rmeta','.so','.d'):continue
                        stem=item.stem.removeprefix('lib').rsplit('-',1)[0]
                        if stem not in normalized:continue
                        shutil.copy2(item,dest/item.name)
                        copied_files+=1;copied_bytes+=item.stat().st_size
                    elif item.is_dir() and item.name.rsplit('-',1)[0] in names:
                        shutil.copytree(item,dest/item.name,symlinks=False)
                        for file in (dest/item.name).rglob('*'):
                            if file.is_file():copied_files+=1;copied_bytes+=file.stat().st_size
        registry_seed_record={'source':str(args.registry_seed),'copied_files':copied_files,'copied_bytes':copied_bytes,
            'selection':'Cargo.lock registry+ packages only; debug/release deps/fingerprint/build; copied bytes, no workspace/path packages, no test ELFs or incremental'}
    elif args.copy_target:
        # copytree copies bytes; there are no shared targets or hardlinks.
        shutil.copytree(args.copy_target.resolve(),target,symlinks=True)
    else:target.mkdir()
    source_hashes={str(path.relative_to(source)):sha(path) for path in source.rglob('*') if path.is_file()}
    save(run/'source-hashes-before.json',source_hashes)
    # Git archive preserves commit mtimes, which can predate copied target
    # artifacts. Invalidate source mtimes without changing any archived bytes.
    source_mtime_reset_at=stamp()
    for path in source.rglob('*'):
        if path.is_file() and not path.is_symlink():os.utime(path,None)
    assert all(sha(source/name)==value for name,value in source_hashes.items())
    env=os.environ.copy()
    for key in list(env):
        if key.startswith('DRIVE_NATIVE_') or key in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','DBUS_SESSION_BUS_ADDRESS','DISPLAY','WAYLAND_DISPLAY'}:env.pop(key,None)
    env.update(CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',
        TMPDIR=str(run/'tmp'),RUST_BACKTRACE='1',PYTHONDONTWRITEBYTECODE='1',
        PYTHONPATH=str(source/'desktop'),QT_QPA_PLATFORM='offscreen',
        DRIVE_NATIVE_FS_INDEX='1',DRIVE_NATIVE_FS_TTL_MS='1000')
    for key,relative in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache'),('XDG_RUNTIME_DIR','runtime')]:
        path=run/relative;path.mkdir(mode=0o700);env[key]=str(path)
    summary={'complete':False,'candidate_commit':args.candidate_commit,'source':manifest,
        'start':stamp(),'environment':'WSL2 Ubuntu-24.04; synthetic/local only; no NAS',
        'target_seed':str(args.copy_target) if args.copy_target else None,
        'registry_seed':registry_seed_record,
        'harness_sha256':sha(Path(__file__)),'commands':[],
        'source_mtime_reset_at':source_mtime_reset_at,
        'source_content_unchanged_after_mtime_reset':True,
        'index0_not_applicable':{name:'Explicitly asserts NSI is enabled; fully exercised by index1 debug/release and attr0 groups.' for name in NSI_ONLY_KERNEL_TESTS},
        'count_note':'Executions are separate; repeated groups are not additive distinct-test coverage.'}
    nautilus_environment={}
    if args.nautilus_private_prefix:
        dependency=args.nautilus_private_prefix.resolve()
        assert dependency.is_relative_to(BASE),'only previously prepared private G4 dependencies'
        assert (dependency/'usr/lib/x86_64-linux-gnu/girepository-1.0/Nautilus-4.0.typelib').is_file()
        isolated=run/'nautilus-prefix'
        shutil.copytree(dependency,isolated,symlinks=True)
        summary['nautilus_private_dependency']={'source_prefix':str(dependency),
            'copied_file_sha256':{str(p.relative_to(isolated)):sha(p) for p in isolated.rglob('*') if p.is_file()},
            'scope':'Ubuntu packages privately extracted, no system installation; initial system-GI failure remains in G4-001/G4-002 originals.'}
        nautilus_environment={'GI_TYPELIB_PATH':str(isolated/'usr/lib/x86_64-linux-gnu/girepository-1.0'),
            'LD_LIBRARY_PATH':str(isolated/'usr/lib/x86_64-linux-gnu')}
    def persist():save(run/'summary.json',summary)
    def execute(name,argv,kind='rust',updates=None,timeout=3600,required_tests=(),result_path=None):
        control_path=run/'coordinator-control.json'
        if control_path.is_file():
            control=json.loads(control_path.read_text())
            if any(fnmatch.fnmatchcase(name,pattern) for pattern in control.get('defer_commands',[])):
                summary['commands'].append({'name':name,'command':argv,'kind':kind,'not_run':True,
                    'coordinator_reason':control['reason'],'counts':{'reported':False},'start':stamp()})
                persist();print(json.dumps({'command':name,'not_run':True,'reason':control['reason']}),flush=True)
                return
        output=run/'logs'/f'{name}.log';command_env=env.copy();command_env.update(updates or {})
        row={'name':name,'command':argv,'environment_overrides':updates or {},'kind':kind,'start':stamp(),'log':str(output.relative_to(run))}
        summary['commands'].append(row);persist();started=time.monotonic()
        interrupted=None
        with output.open('w') as log:
            process=subprocess.Popen(argv,cwd=source,env=command_env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
            try:process.wait(timeout=timeout)
            except BaseException as error:
                row['timeout']=isinstance(error,subprocess.TimeoutExpired)
                row['exception_type']=type(error).__name__
                if not row['timeout']:interrupted=error
                if process.poll() is None:
                    os.killpg(process.pid,signal.SIGTERM)
                    try:process.wait(timeout=15)
                    except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=15)
        original=output.read_text(errors='replace')
        row.update(end=stamp(),elapsed_seconds=time.monotonic()-started,exit_code=process.returncode,log_sha256=sha(output),counts=counts(original,kind))
        if result_path and result_path.is_file():
            data=json.loads(result_path.read_text()); values=data.get('counts',data)
            row['counts']={'reported':True,**{key:values[key] for key in ('passed','failed','ignored')}}
            row['result_path']=str(result_path.relative_to(run));row['result_sha256']=sha(result_path)
        row['required_tests']={name:bool(re.search(r'\b'+re.escape(name)+r' \.\.\. ok\b',original)) for name in required_tests}
        residual=mounts_below(run);row['residual_mounts_after_command']=[str(path) for path in residual]
        if residual:
            row['forced_unmounts']=[]
            for path in residual:
                try:
                    cleanup=subprocess.run(['fusermount3','-u','--',str(path)],capture_output=True,text=True,timeout=30)
                    row['forced_unmounts'].append({'mount':str(path),'exit_code':cleanup.returncode,'stderr':cleanup.stderr})
                except (OSError,subprocess.SubprocessError) as error:row['forced_unmounts'].append({'mount':str(path),'error':type(error).__name__})
        row['passed']=process.returncode==0 and not row.get('exception_type') and not residual and all(row['required_tests'].values()) and (kind not in {'rust','gui','json'} or (row['counts']['reported'] and row['counts']['passed']>0 and row['counts']['failed']==0))
        persist();print(json.dumps({'command':name,'passed':row['passed'],'counts':row['counts']}),flush=True)
        if interrupted:raise interrupted
        if mounts_below(run):raise RuntimeError('private synthetic mount remains; stop before another command')
    cargo=['cargo','test','--offline','--locked','--no-fail-fast']
    try:
        execute('clean-private-product-packages',['cargo','clean','-p','drive-core','-p','drive-fs','-p','drive-runtime','-p','sproto'],'prepare')
        assert summary['commands'][-1]['passed'],'must remove copied product cache before validating archived bytes'
        if (source/'crates/drive-runtime/src/download_retry_clock.rs').is_file():
            clock_cases=('forward_wall_jump_cannot_shorten_a_published_delay',
                'restart_remaining_delay_is_seeded_once_and_reset_releases_it',
                'new_attempts_versions_and_terminal_failures_do_not_reuse_an_old_gate',
                'backward_wall_jump_does_not_bypass_the_durable_deadline')
            execute('retry-clock-preflight',cargo+['-p','drive-runtime','--lib','--all-features','download_retry_clock::tests','--','--nocapture'],required_tests=clock_cases)
            obsolete_case='obsolete_ticket_cannot_displace_current_version_retry_gate'
            execute('obsolete-ticket-preflight',cargo+['-p','drive-runtime','--lib','--all-features',obsolete_case,'--','--nocapture'],required_tests=(obsolete_case,))
            clock_samples=[];clock_stop=threading.Event()
            def sample_retry_clocks():
                while not clock_stop.is_set():
                    before=time.monotonic_ns();wall=time.time_ns();after=time.monotonic_ns()
                    clock_samples.append({'monotonic_before_ns':before,'realtime_ns':wall,'monotonic_after_ns':after})
                    clock_stop.wait(.01)
            clock_thread=threading.Thread(target=sample_retry_clocks);clock_thread.start()
            try:
                original_cases=('concurrent_foreground_reads_wait_for_shared_retry_without_reopen',
                    'concurrent_foreground_reads_wait_for_transport_backoff_after_reconnect')
                execute('original-retry-pair-preflight',cargo+['-p','drive-runtime','--lib','--all-features','concurrent_foreground_reads_wait_for_','--','--nocapture'],required_tests=original_cases)
                execute('runtime-lib-original-parallelism',cargo+['-p','drive-runtime','--lib','--all-features','--','--nocapture'],required_tests=original_cases+clock_cases+(obsolete_case,))
            finally:
                clock_stop.set();clock_thread.join()
                save(run/'retry-preflight-clocks.json',{'scope':'adjacent guest-clock sampling for original unchanged candidate tests; no assertion or test parallelism override',
                    'read_order':'monotonic_before -> realtime -> monotonic_after','requested_interval_seconds':.01,'samples':clock_samples})
        native_read_case='native_read_refusal_learns_only_the_requested_identity_and_recheck_releases_retry'
        execute('native-read-stale-preflight',cargo+['-p','drive-runtime','--lib','--all-features',native_read_case,'--','--test-threads=1'],required_tests=(native_read_case,))
        if args.preflight_only:
            summary['scope']='single stale-read preflight; coordinator requested no wider matrix on superseded candidate'
            summary['acl_screenshots_complete']=True
            summary['complete']=True
            return
        harness=Path(__file__).resolve().parent
        execute('runtime-permission-preflight',cargo+['-p','drive-runtime','--lib','--all-features','permission','--','--test-threads=1'],required_tests=(ACL_RUNTIME_TEST,'local_io_and_transport_errors_are_not_native_permission_evidence'))
        execute('runtime-profile-preflight',cargo+['-p','drive-runtime','--lib','--all-features','profile_only_notification','--','--test-threads=1'],required_tests=('profile_only_notification_invalidates_observed_acl_without_file_events',))
        execute('runtime-version-preflight',cargo+['-p','drive-runtime','--lib','--all-features','protocol_version_mismatch','--','--test-threads=1'],required_tests=('protocol_version_mismatch_disconnects_all_waiters',))
        execute('core-permission-preflight',cargo+['-p','drive-core','--lib','--all-features','write_permissions::tests','--','--test-threads=1'],required_tests=('parent_self_denials_do_not_stall_child_remove_or_rename_journals',))
        migration_case='v6_migration_preserves_queued_move_and_inflight_becomes_uncertain'
        execute('core-v6-migration-preflight',cargo+['-p','drive-core','--test','removals','--all-features',migration_case,'--','--exact','--test-threads=1'],required_tests=(migration_case,))
        parent_case='parent_self_permissions_do_not_deny_independent_child_removal_or_rename'
        execute('kernel-parent-actions-preflight',cargo+['-p','drive-fs','--test','permissions_kernel','--all-features',parent_case,'--','--ignored','--exact','--test-threads=1'],updates={'DRIVE_NATIVE_FS_DIAGNOSTICS':'1'},required_tests=(parent_case,))
        execute('sproto-permission-preflight',cargo+['-p','sproto','--lib','--all-features','permission','--','--test-threads=1'])
        execute('sproto-profile-preflight',cargo+['-p','sproto','--lib','--all-features','profile_only_changes','--','--test-threads=1'],required_tests=('profile_only_changes_require_a_digest_and_preserve_the_notification_bit',))
        execute('native-loopback',['bash',str(harness/'run-native-loopback.sh'),str(run),args.candidate_commit],'json',result_path=run/'native-loopback-results.json')
        execute('worker-trust-preflight',cargo+['-p','drive-runtime','--test','trust_worker','--all-features','--','--test-threads=1'])
        execute('worker-elf-versions',['python3',str(harness/'verify-worker-versions.py'),'--worker',str(target/'debug/drive-native-worker'),'--output',str(run/'worker-versions.json')],'json',result_path=run/'worker-versions.json')
        execute('core-history-probe-prepare',['python3',str(harness/'prepare-core-probe.py'),'--source',str(source),'--output',str(run/'core-history-probe')],'prepare')
        if summary['commands'][-1]['passed']:
            execute('core-history-probe',cargo+['--manifest-path',str(run/'core-history-probe/Cargo.toml'),'--','--test-threads=1'])
        setattr_case='setattr_only_persists_mtime_without_granting_content_writes_or_uploads'
        execute('kernel-setattr-preflight',cargo+['-p','drive-fs','--test','permissions_kernel','--all-features',setattr_case,'--','--ignored','--exact','--test-threads=1'],updates={'DRIVE_NATIVE_FS_DIAGNOSTICS':'1'},required_tests=(setattr_case,))
        execute('workspace-all-features',cargo+['--workspace','--all-features'],required_tests=(ACL_RUNTIME_TEST,)+ACL_PUBLICATION_TESTS)
        execute('workspace-default',cargo+['--workspace'])
        # fuser is excluded from the workspace and has no committed Cargo.lock.
        # Its isolated offline lock is retained; product source stays unchanged.
        execute('vendored-fuser-wire',['cargo','test','--offline','--manifest-path','vendor/fuser/Cargo.toml','--lib','--no-default-features','--features','abi-7-31'])
        for mode in ('debug','release'):
            release=['--release'] if mode=='release' else []
            for index in ('1','0'):
                unsuitable=[part for name in NSI_ONLY_KERNEL_TESTS for part in ('--skip',name)] if index=='0' else []
                execute(f'kernel-fs-{mode}-index{index}',cargo+release+['-p','drive-fs','--lib','--tests','--all-features','--','--ignored','--test-threads=1']+unsuitable,updates={'DRIVE_NATIVE_FS_DIAGNOSTICS':'1','DRIVE_NATIVE_FS_INDEX':index},required_tests=ACL_KERNEL_TESTS)
            execute(f'kernel-runtime-{mode}',cargo+release+['-p','drive-runtime','--lib','--all-features','--','--ignored','--test-threads=1'],updates={'DRIVE_NATIVE_FS_DIAGNOSTICS':'1'})
        execute('kernel-fs-release-attr0',cargo+['--release','-p','drive-fs','--lib','--tests','--all-features','--','--ignored','--test-threads=1'],updates={'DRIVE_NATIVE_FS_DIAGNOSTICS':'1','DRIVE_NATIVE_FS_TTL_MS':'0'},required_tests=ACL_KERNEL_TESTS)
        execute('runtime-private-dbus',['dbus-run-session','--']+cargo+['-p','drive-runtime','--test','desktop','--all-features','recovery_action_real_dbus_preserves_unknown_queue_and_user_pause','--','--ignored','--exact','--test-threads=1'])
        for name in GUI_MODULES:
            screenshot_env={'DRIVE_NATIVE_TEST_SCREENSHOTS':str(run/'acl-screenshots')} if name=='permission_gui_tests' else None
            execute('gui-'+name,['dbus-run-session','--',str(GUI_PYTHON),'desktop/tests/'+name+'.py','-v'],'gui',updates=screenshot_env,timeout=300)
        execute('gui-permission-history-labels',['dbus-run-session','--',str(GUI_PYTHON),str(harness/'verify-history-labels.py'),'-v'],'gui',timeout=300)
        summary['acl_screenshots']={name:sha(run/'acl-screenshots'/f'acl-{name}.png') if (run/'acl-screenshots'/f'acl-{name}.png').is_file() else None for name in ('user','space','directory','unknown')}
        summary['acl_screenshots_complete']=all(summary['acl_screenshots'].values())
        for host in ('nemo','nautilus'):
            provider_env={'DRIVE_NATIVE_TEST_FILEMANAGER':host}
            if host=='nautilus':provider_env.update(nautilus_environment)
            execute('provider-'+host,['dbus-run-session','--','/usr/bin/python3','desktop/tests/filemanager_provider_tests.py','-v'],'gui',updates=provider_env,timeout=300)
        execute('unified-ui-render',['dbus-run-session','--',str(GUI_PYTHON),'desktop/tests/unified_gui_render.py',str(run/'screenshots')],'render',timeout=300)
        summary['complete']=True
    except BaseException as error:
        summary['failure_type']=type(error).__name__
        (run/'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        summary['end']=stamp()
        summary['residual_owned_mounts']=[str(path) for path in mounts_below(run)]
        summary['changed_source_files']=[relative for relative,before in source_hashes.items() if not (source/relative).is_file() or sha(source/relative)!=before]
        summary['new_source_files']=[str(path.relative_to(source)) for path in source.rglob('*') if path.is_file() and str(path.relative_to(source)) not in source_hashes]
        summary['command_counts']={key:sum(row.get('passed') is value for row in summary['commands']) for key,value in [('passed',True),('failed',False)]}
        summary['command_counts']['not_run']=sum(bool(row.get('not_run')) for row in summary['commands'])
        summary['all_commands_passed']=summary['complete'] and all(row.get('passed') for row in summary['commands']) and summary.get('acl_screenshots_complete',False) and not summary['changed_source_files'] and not summary['residual_owned_mounts']
        persist()

if __name__=='__main__':main()
