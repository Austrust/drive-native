#!/usr/bin/env python3
"""Export G4-013 run artifacts into the independent evidence bundle.

Copies only text/JSON/PNG artifacts and harness crate sources; never product
source trees, build targets, ELF binaries, stores or scratch directories.
Runs as the same WSL user that produced the run.
"""
import hashlib
import json
import shutil
import sys
from pathlib import Path

PROJECT = Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/'
               'P032-群晖Drive按需同步')
BUNDLE = PROJECT / 'work/issue7-validation/G4-evidence-bundle-013-final'
BASE = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
RUN = BASE / 'g4-013'
OUT = BUNDLE / 'evidence' / 'g4-013'
CANDIDATE = 'fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4'

SINGLE = ['summary.json', 'source.json', 'source-hashes-before.json',
          'snapshot-verification-after.json', 'retry-preflight-clocks.json',
          'native-loopback-results.json', 'native-loopback-elf.sha256',
          'worker-versions.json', 'package-metadata-equivalence.json']
DIRS = ['logs', 'acl-screenshots', 'screenshots']
CRATES = {
    'native-loopback-crate': ['Cargo.toml', 'Cargo.lock', 'provenance.json',
                              'lock-verification.json', 'cargo-metadata.json',
                              'src/main.rs'],
    'native-loopback-crate-013': ['Cargo.toml', 'Cargo.lock', 'provenance.json',
                                  'lock-verification.json', 'cargo-metadata.json',
                                  'src/main.rs'],
    'native-loopback-updated-001': ['results.json', 'elf.sha256'],
    'core-history-probe': ['Cargo.toml', 'Cargo.lock', 'provenance.json',
                           'metadata.json', 'metadata.log', 'src/lib.rs'],
    'native-worker-chain-build-001': ['build.json', 'prepare.stdout.log',
                                      'prepare.stderr.log', 'metadata.stdout.log',
                                      'metadata.stderr.log', 'verify-lock.stdout.log',
                                      'verify-lock.stderr.log', 'build.stdout.log',
                                      'build.stderr.log',
                                      'native-worker-chain-crate/Cargo.toml',
                                      'native-worker-chain-crate/Cargo.lock',
                                      'native-worker-chain-crate/provenance.json',
                                      'native-worker-chain-crate/lock-verification.json',
                                      'native-worker-chain-crate/src/main.rs',
                                      'native-worker-chain-crate/src/codec.rs'],
    'native-worker-chain-001': ['results.json'],
    'component-policy-handover-001': ['result.json'],
    'supplemental-001': ['summary.json', 'executed-elf-before.json', 'failure.txt'],
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def copy_file(source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    if source.is_file():
        shutil.copy2(source, target)
        return 1
    return 0


def main():
    if OUT.exists():
        raise SystemExit('evidence/g4-013 already exists; refusing to overwrite')
    OUT.mkdir(parents=True)
    copied = 0
    for name in SINGLE:
        copied += copy_file(RUN / name, OUT / name)
    for name in DIRS:
        folder = RUN / name
        if folder.is_dir():
            for path in sorted(folder.rglob('*')):
                if path.is_file():
                    copied += copy_file(path, OUT / name / path.relative_to(folder))
    for name, files in CRATES.items():
        for relative in files:
            copied += copy_file(RUN / name / relative, OUT / name / relative)
    # extra per-case artifacts for both worker-chain runs and handover logs
    for name in ('native-worker-chain-001',):
        for path in sorted((RUN / name).rglob('*')):
            if path.is_file() and path.name in ('case.json', 'output.log'):
                copied += copy_file(path, OUT / name / path.relative_to(RUN / name))
    for path in sorted((RUN / 'component-policy-handover-001/logs').glob('*')) if (
            RUN / 'component-policy-handover-001/logs').is_dir() else []:
        if path.is_file():
            copied += copy_file(path, OUT / 'component-policy-handover-001/logs' / path.name)
    for path in sorted((RUN / 'supplemental-001/logs').glob('*')) if (
            RUN / 'supplemental-001/logs').is_dir() else []:
        if path.is_file():
            copied += copy_file(path, OUT / 'supplemental-001/logs' / path.name)
    # package documentation staging output: inventory only (it duplicates payload docs)
    staged = RUN / 'package-documentation-preflight'
    if staged.is_dir():
        listing = sorted(str(path.relative_to(staged)) for path in staged.rglob('*')
                         if path.is_file())
        (OUT / 'package-documentation-preflight-inventory.json').write_text(
            json.dumps({'file_count': len(listing), 'files': listing}, indent=2) + '\n')
        copied += 1
    # inventory: every main and supplemental command with counts and log SHA
    main_summary = json.loads((RUN / 'summary.json').read_text())
    supplemental = json.loads((RUN / 'supplemental-001/summary.json').read_text())
    inventory = {
        'candidate_commit': CANDIDATE,
        'main': {'complete': main_summary.get('complete'),
                 'all_commands_passed': main_summary.get('all_commands_passed'),
                 'command_counts': main_summary.get('command_counts'),
                 'commands': [{'name': row['name'], 'passed': row.get('passed'),
                               'not_run': bool(row.get('not_run')),
                               'counts': row.get('counts'),
                               'exit_code': row.get('exit_code'),
                               'elapsed_seconds': row.get('elapsed_seconds'),
                               'log': row.get('log'),
                               'log_sha256': row.get('log_sha256')}
                              for row in main_summary['commands']]},
        'supplemental': {'complete': supplemental.get('complete'),
                         'all_passed': supplemental.get('all_passed'),
                         'commands': [{'name': row['name'], 'passed': row.get('passed'),
                                       'not_run': bool(row.get('not_run')),
                                       'counts': row.get('counts'),
                                       'exit_code': row.get('exit_code'),
                                       'elapsed_seconds': row.get('elapsed_seconds'),
                                       'log': row.get('log'),
                                       'log_sha256': row.get('log_sha256')}
                                      for row in supplemental['commands']]},
        'native_loopback_frozen_counts': json.loads(
            (RUN / 'native-loopback-results.json').read_text())['counts'],
        'native_loopback_corrected_counts': json.loads(
            (RUN / 'native-loopback-updated-001/results.json').read_text())['counts'],
    }
    (OUT / 'inventory-001.json').write_text(json.dumps(inventory, indent=2) + '\n')
    files = sorted(path for path in OUT.rglob('*') if path.is_file())
    report = {'candidate_commit': CANDIDATE, 'output': str(OUT), 'copied_files': len(files),
              'total_bytes': sum(path.stat().st_size for path in files),
              'main_command_count': len(main_summary['commands']),
              'supplemental_command_count': len(supplemental['commands']),
              'files': {str(path.relative_to(OUT)): sha(path) for path in files}}
    print(json.dumps({'copied_files': report['copied_files'],
                      'total_bytes': report['total_bytes'],
                      'main_command_count': report['main_command_count'],
                      'supplemental_command_count': report['supplemental_command_count']}))
    return 0


if __name__ == '__main__':
    sys.exit(main())