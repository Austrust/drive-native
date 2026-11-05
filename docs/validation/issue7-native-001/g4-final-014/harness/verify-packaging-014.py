#!/usr/bin/env python3
"""Package-side regression preflight for G4-014 (no deb build; task-5 owns it).

Checks on the verified snapshot / repository identity:
1. packaging inputs (debian/, scripts/, Cargo.toml, Cargo.lock, vendor/) are
   identical between the previous product freeze 9c28992 and b303ac6;
2. the previously frozen local deb delivery is still byte-identical to its own
   freeze record (this verifier must not modify frozen packages).
Output is JSON with counts consumed by the G4 harness row.
"""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--git-dir', type=Path, required=True)
    parser.add_argument('--from-commit', required=True)
    parser.add_argument('--to-commit', required=True)
    parser.add_argument('--freeze', type=Path, required=True)
    parser.add_argument('--delivery', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    base = ['git', '-c', 'safe.directory=*', '--git-dir=' + str(args.git_dir)]
    delta_command = base + ['diff', '--name-status', args.from_commit, args.to_commit,
                            '--', 'debian', 'scripts', 'Cargo.toml', 'Cargo.lock', 'vendor']
    process = subprocess.run(delta_command, capture_output=True, text=True)
    delta_lines = [line for line in process.stdout.splitlines() if line.strip()]
    freeze = json.loads(args.freeze.read_text())
    delivery = args.delivery
    debs = sorted(delivery.glob('*.deb'))
    package_path = debs[0] if debs else None
    checks = {
        'packaging_delta_command': delta_command,
        'packaging_delta_exit_code': process.returncode,
        'packaging_delta_stderr': process.stderr,
        'packaging_delta_lines': delta_lines,
        'packaging_delta_empty': process.returncode == 0 and not delta_lines,
        'frozen_manifest_sha256': sha(delivery / 'MANIFEST.json'),
        'frozen_manifest_matches_record': sha(delivery / 'MANIFEST.json') == freeze['manifest_sha256'],
        'frozen_sha256sums_matches_record': sha(delivery / 'SHA256SUMS') == freeze['sha256sums_sha256'],
        'frozen_package_count': len(debs),
        'frozen_package_path': str(package_path) if package_path else None,
        'frozen_package_sha256': sha(package_path) if package_path else None,
        'frozen_package_matches_record': bool(package_path) and sha(package_path) == freeze['package']['sha256'],
    }
    passed = (checks['packaging_delta_empty'] and checks['frozen_manifest_matches_record']
              and checks['frozen_sha256sums_matches_record'] and checks['frozen_package_matches_record'])
    result = {'scope': 'packaging inputs unchanged vs 9c28992; frozen local deb delivery untouched; no deb build run',
              'from_commit': args.from_commit, 'to_commit': args.to_commit,
              'freeze_record': str(args.freeze), 'delivery': str(delivery),
              'checks': checks, 'passed': passed,
              'counts': {'passed': 1 if passed else 0, 'failed': 0 if passed else 1, 'ignored': 0}}
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': passed}))


if __name__ == '__main__':
    main()