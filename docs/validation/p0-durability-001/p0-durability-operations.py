import hashlib,json,os,pathlib,signal,sqlite3,time
root=pathlib.Path.home()/'.local/state/drive-native-p0/durability-001';cfg=json.loads((root/'config.json').read_text());mount=pathlib.Path(cfg['mount'])/'lab';state=pathlib.Path(cfg['state'])
signal.signal(signal.SIGALRM,lambda *_:(_ for _ in ()).throw(TimeoutError('operation timeout')))
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def snapshot(label):
    db=sqlite3.connect(state.joinpath('state.sqlite').as_uri()+'?mode=ro',uri=True);db.row_factory=sqlite3.Row;db.execute('BEGIN')
    row=db.execute("SELECT id,remote_version,local_version,blob,dirty_blob,base_version,dirty_generation FROM nodes WHERE id='baseline-file'").fetchone()
    result={'label':label,'time':time.time(),'node':dict(row),'drafts':[dict(r) for r in db.execute('SELECT * FROM drafts')],'operations':[dict(r) for r in db.execute('SELECT * FROM operations')],'namespace_jobs':db.execute('SELECT count(*) FROM namespace_ops').fetchone()[0]}
    for field in ['blob','dirty_blob']:
        if row[field]:result[field+'_sha256']=digest(state/'blobs'/row[field])
    for draft in result['drafts']:draft['sha256']=digest(state/'drafts'/draft['token'])
    db.close();(root/(label+'.json')).write_text(json.dumps(result,indent=2));return result
def step(label,fn):
    signal.alarm(15);start=time.monotonic()
    try:
        fn();row={'step':label,'ok':True,'elapsed_s':time.monotonic()-start}
    except Exception as e:
        row={'step':label,'ok':False,'error':type(e).__name__,'elapsed_s':time.monotonic()-start}
        with (root/'operations.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
        raise
    finally:signal.alarm(0)
    with (root/'operations.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
    print(json.dumps(row),flush=True)
baseline=bytes(i%251 for i in range(4096));edited=b'edited!!'+baseline[8:];assert (mount/'A.bin').read_bytes()==baseline
fd=os.open(mount/'A.bin',os.O_RDWR)
try:
    step('write',lambda:os.pwrite(fd,b'edited!!',0))
    before=snapshot('after-write')
    assert before['node']['dirty_blob'] is None
    assert before['drafts'][0]['sha256']==hashlib.sha256(edited).hexdigest()
    step('fsync',lambda:os.fsync(fd))
    after=snapshot('after-fsync')
    assert after['dirty_blob_sha256']==hashlib.sha256(edited).hexdigest()
    assert any(r['node']=='baseline-file' and r['kind']=='upload' and r['state']=='pending' for r in after['operations'])
finally:os.close(fd)
snapshot('after-close')
replacement=bytes((i*7)%251 for i in range(8192))
def save_replace():
    with (mount/'.A.tmp').open('wb') as f:f.write(replacement);f.flush();os.fsync(f.fileno())
    os.replace(mount/'.A.tmp',mount/'A.bin')
step('atomic-save-replace',save_replace)
def sync_directory(path):
    fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY)
    try:os.fsync(fd)
    finally:os.close(fd)
step('parent-fsync',lambda:sync_directory(mount))
def create_child():
    (mount/'created').mkdir()
    with (mount/'created/B.bin').open('wb') as f:f.write(b'child-fsynced\n');f.flush();os.fsync(f.fileno())
    sync_directory(mount/'created');sync_directory(mount)
step('mkdir-child-fsync',create_child)
assert (mount/'A.bin').read_bytes()==replacement
snapshot('final-before-stop')
