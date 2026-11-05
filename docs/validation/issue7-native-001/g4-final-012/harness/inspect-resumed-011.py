"""Read existing isolated results and installed build-tool metadata only."""
import hashlib
import json
from pathlib import Path
import subprocess

base = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-011')
for name in ('native-worker-chain-denials-001', 'native-worker-chain-other-001'):
    path = base / name / 'results.json'
    data = json.loads(path.read_text())
    print(json.dumps({'path':str(path), 'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
        'candidate':data['candidate_commit'], 'complete':data['complete'], 'counts':data['counts'],
        'cases':[{'name':c['name'], 'passed':c['passed'], 'error':c.get('observed',{}).get('error'),
                  'cleanup':c.get('observed',{}).get('cleanup'),
                  'process_group_cleanup':c.get('process_group_cleanup')} for c in data['cases']]}))
for argv in ([ 'dpkg-query', '-W', '-f=${Package} ${Version}\n', 'debhelper', 'gcc', 'cmake', 'pkg-config', 'libglib2.0-dev', 'python3', 'patchelf', 'fakeroot', 'dpkg-dev' ],
             ['/opt/drive-native-build/gui-venv/bin/python', '-c', 'import importlib.metadata,json; print(json.dumps({n:importlib.metadata.version(n) for n in ("PySide6_Essentials","pyinstaller","shiboken6")}))'],
             ['df','-B1',str(base)]):
    p=subprocess.run(argv,capture_output=True,text=True)
    print(json.dumps({'command':argv,'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr}))
