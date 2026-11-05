#!/usr/bin/env python3
"""Collect environment/provenance facts for the G4-014 evidence bundle."""
import hashlib
import importlib.util
import json
import os
import platform
import subprocess
from pathlib import Path

PROJECT = Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/'
               'P032-群晖Drive按需同步')
BUNDLE = PROJECT / 'work/issue7-validation/G4-evidence-bundle-014-final'
BASE = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
RUN = BASE / 'g4-014-rerun'
ABORTED_PARTIAL = BASE / 'g4-014-aborted-partial'
ABORT_EVIDENCE = BUNDLE / 'environment/attempt3-interrupted'
GIT_DIR = PROJECT / 'work/public-summary-20261031/.git'
WORKTREE = PROJECT / 'work/fix-7'
CANDIDATE = 'b303ac6bf01f6ff66f48d1e74cb94740561a841c'
ELVES = PROJECT / 'work/issue7-native-evidence/public-server'


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


g4 = load('frozen_g4', BUNDLE / 'harness/run-g4-014.py')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(argv, cwd=None):
    process = subprocess.run(argv, cwd=cwd, capture_output=True, text=True)
    return {'argv': argv, 'exit_code': process.returncode,
            'stdout': process.stdout.strip()[:4000],
            'stderr': process.stderr.strip()[:4000]}


def main():
    out = BUNDLE / 'environment'
    out.mkdir(parents=True, exist_ok=True)
    versions = {
        'collected_utc': g4.stamp(),
        'platform': platform.platform(),
        'os_release': Path('/etc/os-release').read_text().strip().splitlines(),
        'kernel': platform.release(),
        'nproc': os.cpu_count(),
        'guest_df_home': run(['df', '-h', '/home']),
        'host_c_space': g4.host_c_space(),
        'rustc': run(['rustc', '--version']),
        'cargo': run(['cargo', '--version']),
        'python3': run(['python3', '--version']),
        'gui_python': run(['/opt/drive-native-build/gui-venv/bin/python', '--version']),
        'gui_packages': run(['/opt/drive-native-build/gui-venv/bin/python', '-c',
                             'import importlib.metadata as m; print(m.version("PySide6_Essentials")); '
                             'print(m.version("pyinstaller"))']),
        'dbus_run_session': run(['dbus-run-session', '--', 'true']),
        'fusermount3': run(['fusermount3', '--version']),
        'cargo_home': os.environ.get('CARGO_HOME'),
        'rustup_home': os.environ.get('RUSTUP_HOME'),
        'candidate_commit': CANDIDATE,
    }
    (out / 'environment-014.json').write_text(json.dumps(versions, indent=2) + '\n')
    git = ['git', '-c', 'safe.directory=*', '--git-dir=' + str(GIT_DIR)]
    tree = run(git + ['ls-tree', '-r', CANDIDATE])
    repo = {
        'worktree': str(WORKTREE),
        'git_dir': str(GIT_DIR),
        'head': run(git + ['rev-parse', 'HEAD']),
        'candidate_commit_object': run(git + ['rev-parse', '--verify', CANDIDATE + '^{commit}']),
        'candidate_tree': run(git + ['rev-parse', '--verify', CANDIDATE + '^{tree}']),
        'candidate_tree_entries': len(tree['stdout'].splitlines()) if tree['exit_code'] == 0 else None,
        'status_porcelain': run(git + ['status', '--porcelain']),
        'log': run(git + ['log', '--oneline', '-5', CANDIDATE]),
        'candidate_archived_by': 'git archive from this object database (see prepared source.json)',
    }
    (out / 'author-tree-status-014.json').write_text(json.dumps(repo, indent=2) + '\n')
    inputs = []
    for name in ('syncd.elf', 'libsynosyncserver.so.elf',
                 'libsynosyncservercore.so.elf', 'libsynodrivecommon.so.7.0.elf'):
        path = ELVES / name
        inputs.append({'file': name, 'bytes': path.stat().st_size, 'sha256': sha(path)})
    (out / 'evidence-input-elf-sha.json').write_text(
        json.dumps({'directory': str(ELVES), 'files': inputs,
                    'note': 'local public SPK extraction; no NAS/credential access'},
                   indent=2) + '\n')
    attempts = []
    attempt1_dir = BUNDLE / 'environment/attempt1-missing-nautilus-prefix'
    attempts.append({
        'attempt': 1, 'status': 'aborted_before_product_commands',
        'cause': ('private Nautilus dependency prefix removed by external cleanup; run-g4.py '
                  'startup assertion failed before any product command'),
        'evidence': str(attempt1_dir),
        'launcher_log_sha256': sha(attempt1_dir / 'launcher.log'),
        'recorded_as_product_failure': False})
    attempt2_dir = BUNDLE / 'environment/attempt2-native-loopback-runner-name'
    summary2 = json.loads((attempt2_dir / 'summary.json').read_text())
    attempts.append({
        'attempt': 2, 'status': 'aborted_mid_matrix',
        'cause': ('run-g4.py calls the fixed script name run-native-loopback.sh; the launcher '
                  'referenced a differently named script, so native-loopback failed before the '
                  'remaining commands'),
        'complete': summary2.get('complete'),
        'commands_recorded': len(summary2.get('commands', [])),
        'failed_commands': [row['name'] for row in summary2.get('commands', [])
                            if row.get('passed') is False and not row.get('not_run')],
        'evidence': str(attempt2_dir),
        'recorded_as_product_failure': False})
    abort = json.loads((ABORT_EVIDENCE / 'abort-record.json').read_text())
    attempts.append({
        'attempt': 3, 'status': 'aborted_mid_matrix',
        'cause': 'interrupted by the user during workspace-default; partial run preserved',
        'complete': abort['complete'],
        'commands_recorded': abort['commands_recorded'],
        'failed_commands': abort['failed_commands'],
        'run_dir': abort['run_dir_preserved_in_wsl'],
        'evidence': str(ABORT_EVIDENCE),
        'recorded_as_product_failure': False})
    attempt4_dir = BUNDLE / 'environment/attempt4-root-invalid'
    abort4 = json.loads((attempt4_dir / 'abort-record.json').read_text())
    attempts.append({
        'attempt': 4, 'status': 'invalid_root_user_run',
        'cause': ('launched as root; permissions_kernel asserts getuid != 0, so the FUSE/kernel '
                  'group cannot pass under root; preserved as environment evidence only'),
        'commands_recorded': abort4['commands_recorded'],
        'failed_commands': abort4['failed_commands'],
        'evidence': str(attempt4_dir),
        'recorded_as_product_failure': False})
    (out / 'attempt-history-014.json').write_text(
        json.dumps({'candidate_commit': CANDIDATE, 'attempts': attempts,
                    'formal_run': str(RUN),
                    'formal_run_user': 'drive-native-test (non-root)',
                    'target_reuse': ('formal-run target seeded only from the attempt-3 target via the '
                                     'run-g4.py Cargo.lock registry filter (registry+ packages only); '
                                     'no workspace/path artifact, test ELF or incremental cache copied; '
                                     'CARGO_INCREMENTAL=0. Attempt-4 target was not reused.')},
                   indent=2) + '\n')
    staged = BASE / 'g4-014-snapshot/snapshot-verification-before.json'
    if staged.is_file():
        (out / 'snapshot-verification-before.json').write_bytes(staged.read_bytes())
    prep = PROJECT / 'work/issue7-validation/prepared-g4-014'
    for name in ('source.json', 'preparation-result.json'):
        if (prep / name).is_file():
            (out / ('prepared-' + name)).write_bytes((prep / name).read_bytes())
    print(json.dumps({'environment': str(out), 'candidate_tree': repo['candidate_tree']['stdout']}))


if __name__ == '__main__':
    main()