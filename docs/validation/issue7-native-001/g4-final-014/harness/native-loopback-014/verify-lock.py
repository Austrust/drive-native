"""After offline cargo metadata, require all resolved registry identities to be frozen."""
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
identity = lambda row: (row['name'], row['version'], row.get('source'), row.get('checksum'))
frozen = tomllib.loads((source / 'Cargo.lock').read_text())['package']
resolved = tomllib.loads((crate / 'Cargo.lock').read_text())['package']
old_registry = {identity(row) for row in frozen if row.get('source')}
registry = [row for row in resolved if row.get('source')]
unexpected = [identity(row) for row in registry if identity(row) not in old_registry]
metadata = json.loads(a.metadata.read_text())
assert metadata.get('resolve') and metadata['resolve'].get('nodes'), 'full cargo metadata required; do not use --no-deps'
metadata_registry = {(row['name'], row['version'], row.get('source')) for row in metadata['packages'] if row.get('source')}
lock_registry = {(row['name'], row['version'], row.get('source')) for row in registry}
unlocked_metadata = sorted(metadata_registry - lock_registry)
unexpected_paths = []
for row in metadata['packages']:
    if not row.get('source'):
        manifest = Path(row['manifest_path']).resolve()
        if manifest not in (crate / 'Cargo.toml', source / 'crates/sproto/Cargo.toml'):
            unexpected_paths.append(str(manifest))
report = {
    'registry_dependencies_checked': len(registry),
    'metadata_registry_dependencies_checked': len(metadata_registry),
    'new_or_changed_registry_dependencies': unexpected,
    'metadata_registry_dependencies_absent_from_lock': unlocked_metadata,
    'unexpected_path_dependencies': unexpected_paths,
    'all_registry_dependencies_match_frozen_candidate': not unexpected and not unlocked_metadata and not unexpected_paths,
}
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
report.update(source_cargo_lock_sha256=sha(source / 'Cargo.lock'),
              harness_cargo_lock_sha256=sha(crate / 'Cargo.lock'),
              cargo_metadata_sha256=sha(a.metadata))
destination = crate / 'lock-verification.json'
assert not destination.exists(), 'retain prior lock-verification evidence'
destination.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
raise SystemExit(0 if report['all_registry_dependencies_match_frozen_candidate'] else 1)
