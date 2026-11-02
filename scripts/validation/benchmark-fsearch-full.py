"""Original upstream FSearch complete scans against NEW synthetic real mounts.

Run only in the agreed exclusive timing window, after all binaries are built.
No production mount/profile or NAS endpoint is accepted by this harness.
"""
import argparse,hashlib,json,os,pathlib,select,subprocess,time,resource

def sha(path):return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
def response(process,timeout=120):
    assert select.select([process.stdout],[],[],timeout)[0], 'fixture response timeout'
    line=process.stdout.readline();assert line, 'fixture exited unexpectedly'
    return json.loads(line)
def command(process,text):process.stdin.write(text+'\n');process.stdin.flush();return response(process)
def cpu(pid):
    raw=pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split()
    return {'cpu_ticks':int(raw[11])+int(raw[12]),'rss_pages':int(raw[21])}
def scan(fsearch,audit,root,profile,log,truth):
    conf=profile/'config/fsearch';conf.mkdir(parents=True,exist_ok=True)
    (conf/'fsearch.conf').write_text('[Database]\nfolder_1_path='+str(root)+'\nfolder_1_active=true\nfolder_1_one_file_system=false\nfolder_1_monitor=false\nfolder_1_scan_after_launch=false\nfolder_1_rescan_after=0\n')
    for child in ['data','cache','runtime']: (profile/child).mkdir(exist_ok=True,mode=0o700)
    # The upstream CLI saves directly here; a never-launched GUI has not made it.
    (profile/'data/fsearch').mkdir(exist_ok=True,mode=0o700)
    env=os.environ.copy()
    for key in ['DBUS_SESSION_BUS_ADDRESS','DISPLAY','WAYLAND_DISPLAY','G_MESSAGES_DEBUG']:env.pop(key,None)
    env.update(XDG_CONFIG_HOME=str(profile/'config'),XDG_DATA_HOME=str(profile/'data'),XDG_CACHE_HOME=str(profile/'cache'),XDG_RUNTIME_DIR=str(profile/'runtime'),GSETTINGS_BACKEND='memory',NO_AT_BRIDGE='1')
    database=profile/'data/fsearch/fsearch.db'
    def saved_signature():
        if not database.exists():return None
        st=database.stat();return (st.st_ino,st.st_size,st.st_mtime_ns,st.st_ctime_ns)
    previous_save=saved_signature()
    start=time.monotonic()
    usage_before=resource.getrusage(resource.RUSAGE_CHILDREN)
    with log.open('w') as output:
        completed=subprocess.run(['dbus-run-session','--','xvfb-run','-a','--server-args=-screen 0 1280x800x24 -nolisten tcp',str(fsearch),'--update-database'],env=env,stdout=output,stderr=subprocess.STDOUT,timeout=1800)
    elapsed=time.monotonic()-start
    usage_after=resource.getrusage(resource.RUSAGE_CHILDREN)
    result={'exit_code':completed.returncode,'elapsed_seconds':elapsed,'scan_log_sha256':sha(log),
        'client_tree_user_seconds':usage_after.ru_utime-usage_before.ru_utime,'client_tree_system_seconds':usage_after.ru_stime-usage_before.ru_stime,
        'client_children_maxrss_kib_so_far':usage_after.ru_maxrss}
    assert completed.returncode==0,result
    assert database.is_file(), 'upstream did not save an index'
    assert 'saving failed' not in log.read_text(errors='replace').lower(), 'upstream reported a save failure despite CLI success'
    assert saved_signature()!=previous_save, 'repeat scan left the old database unchanged; no fresh saved-index evidence'
    result['fresh_index_save_verified']=True
    exported=log.with_suffix('.index.tsv')
    with exported.open('w') as output,log.with_suffix('.audit.log').open('w') as error:
        checked=subprocess.run([str(audit),str(database)],env=env,stdout=output,stderr=error,timeout=120)
    assert checked.returncode==0, 'upstream database loader rejected saved index'
    observed={};duplicates=0;invalid=0
    for line in exported.read_text().splitlines():
        fields=line.split('\t',3)
        if len(fields)!=4:invalid+=1;continue
        directory,mtime,size,path=fields
        p=pathlib.Path(path)
        try:relative=p.relative_to(root).as_posix()
        except ValueError:invalid+=1;continue
        if relative=='.':relative=''
        if relative in observed:duplicates+=1
        observed[relative]={'directory':directory=='1','mtime':int(mtime),'size':int(size)}
    missing=set(truth)-set(observed);extra=set(observed)-set(truth)
    mismatches=sum(a!=observed[k] for k,a in truth.items() if k in observed)
    result.update(index_sha256=sha(database),indexed_count=len(observed),missing=len(missing),extra=len(extra),duplicates=duplicates,invalid=invalid,attribute_mismatches=mismatches)
    assert not (missing or extra or duplicates or invalid or mismatches),result
    return result

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--run',required=True,type=pathlib.Path)
    parser.add_argument('--baseline',required=True,type=pathlib.Path)
    parser.add_argument('--fixed',required=True,type=pathlib.Path)
    parser.add_argument('--fsearch',required=True,type=pathlib.Path)
    parser.add_argument('--audit',required=True,type=pathlib.Path)
    parser.add_argument('--groups',type=int,default=2000)
    parser.add_argument('--files',type=int,default=40)
    parser.add_argument('--wide',type=int,default=20000)
    parser.add_argument('--rounds',type=int,default=3)
    parser.add_argument('--exclusive-window-confirmed',action='store_true')
    parser.add_argument('--diagnostics',action='store_true',help='separate diagnostic run, never mix with primary timings')
    args=parser.parse_args();assert args.exclusive_window_confirmed
    args.run=args.run.resolve();assert not args.run.exists();args.run.mkdir(parents=True,mode=0o700)
    summary={'complete':False,'original_fsearch':True,'no_nas':True,'source_environment':'same upgraded WSL environment; not comparable to private Mint strace speed',
        'binaries':{k:sha(getattr(args,k)) for k in ['baseline','fixed','fsearch','audit']},'groups':args.groups,'files_per_group':args.files,'wide_files':args.wide,'rounds':args.rounds,'diagnostics_enabled':args.diagnostics,'results':[]}
    def save(): (args.run/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    save()
    try:
        for round_number in range(args.rounds):
            # Sequential alternating baseline/fixed order; local control uses
            # the identical public synthetic truth and never serves as a mount mirror.
            order=['baseline','fixed','local'] if round_number%2==0 else ['fixed','baseline','local']
            for kind in order:
                case=args.run/f'{round_number+1}-{kind}';case.mkdir()
                binary=args.baseline if kind=='baseline' else args.fixed
                env=os.environ.copy();env['DRIVE_NATIVE_FS_DIAGNOSTICS']='1' if args.diagnostics else '0'
                with (case/'fixture.stderr').open('w') as errors:
                    process=subprocess.Popen([str(binary),str(case/'fixture'),'local' if kind=='local' else 'fuse',str(args.groups),str(args.files),str(args.wide)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=errors,text=True,bufsize=1,env=env)
                    try:
                        ready=response(process,300);assert ready['ready']
                        root=(case/'fixture/mount').resolve()
                        truth={r['path']:{k:r[k] for k in ['directory','size','mtime']} for r in json.loads((case/'fixture/truth.json').read_text())}
                        result={'kind':kind,'round':round_number+1,'ready':ready,'scans':[]};summary['results'].append(result);save()
                        for state in ['first','repeat']:
                            before=cpu(process.pid);sample_before=command(process,'sample')
                            measurement=scan(args.fsearch,args.audit,root,case/'profile',case/f'{state}.log',truth)
                            after=cpu(process.pid);sample_after=command(process,'sample')
                            measurement.update(cache_state=state,fixture_cpu_ticks=after['cpu_ticks']-before['cpu_ticks'],fixture_rss_pages=after['rss_pages'],hydrations_before=sample_before['hydration_attempts'],hydrations_after=sample_after['hydration_attempts'])
                            if args.diagnostics:measurement.update(diagnostics_before=sample_before.get('diagnostics'),diagnostics_after=sample_after.get('diagnostics'))
                            assert measurement['hydrations_after']==0
                            result['scans'].append(measurement);save()
                    finally:
                        if process.poll() is None:
                            stopped=command(process,'stop');assert stopped['stopped']
                        process.wait(timeout=30)
                        assert process.returncode==0,'fixture normal shutdown failed'
        summary['complete']=True;save()
    except BaseException as error:
        summary['failure_type']=type(error).__name__;save();raise

if __name__=='__main__':main()
