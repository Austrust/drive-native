import collections,json,pathlib
root=pathlib.Path.home()/'.local/state/drive-native-p0/review-001'
rows=[json.loads(x) for x in (root/'observations.jsonl').read_text().splitlines()]
result={}
for task in sorted({r['task'] for r in rows if 'task' in r}):
    samples=sorted([r for r in rows if r.get('task')==task],key=lambda r:r['iteration'])
    spans=[];current=None
    for row in samples:
        workers=tuple(p['pid'] for p in row.get('processes',[])[1:])
        if current is None or current['pids']!=workers:
            if current is not None:current['next_observation']=row['time'];spans.append(current)
            current={'pids':workers,'first':row['time'],'last':row['time'],'first_iteration':row['iteration'],'last_iteration':row['iteration']}
        else:current.update(last=row['time'],last_iteration=row['iteration'])
    if current is not None:spans.append(current)
    bounded=[]
    for i,s in enumerate(spans):
        if i==0 or i==len(spans)-1 or len(s['pids'])!=1:continue
        previous=spans[i-1]
        bounded.append({'pid':s['pids'][0],'first_iteration':s['first_iteration'],'last_iteration':s['last_iteration'],
          'observed_lifetime_lower_s':s['last']-s['first'],
          'sample_bounded_lifetime_upper_s':s['next_observation']-previous['last']})
    result[task]={'fully_bounded_segments':bounded,'first_and_last_segments_excluded':True,'sampling_seconds':10,'root_cause_confirmed':False}
(root/'worker-lifetimes.json').write_text(json.dumps(result,indent=2))
print(json.dumps({k:[(round(s['observed_lifetime_lower_s']),round(s['sample_bounded_lifetime_upper_s'])) for s in v['fully_bounded_segments']] for k,v in result.items()}))
