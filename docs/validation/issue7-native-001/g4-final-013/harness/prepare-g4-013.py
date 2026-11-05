#!/usr/bin/env python3
"""Prepare the immutable G4-013 source archive and an independent staging copy.

Steps: host-C 40 GiB gate -> exact `git archive <commit>` from the shared object
database -> staging extraction -> tree-vs-commit verification -> source.json
consumed by run-g4.py. The repository and all frozen products are read-only.
"""
import importlib.util
import json
import re
import sys
import tarfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
PROJECT = Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/'
               'P032-群晖Drive按需同步')
PREP = PROJECT / 'work/issue7-validation/prepared-g4-013'
WORKTREE = PROJECT / 'work/fix-7'
BASE = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
COMMIT = 'fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4'
STAGE = BASE / 'g4-013-snapshot'
RUN = BASE / 'g4-013'


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


g4 = load('frozen_g4', HERE / 'prepared-g4-013' / 'run-g4.py')
snapshot = load('snapshot_verify', HERE / 'snapshot-verify.py')


def windows_path_to_wsl(raw):
    match = re.match(r'^([A-Za-z]):[\\/](.*)$', raw)
    if not match:
        return Path(raw)
    rest = match.group(2).replace('\\', '/')
    return Path('/mnt/' + match.group(1).lower() + '/' + rest)


def resolve_git_dir(worktree):
    dotgit = worktree / '.git'
    if dotgit.is_dir():
        return dotgit
    text = dotgit.read_text().strip()
    assert text.startswith('gitdir:'), text
    gitdir = windows_path_to_wsl(text[len('gitdir:'):].strip())
    common = gitdir / 'commondir'
    if common.is_file():
        gitdir = (gitdir / common.read_text().strip()).resolve()
    return gitdir


def main():
    if STAGE.exists() or RUN.exists():
        raise SystemExit('stage/run directory already exists; refusing to overwrite')
    host = g4.host_c_space()
    report = {'candidate_commit': COMMIT, 'worktree': str(WORKTREE),
              'prepared_utc': g4.stamp(), 'host_space_before_new_run': host,
              'minimum_host_bytes': 40 * 1024 ** 3, 'stage': str(STAGE),
              'archive': str(PREP / 'candidate.tar'), 'complete': False}
    out = PREP / 'preparation-result.json'
    out.write_text(json.dumps(report, indent=2) + '\n')
    if host['free_bytes'] is None or host['free_bytes'] < 40 * 1024 ** 3:
        report['error'] = 'Host C below 40 GiB or unknown; no archive/extraction started'
        out.write_text(json.dumps(report, indent=2) + '\n')
        return 1
    git_dir = resolve_git_dir(WORKTREE)
    base = ['git', '-c', 'safe.directory=*', '--git-dir=' + str(git_dir)]
    head = g4.subprocess.run(base + ['rev-parse', '--verify', COMMIT + '^{commit}'],
                             capture_output=True, check=True).stdout.decode().strip()
    if head != COMMIT:
        raise SystemExit('resolved commit mismatch: ' + head)
    object_type = g4.subprocess.run(base + ['cat-file', '-t', COMMIT],
                                    capture_output=True, check=True).stdout.decode().strip()
    assert object_type == 'commit', object_type
    archive = PREP / 'candidate.tar'
    if archive.exists():
        raise SystemExit('candidate.tar already exists; refusing to overwrite')
    archive_command = base + ['archive', '--format=tar', '--output=' + str(archive), COMMIT]
    g4.subprocess.run(archive_command, check=True)
    archive_sha = g4.sha(archive)
    report.update(git_dir=str(git_dir), archive_command=archive_command,
                  candidate_archive_sha256=archive_sha, archive_bytes=archive.stat().st_size,
                  object_type=object_type)
    out.write_text(json.dumps(report, indent=2) + '\n')
    STAGE.mkdir(parents=True, mode=0o700)
    staged = STAGE / 'source'
    staged.mkdir()
    with tarfile.open(archive) as stream:
        stream.extractall(staged, filter='data')
    verification = snapshot.verify_tree(None, COMMIT, staged, git_dir=git_dir)
    (STAGE / 'snapshot-verification-before.json').write_text(
        json.dumps(verification, indent=2, ensure_ascii=False) + '\n')
    report['snapshot_verification_before'] = {
        'path': str(STAGE / 'snapshot-verification-before.json'),
        'passed': verification['passed'], 'entries': verification['entries'],
        'problem_count': verification['problem_count'],
        'tree_listing_sha256': verification['tree_listing_sha256'],
        'root_tree': verification['root_tree']}
    source_json = {
        'candidate_commit': COMMIT,
        'candidate_archive_sha256': archive_sha,
        'archive_command': archive_command,
        'authorization': ('T4 independent verifier: Lead-authorized G4 re-run on fixed commit '
                          'fefb8ee; local synthetic fixtures only; no NAS, credentials, push or release.'),
        'harness_sha256': g4.sha(HERE / 'prepared-g4-013' / 'run-g4.py'),
        'cache_policy': ('fresh empty private target; no author target and no product build '
                         'artifact copied; registry sources read from the shared offline '
                         'CARGO_HOME only'),
        'compile_cache_control': ('CARGO_INCREMENTAL=0; -j2; --locked --offline; profiles and '
                                  'assertions unchanged'),
        'host_space_gate': ('Host C >= 40 GiB before new run/target; >= 10 GiB before each '
                            'command; unknown stops'),
        'snapshot_verification': report['snapshot_verification_before'],
        'prepared_by': 'T4 independent verifier (verifier)',
        'git_version': g4.subprocess.run(['git', '--version'], capture_output=True,
                                         check=True).stdout.decode().strip(),
    }
    (PREP / 'source.json').write_text(json.dumps(source_json, indent=2, ensure_ascii=False) + '\n')
    report.update(complete=True, source_sha256=g4.sha(PREP / 'source.json'),
                  candidate_archive_sha256=archive_sha)
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'complete': True, 'archive_sha256': archive_sha,
                      'snapshot_passed': verification['passed'],
                      'entries': verification['entries'],
                      'tree_listing_sha256': verification['tree_listing_sha256'],
                      'source_json_sha256': g4.sha(PREP / 'source.json'),
                      'preparation_result_sha256': g4.sha(out)}))
    return 0 if verification['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())