"""Reconcile unittest verbose lines when D-Bus diagnostics interrupt one line."""
import json,pathlib,re
p=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private')
result=json.loads((p/'result.json').read_text());text=(p/'provider.log').read_text()
tests=[]
for match in re.finditer(r'^(test_\S+) \(([^)]+)\) \.\.\. (.*?)(?=^test_|^-{5,}|\Z)',text,re.M|re.S):
    body=match.group(3).strip()
    assert re.search(r'^ok$',body,re.M),body
    tests.append([match.group(1),match.group(2),'ok'])
assert len(tests)==10 and 'Ran 10 tests' in text and re.search(r'^OK$',text,re.M)
result.update(tests=tests,passed=10,failed=0,ignored=0,complete=result['exit_code']==0,
    parsing_note='A D-Bus activation message split the first verbose test line; this corrected projection retains all ten names. Original log and initial parsing result preserved.')
(p/'result-corrected.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'passed':10,'failed':0,'ignored':0,'complete':result['complete']}))
