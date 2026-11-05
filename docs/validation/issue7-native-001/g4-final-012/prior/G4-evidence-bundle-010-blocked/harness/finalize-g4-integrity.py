"""Verify frozen source and run-owned mounts after every G4 supplemental command."""
import hashlib
import json
from pathlib import Path
import sys

run = Path(sys.argv[1]).resolve()
base = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
assert run.parent == base and run.name.startswith('g4-')
filename = sys.argv[2] if len(sys.argv)>2 else 'final-integrity.json'
assert filename in ('final-integrity.json','final-integrity-post-history.json')
output = run / filename
assert not output.exists()
source = run / 'source'
before = json.loads((run / 'source-hashes-before.json').read_text())
def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()
changed = [name for name, expected in before.items() if not (source / name).is_file() or sha(source / name) != expected]
new = [str(path.relative_to(source)) for path in source.rglob('*') if path.is_file() and str(path.relative_to(source)) not in before]
owned = [run, base / ('functional-' + run.name)]
mounts = []
for line in Path('/proc/self/mountinfo').read_text().splitlines():
    raw = line.split()[4]
    for escaped, plain in [('\\040', ' '), ('\\011', '\t'), ('\\012', '\n'), ('\\134', '\\')]:
        raw = raw.replace(escaped, plain)
    path = Path(raw)
    if any(path.is_relative_to(root) for root in owned):
        mounts.append(str(path))
result = {'candidate_commit': json.loads((run / 'source.json').read_text())['candidate_commit'],
          'changed_source_files': changed, 'new_source_files': new,
          'allowed_new_file': 'vendor/fuser/Cargo.lock (excluded vendored crate offline test lock)',
          'residual_owned_mounts': mounts,
          'passed': not changed and not mounts and set(new) <= {'vendor/fuser/Cargo.lock'}}
output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
assert result['passed']
