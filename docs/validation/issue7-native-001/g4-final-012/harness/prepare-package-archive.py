"""After final G4 and coordinator documentation freeze: exact local package source."""
import argparse, datetime, hashlib, json, os, pathlib, signal, subprocess, sys, time, traceback

def sha(path):
    with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def stamp():return datetime.datetime.now(datetime.timezone.utc).isoformat()

def main():
    p=argparse.ArgumentParser();p.add_argument('--commit',required=True);p.add_argument('--authorization',required=True)
    a=p.parse_args();here=pathlib.Path(__file__).resolve().parent;repo=here.parent/'fix-7'
    reference='9c9aabae447d92275f93dae65320788240a03113'
    paths=['crates','desktop','vendor','Cargo.toml','Cargo.lock','debian','scripts']
    out=here/'prepared-package-001';created=False
    result={'candidate_commit':a.commit,'product_reference_commit':reference,'authorization':a.authorization,
        'started_utc':stamp(),'complete':False,'passed':False,'commands':[],'audit_errors':[]}
    def save():
        if created:
            temporary=out/'preparation-result.json.next'
            temporary.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
            temporary.replace(out/'preparation-result.json')
    def interrupted(signum,frame):raise InterruptedError(f'package preparation signal {signum}')
    signal.signal(signal.SIGTERM,interrupted);signal.signal(signal.SIGINT,interrupted)
    def cleanup(process,abnormal):
        if process is None:return {'process_started':False,'confirmed_absent':True}
        record={'pid':process.pid,'pgid':process.pid if os.name=='posix' else None}
        handlers={sig:signal.signal(sig,signal.SIG_IGN) for sig in (signal.SIGTERM,signal.SIGINT)}
        try:
            if os.name=='posix':
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
                for sig,seconds in ((signal.SIGTERM,10),(signal.SIGKILL,5)):
                    if gone:break
                    try:os.killpg(process.pid,sig)
                    except ProcessLookupError:pass
                    gone=wait_group(seconds)
                record['confirmed_absent']=gone
                record['scope']='only POSIX session/process group created by this command'
            else:
                # Git diff/archive have no build/test workers. Preserve any
                # Windows tree-termination output rather than claiming a POSIX audit.
                if process.poll() is None:
                    killed=subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'],
                        capture_output=True,text=True,timeout=15)
                    record['taskkill']={'exit_code':killed.returncode,'stdout':killed.stdout,'stderr':killed.stderr}
                process.wait(timeout=15);record['confirmed_absent']=process.poll() is not None
                record['scope']='owned Windows Git parent/tree termination; no POSIX group enumeration'
        except BaseException as error:record.update(confirmed_absent=False,error=str(error),exception_type=type(error).__name__)
        finally:
            for sig,handler in handlers.items():signal.signal(sig,handler)
        return record
    def command(name,argv):
        row={'name':name,'argv':argv,'cwd':str(repo),'started_utc':stamp(),'passed':False,
            'stdout':f'logs/{name}.stdout.log','stderr':f'logs/{name}.stderr.log'}
        result['commands'].append(row);save();started=time.monotonic();process=None
        try:
            with (out/row['stdout']).open('wb') as stdout,(out/row['stderr']).open('wb') as stderr:
                options={'start_new_session':True} if os.name=='posix' else {'creationflags':subprocess.CREATE_NEW_PROCESS_GROUP}
                process=subprocess.Popen(argv,cwd=repo,stdout=stdout,stderr=stderr,**options)
                row.update(pid=process.pid,pgid=process.pid if os.name=='posix' else None);save()
                row['exit_code']=process.wait(timeout=1200)
        except BaseException as error:row.update(exception_type=type(error).__name__,error=str(error))
        finally:
            row['cleanup']=cleanup(process,'exception_type' in row)
            if process is not None:row['exit_code']=process.poll()
            row.update(finished_utc=stamp(),elapsed_seconds=time.monotonic()-started)
            for field in ('stdout','stderr'):
                try:
                    if (out/row[field]).is_file():row[field+'_sha256']=sha(out/row[field])
                except BaseException as error:result['audit_errors'].append({'audit':name+'/'+field,'error':str(error)})
            row['passed']=row.get('exit_code')==0 and 'exception_type' not in row and row['cleanup']['confirmed_absent'] and not row['cleanup'].get('leftover_after_success_grace')
            save()
        assert row['passed'],f'{name} failed; original output preserved'
        return row
    try:
        assert not out.exists(),'never overwrite an existing package preparation'
        out.mkdir();created=True;(out/'logs').mkdir();save()
        assert len(a.commit)==40 and all(c in '0123456789abcdef' for c in a.commit)
        assert a.authorization.strip()
        equivalence=['git','-C',str(repo),'diff','--exit-code',reference,a.commit,'--',*paths]
        equivalent=command('product-equivalence',equivalence)
        archive=out/'drive-native-source.tar.gz'
        archive_command=['git','-C',str(repo),'archive','--format=tar.gz','--prefix=source/','--output='+str(archive),a.commit]
        command('git-archive',archive_command)
        record={'candidate_commit':a.commit,'product_reference_commit':reference,'equivalent_product_paths':paths,
            'product_equivalence_command':equivalence,'product_equivalence_exit_code':equivalent['exit_code'],
            'source_archive_sha256':sha(archive),'source_archive_name':archive.name,'archive_command':archive_command,
            'authorization':a.authorization,'version':'0.1.0~dev20261105.1'}
        (out/'source.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
        result.update(complete=True,source_json_sha256=sha(out/'source.json'),archive_sha256=record['source_archive_sha256'])
    except BaseException as error:
        result.update(exception_type=type(error).__name__,error=str(error));failure=traceback.format_exc()
        if created:
            try:(out/'failure.txt').write_text(failure,encoding='utf-8')
            except BaseException as secondary:result['audit_errors'].append({'audit':'failure.txt','error':str(secondary)})
        print(failure,file=sys.stderr,flush=True)
    finally:
        result['finished_utc']=stamp()
        result['passed']=result['complete'] and all(row.get('passed') for row in result['commands']) and not result['audit_errors']
        try:save()
        except BaseException as error:
            result['passed']=False;result['audit_errors'].append({'audit':'final preparation result','error':str(error)})
        print(json.dumps(result),flush=True)
    return 0 if result['passed'] else 1
if __name__=='__main__':raise SystemExit(main())
