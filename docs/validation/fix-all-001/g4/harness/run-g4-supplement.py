"""Run explicit local functional-load, tray and full-window screenshot checks."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import time

here = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('g4_helpers', here / 'run-g4.py')
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--candidate-commit', required=True)
    parser.add_argument('--g4-authorized', action='store_true')
    args = parser.parse_args()
    run = args.run.resolve()
    assert args.g4_authorized and run.is_relative_to(helper.BASE)
    original = json.loads((run / 'summary.json').read_text())
    assert original['complete'] and original['candidate_commit'] == args.candidate_commit
    supplemental = run / 'supplement'
    supplemental.mkdir(mode=0o700)
    source = run / 'source'
    before = {str(p.relative_to(source)): helper.sha(p) for p in source.rglob('*') if p.is_file()}
    env = os.environ.copy()
    for key in list(env):
        if key.startswith('DRIVE_NATIVE_') or key in {'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'DBUS_SESSION_BUS_ADDRESS', 'DISPLAY', 'WAYLAND_DISPLAY'}:
            env.pop(key, None)
    env.update(CARGO_TARGET_DIR=str(run / 'target'), CARGO_NET_OFFLINE='true', CARGO_BUILD_JOBS='2',
               TMPDIR=str(run / 'tmp'), PYTHONPATH=str(source / 'desktop'),
               PYTHONDONTWRITEBYTECODE='1', QT_QPA_PLATFORM='offscreen', RUST_BACKTRACE='1',
               DRIVE_NATIVE_FS_INDEX='1', DRIVE_NATIVE_FS_TTL_MS='1000')
    for variable, directory in [('XDG_CONFIG_HOME', 'config'), ('XDG_DATA_HOME', 'data'), ('XDG_CACHE_HOME', 'cache'), ('XDG_RUNTIME_DIR', 'runtime')]:
        path = supplemental / directory
        path.mkdir(mode=0o700)
        env[variable] = str(path)
    summary = {'candidate_commit': args.candidate_commit, 'complete': False,
               'scope': 'local functional validation; never G5 performance evidence',
               'harness_sha256': helper.sha(Path(__file__)), 'commands': []}
    commands = [
        ('core-210k-functional', ['cargo', 'test', '--offline', '--locked', '-p', 'drive-core', '--lib', '--all-features',
                                'status_snapshot::tests::default_budget_admits_210k_node_snapshot_without_holding_store',
                                '--', '--exact', '--ignored', '--test-threads=1'], 'rust'),
        ('tray-synthetic', ['dbus-run-session', '--', str(helper.GUI_PYTHON), str(here / 'verify-tray-supplement.py'), str(source), '-v'], 'gui'),
        ('mainwindow-permission-render', ['dbus-run-session', '--', str(helper.GUI_PYTHON),
                                         str(here / 'render-mainwindow-permissions.py'), str(supplemental / 'mainwindow-screenshots')], 'render'),
    ]
    try:
        for name, command, kind in commands:
            row = {'name': name, 'command': command, 'kind': kind, 'start': helper.stamp()}
            summary['commands'].append(row)
            helper.save(supplemental / 'summary.json', summary)
            log = supplemental / (name + '.log')
            started = time.monotonic()
            with log.open('w') as output:
                process = subprocess.Popen(command, cwd=source, env=env, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
                try:
                    process.wait(timeout=3600 if kind == 'rust' else 300)
                except subprocess.TimeoutExpired:
                    row['timeout'] = True
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait(timeout=15)
            row.update(exit_code=process.returncode, end=helper.stamp(), elapsed_seconds=time.monotonic()-started,
                       log=log.name, log_sha256=helper.sha(log), counts=helper.counts(log.read_text(errors='replace'), kind))
            row['passed'] = process.returncode == 0 and not row.get('timeout')
            helper.save(supplemental / 'summary.json', summary)
            print(json.dumps(row), flush=True)
        summary['complete'] = True
    finally:
        summary['changed_source_files'] = [name for name, value in before.items() if not (source / name).is_file() or helper.sha(source / name) != value]
        summary['new_source_files'] = [str(p.relative_to(source)) for p in source.rglob('*') if p.is_file() and str(p.relative_to(source)) not in before]
        summary['residual_owned_mounts'] = [str(p) for p in helper.mounts_below(run)]
        summary['all_passed'] = summary['complete'] and all(row.get('passed') for row in summary['commands']) and not summary['changed_source_files'] and not summary['residual_owned_mounts']
        helper.save(supplemental / 'summary.json', summary)


if __name__ == '__main__':
    main()
