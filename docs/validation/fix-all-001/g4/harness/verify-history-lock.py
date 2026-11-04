"""Require every external harness registry dependency to match candidate Cargo.lock."""
import hashlib,json,pathlib,sys,tomllib
root,source=map(pathlib.Path,sys.argv[1:3])
identity=lambda p:(p['name'],p['version'],p.get('source'),p.get('checksum'))
old={identity(p) for p in tomllib.loads((source/'Cargo.lock').read_text())['package']}
new=tomllib.loads((root/'probe/Cargo.lock').read_text())['package']
registry=[p for p in new if p.get('source')]
unexpected=[identity(p) for p in registry if identity(p) not in old]
assert not unexpected,unexpected
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
path=root/'provenance.json';data=json.loads(path.read_text())
data.update(candidate_cargo_lock_sha256=sha(source/'Cargo.lock'),
            harness_cargo_lock_sha256=sha(root/'probe/Cargo.lock'),
            registry_dependencies_checked=len(registry),registry_dependencies_match_frozen_candidate=True)
path.write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps({'frozen_registry_dependencies_verified':len(registry)}))
