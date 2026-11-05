"""Run unchanged failing test cases from a copied ELF, with adjacent dual-clock samples."""
import datetime,hashlib,json,os,pathlib,re,shutil,subprocess,threading,time
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-009')
out=run/'clock-probe-001';assert not out.exists();out.mkdir();(out/'tmp').mkdir()
text=(run/'logs/workspace-all-features.log').read_text()
binary=pathlib.Path(re.search(r'Running unittests src/lib.rs \(([^)]+/drive_runtime-[^/()]+)\)',text)[1])
assert binary.is_relative_to(run/'target');copy=out/binary.name;shutil.copy2(binary,copy)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
env=os.environ.copy()
for key in list(env):
    if key.startswith('DRIVE_NATIVE_') or key in {'DBUS_SESSION_BUS_ADDRESS','DISPLAY','WAYLAND_DISPLAY'}:env.pop(key,None)
env.update(TMPDIR=str(out/'tmp'),RUST_BACKTRACE='1')
rows=[];samples=[];stop=threading.Event()
def sample():
    while not stop.is_set():
        # time is between two adjacent monotonic reads; retain the bracket.
        mono_before=time.monotonic_ns();wall=time.time_ns();mono_after=time.monotonic_ns()
        samples.append({'monotonic_before_ns':mono_before,'realtime_ns':wall,'monotonic_after_ns':mono_after})
        stop.wait(.01)
thread=threading.Thread(target=sample);thread.start()
cases=['bridge::download_scheduler_tests::concurrent_foreground_reads_wait_for_shared_retry_without_reopen',
       'bridge::download_scheduler_tests::concurrent_foreground_reads_wait_for_transport_backoff_after_reconnect']
try:
    for index,case in enumerate(cases):
        log=out/f'case-{index+1}.log';argv=[str(copy),'--exact',case,'--nocapture','--test-threads=1']
        start={'realtime_ns':time.time_ns(),'monotonic_ns':time.monotonic_ns()}
        with log.open('w') as stream:
            process=subprocess.run(argv,env=env,cwd=run/'source',stdout=stream,stderr=subprocess.STDOUT,timeout=30)
        rows.append({'name':case,'command':argv,'exit_code':process.returncode,'log':log.name,'log_sha256':sha(log),
          'start':start,'end':{'realtime_ns':time.time_ns(),'monotonic_ns':time.monotonic_ns()}})
    stop.wait(3)
finally:
    stop.set();thread.join()
    (out/'clock-samples.json').write_text(json.dumps(samples,indent=2)+'\n')
    differences=[]
    for left,right in zip(samples,samples[1:]):
        mono=(right['monotonic_before_ns']+right['monotonic_after_ns']-left['monotonic_before_ns']-left['monotonic_after_ns'])/2
        wall=right['realtime_ns']-left['realtime_ns']
        differences.append({'wall_delta_ns':wall,'monotonic_delta_ns':mono,'delta_difference_ns':wall-mono})
    result={'candidate_commit':'4e6aaf7166fd370076adf43a0dad3e6ecdca36d6','copied_elf_sha256':sha(copy),
      'source_elf':str(binary),'copied_elf':str(copy),'cases':rows,'samples':len(samples),
      'clock_read_order':'monotonic_before -> time.time_ns -> monotonic_after; 10ms requested interval',
      'largest_abs_interval_differences':sorted(differences,key=lambda r:abs(r['delta_difference_ns']),reverse=True)[:10],
      'max_monotonic_bracket_ns':max((r['monotonic_after_ns']-r['monotonic_before_ns'] for r in samples),default=0),
      'context':'unchanged exact test ELF; main G4 default/unrelated debug commands may be running; no timing threshold modified; not a benchmark'}
    (out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
