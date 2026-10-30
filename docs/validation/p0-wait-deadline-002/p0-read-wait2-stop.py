import json,pathlib,sqlite3,subprocess
root=pathlib.Path.home()/'.local/state/drive-native-p0/wait-diagnostic-002'
cfg=json.loads((root/'470/config.json').read_text())
db=sqlite3.connect((pathlib.Path(cfg['state'])/'state.sqlite').as_uri()+'?mode=ro',uri=True)
schema=[x[0] for x in db.execute("SELECT name FROM sqlite_master WHERE type='table'")]
state={'nodes':db.execute('SELECT count(*) FROM nodes WHERE deleted=0').fetchone()[0],
 'access_denied':db.execute('SELECT count(*) FROM access_denied').fetchone()[0],
 'committed_cursor_present':bool(db.execute("SELECT length(value)>0 FROM metadata WHERE key='committed_cursor'").fetchone()[0]),
 'pending_operations':db.execute("SELECT count(*) FROM operations WHERE state NOT IN ('committed','superseded')").fetchone()[0],
 'dirty_drafts':db.execute('SELECT count(*) FROM drafts WHERE dirty!=0').fetchone()[0]}
(root/'local-state-counts.json').write_text(json.dumps(state,indent=2))
print(json.dumps(state))
raw=subprocess.check_output(['journalctl','--user','-u',cfg['unit'],'--no-pager','--output=json'],text=True)
counts={};safe=[]
for line in raw.splitlines():
    r=json.loads(line);m=r.get('MESSAGE','')
    # Export only classifications and timestamps. Never include unfiltered logs.
    category=next((kind for needle,kind in [('start operation timed out','start_timeout'),('stop-sigterm','stop_sigterm'),('Failed with result','unit_failed'),('code=killed','process_killed'),('Started ','unit_started'),('Stopping ','unit_stopping'),('status=9','status9')] if needle in m),'other')
    counts[category]=counts.get(category,0)+1
    safe.append({'category':category,'monotonic_us':r.get('__MONOTONIC_TIMESTAMP'),'realtime_us':r.get('__REALTIME_TIMESTAMP'),'pid':r.get('_PID')})
(root/'unit-journal-categories.json').write_text(json.dumps({'counts':counts,'entries':safe},indent=2))
print(json.dumps({'journal_classifications':counts}))
