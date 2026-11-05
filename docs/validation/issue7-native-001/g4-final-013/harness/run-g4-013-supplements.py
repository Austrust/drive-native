#!/usr/bin/env python3
"""Sequential G4-013 supplements after the main matrix.

Verifier-authored driver adapted from the frozen 012 supplement driver with the
same evidence discipline: host-C gates, original logs, exit codes, SHA256 of
every artifact, ELF/source/mount audits. Explicit differences (documented in
the output JSON):
- matrix acceptance allows exactly one known expectation drift: the frozen 012
  native-loopback harness asserts the pre-T1 classification for upload/mkdir
  0x9003. That is the intended T1 change; the frozen run is kept as evidence and
  the corrected verifier harness is run as the first supplement.
- the TLS worker-chain build uses a documented adaptation of the frozen 012
  build wrapper (harness/native-worker-chain-013/build.py + patch record): the
  main matrix all_commands_passed flag is False only because of two documented
  verifier-harness issues, so the reuse gate consumes the explicit acceptance
  record instead; host-space gates and every build step are unchanged.
- package regression is a preflight only (no deb build; packaging task owns it).
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
RUN = BASE / 'g4-013'
SOURCE = RUN / 'source'
TARGET = RUN / 'target'
CANDIDATE = 'fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4'
OLD = BASE / 'g4-011'
OLD_COMMIT = '41365a650eb7d7df696046e21b7671697231f851'
OLD_PROBE = OLD / 'native-worker-chain-target/debug/native-worker-chain-probe'
OLD_PROBE_SHA256 = '68c65939e18dcf2a1d377b4064fd79ac996c884df653a4bc577a4fa741de8ec6'
PREP = PROJECT / 'work/issue7-validation/prepared-g4-013'
FREEZE = PROJECT / 'work/issue7-validation/package-delivery-freeze.json'
DELIVERY = PROJECT / 'work/issue7-deb-001-artifacts'
# The frozen 012 native-loopback harness expected the old classification for
# these two cases; they are the intended T1 delta and are re-verified by the
# corrected harness below.
LOOPBACK_EXPECTED_DELTA = {'upload_reply_9003', 'mkdir_reply_9003'}


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


g4 = load('frozen_g4', HERE / 'prepared-g4-013' / 'run-g4.py')
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
frozen_loop = json.loads((RUN / 'native-loopback-results.json').read_text())
failed_cases = [case['name'] for case in frozen_loop['cases'] if case.get('passed') is not True]
observed = {case['name']: case.get('observed') for case in frozen_loop['cases']}
# Two main-matrix failures are verifier-harness issues, not product results:
# - native-loopback: frozen 012 expectations predate the intended T1 change;
# - core-history-probe-prepare: this bundle initially lacked the probe .rs file;
#   the group is re-run as a supplement with the file restored.
ALLOWED_MAIN_FAILURES = ['native-loopback', 'core-history-probe-prepare']
assert failed == ALLOWED_MAIN_FAILURES, f'unexpected failed commands: {failed}'
assert set(failed_cases) == LOOPBACK_EXPECTED_DELTA, f'unexpected loopback failures: {failed_cases}'
for name in sorted(LOOPBACK_EXPECTED_DELTA):
    kind = observed[name].get('kind')
    message = observed[name].get('message', '')
    assert kind == 'other' and 'mutation permission denial' in message, (name, observed[name])
assert frozen_loop['counts'] == {'passed': 64, 'failed': 2, 'ignored': 0, 'total': 66}, frozen_loop['counts']

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
          'scope': ('corrected native-loopback, 210k functional capacity, real worker TLS chain, '
                    'private component lifecycle, package-side preflight; no NAS/deb/performance claims'),
          'main_summary_sha256': sha(RUN / 'summary.json'),
          'harness_sha256': sha(pathlib.Path(__file__)),
          'matrix_acceptance': {
              'main_failed_commands': failed,
              'frozen_native_loopback_failed_cases': failed_cases,
              'frozen_counts': frozen_loop['counts'],
              'reason': ('frozen 012 harness asserts pre-T1 classification for upload/mkdir wire 9003; '
                         'fefb8ee intentionally returns the typed pre-commit refusal there'),
              'core_history_probe_prepare_failure': ('verifier bundle initially lacked core-history-probe.rs; '
                                                     'the group is re-run as a supplement'),
              'corrected_harness': str(HERE / 'native-loopback-013' / 'main.rs'),
              'corrected_patch_record': str(HERE / 'native-loopback-013-patch.json')},
          'target': str(TARGET),
          'target_provenance': ('fresh empty private target created by run-g4.py for this candidate; '
                                'no author target or product artifact copied')}


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
               'excluded': ('target/debug/native-loopback-probe is rebuilt by the corrected '
                            'verifier harness copy; it is not a product binary'),
               'files': records}
    save(OUT / 'executed-elf-before.json', witness)
    return witness


witness = None
persist()
try:
    witness = executed_elf_witness()
    persist()
    # 1) corrected native-loopback matrix (only the two intended expectation changes)
    command('native-loopback-updated',
            ['bash', str(HERE / 'run-native-loopback-013.sh'), str(RUN), CANDIDATE],
            'json', timeout=1200,
            result_path=RUN / 'native-loopback-updated-001' / 'results.json')
    updated = json.loads((RUN / 'native-loopback-updated-001' / 'results.json').read_text())
    assert updated['all_passed'] is True and updated['counts']['total'] == 70, updated['counts']
    old_by_name = {case['name']: case for case in frozen_loop['cases']}
    new_by_name = {case['name']: case for case in updated['cases']}
    added = sorted(set(new_by_name) - set(old_by_name))
    expected_added = ['download_reply_9003', 'download_reply_a004',
                      't1_setup_Auth_9003_is_generic', 't1_setup_Connect_9003_is_generic']
    assert added == expected_added, added
    assert all(new_by_name[name]['passed'] is True for name in added), added
    assert not (set(old_by_name) - set(new_by_name)), '012 cases must all remain'
    changed = sorted(name for name in old_by_name
                     if old_by_name[name].get('observed') != new_by_name[name].get('observed'))
    assert changed == sorted(LOOPBACK_EXPECTED_DELTA), changed
    changed_expected = sorted(name for name in old_by_name
                              if old_by_name[name].get('expected') != new_by_name[name].get('expected'))
    assert changed_expected == sorted(LOOPBACK_EXPECTED_DELTA), changed_expected
    result['corrected_loopback'] = {
        'counts': updated['counts'],
        'changed_observed_cases': changed,
        'changed_expected_cases': changed_expected,
        'adversarial_added_cases': added,
        'unchanged_012_cases': len(old_by_name) - len(changed),
        'scope': ('70 cases = 66 frozen cases + 4 adversarial additions: download 0x9003/0xa004 '
                  'and auth/connect 0x9003 must stay generic server errors')}
    persist()
    # 2) T1 write-learning targeted tests, counted separately from the workspace matrix
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
    result['write_learning_targeted'] = []
    for name, argv in targeted:
        passed = command('t1-' + name, argv, 'rust', timeout=1800)
        result['write_learning_targeted'].append(
            {'name': name, 'passed': passed,
             'counts': result['commands'][-1]['counts']})
    persist()
    # 2b) core-history probe group re-run (the main-matrix prepare failed only because
    # this bundle initially missed core-history-probe.rs; no product test ran there)
    core_ok = command('core-history-probe-prepare',
                      ['python3', '-B', str(HERE / 'prepare-core-probe.py'),
                       '--source', str(SOURCE),
                       '--output', str(RUN / 'core-history-probe-001')],
                      'prepare', timeout=300)
    result['core_history_probe_repaired'] = {
        'main_matrix_prepare_failed': True,
        'main_matrix_failure_reason': 'verifier bundle initially missed core-history-probe.rs',
        'prepare_passed': core_ok}
    if core_ok:
        command('core-history-probe',
                ['cargo', 'test', '--offline', '--locked',
                 '--manifest-path', str(RUN / 'core-history-probe-001/Cargo.toml'),
                 '--', '--test-threads=1'], 'rust', timeout=1800)
        result['core_history_probe_repaired']['counts'] = result['commands'][-1]['counts']
    persist()
    # 2c) standard compiler check (all targets, all features, locked/offline)
    command('standard-check-all-targets',
            ['cargo', 'check', '--workspace', '--all-targets', '--all-features',
             '--locked', '--offline', '-j2'], 'prepare', timeout=3600)
    # 3) 210k functional capacity (ignored test, explicit name)
    cargo = ['cargo', 'test', '--offline', '--locked', '-p', 'drive-core', '--lib', '--all-features']
    assert command('capacity-ignored-list', cargo + ['--', '--list', '--ignored'])
    text = (OUT / 'logs/capacity-ignored-list.log').read_text()
    names = re.findall(r'^(\S*default_budget_admits_210k_node_snapshot_without_holding_store): test$', text, re.M)
    assert len(names) == 1, names
    command('capacity-210k', cargo + [names[0], '--', '--ignored', '--exact', '--test-threads=1'], 'rust')
    # 3) real worker TLS -> IPC -> Bridge chain
    chain = HERE / 'native-worker-chain-013'
    built = command('tls-harness-build',
                    ['python3', '-B', str(chain / 'build.py'), '--source', str(SOURCE),
                     '--output', str(RUN / 'native-worker-chain-build-001'),
                     '--target', str(TARGET), '--candidate', CANDIDATE,
                     '--reuse-completed-run', str(RUN),
                     '--host-space-helper', str(HERE / 'prepared-g4-013' / 'run-g4.py')],
                    timeout=4200)
    result['tls_harness_build_adapter'] = {
        'harness': str(chain),
        'patch_record': str(HERE / 'native-worker-chain-013-patch.json')}
    persist()
    if built:
        build = json.loads((RUN / 'native-worker-chain-build-001/build.json').read_text())
        assert build['passed']
        command('tls-actual-worker-all-ten',
                ['python3', '-B', str(chain / 'run.py'), '--binary', build['binary']['path'],
                 '--worker', str(TARGET / 'debug/drive-native-worker'), '--crate', build['crate'],
                 '--candidate', CANDIDATE, '--output', str(RUN / 'native-worker-chain-001'),
                 '--host-space-helper', str(HERE / 'prepared-g4-013' / 'run-g4.py')],
                timeout=900)
        tls = json.loads((RUN / 'native-worker-chain-001/results.json').read_text())
        assert tls['counts'] == {'passed': 10, 'failed': 0, 'ignored': 0, 'total': 10}, tls['counts']
        result['tls_worker_chain'] = {'counts': tls['counts'],
                                      'harness_build_sha256': sha(RUN / 'native-worker-chain-build-001/build.json')}
        persist()
    # 4) private component pair replacement (old build is a preserved fixture of the
    #    previous candidate; the new pair is built from this snapshot in this target)
    assert sha(OLD_PROBE) == OLD_PROBE_SHA256, 'preserved old probe identity changed'
    result['old_candidate_fixture'] = {
        'candidate': OLD_COMMIT,
        'probe': {'path': str(OLD_PROBE), 'sha256': sha(OLD_PROBE)},
        'binaries': {name: sha(OLD / 'native-worker-chain-target/debug' / name)
                     for name in ('drive-native-fsd', 'drive-native-worker', 'drive-native-setup')},
        'scope': 'legacy side of the paired component replacement; not an acceptance test of the new candidate'}
    persist()
    binaries = ['cargo', 'build', '--offline', '--locked', '-p', 'drive-runtime',
                '--all-features', '--bins', '--example', 'multi_mount_fixture']
    old_built = command('old-component-build', binaries,
                        custom_env={'CARGO_TARGET_DIR': str(OLD / 'native-worker-chain-target')},
                        cwd=OLD / 'source')
    new_built = command('new-component-build', binaries)
    assert sha(OLD_PROBE) == OLD_PROBE_SHA256, 'preserved old probe changed'
    if old_built and new_built:
        command('private-component-policy-handover',
                ['dbus-run-session', '--', '/usr/bin/python3', '-B',
                 str(HERE / 'component-policy-handover.py'),
                 '--root', str(RUN / 'component-policy-handover-001'),
                 '--fixture', str(TARGET / 'debug/examples/multi_mount_fixture'),
                 '--old-binaries', str(OLD / 'native-worker-chain-target/debug'),
                 '--new-binaries', str(TARGET / 'debug'),
                 '--old-commit', OLD_COMMIT, '--new-commit', CANDIDATE], timeout=600)
        handover = json.loads((RUN / 'component-policy-handover-001/result.json').read_text())
        assert handover['passed'] is True
        result['component_handover'] = {'passed': True, 'cases': len(handover.get('cases', [])),
                                        'result_sha256': sha(RUN / 'component-policy-handover-001/result.json'),
                                        'scope': handover.get('scope')}
        persist()
    # 5) package-side regression preflight (no deb build; packaging task owns it)
    command('package-version', ['dpkg-parsechangelog', '-S', 'Version'], 'prepare')
    command('package-documentation-staging',
            ['python3', 'scripts/stage-docs.py', str(RUN / 'package-documentation-preflight')],
            'prepare', custom_env={'DRIVE_NATIVE_SOURCE_COMMIT': CANDIDATE,
                                   'DRIVE_NATIVE_OFFLINE_DOCS': '1',
                                   'DEB_BUILD_OPTIONS': 'parallel=2'})
    command('package-metadata-equivalence',
            ['python3', '-B', str(HERE / 'verify-packaging-013.py'),
             '--git-dir', str(git_dir), '--from-commit',
             '9c28992eb67a5620bf83b4939dc262dfe239027f', '--to-commit', CANDIDATE,
             '--freeze', str(FREEZE), '--delivery', str(DELIVERY),
             '--output', str(RUN / 'package-metadata-equivalence.json')],
            'json', result_path=RUN / 'package-metadata-equivalence.json')
    packaging = json.loads((RUN / 'package-metadata-equivalence.json').read_text())
    result['package_preflight'] = {
        'version_log': str(OUT / 'logs/package-version.log'),
        'changelog_version_output': (OUT / 'logs/package-version.log').read_text().strip(),
        'packaging_delta_empty': packaging['checks']['packaging_delta_empty'],
        'frozen_delivery_untouched': packaging['passed'],
        'full_deb_build': 'not_run (owned by the packaging task after the final commit)',
        'result_sha256': sha(RUN / 'package-metadata-equivalence.json')}
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
        archive = PREP / 'candidate.tar'
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
        and result.get('snapshot_verification_after', {}).get('same_as_before') is True)
    persist()
    print(json.dumps({'summary': str(OUT / 'summary.json'),
                      'all_passed': result['all_passed']}), flush=True)
raise SystemExit(0 if result['all_passed'] else 1)
