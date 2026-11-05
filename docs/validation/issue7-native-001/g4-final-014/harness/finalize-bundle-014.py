#!/usr/bin/env python3
"""Finalize the G4-014 evidence bundle: SHA256SUMS, MANIFEST.json, STATUS.json.

Ordering avoids self-reference cycles:
1. STATUS.json is written first (no manifest hash inside it);
2. SHA256SUMS lists every regular file under this bundle except MANIFEST.json
   and SHA256SUMS itself (so SHA256SUMS is NOT included);
3. MANIFEST.json lists every regular file except itself (so SHA256SUMS and
   STATUS.json are included, with their final hashes);
4. harness/check-bundle-014.py re-verifies everything from disk; its JSON result
   is written OUTSIDE the bundle (prepared-g4-014/bundle-self-check.json) so it
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
PREP = BUNDLE.parent / 'prepared-g4-014'
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
    main_summary = json.loads((BUNDLE / 'evidence/g4-014/summary.json').read_text(encoding='utf-8'))
    supplemental = json.loads((BUNDLE / 'evidence/g4-014/supplemental-001/summary.json').read_text(encoding='utf-8'))
    inventory = json.loads((BUNDLE / 'evidence/g4-014/inventory-001.json').read_text(encoding='utf-8'))
    loopback = inventory['native_loopback_counts']
    t1_targeted = supplemental.get('t1_write_learning_targeted')
    if t1_targeted is None:
        t1_path = BUNDLE / 'environment/t1-write-learning-counts.json'
        t1_targeted = json.loads(t1_path.read_text(encoding='utf-8')) if t1_path.is_file() else None
    snapshot_before_path = BUNDLE / 'environment/snapshot-verification-before.json'
    if snapshot_before_path.is_file():
        snapshot_before = json.loads(snapshot_before_path.read_text(encoding='utf-8'))
    else:
        snapshot_before = json.loads(
            (PREP / 'preparation-result.json').read_text(encoding='utf-8'))['snapshot_verification_before']
    abort_record = json.loads(
        (BUNDLE / 'environment/attempt3-interrupted/abort-record.json').read_text(encoding='utf-8'))
    abort_record4 = json.loads(
        (BUNDLE / 'environment/attempt4-root-invalid/abort-record.json').read_text(encoding='utf-8'))
    failed_main = [row['name'] for row in main_summary['commands']
                   if row.get('passed') is False and not row.get('not_run')]
    not_run = [row['name'] for row in main_summary['commands'] if row.get('not_run')]
    not_run += [row['name'] for row in supplemental['commands'] if row.get('not_run')]
    status = {
        'candidate_commit': 'b303ac6bf01f6ff66f48d1e74cb94740561a841c',
        'prior_baseline': {'bundle': 'G4-evidence-bundle-013-final',
                           'candidate_commit': 'fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4'},
        'final_run': 'g4-014-rerun',
        'run': {
            'run_dir_wsl': '/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-014-rerun',
            'summary_sha256': sha(BUNDLE / 'evidence/g4-014/summary.json'),
            'complete': main_summary.get('complete'),
            'all_commands_passed': main_summary.get('all_commands_passed'),
            'command_counts': main_summary.get('command_counts'),
            'supplemental_dir_wsl': ('/home/drive-native-test/.local/state/drive-native-ultra-validation/'
                                     'g4-014-rerun/supplemental-001'),
            'supplemental_summary_sha256': sha(BUNDLE / 'evidence/g4-014/supplemental-001/summary.json'),
            'supplemental_complete': supplemental.get('complete'),
            'supplemental_all_passed': supplemental.get('all_passed'),
            'supplemental_records_main_summary_sha256': supplemental.get('main_summary_sha256'),
        },
        'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'verifier': 'T4 independent verifier (no product source edits)',
        'acceptance_status': ('passed'
                              if (main_summary.get('complete') is True
                                  and main_summary.get('all_commands_passed') is True
                                  and not failed_main
                                  and supplemental.get('all_passed') is True)
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
        'native_loopback': {'g4_014_counts': loopback,
                            'baseline_013_sha256': inventory['native_loopback_baseline_013_sha256'],
                            'comparison': supplemental.get('matrix_acceptance', {}).get(
                                'loopback_vs_g4_013_corrected'),
                            'note': ('G4-014 uses the corrected 70-case harness; case-by-case '
                                     'observed classification is compared with the G4-013 corrected '
                                     'baseline')},
        't1_write_learning_targeted_tests': t1_targeted,
        'snapshot': snapshot_before,
        'aborted_partial_run': {'path_in_wsl': abort_record['run_dir_preserved_in_wsl'],
                                'reason': 'interrupted by the user during workspace-default',
                                'commands_recorded': abort_record['commands_recorded'],
                                'failed_commands': abort_record['failed_commands'],
                                'target_reuse': ('its target was used only as a Cargo.lock '
                                                 'registry-dependency seed for the formal rerun')},
        'invalid_root_user_run': {
            'evidence_dir': 'environment/attempt4-root-invalid',
            'reason': ('launched as root; permissions_kernel asserts getuid != 0, so the '
                       'FUSE/kernel group cannot pass under root'),
            'commands_recorded': abort_record4['commands_recorded'],
            'failed_commands': abort_record4['failed_commands'],
            'recorded_as_product_failure': False,
            'target_reuse': 'target not reused; formal run is executed as drive-native-test'},
        'not_run_commands': not_run,
        'exclusions': {
            'manifest_self': 'MANIFEST.json is not listed in its own files map',
            'sha256sums_self': 'SHA256SUMS lists all files except MANIFEST.json and SHA256SUMS',
            'absent_by_design': [
                'product source archive candidate.tar and run/source (retained at '
                'work/issue7-validation/prepared-g4-014/candidate.tar; SHA in source.json)',
                'build targets and all ELF binaries',
                'Store databases, scratch fixtures and temporary files',
            ],
        },
    }
    # Record self-reported failures that are not command exit failures, so the
    # bundle never hides them behind a green command row (honesty requirement).
    pkg_path = BUNDLE / 'evidence/g4-014/package-metadata-equivalence.json'
    pkg = json.loads(pkg_path.read_text(encoding='utf-8')) if pkg_path.is_file() else {}
    status['supplemental']['self_reported_failed_checks'] = []
    status['allowed_failed_checks'] = []
    if pkg.get('passed') is False:
        allowed = {
            'name': 'package-metadata-equivalence',
            'command_exit_ok': True,
            'self_reported_passed': False,
            'reason': ('packaging delta vs 9c28992 is non-empty: debian/changelog adds the '
                       '0.1.0~dev20261105.2 entry (commit d7d8b6a); frozen deb-001 manifest/'
                       'SHA256SUMS/package SHA still match their records; no deb build was run'),
            'result_sha256': sha(pkg_path),
            'product_behavior_impact': ('none observed; the delta is release metadata only and '
                                        'no product test asserted the frozen baseline'),
            'follow_up': ('T5 packaging must confirm the version bump is intended and rebuild/'
                          'verify the deb; this entry is not counted as a product failure'),
        }
        status['supplemental']['self_reported_failed_checks'].append(allowed)
        status['allowed_failed_checks'].append(allowed)
    measure_path = BUNDLE / 'evidence/g4-014/t9-measure-audit.json'
    measure = json.loads(measure_path.read_text(encoding='utf-8')) if measure_path.is_file() else {}
    if measure.get('passed') is False:
        allowed = {
            'name': 't9-measure-log-audit (JSON artefact self-report)',
            'command_exit_ok': True,
            'self_reported_passed': False,
            'reason': ('all four report claims are reproduced, but the home_attempts5 row is '
                       'internally inconsistent: retry phase reports 19 retries/298 requests in '
                       'the histogram against 24 disconnects and pages=311; the other three rows '
                       'reconcile. T9 report figures match the raw fields, so this is a '
                       'measurement-accounting caveat, not a product behaviour failure'),
            'result_sha256': sha(measure_path),
            'product_behavior_impact': 'none directly measured; T9 real-NAS claims remain as documented but the retry accounting in one row is not reconciled',
            'follow_up': ('T9 author should clarify disconnect/page/request accounting on the HOME '
                          'attempts5 row or annotate the report; no new NAS run was performed here'),
        }
        status['supplemental']['self_reported_failed_checks'].append(allowed)
        status['allowed_failed_checks'].append(allowed)
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
        'package': 'G4-evidence-bundle-014-final',
        'candidate_commit': status['candidate_commit'],
        'prior_baseline': status['prior_baseline'],
        'final_run': 'g4-014',
        'created_utc': status['created_utc'],
        'verifier': status['verifier'],
        'acceptance_status': status['acceptance_status'],
        'evidence_summary': {
            'main_command_count': status['main']['command_count'],
            'main_failed_commands': failed_main,
            'main_all_commands_passed': status['main']['all_commands_passed'],
            'native_loopback_counts': loopback,
            'native_loopback_baseline_013_sha256': inventory['native_loopback_baseline_013_sha256'],
            'supplemental_command_count': status['supplemental']['command_count'],
            'supplemental_all_passed': status['supplemental']['all_passed'],
            'allowed_failed_checks': [row['name'] for row in status['allowed_failed_checks']],
            'not_run_commands': not_run,
        },
        'exclusions': status['exclusions'],
        'self_check': ('python3 harness/check-bundle-014.py re-computes every hash; result recorded at '
                       '../prepared-g4-014/bundle-self-check.json (outside this bundle to avoid a '
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
    check = subprocess.run([sys.executable, str(HERE / 'check-bundle-014.py')],
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
