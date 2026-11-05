"""Authorized build phase only: preserve every preparation/build stream and exit."""
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

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()
def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()
def main():
    def interrupted(signum, frame):
        raise InterruptedError(f'build wrapper received signal {signum}')
    signal.signal(signal.SIGTERM, interrupted)
    p = argparse.ArgumentParser()
    p.add_argument('--source', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--target', type=Path, required=True, help='coordinator-owned independent target; never the active G4 target')
    p.add_argument('--candidate', required=True)
    p.add_argument('--host-space-helper', type=Path, required=True)
    p.add_argument('--reuse-completed-run', type=Path,
                   help='explicit coordinator-authorized sequential reuse of the same candidate completed G4 target')
    a = p.parse_args()
    assert os.name == 'posix', 'Linux build only after coordinator authorization'
    assert all(path.is_absolute() for path in (a.source, a.output, a.target))
    source, output, target = (path.resolve() for path in (a.source, a.output, a.target))
    reuse_proof = None
    if a.reuse_completed_run:
        previous = a.reuse_completed_run.resolve()
        proof_path = previous / 'summary.json'
        proof = json.loads(proof_path.read_text())
        # T4 verifier adaptation: all_commands_passed is False only because of the
        # two documented verifier-harness issues; require the explicit acceptance
        # record instead of that boolean.
        assert proof['complete']
        assert proof['candidate_commit'] == a.candidate
        assert source == previous / 'source' and target == previous / 'target' and target.is_dir()
        witness = previous / 'supplemental-001/executed-elf-before.json'
        assert witness.is_file(), 'preserve executed ELF identities before reusing completed target'
        acceptance_path = previous / 'supplemental-001/summary.json'
        acceptance = json.loads(acceptance_path.read_text())
        expected_failures = ['native-loopback', 'core-history-probe-prepare']
        assert acceptance['matrix_acceptance']['main_failed_commands'] == expected_failures, \
            acceptance['matrix_acceptance']['main_failed_commands']
        reuse_proof = {'run':str(previous),'summary_sha256':sha(proof_path),
                      'elf_witness_sha256':sha(witness),
                      'acceptance_record_sha256':sha(acceptance_path),
                      'accepted_main_failures':expected_failures,
                      'authorization':('T4 verifier: main matrix complete; all_commands_passed False only for the two '
                                       'documented verifier-harness issues; same-candidate private target reused for a '
                                       'sequential harness build, no parallel build and no cross-candidate sharing')}
    host_space = load_host_space(a.host_space_helper)
    assert not output.exists()
    assert not output.is_relative_to(source) and not target.is_relative_to(source)
    output.mkdir(parents=True)
    crate = output / 'native-worker-chain-crate'
    here = Path(__file__).resolve().parent
    env = os.environ.copy()
    for key in list(env):
        if key.startswith('DRIVE_NATIVE_') or key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS'):
            env.pop(key, None)
    env.update(CARGO_TARGET_DIR=str(target), CARGO_NET_OFFLINE='true', CARGO_BUILD_JOBS='2', CARGO_TERM_COLOR='never', PYTHONDONTWRITEBYTECODE='1')
    result = {'candidate_commit': a.candidate, 'source': str(source), 'target': str(target),
              'completed_same_candidate_target_reuse': reuse_proof,
              'crate': str(crate), 'started_utc': utc(), 'complete': False, 'passed': False,
              'source_lock_before': sha(source / 'Cargo.lock'), 'commands': [],
              'driver_files': {name: sha(here / name) for name in ('build.py', 'prepare.py', 'verify-lock.py', 'space.py', 'Cargo.toml.in', 'main.rs', 'codec.rs')}}
    summary = output / 'build.json'
    def persist():
        summary.write_text(json.dumps(result, indent=2) + '\n')
    def command(name, argv, timeout=3600):
        row = {'name': name, 'argv': argv, 'started_utc': utc(),
               'stdout': name + '.stdout.log', 'stderr': name + '.stderr.log'}
        row['host_space_before'] = host_space(10)
        result['commands'].append(row)
        persist()
        if not row['host_space_before']['passed']:
            row.update(passed=False, not_run=True, error='host C below 10 GiB or unknown; no command started')
            persist()
            return False
        start = time.monotonic()
        process = None
        with (output / row['stdout']).open('w') as stdout, (output / row['stderr']).open('w') as stderr:
            try:
                process = subprocess.Popen(argv, cwd=output, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
                row['process_group'] = process.pid
                row['exit_code'] = process.wait(timeout=timeout)
            except BaseException as error:
                row['exception_type'] = type(error).__name__
                row['error'] = str(error)
                if process is not None:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    except Exception as cleanup:
                        row['cleanup_error'] = f'{type(cleanup).__name__}: {cleanup}'
                    try:
                        process.wait(timeout=10)
                    except Exception as cleanup:
                        row['cleanup_wait_error'] = f'{type(cleanup).__name__}: {cleanup}'
        row.update(elapsed_seconds=time.monotonic() - start, finished_utc=utc(),
                   stdout_sha256=sha(output / row['stdout']), stderr_sha256=sha(output / row['stderr']))
        row['passed'] = row.get('exit_code') == 0 and 'exception_type' not in row
        persist()
        return row['passed']
    try:
        result['host_space_before_new_target'] = host_space(10 if reuse_proof else 40)
        persist()
        if not result['host_space_before_new_target']['passed']:
            result['error'] = 'host C below applicable 10 GiB existing-target / 40 GiB new-target gate or unknown; no preparation/build started'
            return 1
        if not command('prepare', [sys.executable, str(here / 'prepare.py'), '--source', str(source), '--output', str(crate), '--candidate', a.candidate], 60):
            return 1
        if not command('metadata', ['cargo', 'metadata', '--offline', '--format-version', '1', '--manifest-path', str(crate / 'Cargo.toml')], 180):
            return 1
        if not command('verify-lock', [sys.executable, str(here / 'verify-lock.py'), '--source', str(source), '--crate', str(crate), '--metadata', str(output / 'metadata.stdout.log')], 60):
            return 1
        if not command('build', ['cargo', 'build', '--offline', '--locked', '--manifest-path', str(crate / 'Cargo.toml'), '--message-format=json']):
            return 1
        artifacts = []
        for line in (output / 'build.stdout.log').read_text().splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if message.get('reason') == 'compiler-artifact' and message.get('target', {}).get('name') == 'native-worker-chain-probe' and message.get('executable'):
                artifacts.append(message['executable'])
        assert len(artifacts) == 1, f'harness executables: {artifacts}'
        result['binary'] = {'path': artifacts[0], 'sha256': sha(Path(artifacts[0]))}
        assert sha(source / 'Cargo.lock') == result['source_lock_before'], 'source lock changed during external build'
        result['passed'] = True
    except Exception as error:
        result['exception_type'] = type(error).__name__
        result['error'] = str(error)
        import traceback
        (output / 'failure.txt').write_text(traceback.format_exc())
        return 1
    finally:
        result.update(complete=True, finished_utc=utc(), source_lock_after=sha(source / 'Cargo.lock'))
        result['source_lock_unchanged'] = result['source_lock_before'] == result['source_lock_after']
        result['passed'] = result['passed'] and result['source_lock_unchanged']
        persist()
        print(json.dumps({'build': str(summary), 'passed': result['passed'], 'sha256': sha(summary)}))
    return 0 if result['passed'] else 1
if __name__ == '__main__':
    raise SystemExit(main())
