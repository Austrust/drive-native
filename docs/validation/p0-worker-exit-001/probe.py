"""Observe first kernel exit notification for each existing worker; no attachment or signals."""
import asyncio,hashlib,json,os,pathlib,subprocess,time
base=pathlib.Path.home()/'.local/state/drive-native-p0'
out=base/'worker-exit-probe-001'
out.mkdir(mode=0o700,exist_ok=False)
(out/'probe.py').write_bytes(pathlib.Path(__file__).read_bytes())
catalog=json.loads((base.parent/'drive-native-soak/catalog.json').read_text())
allowed=json.loads((base/'manifest.json').read_text())['allowed_binary_sha256']
def append(row):
    with (out/'events.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
def stat(pid):
    return (pathlib.Path('/proc')/str(pid)/'stat').read_text().rsplit(')',1)[1].split()
async def observe(task):
    fsd=int(subprocess.check_output(['systemctl','--user','show','drive-native-soak@'+task['instance']+'.service','-p','MainPID','--value']))
    children=set(int(c) for p in pathlib.Path(f'/proc/{fsd}/task').glob('*/children') for c in p.read_text().split())
    if len(children)!=1:return {'task':task['instance'],'result':'initial_worker_count_not_one','count':len(children)}
    worker=children.pop();fd=os.pidfd_open(worker)
    try:
        fields=stat(worker);start_ticks=int(fields[19]);hz=os.sysconf('SC_CLK_TCK')
        digest=hashlib.sha256(pathlib.Path(f'/proc/{worker}/exe').read_bytes()).hexdigest()
        if digest not in allowed:raise RuntimeError('binary identity mismatch')
        initial={'task':task['instance'],'fsd':fsd,'worker':worker,'start_ticks':start_ticks,'hz':hz,'binary_sha256':digest,
          'time':time.time(),'boot_time_s':time.clock_gettime(time.CLOCK_BOOTTIME),'kind':'watching_worker_exit'}
        append(initial)
        loop=asyncio.get_running_loop();future=loop.create_future()
        def exited():
            if not future.done():future.set_result(time.clock_gettime(time.CLOCK_BOOTTIME))
        loop.add_reader(fd,exited)
        try:
            exit_boot=await asyncio.wait_for(future,600)
            result={**initial,'kind':'worker_exit','observed_at':time.time(),'exit_boot_s':exit_boot,
              'lifetime_s':exit_boot-start_ticks/hz,'fsd_still_present':pathlib.Path(f'/proc/{fsd}').exists(),
              'exit_reason':'not available from non-parent pidfd; no assertion about TCP failure or signal'}
        except asyncio.TimeoutError:
            result={**initial,'kind':'deadline_no_exit','observed_at':time.time(),'wait_limit_s':600}
        finally:loop.remove_reader(fd)
        append(result);return result
    finally:os.close(fd)
async def main():
    with (base/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'start':time.time(),'run':'worker-exit-probe-001','kind':'read-only pidfd exit notifications; first existing worker per task; 600s bound; no NAS calls or process signals'})+'\n')
    results=await asyncio.gather(*(observe(t) for t in catalog),return_exceptions=True)
    serialized=[{'probe_error':type(x).__name__} if isinstance(x,BaseException) else x for x in results]
    (out/'result.json').write_text(json.dumps({'finished_at':time.time(),'results':serialized,'scope':'first observed worker per task; not full stability or protocol root cause'},indent=2))
    with (base/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'end':time.time(),'run':'worker-exit-probe-001'})+'\n')
    print(json.dumps({'complete':True,'results':[{k:r.get(k) for k in ['task','kind','lifetime_s','probe_error']} for r in serialized]}))
asyncio.run(main())
