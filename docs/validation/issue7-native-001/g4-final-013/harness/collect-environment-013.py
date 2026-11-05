#!/usr/bin/env python3
"""Collect environment/provenance facts for the G4-013 evidence bundle."""
import hashlib
import importlib.util
import json
import os
import platform
import subprocess
from pathlib import Path

PROJECT = Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/'
               'P032-群晖Drive按需同步')
BUNDLE = PROJECT / 'work/issue7-validation/G4-evidence-bundle-013-final'
BASE = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
RUN = BASE / 'g4-013'
ATTEMPT1 = BASE / 'g4-013-attempt1-harness-path-failure'
GIT_DIR = PROJECT / 'work/public-summary-20261031/.git'
WORKTREE = PROJECT / 'work/fix-7'
CANDIDATE = 'fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4'
ELVES = PROJECT / 'work/issue7-native-evidence/public-server'


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


g4 = load('frozen_g4', BUNDLE / 'harness/prepared-g4-013/run-g4.py')


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
    (out / 'environment-013.json').write_text(json.dumps(versions, indent=2) + '\n')
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
    (out / 'author-tree-status-013.json').write_text(json.dumps(repo, indent=2) + '\n')
    inputs = []
    for name in ('syncd.elf', 'libsynosyncserver.so.elf',
                 'libsynosyncservercore.so.elf', 'libsynodrivecommon.so.7.0.elf'):
        path = ELVES / name
        inputs.append({'file': name, 'bytes': path.stat().st_size, 'sha256': sha(path)})
    (out / 'evidence-input-elf-sha.json').write_text(
        json.dumps({'directory': str(ELVES), 'files': inputs,
                    'note': 'local public SPK extraction; no NAS/credential access'},
                   indent=2) + '\n')
    attempt1 = {}
    if (ATTEMPT1 / 'summary.json').is_file():
        summary = json.loads((ATTEMPT1 / 'summary.json').read_text())
        attempt1 = {
            'path': str(ATTEMPT1),
            'candidate_commit': summary.get('candidate_commit'),
            'complete': summary.get('complete'),
            'all_commands_passed': summary.get('all_commands_passed'),
            'commands': [{'name': row['name'], 'passed': row.get('passed'),
                          'counts': row.get('counts')}
                         for row in summary.get('commands', [])],
            'failure_tail': (ATTEMPT1 / 'failure.txt').read_text()[-1500:]
            if (ATTEMPT1 / 'failure.txt').is_file() else None,
            'cause': ('verifier harness path bug: run-g4.py was launched from prepared-g4-013/ '
                      'where its helper scripts are absent; not a product failure'),
        }
    (out / 'attempt1-harness-path-failure.json').write_text(
        json.dumps(attempt1, indent=2) + '\n')
    staged = BASE / 'g4-013-snapshot/snapshot-verification-before.json'
    if staged.is_file():
        (out / 'snapshot-verification-before.json').write_bytes(staged.read_bytes())
    prep = PROJECT / 'work/issue7-validation/prepared-g4-013'
    for name in ('source.json', 'preparation-result.json'):
        if (prep / name).is_file():
            (out / ('prepared-' + name)).write_bytes((prep / name).read_bytes())
    print(json.dumps({'environment': str(out), 'candidate_tree': repo['candidate_tree']['stdout']}))


if __name__ == '__main__':
    main()