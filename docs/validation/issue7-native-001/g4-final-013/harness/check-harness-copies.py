#!/usr/bin/env python3
"""Cross-check the harness files copied into this bundle against the frozen
G4-012-final MANIFEST (prior-round frozen evidence, read-only input).

The only intentional difference is harness/run-native-loopback.sh (path
resolution adapted so this bundle is self-contained; header documents it).
Verifier-authored drivers/scripts are listed separately: they are new files,
not expected in the 012 MANIFEST, and their SHA256 is recorded by this package.
"""
import hashlib
import json
from pathlib import Path

BUNDLE = Path(__file__).resolve().parent.parent
PRIOR = BUNDLE.parent / 'G4-evidence-bundle-012-final'
HERE = BUNDLE / 'harness'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    prior_files = json.loads((PRIOR / 'MANIFEST.json').read_text())['files']
    mappings = {}
    for base in ('native-loopback', 'native-worker-chain-012'):
        for path in sorted((HERE / base).rglob('*')):
            if path.is_file():
                rel = path.relative_to(HERE).as_posix()
                mappings[rel] = 'harness/' + rel
    for name in ('component-policy-handover.py', 'prepare-core-probe.py',
                 'verify-worker-versions.py', 'verify-history-labels.py',
                 'core-history-probe.rs'):
        mappings[name] = 'harness/' + name
    mappings['prepared-g4-013/run-g4.py'] = 'harness/prepared-g4-012/run-g4.py'
    results = []
    for local, prior in sorted(mappings.items()):
        actual = sha(HERE / local)
        expected = prior_files.get(prior)
        results.append({'local': 'harness/' + local, 'prior_manifest_path': prior,
                        'prior_sha256': expected, 'actual_sha256': actual,
                        'match': expected == actual})
    known_extra = {
        'run-native-loopback.sh': 'intentional path-resolution adaptation; original bytes differ (header documents why)',
    }
    verifier_authored = []
    for path in sorted(HERE.rglob('*')):
        if not path.is_file():
            continue
        rel = path.relative_to(HERE).as_posix()
        if rel.startswith(('native-loopback/', 'native-worker-chain-012/')):
            continue
        if rel in mappings or rel in known_extra:
            continue
        if rel == 'prepared-g4-013/source.json':
            continue
        if rel == 'prepared-g4-013/launcher.log':
            continue
        verifier_authored.append({'path': 'harness/' + rel, 'sha256': sha(path)})
    mismatches = [row for row in results if not row['match']]
    result = {
        'scope': 'copied harness files vs frozen G4-012-final MANIFEST (read-only prior evidence)',
        'prior_manifest': str(PRIOR / 'MANIFEST.json'),
        'prior_manifest_sha256': sha(PRIOR / 'MANIFEST.json'),
        'checked': len(results),
        'mismatches': mismatches,
        'known_intentional_differences': known_extra,
        'verifier_authored_files': verifier_authored,
        'passed': not mismatches,
    }
    output = BUNDLE / 'environment' / 'harness-crosscheck-013.json'
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'checked': len(results), 'mismatches': len(mismatches),
                      'passed': result['passed'], 'output': str(output)}))


if __name__ == '__main__':
    main()