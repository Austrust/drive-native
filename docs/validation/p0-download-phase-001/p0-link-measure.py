import json,math,os,pathlib,subprocess,time
base=pathlib.Path.home()/'.local/state/drive-native-p0';root=base/'link-001';root.mkdir(mode=0o700,exist_ok=False)
tool=base/'link-tool-001/unpacked';env=os.environ.copy();env['LD_LIBRARY_PATH']=str(tool/'usr/lib/x86_64-linux-gnu')
manifest={'registered_at':time.time(),'server':'10.114.114.3','port':5201,'direction':'reverse NAS to WSL','parallel_streams':1,'warmup_runs':1,'warmup_seconds':2,'planned_runs':3,'seconds_per_run':5,'process_timeout_s':12,'not_drive_protocol':True,'interference':'six native tasks, probes, idle phase clone; no controlled NAS or LAN background load','stop_rule':'first tool/protocol failure ends experiment; retain failures'}
(root/'manifest.json').write_text(json.dumps(manifest,indent=2))
with (base/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'start':time.time(),'run':'link-001','kind':'preregistered single-stream reverse iperf3 baseline'})+'\n')
rows=[]
for index in range(-1,3):
    command=[str(tool/'usr/bin/iperf3'),'-c','10.114.114.3','-p','5201','-R','-P','1','-t',str(2 if index<0 else 5),'--connect-timeout','2000','-J']
    started=time.monotonic();row={'index':index,'time':time.time()}
    try:
        p=subprocess.run(command,capture_output=True,text=True,env=env,timeout=12)
        result=json.loads(p.stdout);(root/f'raw-{index}.json').write_text(json.dumps(result,indent=2))
        row.update(ok=p.returncode==0 and 'error' not in result,returncode=p.returncode)
        if row['ok']:row.update({k:result['end']['sum_received'][k] for k in ['seconds','bytes','bits_per_second']})
        else:row['error']=result.get('error','tool_failure')
    except Exception as error:row.update(ok=False,error_type=type(error).__name__)
    row['wall_s']=time.monotonic()-started;rows.append(row)
    with (root/'samples.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
    print(json.dumps(row),flush=True)
    if not row['ok']:break
    time.sleep(.5)
summary={'planned':3,'executed':sum(r['index']>=0 for r in rows),'errors':sum(not r['ok'] for r in rows)}
values=sorted(r['bits_per_second'] for r in rows if r['index']>=0 and r['ok'])
if values:summary['received_bps']={**{f'p{p}':values[math.ceil(len(values)*p/100)-1] for p in [50,95,99]},'max':max(values),'min':min(values)}
(root/'summary.json').write_text(json.dumps(summary,indent=2))
with (base/'planned-loads.jsonl').open('a') as f:f.write(json.dumps({'end':time.time(),'run':'link-001'})+'\n')
