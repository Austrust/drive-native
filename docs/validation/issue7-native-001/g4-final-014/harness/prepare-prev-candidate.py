#!/usr/bin/env python3
"""Prepare an immutable snapshot of the previous-candidate fixture (41365a6)
for the component-pair replacement supplement. Read-only repository access."""
import importlib.util
import json
import tarfile
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
PROJECT = Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/'
               'P032-群晖Drive按需同步')
BASE = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
OLD = BASE / 'g4-014-old-candidate-snapshot'
GIT_DIR = PROJECT / 'work/public-summary-20261031/.git'
COMMIT = '41365a650eb7d7df696046e21b7671697231f851'


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


g4 = load('frozen_g4', HERE / 'run-g4-014.py')
snapshot = load('snapshot_verify', HERE / 'snapshot-verify.py')
PREV = PROJECT / 'work/issue7-validation/prepared-g4-014-prev'


def main():
    assert not OLD.exists(), 'old-candidate staging already exists'
    assert not PREV.exists(), 'old-candidate prepared dir already exists'
    PREV.mkdir(parents=True)
    host = g4.host_c_space()
    report = {'candidate_commit': COMMIT, 'prepared_utc': g4.stamp(),
              'host_space_before': host, 'complete': False}
    out = PREV / 'preparation-result.json'
    out.write_text(json.dumps(report, indent=2) + '\n')
    if host['free_bytes'] is None or host['free_bytes'] < 40 * 1024 ** 3:
        raise SystemExit('host gate rejected old-candidate preparation')
    archive = PREV / 'candidate.tar'
    command = ['git', '-c', 'safe.directory=*', '--git-dir=' + str(GIT_DIR),
               'archive', '--format=tar', '--output=' + str(archive), COMMIT]
    g4.subprocess.run(command, check=True)
    report['archive_command'] = command
    report['candidate_archive_sha256'] = g4.sha(archive)
    OLD.mkdir(parents=True, mode=0o700)
    staged = OLD / 'source'
    staged.mkdir()
    with tarfile.open(archive) as stream:
        stream.extractall(staged, filter='data')
    verification = snapshot.verify_tree(None, COMMIT, staged, git_dir=GIT_DIR)
    report['snapshot_verification'] = {'passed': verification['passed'],
                                       'entries': verification['entries'],
                                       'problem_count': verification['problem_count'],
                                       'root_tree': verification['root_tree'],
                                       'tree_listing_sha256': verification['tree_listing_sha256']}
    report['complete'] = True
    out.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'complete': True, 'entries': verification['entries'],
                      'archive_sha256': report['candidate_archive_sha256'],
                      'snapshot_passed': verification['passed']}))
    return 0 if verification['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())