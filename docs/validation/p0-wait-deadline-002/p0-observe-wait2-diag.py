import asyncio,json,pathlib,re,subprocess,time
from gi.repository import Gio,GLib
root=pathlib.Path.home()/'.local/state/drive-native-p0/wait-diagnostic-002'
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
    assert set(parsed)<=set('layer phase pid outer_s inner_s elapsed_us outcome ok kind changes profile_changed'.split())
    parsed.update(monotonic_us=int(entry['__MONOTONIC_TIMESTAMP']),realtime_us=int(entry['__REALTIME_TIMESTAMP']),journal_pid=int(entry['_PID']))
    return parsed
async def observe(seconds):
    run=root/str(seconds);cfg=json.loads((run/'config.json').read_text());result={'outer_seconds':seconds,'inner_seconds':450}
    follower=None;records=[];start=time.monotonic()
    try:
        while True:
            state=overview(cfg)
            if state['connection']=='online' and state['mounted']:break
            if state['connection']=='needs_attention' or time.monotonic()-start>45:raise RuntimeError('initial_online_not_verified')
            await asyncio.sleep(.2)
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
    except Exception as e:result.update(end_reason='observer_error',error_type=type(e).__name__)
    finally:
        if follower:
            follower.terminate();await follower.wait()
        stop_start=time.monotonic();result['stop_requested_at']=time.time()
        subprocess.run(['systemctl','--user','stop','--no-block',cfg['unit']],check=True)
        while True:
            props=properties(cfg['unit'])
            if props['MainPID']=='0':break
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
