"""Freeze and audit observation evidence; never declare product stability."""
import collections,hashlib,json,math,pathlib,subprocess,time
root=pathlib.Path.home()/'.local/state/drive-native-p0'
out=root/'review-001';out.mkdir(exist_ok=False)
raw=(root/'observations.jsonl').read_bytes();(out/'observations.jsonl').write_bytes(raw)
manifest=json.loads((root/'manifest.json').read_text())
tasks=[t['instance'] for t in json.loads((root.parent/'drive-native-soak/catalog.json').read_text())]
status=json.loads((root/'status.json').read_text())
load=(root/'planned-loads.jsonl').read_bytes();(out/'planned-loads.jsonl').write_bytes(load)
rows=[];parse_errors=[]
for index,line in enumerate(raw.splitlines()):
    try:rows.append(json.loads(line))
    except ValueError:parse_errors.append(index+1)
def dist(values):
    v=sorted(values)
    if not v:return {'n':0}
    return {'n':len(v),**{k:v[max(0,math.ceil(q*len(v))-1)] for k,q in [('p50',.5),('p95',.95),('p99',.99),('max',1)]}}
result={'scope':'partial .6 observational baseline; GetOverview and process statistics, not FUSE correctness or performance acceptance',
 'captured_at':time.time(),'observations_sha256':hashlib.sha256(raw).hexdigest(),'observations_bytes':len(raw),'planned_loads_sha256':hashlib.sha256(load).hexdigest(),
 'parse_error_lines':parse_errors,'status_snapshot':status,'tasks':{},'new_product_code':False,'runtime_touched':False}
starts=[r for r in rows if r.get('kind')=='start']
ends=[r for r in rows if r.get('state')=='finished_requires_review']
result['start_records']=len(starts);result['finished_records']=len(ends)
result['unknown_task_ids']=sorted({r['task'] for r in rows if 'task' in r}-set(tasks))
all_failure=[]
for task in tasks:
    samples=sorted([r for r in rows if r.get('task')==task],key=lambda r:r['iteration'])
    assert samples
    iterations=[r['iteration'] for r in samples]
    count=collections.Counter(iterations)
    gaps=[{'from':a['iteration'],'to':b['iteration'],'seconds':b['time']-a['time']} for a,b in zip(samples,samples[1:]) if b['time']-a['time']>15 or b['time']<=a['time']]
    failures=[r for r in samples if r.get('ok') is not True]
    all_failure.extend(failures)
    fsd_pids=sorted({r['processes'][0]['pid'] for r in samples if r.get('processes')})
    worker_pids=sorted({p['pid'] for r in samples for p in r.get('processes',[])[1:]})
    binary_checks=[p for r in samples for p in r.get('processes',[]) if 'binary_sha256' in p]
    binary_mismatches=[p for p in binary_checks if p['binary_sha256'] not in manifest['allowed_binary_sha256'] or not p.get('expected_binary')]
    resource=[r for r in samples if r.get('processes')]
    cpu_pairs=[]
    for a,b in zip(samples,samples[1:]):
        dt=b['time']-a['time']
        if not 0<dt<=15:continue
        aa={p['pid']:p for p in a.get('processes',[])};bb={p['pid']:p for p in b.get('processes',[])}
        # Do not combine across worker replacements or incomplete process snapshots.
        if not aa or aa.keys()!=bb.keys():continue
        delta=sum(bb[p]['cpu_seconds']-aa[p]['cpu_seconds'] for p in aa)
        if delta>=0:cpu_pairs.append({'cpu_s':delta,'wall_s':dt})
    result['tasks'][task]={'sample_count':len(samples),'first_iteration':min(iterations),'last_iteration':max(iterations),
      'missing_iterations':sorted(set(range(min(iterations),max(iterations)+1))-set(iterations)),
      'duplicate_iterations':sorted(i for i,n in count.items() if n>1),'failed_samples':len(failures),
      'connection_counts':dict(collections.Counter(r.get('connection','unknown') for r in samples)),
      'errors':dict(collections.Counter(r.get('error_type') or r.get('error') or 'unclassified' for r in failures)),
      'mounted_false_or_unknown':sum(r.get('mounted') is not True for r in samples),'intervals_over_15s_or_nonpositive':gaps,
      'fsd_pids':fsd_pids,'worker_pid_count':len(worker_pids),'binary_check_count':len(binary_checks),'binary_mismatches':binary_mismatches,
      'overview_ms_all_observed':dist([r['overview_ms'] for r in samples if 'overview_ms' in r]),'overview_missing':sum('overview_ms' not in r for r in samples),
      'rss_sum_kib':dist([sum(p['rss_kib'] for p in r['processes']) for r in resource]),
      'pss_sum_kib':dist([sum(p['pss_kib'] for p in r['processes']) for r in resource]),'resource_missing':len(samples)-len(resource),
      'cpu_same_pid_segments':{'n':len(cpu_pairs),'covered_s':sum(p['wall_s'] for p in cpu_pairs),'mean_percent_one_core':100*sum(p['cpu_s'] for p in cpu_pairs)/sum(p['wall_s'] for p in cpu_pairs) if cpu_pairs else None}}
result['failures']=all_failure
result['complete_24h_record_present']=bool(len(starts)==1 and len(ends)==1 and ends[0]['elapsed_s']>=manifest['duration_s'])
result['product_stability_accepted']=False
result['limitations']=['Active observer may append after the frozen snapshot; status and snapshot end are not an atomic pair.',
 '10-second sampling cannot establish exact disconnection or recovery duration.',
 'Resource distributions include recorded development/network loads, not idle-window acceptance.',
 'CPU pairs exclude PID changes and gaps; omitted intervals are not counted as zero usage.',
 'No FUSE access, content consistency or server convergence assertion follows from GetOverview.']
(out/'summary.json').write_text(json.dumps(result,indent=2))
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
(out/'review-script.py').write_bytes(pathlib.Path(__file__).read_bytes())
print(json.dumps({'snapshot':str(out),'rows':len(rows),'parse_errors':parse_errors,'failed_samples':len(all_failure),'complete_24h':result['complete_24h_record_present'],
 'tasks':{k:{n:v[n] for n in ['sample_count','failed_samples','missing_iterations','duplicate_iterations','fsd_pids','worker_pid_count','binary_mismatches']} for k,v in result['tasks'].items()}}))
