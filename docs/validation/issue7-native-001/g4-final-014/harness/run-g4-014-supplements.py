#!/usr/bin/env python3
"""Sequential G4-014 supplements: T9 directed + adversarial, T1 targeted,
core-history, capacity, release-bins, TLS worker chain, component replacement,
package preflight and full audits.

Verifier-authored driver (same evidence discipline as G4-013). The main matrix
runs the G4-013-corrected native-loopback harness, so no expectation delta is
expected here; the loopback observed matrix is compared case by case with the
G4-013 corrected baseline to detect any T9-induced behaviour drift.
"""
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import signal
import subprocess
import time
import traceback

HERE = pathlib.Path(__file__).resolve().parent
PROJECT = pathlib.Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/'
                       'P032-群晖Drive按需同步')
BASE = pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
RUN = BASE / 'g4-014-rerun'
SOURCE = RUN / 'source'
TARGET = RUN / 'target'
CANDIDATE = 'b303ac6bf01f6ff66f48d1e74cb94740561a841c'
PREV_BUNDLE = PROJECT / 'work/issue7-validation/G4-evidence-bundle-013-final/evidence/g4-013'
OLD_STAGE = BASE / 'g4-014-old-candidate-snapshot'
OLD_TARGET = BASE / 'g4-014-old-target'
OLD_COMMIT = '41365a650eb7d7df696046e21b7671697231f851'
FREEZE = PROJECT / 'work/issue7-validation/package-delivery-freeze.json'
DELIVERY = PROJECT / 'work/issue7-deb-001-artifacts'
AUTHOR_RELEASE = {
    'drive-native-fsd': 'a3ed4ce5b5a83754c14b3257130f6da2a528408e9151edf329f3e647073d3d75',
    'drive-native-worker': '54e96e175ee4af6d84fdf47528c812f47cdefed1c5d9aaa2fb8f7c2329587f59',
    'drive-native-setup': 'dafc0249aefc254594a82e5e799c20cde7e5f1373fdd7d9c68c4df52200abe09',
}


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


g4 = load('frozen_g4', HERE / 'run-g4-014.py')
snapshot = load('snapshot_verify', HERE / 'snapshot-verify.py')


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


main = json.loads((RUN / 'summary.json').read_text())
assert main['candidate_commit'] == CANDIDATE, 'main matrix candidate mismatch'
assert main.get('complete') is True, 'main matrix did not complete'
failed = [row['name'] for row in main['commands']
          if row.get('passed') is False and not row.get('not_run')]
assert failed == [], f'unexpected and unaccepted main-matrix failures: {failed}'
baseline = json.loads((PREV_BUNDLE / 'native-loopback-updated-001/results.json').read_text())
loopback = json.loads((RUN / 'native-loopback-results.json').read_text())
assert loopback['counts'] == {'passed': 70, 'failed': 0, 'ignored': 0, 'total': 70}, loopback['counts']
loop_by_name = {case['name']: case for case in loopback['cases']}
base_by_name = {case['name']: case for case in baseline['cases']}
assert set(loop_by_name) == set(base_by_name), 'loopback case set changed vs G4-013'
loop_changes = []
for name in sorted(loop_by_name):
    if loop_by_name[name].get('observed') != base_by_name[name].get('observed'):
        loop_changes.append({'case': name,
                             'g4_013_observed': base_by_name[name].get('observed'),
                             'g4_014_observed': loop_by_name[name].get('observed')})
assert not loop_changes, f'T9 round changed loopback classification: {loop_changes}'
loopback_compare = {'cases': len(loop_by_name), 'counts': loopback['counts'],
                    'changed_observed_cases': loop_changes,
                    'baseline': str(PREV_BUNDLE / 'native-loopback-updated-001/results.json'),
                    'verdict': 'observed classification identical to G4-013 corrected baseline'}

OUT = RUN / 'supplemental-001'
OUT.mkdir(exist_ok=False)
(OUT / 'logs').mkdir()
source_json = json.loads((RUN / 'source.json').read_text())
git_dir = pathlib.Path([arg.split('=', 1)[1] for arg in source_json['archive_command']
                        if arg.startswith('--git-dir=')][0])
env = os.environ.copy()
for key in list(env):
    if key.startswith('DRIVE_NATIVE_') or key in (
            'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'DBUS_SESSION_BUS_ADDRESS', 'DISPLAY',
            'WAYLAND_DISPLAY'):
        env.pop(key, None)
env.update(CARGO_TARGET_DIR=str(TARGET), CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2',
           CARGO_NET_OFFLINE='true', PYTHONDONTWRITEBYTECODE='1', RUST_BACKTRACE='1',
           QT_QPA_PLATFORM='offscreen', PYTHONPATH=str(SOURCE / 'desktop'),
           TMPDIR=str(RUN / 'tmp'))
for key, name in [('XDG_CONFIG_HOME', 'config'), ('XDG_DATA_HOME', 'data'),
                  ('XDG_CACHE_HOME', 'cache'), ('XDG_RUNTIME_DIR', 'runtime')]:
    env[key] = str(RUN / name)

result = {'candidate_commit': CANDIDATE, 'complete': False, 'all_passed': False, 'commands': [],
          'scope': ('T9 retry directed/adversarial, T1 targeted, core-history, capacity, release '
                    'bins, real worker TLS chain, component replacement, package preflight; no '
                    'NAS/deb/performance claims'),
          'main_summary_sha256': sha(RUN / 'summary.json'),
          'harness_sha256': sha(pathlib.Path(__file__)),
          'matrix_acceptance': {
              'main_failed_commands': failed,
              'main_all_commands_passed': main.get('all_commands_passed'),
              'loopback_vs_g4_013_corrected': loopback_compare},
          'target': str(TARGET),
          'target_provenance': ('fresh empty private target created by run-g4-014.py for this '
                                'candidate; no author target or product artifact copied')}
prev_candidate = BASE / 'g4-014-old-candidate-snapshot'
if (PROJECT / 'work/issue7-validation/prepared-g4-014-prev/preparation-result.json').is_file():
    result['old_candidate_snapshot'] = json.loads(
        (PROJECT / 'work/issue7-validation/prepared-g4-014-prev/preparation-result.json').read_text())


def persist():
    save(OUT / 'summary.json', result)


def interrupted(signum, frame):
    raise InterruptedError(f'supplement driver received signal {signum}')


signal.signal(signal.SIGTERM, interrupted)


def command(name, argv, kind='prepare', custom_env=None, timeout=3600, cwd=SOURCE,
            result_path=None):
    row = {'name': name, 'argv': argv, 'cwd': str(cwd), 'kind': kind,
           'log': 'logs/' + name + '.log', 'host_space_before': g4.host_c_space(),
           'started_utc': g4.stamp()}
    result['commands'].append(row)
    persist()
    free = row['host_space_before']['free_bytes']
    if free is None or free < 10 * 1024 ** 3:
        row.update(passed=False, not_run=True,
                   reason='host C below 10GiB/unknown; stopped before command')
        persist()
        raise RuntimeError(row['reason'])
    start = time.monotonic()
    active_env = env.copy()
    active_env.update(custom_env or {})
    with (OUT / row['log']).open('wb') as log:
        process = subprocess.Popen(argv, cwd=cwd, env=active_env, stdout=log,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        row['process_group'] = process.pid
        persist()
        try:
            row['exit_code'] = process.wait(timeout=timeout)
        except BaseException as error:
            row.update(exception_type=type(error).__name__, error=str(error))
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=15)
            row.update(exit_code=process.returncode,
                       timed_out=isinstance(error, subprocess.TimeoutExpired))
    row.update(finished_utc=g4.stamp(), elapsed_seconds=time.monotonic() - start,
               log_sha256=sha(OUT / row['log']))
    row['counts'] = g4.counts((OUT / row['log']).read_text(errors='replace'), kind)
    if result_path and result_path.is_file():
        data = json.loads(result_path.read_text())
        values = data.get('counts', data)
        row['counts'] = {'reported': True, **{key: values[key] for key in
                                              ('passed', 'failed', 'ignored')}}
        row['result_path'] = str(result_path.relative_to(RUN))
        row['result_sha256'] = sha(result_path)
    row['passed'] = row['exit_code'] == 0 and 'exception_type' not in row
    persist()
    print(json.dumps({'name': name, 'passed': row['passed'], 'counts': row['counts']}),
          flush=True)
    if 'exception_type' in row or row['exit_code'] < 0 or row['exit_code'] >= 128:
        raise RuntimeError('supplement interrupted or timed out; do not enter another command')
    return row['passed']


def executed_elf_witness():
    paths = set()
    for row in main['commands']:
        if not row.get('log') or 'exit_code' not in row:
            continue
        text = (RUN / row['log']).read_text(errors='replace')
        for item in re.findall(r'^\s*Running .*\(([^)]+)\)\s*$', text, re.M):
            path = pathlib.Path(item)
            if not path.is_absolute():
                path = SOURCE / path
            paths.add(path.resolve())
    for name in ('drive-native-fsd', 'drive-native-worker', 'drive-native-setup'):
        path = TARGET / 'debug' / name
        if path.is_file():
            paths.add(path)
    records = []
    for path in sorted(paths):
        assert path.is_relative_to(TARGET) and path.is_file(), path
        with path.open('rb') as stream:
            assert stream.read(4) == b'\x7fELF', path
        records.append({'path': str(path), 'sha256': sha(path), 'bytes': path.stat().st_size})
    assert records
    witness = {'candidate_commit': CANDIDATE,
               'executed_tests_from': 'Cargo Running lines in preserved main matrix logs',
               'observation_boundary': ('ELFs still present after the main matrix, hashed before '
                                        'supplements and compared after'),
               'excluded': ('native-loopback probe and t9-retry-probe are verifier harness '
                            'binaries rebuilt by supplements; not product binaries'),
               'files': records}
    save(OUT / 'executed-elf-before.json', witness)
    return witness


witness = None
persist()
try:
    witness = executed_elf_witness()
    persist()
    persist()
    # A. T9 directed deterministic tests (author claims 4 retry / 10 seed)
    t9_core = ['cargo', 'test', '--offline', '--locked', '-p', 'drive-core', '--all-features', '--lib']
    command('t9-retry-unit-tests', t9_core + ['retry::tests', '--', '--nocapture'],
            'rust', timeout=1800)
    retry_counts = result['commands'][-1]['counts']
    assert retry_counts.get('passed') == 4 and retry_counts.get('failed') == 0, retry_counts
    command('t9-seed-retry-tests', t9_core + ['seed_tests', '--', '--nocapture'],
            'rust', timeout=1800)
    seed_counts = result['commands'][-1]['counts']
    assert seed_counts.get('passed') == 10 and seed_counts.get('failed') == 0, seed_counts
    command('t9-drive-core-lib', t9_core + ['--', '--nocapture'], 'rust', timeout=3600)
    command('t9-drive-runtime-lib',
            ['cargo', 'test', '--offline', '--locked', '-p', 'drive-runtime', '--all-features',
             '--lib', '--', '--nocapture'], 'rust', timeout=1800)
    result['t9_directed'] = {
        'retry_unit_tests': retry_counts, 'seed_retry_tests': seed_counts,
        'drive_core_lib_all_features': result['commands'][-2]['counts'],
        'drive_runtime_lib_all_features': result['commands'][-1]['counts']}
    persist()
    # B. external adversarial probe of the actual native_retry.rs source
    probe_prepared = command(
        't9-retry-probe-prepare',
        ['python3', '-B', str(HERE / 't9-retry-probe' / 'prepare.py'),
         '--source', str(SOURCE), '--output', str(RUN / 't9-retry-probe'),
         '--candidate', CANDIDATE], 'prepare', timeout=300)
    if probe_prepared:
        command('t9-retry-probe-tests',
                ['cargo', 'test', '--offline', '--locked', '--manifest-path',
                 str(RUN / 't9-retry-probe' / 'Cargo.toml'), '--', '--nocapture'],
                'rust', timeout=1800)
        probe_counts = result['commands'][-1]['counts']
        assert probe_counts.get('passed') == 13 and probe_counts.get('failed') == 0, probe_counts
        result['t9_adversarial_probe'] = {
            'counts': probe_counts,
            'provenance': str(RUN / 't9-retry-probe' / 'provenance.json'),
            'provenance_sha256': sha(RUN / 't9-retry-probe' / 'provenance.json'),
            'scope': ('actual product native_retry.rs included byte-for-byte except stripped module '
                      'doc lines; real sproto::Error; tests attempt cap/backoff bound/non-retryable '
                      'classes/each attempt re-creates the request future/cancellation')}
        persist()
    # C. independent re-derivation of the committed real-NAS measurement claims
    command('t9-measure-log-audit',
            ['python3', '-B', str(HERE / 'analyze-t9-measure.py'),
             '--docs', str(SOURCE / 'docs/validation/real-nas-initial-index-20261105'),
             '--output', str(RUN / 't9-measure-audit.json')],
            'json', result_path=RUN / 't9-measure-audit.json')
    # D. T1 write-learning targeted tests (unchanged by T9, re-run for regression)
    targeted = [
        ('sproto-error-tests',
         ['cargo', 'test', '--offline', '--locked', '-p', 'sproto', '--lib',
          '--all-features', 'error::tests', '--', '--nocapture']),
        ('worker-mutation-permission-tests',
         ['cargo', 'test', '--offline', '--locked', '-p', 'drive-runtime', '--lib',
          '--all-features', 'mutation_permission_tests', '--', '--nocapture']),
        ('bridge-learned-upload-denial',
         ['cargo', 'test', '--offline', '--locked', '-p', 'drive-runtime', '--lib',
          '--all-features',
          'learned_upload_denial_gates_the_queue_and_recheck_releases_the_same_intent',
          '--', '--nocapture']),
        ('bridge-ambiguous-upload-hint',
         ['cargo', 'test', '--offline', '--locked', '-p', 'drive-runtime', '--lib',
          '--all-features', 'ambiguous_upload_permission_hint_never_learns_a_denial',
          '--', '--nocapture']),
        ('bridge-transport-loss-no-learn',
         ['cargo', 'test', '--offline', '--locked', '-p', 'drive-runtime', '--lib',
          '--all-features', 'transport_loss_during_upload_never_learns_an_acl_denial',
          '--', '--nocapture']),
        ('core-learned-upload-denial-revision',
         ['cargo', 'test', '--offline', '--locked', '-p', 'drive-core', '--lib',
          '--all-features', 'learned_upload_denial_uses_current_revision_persists_and_expires',
          '--', '--nocapture']),
        ('core-definite-refusal-target-guard',
         ['cargo', 'test', '--offline', '--locked', '-p', 'drive-core', '--lib',
          '--all-features', 'definite_refusal_must_match_an_authorized_target',
          '--', '--nocapture']),
        ('core-rename-remove-hints-no-rows',
         ['cargo', 'test', '--offline', '--locked', '-p', 'drive-core', '--lib',
          '--all-features', 'rename_and_remove_permission_hints_never_write_acl_rows',
          '--', '--nocapture']),
    ]
    result['t1_write_learning_targeted'] = []
    for name, argv in targeted:
        passed = command('t1-' + name, argv, 'rust', timeout=1800)
        result['t1_write_learning_targeted'].append(
            {'name': name, 'passed': passed, 'counts': result['commands'][-1]['counts']})
    persist()
    # E. standard compiler check
    command('standard-check-all-targets',
            ['cargo', 'check', '--workspace', '--all-targets', '--all-features',
             '--locked', '--offline', '-j2'], 'prepare', timeout=3600)
    # G. 210k functional capacity
    cargo = ['cargo', 'test', '--offline', '--locked', '-p', 'drive-core', '--lib', '--all-features']
    assert command('capacity-ignored-list', cargo + ['--', '--list', '--ignored'])
    text = (OUT / 'logs/capacity-ignored-list.log').read_text()
    names = re.findall(r'^(\S*default_budget_admits_210k_node_snapshot_without_holding_store): test$', text, re.M)
    assert len(names) == 1, names
    command('capacity-210k', cargo + [names[0], '--', '--ignored', '--exact', '--test-threads=1'], 'rust')
    # H. T9 release binaries (author-supplied fixtures for T3); compare SHA with author record
    release_ok = command('t9-release-bins-build',
                         ['cargo', 'build', '--offline', '--locked', '-p', 'drive-runtime',
                          '--all-features', '--release', '--bins', '-j2'],
                         'prepare', timeout=2400)
    if release_ok:
        hashes = {name: sha(TARGET / 'release' / name) for name in AUTHOR_RELEASE}
        comparison = {name: {'built_sha256': hashes[name], 'author_sha256': AUTHOR_RELEASE[name],
                             'match': hashes[name] == AUTHOR_RELEASE[name]}
                      for name in AUTHOR_RELEASE}
        save(RUN / 'release-bins-014.json', {'candidate_commit': CANDIDATE,
                                             'binaries': comparison,
                                             'scope': 'local release build from the verified snapshot'})
        result['t9_release_bins'] = {
            'comparison': comparison,
            'record': str(RUN / 'release-bins-014.json'),
            'all_match_author': all(row['match'] for row in comparison.values())}
        persist()
    # I. real worker TLS -> IPC -> Bridge chain (frozen 012 harness; reuse gate valid only if main all-pass)
    tls_ok = False
    if main.get('all_commands_passed') is True:
        chain = HERE / 'native-worker-chain-012'
        built = command('tls-harness-build',
                        ['python3', '-B', str(chain / 'build.py'), '--source', str(SOURCE),
                         '--output', str(RUN / 'native-worker-chain-build-001'),
                         '--target', str(TARGET), '--candidate', CANDIDATE,
                         '--reuse-completed-run', str(RUN),
                         '--host-space-helper', str(HERE / 'run-g4-014.py')],
                        timeout=4200)
        if built:
            build = json.loads((RUN / 'native-worker-chain-build-001/build.json').read_text())
            assert build['passed']
            command('tls-actual-worker-all-ten',
                    ['python3', '-B', str(chain / 'run.py'), '--binary', build['binary']['path'],
                     '--worker', str(TARGET / 'debug/drive-native-worker'), '--crate', build['crate'],
                     '--candidate', CANDIDATE, '--output', str(RUN / 'native-worker-chain-001'),
                     '--host-space-helper', str(HERE / 'run-g4-014.py')],
                    timeout=900)
            tls = json.loads((RUN / 'native-worker-chain-001/results.json').read_text())
            assert tls['counts'] == {'passed': 10, 'failed': 0, 'ignored': 0, 'total': 10}, tls['counts']
            result['tls_worker_chain'] = {'counts': tls['counts'],
                                          'harness_build_sha256': sha(RUN / 'native-worker-chain-build-001/build.json')}
            tls_ok = True
            persist()
    if not tls_ok:
        result['tls_worker_chain'] = {'status': 'blocked',
                                      'reason': 'reuse gate requires main all_commands_passed'}
        persist()
    # J. private component pair replacement (old side rebuilt from 41365a6 snapshot by this verifier)
    seed_ok = command('old-target-seed',
                      ['python3', '-B', str(HERE / 'seed-old-target.py'),
                       '--source', str(OLD_STAGE / 'source'), '--seed', str(TARGET),
                       '--target', str(OLD_TARGET),
                       '--output', str(RUN / 'old-target-seed.json')], 'prepare', timeout=1800)
    binaries = ['cargo', 'build', '--offline', '--locked', '-p', 'drive-runtime',
                '--all-features', '--bins', '--example', 'multi_mount_fixture']
    old_built = seed_ok and command('old-component-build', binaries,
                                    custom_env={'CARGO_TARGET_DIR': str(OLD_TARGET)},
                                    cwd=OLD_STAGE / 'source')
    new_built = seed_ok and command('new-component-build', binaries)
    if old_built and new_built:
        command('private-component-policy-handover',
                ['dbus-run-session', '--', '/usr/bin/python3', '-B',
                 str(HERE / 'component-policy-handover.py'),
                 '--root', str(RUN / 'component-policy-handover-001'),
                 '--fixture', str(TARGET / 'debug/examples/multi_mount_fixture'),
                 '--old-binaries', str(OLD_TARGET / 'debug'),
                 '--new-binaries', str(TARGET / 'debug'),
                 '--old-commit', OLD_COMMIT, '--new-commit', CANDIDATE], timeout=600)
        handover = json.loads((RUN / 'component-policy-handover-001/result.json').read_text())
        assert handover['passed'] is True
        result['component_handover'] = {
            'passed': True, 'cases': len(handover.get('cases', [])),
            'result_sha256': sha(RUN / 'component-policy-handover-001/result.json'),
            'old_side': 'rebuilt by this verifier from an immutable 41365a6 snapshot; not the author build',
            'scope': handover.get('scope')}
        persist()
    else:
        result['component_handover'] = {'status': 'blocked',
                                        'reason': 'old-candidate snapshot or build unavailable'}
        persist()
    # K. package-side regression preflight (no deb build; packaging task owns it)
    command('package-version', ['dpkg-parsechangelog', '-S', 'Version'], 'prepare')
    command('package-documentation-staging',
            ['python3', 'scripts/stage-docs.py', str(RUN / 'package-documentation-preflight')],
            'prepare', custom_env={'DRIVE_NATIVE_SOURCE_COMMIT': CANDIDATE,
                                   'DRIVE_NATIVE_OFFLINE_DOCS': '1',
                                   'DEB_BUILD_OPTIONS': 'parallel=2'})
    command('package-metadata-equivalence',
            ['python3', '-B', str(HERE / 'verify-packaging-014.py'),
             '--git-dir', str(git_dir), '--from-commit',
             '9c28992eb67a5620bf83b4939dc262dfe239027f', '--to-commit', CANDIDATE,
             '--freeze', str(FREEZE), '--delivery', str(DELIVERY),
             '--output', str(RUN / 'package-metadata-equivalence.json')],
            'json', result_path=RUN / 'package-metadata-equivalence.json')
    packaging = json.loads((RUN / 'package-metadata-equivalence.json').read_text())
    result['package_preflight'] = {
        'changelog_version_output': (OUT / 'logs/package-version.log').read_text().strip(),
        'packaging_delta_empty': packaging['checks']['packaging_delta_empty'],
        'frozen_delivery_untouched': packaging['passed'],
        'full_deb_build': 'not_run (owned by the packaging task)',
        'result_sha256': sha(RUN / 'package-metadata-equivalence.json')}
    deb2 = PROJECT / 'work/issue7-deb-002-artifacts'
    if deb2.is_dir():
        manifests = sorted(deb2.glob('MANIFEST.json'))
        result['package_preflight']['deb_002_artifacts_present'] = {
            'path': str(deb2),
            'manifest_sha256': sha(manifests[0]) if manifests else None,
            'note': 'observed only; not validated by this bundle'}
    persist()
    result['complete'] = True
except BaseException as error:
    result.update(exception_type=type(error).__name__, error=str(error))
    (OUT / 'failure.txt').write_text(traceback.format_exc())
finally:
    result['audit_errors'] = []
    result['changed_executed_elf'] = []
    result['changed_source_files'] = []
    result['residual_owned_mounts'] = []
    try:
        assert witness is not None, 'ELF witness was not completed'
        result['changed_executed_elf'] = [
            row['path'] for row in witness['files']
            if not pathlib.Path(row['path']).is_file() or sha(row['path']) != row['sha256']]
    except BaseException as error:
        result['audit_errors'].append({'audit': 'executed ELF', 'type': type(error).__name__,
                                       'error': str(error)})
    try:
        before = json.loads((RUN / 'source-hashes-before.json').read_text())
        result['changed_source_files'] = [
            name for name, expected in before.items()
            if not (SOURCE / name).is_file() or sha(SOURCE / name) != expected]
    except BaseException as error:
        result['audit_errors'].append({'audit': 'frozen source', 'type': type(error).__name__,
                                       'error': str(error)})
    try:
        result['residual_owned_mounts'] = [str(path) for path in g4.mounts_below(RUN)]
    except BaseException as error:
        result['audit_errors'].append({'audit': 'mountinfo', 'type': type(error).__name__,
                                       'error': str(error)})
    try:
        archive = PROJECT / 'work/issue7-validation/prepared-g4-014/candidate.tar'
        result['archive_integrity'] = {'sha256': sha(archive),
                                       'expected': source_json['candidate_archive_sha256'],
                                       'unchanged': sha(archive) == source_json['candidate_archive_sha256']}
    except BaseException as error:
        result['audit_errors'].append({'audit': 'archive', 'type': type(error).__name__,
                                       'error': str(error)})
    try:
        after = snapshot.verify_tree(None, CANDIDATE, SOURCE, git_dir=git_dir,
                                     allow_extra=['vendor/fuser/Cargo.lock'])
        save(RUN / 'snapshot-verification-after.json', after)
        before_tree = source_json['snapshot_verification']['tree_listing_sha256']
        result['snapshot_verification_after'] = {
            'path': str(RUN / 'snapshot-verification-after.json'),
            'passed': after['passed'], 'entries': after['entries'],
            'problem_count': after['problem_count'],
            'tree_listing_sha256': after['tree_listing_sha256'],
            'same_as_before': after['tree_listing_sha256'] == before_tree}
    except BaseException as error:
        result['audit_errors'].append({'audit': 'snapshot tree', 'type': type(error).__name__,
                                       'error': str(error)})
    result['all_passed'] = (
        result['complete'] and all(row.get('passed') for row in result['commands'])
        and not result['changed_executed_elf'] and not result['changed_source_files']
        and not result['residual_owned_mounts'] and not result['audit_errors']
        and result.get('archive_integrity', {}).get('unchanged') is True
        and result.get('snapshot_verification_after', {}).get('passed') is True
        and result.get('snapshot_verification_after', {}).get('same_as_before') is True
        and result.get('tls_worker_chain', {}).get('counts', {}).get('passed') == 10
        and result.get('component_handover', {}).get('passed') is True)
    persist()
    print(json.dumps({'summary': str(OUT / 'summary.json'),
                      'all_passed': result['all_passed']}), flush=True)
raise SystemExit(0 if result['all_passed'] else 1)