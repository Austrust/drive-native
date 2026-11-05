#!/usr/bin/env python3
"""Finalize the G4-013 evidence bundle: SHA256SUMS, MANIFEST.json, STATUS.json.

Ordering avoids self-reference cycles:
1. STATUS.json is written first (no manifest hash inside it);
2. SHA256SUMS lists every regular file under this bundle except MANIFEST.json
   and SHA256SUMS itself (so SHA256SUMS is NOT included);
3. MANIFEST.json lists every regular file except itself (so SHA256SUMS and
   STATUS.json are included, with their final hashes);
4. harness/check-bundle-013.py re-verifies everything from disk; its JSON result
   is written OUTSIDE the bundle (prepared-g4-013/bundle-self-check.json) so it
   cannot create another cycle. The manifest SHA256 is also recorded there for
   the final report message.

The script refuses to finalize if any ELF binary, product target/source tree or
oversized file is present.
"""
import datetime
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

BUNDLE = Path(__file__).resolve().parent.parent
HERE = BUNDLE / 'harness'
PREP = BUNDLE.parent / 'prepared-g4-013'
EXCLUDE_MANIFEST = {'MANIFEST.json'}
EXCLUDE_SUMS = {'MANIFEST.json', 'SHA256SUMS'}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def files():
    return sorted(path for path in BUNDLE.rglob('*') if path.is_file())


def main():
    manifest_path = BUNDLE / 'MANIFEST.json'
    sums_path = BUNDLE / 'SHA256SUMS'
    for stale in (manifest_path, sums_path, BUNDLE / 'STATUS.json'):
        if stale.exists():
            stale.unlink()
    all_files = files()
    unsafe = []
    for path in all_files:
        head = path.read_bytes()[:4]
        if head == b'\x7fELF':
            unsafe.append({'path': str(path.relative_to(BUNDLE)), 'why': 'ELF binary'})
        if path.stat().st_size > 64 * 1024 * 1024:
            unsafe.append({'path': str(path.relative_to(BUNDLE)), 'why': 'over 64 MiB'})
        if '/target/' in '/' + str(path) + '/' or '/tmp/' in '/' + str(path) + '/':
            unsafe.append({'path': str(path.relative_to(BUNDLE)), 'why': 'build/temp tree'})
    if unsafe:
        print(json.dumps({'refused': unsafe[:20], 'count': len(unsafe)}))
        return 1
    main_summary = json.loads((BUNDLE / 'evidence/g4-013/summary.json').read_text(encoding='utf-8'))
    supplemental = json.loads((BUNDLE / 'evidence/g4-013/supplemental-001/summary.json').read_text(encoding='utf-8'))
    inventory = json.loads((BUNDLE / 'evidence/g4-013/inventory-001.json').read_text(encoding='utf-8'))
    corrected = inventory['native_loopback_corrected_counts']
    frozen = inventory['native_loopback_frozen_counts']
    failed_main = [row['name'] for row in main_summary['commands']
                   if row.get('passed') is False and not row.get('not_run')]
    not_run = [row['name'] for row in main_summary['commands'] if row.get('not_run')]
    not_run += [row['name'] for row in supplemental['commands'] if row.get('not_run')]
    status = {
        'candidate_commit': 'fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4',
        'prior_baseline': {'bundle': 'G4-evidence-bundle-012-final',
                           'candidate_commit': '9c9aabae447d92275f93dae65320788240a03113'},
        'final_run': 'g4-013',
        'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'verifier': 'T4 independent verifier (no product source edits)',
        'acceptance_status': ('passed_with_documented_verifier_harness_deltas'
                              if (main_summary.get('complete') and supplemental.get('all_passed'))
                              else 'not_accepted'),
        'main': {'command_count': len(main_summary['commands']),
                 'passed_commands': main_summary.get('command_counts', {}).get('passed'),
                 'failed_commands': failed_main,
                 'not_run_commands': [row['name'] for row in main_summary['commands']
                                      if row.get('not_run')],
                 'complete': main_summary.get('complete'),
                 'all_commands_passed': main_summary.get('all_commands_passed'),
                 'workspace_all_features': next(
                     (row['counts'] for row in main_summary['commands']
                      if row['name'] == 'workspace-all-features'), None),
                 'workspace_default': next(
                     (row['counts'] for row in main_summary['commands']
                      if row['name'] == 'workspace-default'), None)},
        'supplemental': {'command_count': len(supplemental['commands']),
                         'complete': supplemental.get('complete'),
                         'all_passed': supplemental.get('all_passed'),
                         'failed_commands': [row['name'] for row in supplemental['commands']
                                             if row.get('passed') is False]},
        'native_loopback': {'frozen_012_expectation': frozen,
                            'corrected_verifier_expectation': corrected,
                            'changed_cases': ['upload_reply_9003', 'mkdir_reply_9003'],
                            'adversarial_added_cases': 4},
        't1_write_learning_targeted_tests': json.loads(
            (BUNDLE / 'environment/t1-write-learning-counts.json').read_text(encoding='utf-8')),
        'snapshot': json.loads(
            (BUNDLE / 'environment/snapshot-verification-before.json').read_text(encoding='utf-8')),
        'not_run_commands': not_run,
        'exclusions': {
            'manifest_self': 'MANIFEST.json is not listed in its own files map',
            'sha256sums_self': 'SHA256SUMS lists all files except MANIFEST.json and SHA256SUMS',
            'absent_by_design': [
                'product source archive candidate.tar and run/source (retained at '
                'work/issue7-validation/prepared-g4-013/candidate.tar; SHA in source.json)',
                'build targets and all ELF binaries',
                'Store databases, scratch fixtures and temporary files',
            ],
        },
    }
    status_path = BUNDLE / 'STATUS.json'
    status_path.write_text(json.dumps(status, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    sums_lines = []
    for path in files():
        relative = path.relative_to(BUNDLE).as_posix()
        if relative in EXCLUDE_SUMS:
            continue
        sums_lines.append(f'{sha(path)}  {relative}')
    sums_path.write_text('\n'.join(sums_lines) + '\n', encoding='utf-8')
    manifest_entries = {}
    for path in files():
        relative = path.relative_to(BUNDLE).as_posix()
        if relative in EXCLUDE_MANIFEST:
            continue
        manifest_entries[relative] = sha(path)
    manifest = {
        'package': 'G4-evidence-bundle-013-final',
        'candidate_commit': status['candidate_commit'],
        'prior_baseline': status['prior_baseline'],
        'final_run': 'g4-013',
        'created_utc': status['created_utc'],
        'verifier': status['verifier'],
        'acceptance_status': status['acceptance_status'],
        'evidence_summary': {
            'main_command_count': status['main']['command_count'],
            'main_failed_commands': failed_main,
            'main_all_commands_passed': status['main']['all_commands_passed'],
            'frozen_012_native_loopback_counts': frozen,
            'corrected_native_loopback_counts': corrected,
            'supplemental_command_count': status['supplemental']['command_count'],
            'supplemental_all_passed': status['supplemental']['all_passed'],
            'not_run_commands': not_run,
        },
        'exclusions': status['exclusions'],
        'self_check': ('python3 harness/check-bundle-013.py re-computes every hash; result recorded at '
                       '../prepared-g4-013/bundle-self-check.json (outside this bundle to avoid a '
                       'manifest/status self-reference cycle)'),
        'file_count': len(manifest_entries),
        'files': manifest_entries,
    }
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    link_errors = []
    for report in ('REPORT.md', 'COVERAGE.md'):
        path = BUNDLE / report
        if not path.is_file():
            link_errors.append({'report': report, 'error': 'missing'})
            continue
        for target in re.findall(r'\[[^\]]*\]\(([^)\s]+)\)', path.read_text(encoding='utf-8')):
            if target.startswith(('http://', 'https://', '#')):
                continue
            relative = target.split('#', 1)[0]
            if not (BUNDLE / relative).exists():
                link_errors.append({'report': report, 'link': target})
    check = subprocess.run([sys.executable, str(HERE / 'check-bundle-013.py')],
                           capture_output=True, text=True, encoding='utf-8')
    record = {'manifest_sha256': sha(manifest_path),
              'sha256sums_sha256': sha(sums_path),
              'manifest_files': len(manifest_entries),
              'sha256sums_entries': len(sums_lines),
              'link_errors': link_errors,
              'self_check_exit_code': check.returncode,
              'self_check_stdout': check.stdout.strip(),
              'self_check_stderr': check.stderr.strip(),
              'finished_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}
    PREP.mkdir(parents=True, exist_ok=True)
    (PREP / 'bundle-self-check.json').write_text(json.dumps(record, indent=2, ensure_ascii=False) + '\n',
                                                 encoding='utf-8')
    print(json.dumps({'manifest_sha256': record['manifest_sha256'],
                      'sha256sums_sha256': record['sha256sums_sha256'],
                      'files': len(manifest_entries), 'sha256sums_entries': len(sums_lines),
                      'link_errors': len(link_errors),
                      'self_check_exit': check.returncode}))
    return 0 if check.returncode == 0 and not link_errors else 1


if __name__ == '__main__':
    sys.exit(main())