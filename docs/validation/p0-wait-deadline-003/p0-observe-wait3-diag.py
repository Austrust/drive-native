import asyncio,json,pathlib,re,subprocess,time
from gi.repository import Gio,GLib
root=pathlib.Path.home()/'.local/state/drive-native-p0/wait-diagnostic-003'
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
def overview(cfg):
    reply=bus.call_sync(cfg['bus'],'/org/drivenative/Client1','org.drivenative.Client1','GetOverview',None,GLib.VariantType.new('(s)'),Gio.DBusCallFlags.NONE,3000,None)
    d=json.loads(reply.unpack()[0]);return {k:d.get(k) for k in ['connection','mounted','error','pending_operations']}
def properties(unit):
    raw=subprocess.check_output(['systemctl','--user','show',unit,'-p','MainPID','-p','ActiveState','-p','Result','-p','ExecMainStatus'],text=True)
    return dict(line.split('=',1) for line in raw.splitlines())
def trace(entry):
    message=entry.get('MESSAGE','')
    if not message.startswith('P0_WAIT '):return None
    assert re.fullmatch(r'P0_WAIT(?: [a-z_]+=[A-Za-z0-9_]+)+',message),'unexpected trace format'
    parsed=dict(part.split('=',1) for part in message.split()[1:])
    assert set(parsed)<=set('layer phase pid outer_s inner_s elapsed_us outcome ok kind changes profile_changed count'.split())
    parsed.update(monotonic_us=int(entry['__MONOTONIC_TIMESTAMP']),realtime_us=int(entry['__REALTIME_TIMESTAMP']),journal_pid=int(entry['_PID']))
    return parsed

def thread_waits(unit):
    props=properties(unit);pid=int(props['MainPID']);pids=[pid] if pid else []
    if pid:
        try:pids += [int(x) for x in pathlib.Path('/proc',str(pid),'task',str(pid),'children').read_text().split()]
        except (FileNotFoundError,PermissionError):pass
    rows=[]
    for current in pids:
        counts={};states={};base=pathlib.Path('/proc',str(current))
        try:
            for t in (base/'task').iterdir():
                try:
                    wait=(t/'wchan').read_text().strip();counts[wait]=counts.get(wait,0)+1
                    stat=(t/'stat').read_text().rsplit(')',1)[1].split()[0];states[stat]=states.get(stat,0)+1
                except (FileNotFoundError,PermissionError):pass
            rows.append({'pid':current,'wait_counts':counts,'state_counts':states,'fd_count':len(list((base/'fd').iterdir()))})
        except (FileNotFoundError,PermissionError):pass
    return {'properties':props,'processes':rows}

async def observe(seconds):
    run=root/str(seconds);cfg=json.loads((run/'config.json').read_text());result={'outer_seconds':seconds,'inner_seconds':450}
    follower=None;records=[];start=time.monotonic()
    try:
        result['initial_samples']=[]
        while True:
            try:
                state=overview(cfg)
                sample={'elapsed_s':time.monotonic()-start,**state}
            except Exception as e:
                state={};sample={'elapsed_s':time.monotonic()-start,'error_type':type(e).__name__}
            result['initial_samples'].append(sample)
            with (run/'initial.jsonl').open('a') as f:f.write(json.dumps(sample)+'\n')
            if state.get('connection')=='online' and state.get('mounted'):break
            if state.get('connection')=='needs_attention':raise RuntimeError('initial_needs_attention')
            if time.monotonic()-start>300:raise RuntimeError('initial_online_deadline')
            await asyncio.sleep(1)
        result['initial_elapsed_s']=time.monotonic()-start
        result['initial_overview']=state;result['observed_from']=time.time()
        print(json.dumps({'outer_seconds':seconds,'online':True,'mounted':state['mounted']}),flush=True)
        follower=await asyncio.create_subprocess_exec('journalctl','--user','-u',cfg['unit'],'--follow','--no-tail','--no-pager','--output=json',stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.DEVNULL)
        deadline=time.monotonic()+900
        while True:
            try:line=await asyncio.wait_for(follower.stdout.readline(),max(.01,deadline-time.monotonic()))
            except asyncio.TimeoutError:result['end_reason']='900s_observation_deadline';break
            if not line:raise RuntimeError('journal_follow_ended')
            row=trace(json.loads(line))
            if row is None:continue
            records.append(row)
            with (run/'trace.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
            if row['layer']=='bridge' and (row['phase']=='remote_failure' or (row['phase']=='reply' and row.get('outcome','').startswith('io_'))):
                result['end_reason']='bridge_terminal_error';result['first_terminal']=row;break
        result['before_stop_overview']=overview(cfg)
    except Exception as e:
        reason=str(e) if isinstance(e,RuntimeError) and str(e) in ['initial_needs_attention','initial_online_deadline','journal_follow_ended'] else 'unclassified_observer_exception'
        result.update(end_reason='observer_error',error_type=type(e).__name__,error_reason=reason)
    finally:
        if follower:
            follower.terminate();await follower.wait()
        result['before_stop_threads']=thread_waits(cfg['unit'])
        stop_start=time.monotonic();result['stop_requested_at']=time.time();result['stop_thread_samples']=[];next_sample=5
        subprocess.run(['systemctl','--user','stop','--no-block',cfg['unit']],check=True)
        while True:
            props=properties(cfg['unit'])
            if props['MainPID']=='0':break
            if time.monotonic()-stop_start>=next_sample:
                result['stop_thread_samples'].append({'elapsed_s':time.monotonic()-stop_start,**thread_waits(cfg['unit'])});next_sample+=5
            if time.monotonic()-stop_start>30:raise RuntimeError('diagnostic_stop_not_confirmed')
            await asyncio.sleep(.2)
        result['stop']={'elapsed_s':time.monotonic()-stop_start,'properties':props,'mount_remaining':subprocess.run(['findmnt','-M',cfg['mount']],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode==0}
        result['observed_rows']=len(records)
        # Final trace is filtered, including any events caused by explicit stop.
        raw=subprocess.check_output(['journalctl','--user','-u',cfg['unit'],'--no-pager','--output=json'],text=True)
        final=[v for line in raw.splitlines() if (v:=trace(json.loads(line))) is not None]
        (run/'final-trace.jsonl').write_text(''.join(json.dumps(v)+'\n' for v in final))
        (run/'result.json').write_text(json.dumps(result,indent=2))
        print(json.dumps({'outer_seconds':seconds,'end_reason':result.get('end_reason'),'terminal':result.get('first_terminal'),'stop':result['stop']}),flush=True)
    return result
async def main():
    results=await asyncio.gather(observe(470))
    (root/'result.json').write_text(json.dumps({'finished_at':time.time(),'results':results,'runtime_product_changed':False},indent=2))
    with (root.parent/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'end':time.time(),'run':root.name})+'\n')
asyncio.run(main())
