"""Create an external crate from frozen source; this script never invokes Cargo."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

p = argparse.ArgumentParser()
p.add_argument('--source', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--candidate', required=True)
a = p.parse_args()
assert a.source.is_absolute() and a.output.is_absolute()
source, output = a.source.resolve(), a.output.resolve()
assert len(a.candidate) == 40 and all(c in '0123456789abcdef' for c in a.candidate)
assert not output.is_relative_to(source), 'never add a crate to frozen product source'
assert not output.exists(), 'new isolated output required'
here = Path(__file__).resolve().parent
paths = {'@CORE_PATH@': source / 'crates/drive-core',
         '@RUNTIME_PATH@': source / 'crates/drive-runtime',
         '@FUSER_PATH@': source / 'vendor/fuser'}
assert all((path / 'Cargo.toml').is_file() for path in paths.values())
output.mkdir(parents=True)
(output / 'src').mkdir()
(output / 'fixtures').mkdir()
for name in ('main.rs', 'codec.rs'):
    shutil.copy2(here / name, output / 'src' / name)
shutil.copy2(source / 'Cargo.lock', output / 'Cargo.lock')
manifest = (here / 'Cargo.toml.in').read_text()
for key, path in paths.items():
    manifest = manifest.replace(key, json.dumps(str(path)))
(output / 'Cargo.toml').write_text(manifest)
for name in ('ca.pem', 'server.pem', 'server-key.pem'):
    shutil.copy2(source / 'crates/sproto/tests/fixtures/trust' / name, output / 'fixtures' / name)
def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()
data = {'candidate_commit': a.candidate, 'frozen_source': str(source),
        'prepared_only': True,
        'scope': 'synthetic localhost native TLS service; actual worker ELF and public Bridge; no relay or upstream',
        'source_cargo_lock_sha256': sha(source / 'Cargo.lock'),
        'initial_harness_cargo_lock_sha256': sha(output / 'Cargo.lock'),
        'files': {str(path.relative_to(output)): sha(path) for path in output.rglob('*') if path.is_file()}}
(output / 'provenance.json').write_text(json.dumps(data, indent=2) + '\n')
print(json.dumps(data))
