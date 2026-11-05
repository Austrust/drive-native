"""Prepare an external crate without compiling/running or changing frozen source."""
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
source, output = a.source.resolve(), a.output.resolve()
assert a.source.is_absolute() and a.output.is_absolute()
assert len(a.candidate) == 40 and all(c in '0123456789abcdef' for c in a.candidate)
assert not output.is_relative_to(source), 'external harness must not change frozen source'
assert (source / 'Cargo.lock').is_file()
assert (source / 'crates/sproto/Cargo.toml').is_file()
here = Path(__file__).resolve().parent
output.mkdir(parents=True, exist_ok=False)
(output / 'src').mkdir()
shutil.copy2(here / 'main.rs', output / 'src/main.rs')
shutil.copy2(source / 'Cargo.lock', output / 'Cargo.lock')
manifest = (here / 'Cargo.toml.in').read_text().replace(
    '@SPROTO_PATH@', json.dumps(str(source / 'crates/sproto')))
(output / 'Cargo.toml').write_text(manifest)
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
data = {
    'candidate_commit': a.candidate,
    'frozen_source': str(source),
    'scope': 'synthetic 127.0.0.1 pstream server and public sproto Client APIs; no NAS, real credentials, worker ELF or live ACL verification',
    'prepared_only': True,
    'source_cargo_lock_sha256': sha(source / 'Cargo.lock'),
    'initial_harness_cargo_lock_sha256': sha(output / 'Cargo.lock'),
    'harness_source_sha256': sha(output / 'src/main.rs'),
    'harness_manifest_sha256': sha(output / 'Cargo.toml'),
}
(output / 'provenance.json').write_text(json.dumps(data, indent=2) + '\n')
print(json.dumps(data))
