#!/usr/bin/env python3
"""Cross-check the harness files carried into this bundle against the frozen
G4-013-final MANIFEST (prior-round frozen evidence, read-only input).

The G4-014 bundle intentionally renames several verifier scripts (013 -> 014),
replaces the G4-013-corrected loopback copy (native-loopback-013 -> -014) and
adds new T9 tooling. Those are listed as intentional differences; everything
else must match the 013 frozen bytes.
"""
import hashlib
import json
from pathlib import Path

BUNDLE = Path(__file__).resolve().parent.parent
PRIOR = BUNDLE.parent / 'G4-evidence-bundle-013-final'
HERE = BUNDLE / 'harness'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


MAPPINGS = {
    'run-g4-014.py': 'harness/run-g4-013.py',
    'snapshot-verify.py': 'harness/snapshot-verify.py',
    'check-bundle-014.py': 'harness/check-bundle-013.py',
    'verify-history-labels.py': 'harness/verify-history-labels.py',
    'verify-worker-versions.py': 'harness/verify-worker-versions.py',
    'prepare-core-probe.py': 'harness/prepare-core-probe.py',
    'core-history-probe.rs': 'harness/core-history-probe.rs',
    'component-policy-handover.py': 'harness/component-policy-handover.py',
}
for base, prior_base in (('native-loopback-014', 'native-loopback-013'),
                         ('native-worker-chain-012', 'native-worker-chain-012'),
                         ('native-worker-chain-013', 'native-worker-chain-013')):
    for path in sorted((HERE / base).rglob('*')):
        if path.is_file():
            rel = path.relative_to(HERE).as_posix()
            prior = 'harness/' + prior_base + '/' + path.relative_to(HERE / base).as_posix()
            MAPPINGS[rel] = prior
KNOWN_INTENTIONAL = {
    'run-native-loopback.sh': 'renamed/adapted for the 014 corrected loopback harness',
    'prepare-g4-014.py': '013 script renamed+commit/paths updated to b303ac6/g4-014',
    'export-014.py': '013 script renamed+run/out paths and T9 artifacts list updated',
    'finalize-bundle-014.py': '013 script renamed+bundle/candidate paths updated',
    'collect-environment-014.py': '013 collector renamed+candidate/run updated',
    'verify-packaging-014.py': '013 preflight renamed+commit labels updated',
    'seed-old-target.py': 'new T9 component-replacement helper',
    'analyze-t9-measure.py': 'new T9 measurement re-derivation tool',
    'prepare-prev-candidate.py': 'new T9 component-replacement helper',
    'run-g4-014-supplements.py': 'new G4-014 supplement driver',
    'start-g4-014.sh': 'new matrix launcher',
    'start-g4-014-supplements.sh': 'new supplement launcher',
    'check-harness-copies-014.py': 'this checker (renamed from 013)',
}


def main():
    prior_files = json.loads((PRIOR / 'MANIFEST.json').read_text())['files']
    results = []
    for local, prior in sorted(MAPPINGS.items()):
        actual = sha(HERE / local)
        expected = prior_files.get(prior)
        results.append({'local': 'harness/' + local, 'prior_manifest_path': prior,
                        'prior_sha256': expected, 'actual_sha256': actual,
                        'match': expected == actual})
    extras = []
    for path in sorted(HERE.rglob('*')):
        if not path.is_file():
            continue
        rel = path.relative_to(HERE).as_posix()
        if rel in MAPPINGS or rel in KNOWN_INTENTIONAL:
            continue
        extras.append({'path': 'harness/' + rel, 'sha256': sha(path)})
    mismatches = [row for row in results if not row['match']]
    result = {
        'scope': 'copied harness files vs frozen G4-013-final MANIFEST (read-only prior evidence)',
        'prior_manifest': str(PRIOR / 'MANIFEST.json'),
        'prior_manifest_sha256': sha(PRIOR / 'MANIFEST.json'),
        'checked': len(results),
        'mismatches': mismatches,
        'known_intentional_differences': KNOWN_INTENTIONAL,
        'verifier_authored_files': extras,
        'passed': not mismatches,
    }
    output = BUNDLE / 'environment' / 'harness-crosscheck-014.json'
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'checked': len(results), 'mismatches': len(mismatches),
                      'passed': result['passed']}))


if __name__ == '__main__':
    main()