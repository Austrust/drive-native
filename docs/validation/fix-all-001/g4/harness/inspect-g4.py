"""Compact read-only status of one private validation run."""
import json,pathlib,sys
p=pathlib.Path(sys.argv[1]);summary=json.loads((p/'summary.json').read_text())
for row in summary['commands']:
    if 'exit_code' in row:print(json.dumps({k:row.get(k) for k in ('name','exit_code','passed','counts')}))
    else:
        print('RUNNING '+row['name'])
        log=p/row['log']
        if log.exists():
            lines=log.read_text(errors='replace').splitlines()
            print('\n'.join(lines[-8:]))
print('complete='+str(summary['complete']))
