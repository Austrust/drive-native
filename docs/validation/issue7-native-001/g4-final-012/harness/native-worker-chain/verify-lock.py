"""Verify offline metadata uses only registry identities in the candidate lock."""
import argparse
import hashlib
import json
from pathlib import Path
import tomllib

p = argparse.ArgumentParser()
p.add_argument('--crate', type=Path, required=True)
p.add_argument('--source', type=Path, required=True)
p.add_argument('--metadata', type=Path, required=True)
a = p.parse_args()
crate, source = a.crate.resolve(), a.source.resolve()
def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()
provenance = json.loads((crate / 'provenance.json').read_text())
assert sha(source / 'Cargo.lock') == provenance['source_cargo_lock_sha256'], 'candidate lock changed'
identity = lambda row: (row['name'], row['version'], row.get('source'), row.get('checksum'))
frozen = tomllib.loads((source / 'Cargo.lock').read_text())['package']
resolved = tomllib.loads((crate / 'Cargo.lock').read_text())['package']
old_registry = {identity(row) for row in frozen if row.get('source')}
registry = [row for row in resolved if row.get('source')]
unexpected = [identity(row) for row in registry if identity(row) not in old_registry]
metadata = json.loads(a.metadata.read_text())
assert metadata.get('resolve') and metadata['resolve'].get('nodes'), 'full metadata required'
metadata_registry = {(r['name'], r['version'], r.get('source')) for r in metadata['packages'] if r.get('source')}
lock_registry = {(r['name'], r['version'], r.get('source')) for r in registry}
unlocked = sorted(metadata_registry - lock_registry)
allowed = {crate / 'Cargo.toml', source / 'vendor/fuser/Cargo.toml'}
allowed.update(source / 'crates' / name / 'Cargo.toml' for name in ('drive-core', 'drive-fs', 'drive-runtime', 'sproto'))
unexpected_paths = [r['manifest_path'] for r in metadata['packages']
                    if not r.get('source') and Path(r['manifest_path']).resolve() not in allowed]
source_changes = [name for name, digest in provenance['files'].items()
                  if name != 'Cargo.lock' and sha(crate / name) != digest]
report = {'registry_dependencies_checked': len(registry),
          'metadata_registry_dependencies_checked': len(metadata_registry),
          'new_or_changed_registry_dependencies': unexpected,
          'metadata_registry_dependencies_absent_from_lock': unlocked,
          'unexpected_path_dependencies': unexpected_paths,
          'changed_prepared_files': source_changes,
          'all_registry_dependencies_match_frozen_candidate': not unexpected and not unlocked and not unexpected_paths and not source_changes,
          'source_cargo_lock_sha256': sha(source / 'Cargo.lock'),
          'harness_cargo_lock_sha256': sha(crate / 'Cargo.lock'),
          'cargo_metadata_sha256': sha(a.metadata)}
destination = crate / 'lock-verification.json'
assert not destination.exists(), 'preserve earlier verification'
destination.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
raise SystemExit(0 if report['all_registry_dependencies_match_frozen_candidate'] else 1)
