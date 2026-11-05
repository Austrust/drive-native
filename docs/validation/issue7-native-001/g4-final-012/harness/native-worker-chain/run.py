"""After explicit execution authorization: bound every case in its own process group."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
sys.dont_write_bytecode = True
from space import load_host_space

CASES = [
    'bridge_download_denied_3002', 'bridge_download_denied_3008',
    'bridge_profile_false_empty', 'bridge_profile_true_empty', 'bridge_profile_true_other_view',
    'bridge_late_read_after_profile',
    'worker_rpc_generic_3004', 'worker_rpc_generic_8004', 'worker_rpc_generic_9005', 'worker_rpc_generic_a005',
]
def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()
def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()
def group_exists(pid):
    try:
        os.killpg(pid, 0)
        return True
    except ProcessLookupError:
        return False
def terminate_group(process):
    # Only the new session created for this invocation is eligible for cleanup.
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        return False
    deadline = time.monotonic() + 2
    while group_exists(process.pid) and time.monotonic() < deadline:
        time.sleep(.02)
    return not group_exists(process.pid)
def main():
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--worker', type=Path, required=True)
    p.add_argument('--crate', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--candidate', required=True)
    p.add_argument('--host-space-helper', type=Path, required=True)
    p.add_argument('--case', choices=CASES, action='append')
    a = p.parse_args()
    selected = a.case or CASES
    assert len(selected) == len(set(selected)), 'duplicate case arguments are not permitted'
    host_space = load_host_space(a.host_space_helper)
    assert os.name == 'posix' and os.geteuid() != 0, 'ordinary Linux validation user required'
    for path in (a.binary, a.worker, a.crate, a.output):
        assert path.is_absolute(), 'absolute isolated paths required'
    binary, worker, crate, output = (path.resolve() for path in (a.binary, a.worker, a.crate, a.output))
    assert not output.exists(), 'preserve previous evidence'
    assert not output.is_relative_to(crate), 'run outside prepared source'
    provenance = json.loads((crate / 'provenance.json').read_text())
    assert provenance['candidate_commit'] == a.candidate
    assert not output.is_relative_to(Path(provenance['frozen_source']).resolve())
    for name, digest in provenance['files'].items():
        if name != 'Cargo.lock':
            assert sha(crate / name) == digest, f'prepared file changed: {name}'
    lock = json.loads((crate / 'lock-verification.json').read_text())
    assert lock['all_registry_dependencies_match_frozen_candidate']
    assert sha(crate / 'Cargo.lock') == lock['harness_cargo_lock_sha256']
    for path in (binary, worker):
        with path.open('rb') as stream:
            assert stream.read(4) == b'\x7fELF', 'real Linux executable required'
    output.mkdir(parents=True)
    result = {'candidate_commit': a.candidate, 'started_utc': utc(), 'complete': False,
              'worker': {'path': str(worker), 'sha256': sha(worker)},
              'harness': {'path': str(binary), 'sha256': sha(binary), 'crate': str(crate),
                          'source_files': {name: sha(crate / name) for name in ('src/main.rs', 'src/codec.rs', 'Cargo.toml', 'Cargo.lock', 'provenance.json', 'lock-verification.json')}},
              'certificate_fixture_hashes': {path.name: sha(path) for path in sorted((crate / 'fixtures').iterdir()) if path.is_file()},
              'cases': [], 'scope': 'isolated synthetic native TLS service; no relay, NAS, real credentials, FUSE or GUI',
              'case_timeout_seconds': 45, 'cleanup_timeout_seconds': 12}
    destination = output / 'results.json'
    def persist():
        destination.write_text(json.dumps(result, indent=2) + '\n')
    persist()
    for index, name in enumerate(selected):
        observation = host_space(10)
        if not observation['passed']:
            result['stopped_for_host_space'] = observation
            for pending in selected[index:]:
                result['cases'].append({'name': pending, 'passed': False, 'status': 'blocked', 'not_run': True,
                                       'reason': 'host C below 10 GiB or unknown; stop before any further case',
                                       'host_space_before': observation})
            persist()
            break
        directory = output / name
        directory.mkdir()
        (directory / 'tmp').mkdir()
        env = os.environ.copy()
        for key in list(env):
            if key.startswith('DRIVE_NATIVE_') or key in ('DBUS_SESSION_BUS_ADDRESS', 'DISPLAY', 'WAYLAND_DISPLAY'):
                env.pop(key, None)
        env.update(TMPDIR=str(directory / 'tmp'), RUST_BACKTRACE='1')
        command = [str(binary), name, str(worker), str(crate / 'fixtures'), str(directory)]
        row = {'name': name, 'command': command, 'started_utc': utc(), 'passed': False, 'status': 'running', 'log': str(Path(name) / 'output.log'),
               'host_space_before': observation}
        result['cases'].append(row)
        persist()
        process = None
        started = time.monotonic()
        try:
            with (directory / 'output.log').open('w') as log:
                process = subprocess.Popen(command, cwd=directory, env=env, stdout=log,
                                           stderr=subprocess.STDOUT, start_new_session=True)
                row['process_group'] = process.pid
                persist()
                try:
                    row['exit_code'] = process.wait(timeout=45)
                except subprocess.TimeoutExpired:
                    row['status'] = 'timeout'
                    row['error'] = 'case exceeded 45 seconds; own process group killed'
                child_result = directory / 'case.json'
                if child_result.exists():
                    observed = json.loads(child_result.read_text())
                    assert observed['name'] == name
                    row['observed'] = observed
                    if row['status'] != 'timeout':
                        row['status'] = observed['status']
                        row['passed'] = observed['passed'] and row.get('exit_code') == 0
                elif row['status'] != 'timeout':
                    row['status'] = 'failed'
                    row['error'] = 'no preserved case.json'
        except Exception as error:
            row.update(status='failed', error=f'{type(error).__name__}: {error}', passed=False)
        finally:
            # Even an apparently successful child must leave no live worker in
            # its process group. A leftover forces failure; only own group killed.
            cleanup_errors = []
            try:
                leftover = process is not None and group_exists(process.pid)
            except Exception as error:
                leftover = True
                cleanup_errors.append(f'group inspection: {type(error).__name__}: {error}')
            try:
                cleaned = True if process is None else terminate_group(process)
            except Exception as error:
                cleaned = False
                cleanup_errors.append(f'group cleanup: {type(error).__name__}: {error}')
            row['process_group_cleanup'] = {'leftover_before_cleanup': leftover, 'absent_after_cleanup': cleaned}
            if leftover or not cleaned or cleanup_errors:
                row['passed'] = False
                if row['status'] != 'timeout':
                    row['status'] = 'failed'
            row['elapsed_seconds'] = time.monotonic() - started
            row['finished_utc'] = utc()
            try:
                if (directory / 'output.log').exists():
                    row['log_sha256'] = sha(directory / 'output.log')
                if (directory / 'case.json').exists():
                    row['case_json_sha256'] = sha(directory / 'case.json')
            except Exception as error:
                row['passed'] = False
                row['status'] = 'failed'
                cleanup_errors.append(f'evidence hash: {type(error).__name__}: {error}')
            row['cleanup_errors'] = cleanup_errors
            try:
                persist()
            except Exception as error:
                row['passed'] = False
                row['status'] = 'failed'
                row['evidence_write_error'] = f'{type(error).__name__}: {error}'
                print(json.dumps(row), file=sys.stderr, flush=True)
    result.update(complete='stopped_for_host_space' not in result, finished_utc=utc())
    result['unchanged_executables'] = sha(worker) == result['worker']['sha256'] and sha(binary) == result['harness']['sha256']
    result['counts'] = {'passed': sum(row['passed'] for row in result['cases']),
                        'failed': sum(not row['passed'] and row['status'] != 'blocked' for row in result['cases']),
                        'ignored': sum(row['status'] == 'blocked' for row in result['cases']), 'total': len(result['cases'])}
    result['all_passed'] = all(row['passed'] for row in result['cases']) and result['unchanged_executables']
    persist()
    print(json.dumps({'result': str(destination), 'sha256': sha(destination), 'counts': result['counts'], 'all_passed': result['all_passed']}))
    return 0 if result['all_passed'] else 1
if __name__ == '__main__':
    raise SystemExit(main())
