"""Remove only the never-used partial G4-010 cache after its failed copy."""
from pathlib import Path
import json
import shutil

base = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
run = base / 'g4-010'
target = run / 'target'
expected = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-010/target')
assert target.resolve(strict=True) == expected and not target.is_symlink()
assert json.loads((run / 'source.json').read_text())['candidate_commit'] == '41365a650eb7d7df696046e21b7671697231f851'
assert not (run / 'summary.json').exists(), 'Do not remove a completed or tested target'
for line in Path('/proc/self/mountinfo').read_text().splitlines():
    mount = Path(line.split()[4])
    assert not mount.is_relative_to(target), 'Mounted cache must not be deleted'
record = Path(__file__).resolve().parent / 'partial-target-cleanup.json'
data = {'target': str(target), 'candidate': '41365a650eb7d7df696046e21b7671697231f851',
        'reason': 'Never-used partial cache; copy failed with I/O and read-only filesystem before clean/build/test.',
        'preserved': ['source', 'source.json', 'logs', 'all prior run targets and evidence'], 'removed': False}
record.write_text(json.dumps(data, indent=2) + '\n')
shutil.rmtree(target)
data['removed'] = True
record.write_text(json.dumps(data, indent=2) + '\n')
print(json.dumps(data))
