import collections,json,pathlib
root=pathlib.Path.home()/'.local/state/drive-native-p0/relay-phase-001/phases-001'
summary=[]
for line in (root/'samples.jsonl').read_text().splitlines():
    row=json.loads(line);transitions=[];totals=collections.defaultdict(float);last_time=0;last_phase='no_rpc';max_query=0
    for sample in row['phase_samples']:
        entries=sample.get('entries',[])
        phase=entries[0].get('progress',{}).get('phase','no_progress') if entries and entries[0].get('progress') else ('no_progress' if entries else 'no_rpc')
        totals[last_phase]+=sample['elapsed_s']-last_time;last_time=sample['elapsed_s']
        if phase!=last_phase:transitions.append({'at':last_time,'phase':phase,'progress':entries[0].get('progress') if entries else None})
        last_phase=phase;max_query=max(max_query,sample.get('query_s',0))
    summary.append({'index':row['index'],'complete_s':row.get('complete_s'),'phase_interval_estimates_s':dict(totals),'transitions':transitions,'max_query_s':max_query})
(root/'phase-summary.json').write_text(json.dumps(summary,indent=2))
for row in summary:print(json.dumps(row))
