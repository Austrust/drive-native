"""Standard offline deb build in a fresh private root; no package installation."""
import datetime,hashlib,importlib.util,json,os,pathlib,shutil,signal,subprocess,sys,tarfile,time,tomllib,traceback
sys.dont_write_bytecode=True
HERE=pathlib.Path(__file__).resolve().parent
BASE=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
G4=BASE/'g4-012';PREPARED=HERE/'prepared-package-001';RUN=BASE/'package-issue7-001'
SOURCE=RUN/'source';TARGET=RUN/'target';REFERENCE='9c9aabae447d92275f93dae65320788240a03113'
def sha(path):
    with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def stamp():return datetime.datetime.now(datetime.timezone.utc).isoformat()

def main():
    # Tiny preparation evidence is outside RUN; a new RUN/target still requires 40 GiB.
    attempt=PREPARED/f'build-attempt-{time.time_ns()}-{os.getpid()}'
    attempt_created=run_created=False;helper=before=None;env={}
    result={'scope':'new private offline standard dpkg build and extracted payload validation; no system installation',
        'started_utc':stamp(),'complete':False,'passed':False,'commands':[],'preparation_steps':[],
        'audit_errors':[],'evidence_errors':[],'attempt_evidence':str(attempt),'run':str(RUN),
        'compiler_cache':'CARGO_INCREMENTAL=0; original profiles and assertions retained'}
    def save(final=False):
        destinations=([attempt/'result.json'] if attempt_created else [])+([RUN/'result.json'] if run_created else [])
        failures=[];written=[]
        for path in destinations:
            try:
                temporary=path.with_name(path.name+'.next')
                temporary.write_text(json.dumps(result,indent=2)+'\n');temporary.replace(path);written.append(path)
            except BaseException as error:failures.append({'path':str(path),'type':type(error).__name__,'error':str(error)})
        if failures:
            result['evidence_errors'].extend(failures);result['passed']=False
            # A successful first destination must not retain passed=True when
            # the second write failed. Rewrite the surviving evidence as failed.
            for path in written:
                try:
                    temporary=path.with_name(path.name+'.next')
                    temporary.write_text(json.dumps(result,indent=2)+'\n');temporary.replace(path)
                except BaseException as error:
                    result['evidence_errors'].append({'path':str(path),'phase':'failure-status-rewrite',
                        'type':type(error).__name__,'error':str(error)})
            print(json.dumps({'evidence_write_errors':failures}),file=sys.stderr,flush=True)
            if not final:raise RuntimeError('cannot persist package evidence; stop before further work')
    def interrupted(signum,frame):raise InterruptedError(f'package driver signal {signum}')
    signal.signal(signal.SIGTERM,interrupted);signal.signal(signal.SIGINT,interrupted)
    def observe(minimum_gib):
        try:observation=helper.host_c_space()
        except BaseException as error:observation={'free_bytes':None,'exception_type':type(error).__name__,'error':str(error)}
        free=observation.get('free_bytes')
        return {**observation,'minimum_free_gib':minimum_gib,'minimum_free_bytes':minimum_gib*1024**3,
            'passed':isinstance(free,int) and free>=minimum_gib*1024**3}
    def preparation(name,action):
        row={'name':name,'started_utc':stamp(),'passed':False}
        result['preparation_steps'].append(row);save()
        try:action();row['passed']=True
        except BaseException as error:
            row.update(exception_type=type(error).__name__,error=str(error));raise
        finally:row['finished_utc']=stamp();save()
    def cleanup_group(process,abnormal):
        if process is None:return {'process_started':False,'confirmed_absent':True}
        record={'pid':process.pid,'pgid':process.pid,'scope':'only session/process group started by this command'}
        handlers={sig:signal.signal(sig,signal.SIG_IGN) for sig in (signal.SIGTERM,signal.SIGINT)}
        try:
            def exists():
                try:os.killpg(process.pid,0);return True
                except ProcessLookupError:return False
            def wait_group(seconds):
                end=time.monotonic()+seconds
                while time.monotonic()<end:
                    process.poll()
                    if not exists():return True
                    time.sleep(.05)
                process.poll();return not exists()
            gone=wait_group(0 if abnormal else 1)
            record['leftover_after_success_grace']=not abnormal and not gone
            for sig,seconds in ((signal.SIGTERM,20),(signal.SIGKILL,10)):
                if gone:break
                record['sent_'+signal.Signals(sig).name]=True
                try:os.killpg(process.pid,sig)
                except ProcessLookupError:pass
                gone=wait_group(seconds)
            record['confirmed_absent']=gone
        except BaseException as error:record.update(confirmed_absent=False,exception_type=type(error).__name__,error=str(error))
        finally:
            for sig,handler in handlers.items():signal.signal(sig,handler)
        return record
    def command(name,argv,timeout=7200,cwd=SOURCE):
        row={'name':name,'argv':argv,'log':'logs/'+name+'.log','cwd':str(cwd),'started_utc':stamp(),'passed':False}
        result['commands'].append(row);save();started=time.monotonic();process=None
        try:
            row['host_space_before']=observe(10);save()
            if not row['host_space_before']['passed']:
                row['not_run']=True;raise RuntimeError('host C >=10GiB required before every package command; unknown/below threshold')
            with (RUN/row['log']).open('wb') as log:
                process=subprocess.Popen(argv,cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
                row['pid']=row['pgid']=process.pid;save()
                row['exit_code']=process.wait(timeout=timeout)
        except BaseException as error:row.update(exception_type=type(error).__name__,error=str(error))
        finally:
            row['process_group_cleanup']=cleanup_group(process,'exception_type' in row)
            if process is not None:row['exit_code']=process.poll()
            row.update(elapsed_seconds=time.monotonic()-started,finished_utc=stamp())
            try:
                if (RUN/row['log']).is_file():row['log_sha256']=sha(RUN/row['log'])
            except BaseException as error:result['audit_errors'].append({'audit':name+' log','type':type(error).__name__,'error':str(error)})
            cleanup=row['process_group_cleanup']
            row['passed']=row.get('exit_code')==0 and 'exception_type' not in row and cleanup['confirmed_absent'] and not cleanup.get('leftover_after_success_grace')
            save()
        assert row['passed'],f'{name} failed; stop before another package command'
        return (RUN/row['log']).read_text(errors='replace')

    try:
        assert os.name=='posix' and os.geteuid()!=0,'ordinary Linux validation user required'
        assert PREPARED.is_dir(),'prepare immutable source before package build'
        attempt.mkdir();attempt_created=True;save()
        assert not RUN.exists(),'never overwrite an existing package run'
        source_path=PREPARED/'source.json';main_path=G4/'summary.json';supplemental_path=G4/'supplemental-001/summary.json'
        source_identity=json.loads(source_path.read_text())
        main_summary=json.loads(main_path.read_text());supplements=json.loads(supplemental_path.read_text())
        result['source']=source_identity
        result['g4_prerequisites']={'reference_commit':REFERENCE,
            'source_identity':{'path':str(source_path),'sha256':sha(source_path)},
            'main_summary':{'path':str(main_path),'sha256':sha(main_path),'candidate_commit':main_summary.get('candidate_commit')},
            'supplemental_summary':{'path':str(supplemental_path),'sha256':sha(supplemental_path),'candidate_commit':supplements.get('candidate_commit')}}
        save()
        assert source_identity['product_reference_commit']==main_summary['candidate_commit']==supplements['candidate_commit']==REFERENCE
        assert source_identity['product_equivalence_exit_code']==0
        assert main_summary['complete'] and main_summary['all_commands_passed']
        assert supplements['complete'] and supplements['all_passed']
        assert supplements['main_summary_sha256']==sha(main_path)
        prepared_result=PREPARED/'preparation-result.json';prepared_summary=json.loads(prepared_result.read_text())
        assert prepared_summary['complete'] and prepared_summary['passed']
        assert prepared_summary['source_json_sha256']==sha(source_path)
        result['g4_prerequisites']['preparation_result']={'path':str(prepared_result),'sha256':sha(prepared_result)}
        helper_path=HERE/'prepared-g4-012/run-g4.py'
        spec=importlib.util.spec_from_file_location('package_g4',helper_path)
        helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
        result['host_space_helper']={'path':str(helper_path),'sha256':sha(helper_path)}
        result['host_space_before_new_target']=observe(40);save()
        assert result['host_space_before_new_target']['passed'],'new package root requires host C >=40GiB; unknown/below threshold'
        RUN.mkdir(mode=0o700);run_created=True;(RUN/'logs').mkdir();save()
        def extract():
            nonlocal before
            archive=PREPARED/source_identity['source_archive_name']
            assert archive.resolve().parent==PREPARED.resolve()
            assert sha(archive)==source_identity['source_archive_sha256']
            with tarfile.open(archive) as stream:stream.extractall(RUN,filter='data')
            before={str(path.relative_to(SOURCE)):sha(path) for path in SOURCE.rglob('*') if path.is_file()}
            result['source_hashes_before']=before
        preparation('verify-and-extract-archive',extract)
        def seed():
            packages=tomllib.loads((SOURCE/'Cargo.lock').read_text())['package']
            names={p['name'] for p in packages if p.get('source','').startswith('registry+')}
            normalized={name.replace('-','_') for name in names}
            record={'source':str(G4/'target'),'copied_files':0,'copied_bytes':0,
                'selection':'Cargo.lock registry names only; no workspace/path packages, test ELF or incremental','product_artifacts_copied':False}
            result['registry_seed']=record
            for mode in ('debug','release'):
                for folder in ('deps','.fingerprint','build'):
                    origin=G4/'target'/mode/folder;destination=TARGET/mode/folder;destination.mkdir(parents=True,exist_ok=True)
                    for item in origin.iterdir():
                        if folder=='deps':
                            if not item.is_file() or item.suffix not in ('.rlib','.rmeta','.so','.d'):continue
                            if item.stem.removeprefix('lib').rsplit('-',1)[0] not in normalized:continue
                            shutil.copy2(item,destination/item.name);record['copied_files']+=1;record['copied_bytes']+=item.stat().st_size
                        elif item.is_dir() and item.name.rsplit('-',1)[0] in names:
                            shutil.copytree(item,destination/item.name)
                            for path in (destination/item.name).rglob('*'):
                                if path.is_file():record['copied_files']+=1;record['copied_bytes']+=path.stat().st_size
                    save()
        preparation('registry-only-seed',seed)
        def environment():
            for path in SOURCE.rglob('*'):
                if path.is_file() and not path.is_symlink():os.utime(path,None)
            env.update(os.environ)
            for key in list(env):
                if key.startswith('DRIVE_NATIVE_') or key in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','DBUS_SESSION_BUS_ADDRESS','DISPLAY','WAYLAND_DISPLAY','DEB_BUILD_OPTIONS','DEB_BUILD_PROFILES'):
                    env.pop(key,None)
            env.update(CARGO_TARGET_DIR=str(TARGET),CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',
                GUI_PYTHON='/opt/drive-native-build/gui-venv/bin/python',PYTHONPATH=str(SOURCE/'desktop'),
                PYTHONDONTWRITEBYTECODE='1',QT_QPA_PLATFORM='offscreen',DRIVE_NATIVE_SOURCE_COMMIT=source_identity['candidate_commit'],
                DRIVE_NATIVE_OFFLINE_DOCS='1',DEB_BUILD_OPTIONS='parallel=2')
            for key,name in [('HOME','home'),('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache'),('XDG_RUNTIME_DIR','runtime'),('TMPDIR','tmp')]:
                folder=RUN/name;folder.mkdir(mode=0o700);env[key]=str(folder)
        preparation('isolated-environment-and-source-mtimes',environment)

        assert command('version',['dpkg-parsechangelog','-S','Version']).strip()==source_identity['version']
        command('documentation-preflight',['python3','scripts/stage-docs.py',str(RUN/'documentation-preflight')])
        command('standard-dpkg-build',['dbus-run-session','--','dpkg-buildpackage','-b','-us','-uc','-j2'])
        package=RUN/f"drive-native_{source_identity['version']}_amd64.deb";assert package.is_file()
        result['package']={'path':str(package),'sha256':sha(package),'bytes':package.stat().st_size}
        command('package-control',['dpkg-deb','--info',str(package)])
        command('package-contents',['dpkg-deb','--contents',str(package)])
        payload=RUN/'payload';control=RUN/'control';payload.mkdir();control.mkdir()
        command('extract-payload',['dpkg-deb','--extract',str(package),str(payload)])
        command('extract-control',['dpkg-deb','--control',str(package),str(control)])
        docs=payload/'usr/share/doc/drive-native'
        assert (docs/'SOURCE_COMMIT').read_text().strip()==source_identity['candidate_commit']
        assert (docs/'BUILD_VERSION').read_text().strip()==source_identity['version']
        assert (docs/'SOURCE_REFERENCE.md').is_file()
        command('packaged-worker-wire',['python3','-B',str(HERE/'verify-worker-versions.py'),
            '--worker',str(payload/'usr/lib/drive-native/drive-native-worker'),'--output',str(RUN/'packaged-worker-versions.json')])
        command('packaged-gui-runtime',[str(payload/'usr/lib/drive-native/gui/drive-native-gui'),'--verify-runtime'],timeout=300)
        result['payload_elfs']={name:sha(payload/'usr/lib/drive-native'/name) for name in ('drive-native-fsd','drive-native-worker','drive-native-setup')}
        result['maintainer_scripts']={path.name:sha(path) for path in control.iterdir() if path.is_file()}
        result['complete']=True
    except BaseException as error:
        result.update(exception_type=type(error).__name__,error=str(error));failure=traceback.format_exc()
        for directory in ([attempt] if attempt_created else [])+([RUN] if run_created else []):
            try:(directory/'failure.txt').write_text(failure)
            except BaseException as secondary:result['evidence_errors'].append({'path':str(directory/'failure.txt'),'error':str(secondary)})
        print(failure,file=sys.stderr,flush=True)
    finally:
        result['changed_source_files']=[];result['residual_owned_mounts']=[]
        try:
            if before is None:result['source_integrity_audit']={'not_run':True,'reason':'source extraction/hash baseline was not completed'}
            else:
                result['changed_source_files']=[name for name,digest in before.items() if not (SOURCE/name).is_file() or sha(SOURCE/name)!=digest]
                result['source_integrity_audit']={'not_run':False}
        except BaseException as error:result['audit_errors'].append({'audit':'source hashes','type':type(error).__name__,'error':str(error)})
        try:
            if run_created and helper is not None:result['residual_owned_mounts']=[str(path) for path in helper.mounts_below(RUN)]
            else:result['mount_audit']={'not_run':True,'reason':'package run was not created'}
        except BaseException as error:result['audit_errors'].append({'audit':'mountinfo','type':type(error).__name__,'error':str(error)})
        result['passed']=result['complete'] and all(row.get('passed') for row in result['commands']) and all(row.get('passed') for row in result['preparation_steps']) and not result['changed_source_files'] and not result['residual_owned_mounts'] and not result['audit_errors'] and not result['evidence_errors']
        result['finished_utc']=stamp();save(final=True)
        print(json.dumps({'result':str(RUN/'result.json') if run_created else None,'attempt_result':str(attempt/'result.json') if attempt_created else None,
            'passed':result['passed'],'error':result.get('error'),'audit_errors':result['audit_errors'],'evidence_errors':result['evidence_errors']}),flush=True)
    return 0 if result['passed'] else 1
if __name__=='__main__':raise SystemExit(main())
