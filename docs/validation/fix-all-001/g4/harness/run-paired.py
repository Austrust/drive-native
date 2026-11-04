"""Private, synthetic-only G5 pairing. Requires an explicit exclusive window.

Uses unmodified upstream FSearch and the same fixture source for both products.
This script creates new run directories and never deletes historical evidence.
Preparation/build are separate coordinator-authorized actions.
"""
import argparse
import datetime
import hashlib
import json
import os
import pathlib
import resource
import select
import signal
import statistics
import subprocess
import threading
import time
import traceback

BASE = pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-benchmark')
FSEARCH = pathlib.Path('/home/drive-native-test/fsearch-bench/bin/fsearch-0.3.1-canonical')
AUDIT = FSEARCH.with_name('fsearch-index-audit')
MANIFEST = pathlib.Path('/home/drive-native-test/real-tree/manifest.tsv')
EXPECTED = {
    'fsearch': 'b6f5406a24e82e6263c9490f20db90b9939e9d627763e620cb2bc0f793d05d8c',
    'audit': '761d6f36315c552af68e2f651d65819cbd8c83550acd135ff075d8991ba063c2',
    'manifest': 'bfed8ecb6a2b87bea417460e10aa0f4874fc0929b5e8ef4bf9998eb74269c193',
}
TREES = {
    '100k': {'entries': 102130, 'directories': 2066, 'generator': ['2000','40','20000'], 'targets': {'first':2.0,'repeat':1.6}},
    'dense-319k': {'entries':319679, 'directories':56073, 'generator':['56007','4','39514'], 'targets':{'first':13.0,'repeat':12.0}},
    'manifest-327750': {'entries':327750, 'directories':56554, 'generator':None, 'targets':{'first':15.0}},
}

def sha(path):
    with pathlib.Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def stamp():
    now=datetime.datetime.now(datetime.timezone.utc)
    return {'utc':now.isoformat(), 'asia_shanghai':now.astimezone(datetime.timezone(datetime.timedelta(hours=8))).isoformat(), 'unix':now.timestamp()}

def write_json(path, value):
    temporary=path.with_name(path.name+'.next')
    temporary.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')
    temporary.replace(path)

def load_sample():
    cpu=pathlib.Path('/proc/stat').read_text().splitlines()[0].split()[1:]
    return {'time':stamp(), 'loadavg':pathlib.Path('/proc/loadavg').read_text().strip(), 'cpu_ticks':list(map(int,cpu))}

def processes():
    # Command names only; never record potentially sensitive process arguments.
    try:
        return subprocess.check_output(['ps','-eo','pcpu,comm','--sort=-pcpu'],text=True,timeout=10).splitlines()[:16]
    except (OSError,subprocess.SubprocessError) as error:
        return {'unavailable':type(error).__name__}

def fixture_cpu(pid):
    fields=pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split()
    return {'user_ticks':int(fields[11]),'system_ticks':int(fields[12]),'rss_pages':int(fields[21])}

def response(process, timeout=300):
    if not select.select([process.stdout],[],[],timeout)[0]:
        raise TimeoutError('fixture response timeout')
    line=process.stdout.readline()
    if not line:
        raise RuntimeError('fixture exited before response')
    return json.loads(line)

def command(process, text, timeout=120):
    process.stdin.write(text+'\n'); process.stdin.flush()
    return response(process,timeout)

def mounted(path):
    target=str(path)
    def decode(value):
        for escaped,plain in [('\\040',' '),('\\011','\t'),('\\012','\n'),('\\134','\\')]:
            value=value.replace(escaped,plain)
        return value
    return any(decode(line.split()[4])==target for line in pathlib.Path('/proc/self/mountinfo').read_text().splitlines())

def stop_fixture(process, mount):
    outcome={'normal':False,'residual_mount':None,'forced_cleanup':False}
    try:
        if process.poll() is None:
            outcome['response']=command(process,'stop',60)
            assert outcome['response'].get('stopped')
            assert outcome['response'].get('hydration_attempts')==0
        process.wait(timeout=30)
        outcome['exit_code']=process.returncode
        outcome['normal']=process.returncode==0 and outcome.get('response',{}).get('stopped') is True
    except BaseException as error:
        outcome['error_type']=type(error).__name__
    finally:
        # The mount/process belong to this freshly created private case only.
        if mounted(mount):
            outcome['forced_cleanup']=True
            try:
                released=subprocess.run(['fusermount3','-u','--',str(mount)],capture_output=True,text=True,timeout=30)
                outcome['unmount_exit']=released.returncode
                outcome['unmount_stderr']=released.stderr
            except (OSError,subprocess.SubprocessError) as error:
                outcome['unmount_error']=type(error).__name__
        if process.poll() is None:
            outcome['forced_cleanup']=True
            try:
                os.killpg(process.pid,signal.SIGTERM)
                try: process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid,signal.SIGKILL); process.wait(timeout=15)
            except (ProcessLookupError,subprocess.SubprocessError) as error:
                outcome['terminate_error']=type(error).__name__
        outcome['residual_mount']=mounted(mount)
    return outcome

def delta_diagnostics(before,after):
    old={entry['stage']:entry for entry in before.get('diagnostics',[])}
    result={}
    for entry in after.get('diagnostics',[]):
        previous=old.get(entry['stage'],{})
        result[entry['stage']]={'count':entry['count']-previous.get('count',0),
            'total_us':entry['total_us']-previous.get('total_us',0),
            'buckets':[a-b for a,b in zip(entry.get('buckets',[]),previous.get('buckets',[0]*32))]}
    return result

def scan(fsearch,audit,root,profile,log,truth):
    conf=profile/'config/fsearch'; conf.mkdir(parents=True,exist_ok=True)
    (conf/'fsearch.conf').write_text('[Database]\nfolder_1_path='+str(root)+'\nfolder_1_active=true\nfolder_1_one_file_system=false\nfolder_1_monitor=false\nfolder_1_scan_after_launch=false\nfolder_1_rescan_after=0\n')
    for child in ['data','cache','runtime']: (profile/child).mkdir(exist_ok=True,mode=0o700)
    (profile/'data/fsearch').mkdir(exist_ok=True,mode=0o700)
    env=os.environ.copy()
    for key in ['DBUS_SESSION_BUS_ADDRESS','DISPLAY','WAYLAND_DISPLAY','G_MESSAGES_DEBUG']:env.pop(key,None)
    env.update(XDG_CONFIG_HOME=str(profile/'config'),XDG_DATA_HOME=str(profile/'data'),XDG_CACHE_HOME=str(profile/'cache'),XDG_RUNTIME_DIR=str(profile/'runtime'),GSETTINGS_BACKEND='memory',NO_AT_BRIDGE='1')
    database=profile/'data/fsearch/fsearch.db'
    def signature():
        if not database.exists():return None
        st=database.stat();return st.st_ino,st.st_size,st.st_mtime_ns,st.st_ctime_ns
    previous=signature(); started=stamp(); load_before=load_sample()
    argv=['dbus-run-session','--','xvfb-run','-a','--server-args=-screen 0 1280x800x24 -nolisten tcp',str(fsearch),'--update-database']
    usage_before=resource.getrusage(resource.RUSAGE_CHILDREN); start=time.monotonic()
    with log.open('w') as output:
        process=subprocess.Popen(argv,env=env,stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
        try: process.wait(timeout=1800)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid,signal.SIGTERM)
            try:process.wait(timeout=15)
            except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=15)
            raise
    elapsed=time.monotonic()-start; usage_after=resource.getrusage(resource.RUSAGE_CHILDREN)
    result={'command':argv,'start':started,'end':stamp(),'load_before':load_before,'load_after':load_sample(),
        'exit_code':process.returncode,'elapsed_seconds':elapsed,'scan_log_sha256':sha(log),
        'client_tree_user_seconds':usage_after.ru_utime-usage_before.ru_utime,
        'client_tree_system_seconds':usage_after.ru_stime-usage_before.ru_stime,
        'client_children_maxrss_kib_so_far':usage_after.ru_maxrss}
    assert process.returncode==0,result
    assert database.is_file(),'FSearch did not save an index'
    assert 'saving failed' not in log.read_text(errors='replace').lower(),'FSearch reported save failure'
    assert signature()!=previous,'repeat did not create a fresh saved-index version'
    result['fresh_index_save_verified']=True
    exported=log.with_suffix('.index.tsv')
    with exported.open('w') as output,log.with_suffix('.audit.log').open('w') as error:
        checked=subprocess.run([str(audit),str(database)],env=env,stdout=output,stderr=error,timeout=180)
    assert checked.returncode==0,'FSearch audit loader rejected saved index'
    observed={}; duplicates=invalid=0
    for line in exported.read_text().splitlines():
        fields=line.split('\t',3)
        if len(fields)!=4:invalid+=1;continue
        directory,mtime,size,path=fields
        try:relative=pathlib.Path(path).relative_to(root).as_posix()
        except ValueError:invalid+=1;continue
        if relative=='.':relative=''
        if relative in observed:duplicates+=1
        observed[relative]={'directory':directory=='1','mtime':int(mtime),'size':int(size)}
    missing=set(truth)-set(observed);extra=set(observed)-set(truth)
    mismatches=sum(value!=observed[key] for key,value in truth.items() if key in observed)
    result.update(index_sha256=sha(database),export_sha256=sha(exported),indexed_count=len(observed),missing=len(missing),extra=len(extra),duplicates=duplicates,invalid=invalid,attribute_mismatches=mismatches)
    write_json(log.with_suffix('.result.json'),result)
    assert not(missing or extra or duplicates or invalid or mismatches),result
    return result

def directory_probe(process,root,truth):
    """Diagnostic only: time serial scandir calls, including FUSE and handler work.

    This is not a pure kernel latency measurement and not a primary FSearch time.
    """
    before=command(process,'sample'); cpu_before=fixture_cpu(process.pid)
    start=time.monotonic(); calls=entries=0; in_call=0.0
    for relative,record in truth.items():
        if not record['directory']:continue
        call_start=time.monotonic()
        with os.scandir(root/relative) as iterator:
            entries+=sum(1 for _ in iterator)
        in_call+=time.monotonic()-call_start; calls+=1
    elapsed=time.monotonic()-start; cpu_after=fixture_cpu(process.pid);after=command(process,'sample')
    assert after['hydration_attempts']==0
    assert entries==len(truth)-1,'scandir diagnostic did not enumerate every non-root entry exactly once'
    return {'kind':'serial-scandir-diagnostic','directories':calls,'enumerated_children':entries,
        'elapsed_seconds':elapsed,'scandir_inflight_seconds':in_call,
        'fixture_cpu_seconds':sum(cpu_after[k]-cpu_before[k] for k in ('user_ticks','system_ticks'))/os.sysconf('SC_CLK_TCK'),
        'diagnostics_delta':delta_diagnostics(before,after),
        'scope':'scandir in-flight time includes user-space FUSE handler, kernel, and scheduling; stage times overlap and cannot be subtracted or added as disjoint components'}

def aggregates(summary):
    result=[]
    for name,tree in TREES.items():
        for role in ('baseline','candidate','local'):
            for state in ('first','repeat'):
                rows=[scan for case in summary['cases'] if case['tree']==name and case['role']==role for scan in case['scans'] if scan['cache_state']==state]
                if not rows:continue
                median=statistics.median(row['elapsed_seconds'] for row in rows)
                target=tree['targets'].get(state) if role=='candidate' else None
                result.append({'tree':name,'role':role,'state':state,'n':len(rows),'median_seconds':median,
                    'median_ms_per_directory':median*1000/tree['directories'],'target_seconds':target,
                    'target_met':median<=target if target is not None else None,
                    'directory_target_ms':0.15 if role=='candidate' else None,
                    'directory_target_met':median*1000/tree['directories']<=0.15 if role=='candidate' else None,
                    'median_fixture_cpu_seconds':statistics.median(row['fixture_cpu_seconds'] for row in rows),
                    'median_client_cpu_seconds':statistics.median(row['client_tree_user_seconds']+row['client_tree_system_seconds'] for row in rows)})
    return result

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--run',required=True,type=pathlib.Path)
    parser.add_argument('--build-manifest',required=True,type=pathlib.Path)
    parser.add_argument('--candidate-commit',required=True)
    parser.add_argument('--mode',choices=('primary','diagnostic'),required=True)
    parser.add_argument('--rounds',type=int)
    parser.add_argument('--only',choices=tuple(TREES),nargs='+')
    parser.add_argument('--exclusive-window-confirmed',action='store_true')
    args=parser.parse_args()
    assert args.exclusive_window_confirmed,'coordinator must authorize exclusive window'
    rounds=args.rounds if args.rounds is not None else (3 if args.mode=='primary' else 1)
    assert rounds>=1 and (args.mode!='primary' or rounds==3),'primary requires three paired rounds'
    run=args.run.resolve();assert run.is_relative_to(BASE.resolve()) and run!=BASE.resolve(),'run must be a new private benchmark child'
    assert not run.exists(),'never overwrite a prior run'
    build=json.loads(args.build_manifest.read_text());assert build.get('build_complete')
    assert build['candidate_commit']==args.candidate_commit
    assert build['baseline_commit'].startswith('d555944')
    for key,path in [('fsearch',FSEARCH),('audit',AUDIT),('manifest',MANIFEST)]:assert sha(path)==EXPECTED[key],key
    suffix='-diagnostic' if args.mode=='diagnostic' else ''
    binaries={role:pathlib.Path(build['binaries'][role+suffix]['path']) for role in ('baseline','candidate')}
    for role,path in binaries.items():assert sha(path)==build['binaries'][role+suffix]['sha256'],role
    run.mkdir(parents=True,mode=0o700);(run/'tmp').mkdir(mode=0o700)
    summary={'complete':False,'mode':args.mode,'build':build,'source_environment':'WSL2; not a Mint/bare-metal measurement',
        'start':stamp(),'start_load':load_sample(),'start_processes':processes(),'kernel':os.uname().release,
        'cpu_count':os.cpu_count(),'clock_ticks':os.sysconf('SC_CLK_TCK'),'page_size':os.sysconf('SC_PAGE_SIZE'),
        'tools_sha256':EXPECTED,'harness_sha256':sha(pathlib.Path(__file__)),
        'rounds':rounds,'first_definition':'new fixture and new FSearch saved database; host page cache is not dropped',
        'scan_clock_definition':'wall clock starts when launching dbus-run-session/xvfb-run/FSearch after fixture ready and any NamespaceIndex build; includes client startup, traversal, sorting and saved-index write, excludes subsequent audit',
        'index_build_accounting':'ready.namespace_index separately records index build microseconds, memory bytes and degradation; these pre-scan costs are excluded from scan wall and must be reported separately',
        'per_directory_definition':'FSearch total wall milliseconds divided by directories including root; includes client/helper startup, sorting and saving; not a FUSE-only cost',
        'directory_probe_definition':'separate diagnostic serial scandir inflight timing; not pooled with primary wall/directory and not pure kernel transport latency',
        'entry_ttl_ms':0,'attribute_ttl_ms':1000,'cases':[],'wsl_guest_samples':[],
        'load_samples_scope':'/proc/stat, /proc/loadavg and process snapshots are WSL guest metrics, not Windows host load. Windows sampling, when available, is separately recorded.',
        'primary_instrumentation':'diagnostics disabled' if args.mode=='primary' else 'diagnostic ELF and counters; never pooled with primary times'}
    save=lambda:write_json(run/'summary.json',summary)
    stopped=threading.Event()
    def sample_guest():
        while not stopped.wait(1):summary['wsl_guest_samples'].append(load_sample())
    thread=threading.Thread(target=sample_guest,daemon=True);thread.start();save()
    try:
        for tree_name in args.only or TREES:
            tree=TREES[tree_name];expected_truth=None
            for round_number in range(1,rounds+1):
                order=(('baseline','candidate','local'),('local','candidate','baseline'),('candidate','baseline','local'))[(round_number-1)%3]
                for role in order:
                    case_dir=run/tree_name/f'{round_number}-{role}';case_dir.mkdir(parents=True)
                    fixture=case_dir/'fixture';mount=fixture/'mount'
                    binary=binaries['baseline' if role=='baseline' else 'candidate']
                    local=role=='local';manifest=tree['generator'] is None
                    mode=('local' if local else 'fuse')+('-manifest' if manifest else '')
                    argv=[str(binary),str(fixture),mode]+([str(MANIFEST)] if manifest else tree['generator'])
                    env=os.environ.copy();env.update(DRIVE_NATIVE_FS_DIAGNOSTICS='1' if args.mode=='diagnostic' else '0',DRIVE_NATIVE_FS_INDEX='1',DRIVE_NATIVE_FS_TTL_MS='1000',TMPDIR=str(run/'tmp'))
                    case={'tree':tree_name,'round':round_number,'role':role,'fixture_command':argv,'mount':str(mount),'scans':[]}
                    summary['cases'].append(case);save()
                    with (case_dir/'fixture.stderr').open('w') as errors:
                        process=subprocess.Popen(argv,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=errors,text=True,bufsize=1,env=env,start_new_session=True)
                        try:
                            ready=response(process,600);case['ready']=ready
                            assert ready.get('ready')
                            if role=='candidate':
                                index=ready.get('namespace_index',{})
                                case['index_ready_without_degradation']=bool(index.get('index_enabled')) and index.get('index_degraded') is False
                            assert ready['directories_including_root']==tree['directories']
                            assert ready['files']+ready['directories_including_root']==tree['entries']
                            truth_path=fixture/'truth.json';truth_sha=sha(truth_path)
                            if expected_truth is None:expected_truth=truth_sha
                            assert expected_truth==truth_sha,'paired fixture truth differs'
                            raw=json.loads(truth_path.read_text());truth={row['path']:{k:row[k] for k in ('directory','size','mtime')} for row in raw}
                            assert len(raw)==len(truth)==tree['entries']
                            case['truth_sha256']=truth_sha;save()
                            for state in ('first','repeat'):
                                before=fixture_cpu(process.pid);sample_before=command(process,'sample')
                                measurement=scan(FSEARCH,AUDIT,mount,case_dir/'profile',case_dir/f'{state}.log',truth)
                                after=fixture_cpu(process.pid);sample_after=command(process,'sample')
                                measurement.update(cache_state=state,
                                    fixture_cpu_seconds=sum(after[k]-before[k] for k in ('user_ticks','system_ticks'))/os.sysconf('SC_CLK_TCK'),
                                    fixture_user_seconds=(after['user_ticks']-before['user_ticks'])/os.sysconf('SC_CLK_TCK'),
                                    fixture_system_seconds=(after['system_ticks']-before['system_ticks'])/os.sysconf('SC_CLK_TCK'),
                                    fixture_rss_bytes=after['rss_pages']*os.sysconf('SC_PAGE_SIZE'),
                                    ms_per_directory=measurement['elapsed_seconds']*1000/tree['directories'],
                                    hydration_before=sample_before['hydration_attempts'],hydration_after=sample_after['hydration_attempts'])
                                if args.mode=='diagnostic':
                                    measurement['diagnostics_delta']=delta_diagnostics(sample_before,sample_after)
                                    if role=='candidate':
                                        measurement['index_hit_observed']=measurement['diagnostics_delta'].get('IndexHits',{}).get('count',0)>0
                                    assert measurement['diagnostics_delta'].get('EntryTtlNonZero',{}).get('count',0)==0
                                assert measurement['hydration_before']==measurement['hydration_after']==0
                                case['scans'].append(measurement);save()
                                print(json.dumps({'tree':tree_name,'round':round_number,'role':role,'state':state,'seconds':measurement['elapsed_seconds'],'truth_ok':True}),flush=True)
                            if args.mode=='diagnostic':case['directory_probe']=directory_probe(process,mount,truth)
                        finally:
                            case['shutdown']=stop_fixture(process,mount);save()
                        assert case['shutdown']['normal'] and not case['shutdown']['forced_cleanup'] and not case['shutdown']['residual_mount'],'fixture did not shut down normally'
        summary['complete']=True
    except BaseException as error:
        summary['failure_type']=type(error).__name__
        (run/'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        stopped.set();thread.join(timeout=5)
        summary.update(end=stamp(),end_load=load_sample(),end_processes=processes(),aggregates=aggregates(summary))
        summary['residual_owned_mounts']=[case['mount'] for case in summary['cases'] if mounted(case['mount'])]
        candidates=[case for case in summary['cases'] if case['role']=='candidate']
        summary['index_ready_without_degradation']=bool(candidates) and all(case.get('index_ready_without_degradation',False) for case in candidates)
        summary['index_hits_all_scans']=all(scan.get('index_hit_observed',False) for case in candidates for scan in case['scans']) if args.mode=='diagnostic' and candidates else None
        save()

if __name__=='__main__':main()
