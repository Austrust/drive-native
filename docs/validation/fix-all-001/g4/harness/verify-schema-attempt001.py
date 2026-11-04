"""Run frozen old/new readers only against newly-created private synthetic DBs."""
import hashlib,json,pathlib,shutil,sqlite3,subprocess,sys
run=pathlib.Path(sys.argv[1]).resolve()
assert run.name=='schema-g4-001' and run.parent==pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
results=[]
def call(schema,state,accepted):
    command=[str(run/'bins'/f'schema{schema}'),str(state)]
    result=subprocess.run(command,capture_output=True,text=True,timeout=60)
    row={'command':command,'exit_code':result.returncode,'stdout':result.stdout,'stderr':result.stderr,'expected_accept':accepted}
    row['passed']=(result.returncode==0) if accepted else (result.returncode==23 and 'newer' in result.stderr.lower())
    results.append(row)
    (run/'results.json').write_text(json.dumps({'complete':False,'results':results},indent=2)+'\n')
    assert row['passed'],row
def version(state):
    with sqlite3.connect(state/'state.sqlite') as db:return db.execute('PRAGMA user_version').fetchone()[0]
def fingerprint(state):
    return {p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in state.iterdir() if p.is_file()}
state23=run/'state'/'v23';call(23,state23,True);assert version(state23)==23
state24=run/'state'/'v24';shutil.copytree(state23,state24);call(24,state24,True);assert version(state24)==24
before24=fingerprint(state24);call(23,state24,False);assert fingerprint(state24)==before24
state25=run/'state'/'v25';shutil.copytree(state24,state25);call(25,state25,True);assert version(state25)==25
before25=fingerprint(state25);call(23,state25,False);call(24,state25,False);assert fingerprint(state25)==before25
for state in (state23,state24,state25):
    with sqlite3.connect(state/'state.sqlite') as db:assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
result={'complete':True,'passed':len(results),'failed':0,'ignored':0,'results':results,
        'readers':{str(v):hashlib.sha256((run/'bins'/f'schema{v}').read_bytes()).hexdigest() for v in (23,24,25)},
        'versions':[version(s) for s in (state23,state24,state25)],'rejected_readers_left_database_bytes_unchanged':True}
(run/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'schema_probe_passed':len(results)}))
