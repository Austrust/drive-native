#!/usr/bin/env python3
"""Independent checks on the T2 (L2 source) conclusion package.

Verifies only evidence integrity and non-modification claims; the argument
review is in review/T2-REVIEW.md. The T2 report directory is UNTRACKED in the
fefb8ee commit (recording that fact is part of the check).
"""
import hashlib
import json
import subprocess
from pathlib import Path

PROJECT = Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步')
WORKTREE = PROJECT / 'work/fix-7'
EVIDENCE = PROJECT / 'work/issue7-native-evidence/l2-source-002'
BUNDLE = Path(__file__).resolve().parent.parent
GIT_DIR = PROJECT / 'work/public-summary-20261031/.git'
T2_REPORT = WORKTREE / 'docs/validation/issue7-native-001/l2-source-002/REPORT.md'
L2_STATIC = WORKTREE / 'docs/validation/issue7-native-001/l2-static'
SERVER_STATIC = WORKTREE / 'docs/validation/issue7-native-001/server-static'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path):
    return json.loads(Path(path).read_text())


def git(*args):
    process = subprocess.run(['git', '-c', 'safe.directory=*', '--git-dir=' + str(GIT_DIR)]
                             + list(args), capture_output=True, text=True)
    return process


def main():
    report = {'scope': 'T2 l2-source-002 evidence integrity + non-modification checks',
              't2_report': {'path': str(T2_REPORT), 'sha256': sha(T2_REPORT),
                            'bytes': T2_REPORT.stat().st_size},
              'evidence_root': str(EVIDENCE)}
    manifest = load(EVIDENCE / 'MANIFEST.json')
    sums_lines = [line.split(None, 1) for line in
                  (EVIDENCE / 'SHA256SUMS.txt').read_text().splitlines() if line.strip()]
    sums = {name.strip(): digest for digest, name in sums_lines}
    files = {str(path.relative_to(EVIDENCE)) for path in EVIDENCE.rglob('*') if path.is_file()}
    sums_mismatch = [name for name, digest in sums.items()
                     if not (EVIDENCE / name).is_file() or sha(EVIDENCE / name) != digest]
    listed = {entry['file'] for group in manifest.get('evidence', {}).values()
              if isinstance(group, dict) for entry in group.get('functions', [])
              if isinstance(entry, dict) and 'file' in entry}
    listed |= {group['window']['file'] for group in manifest.get('evidence', {}).values()
               if isinstance(group, dict) and isinstance(group.get('window'), dict)
               and 'file' in group['window']}
    listed_mismatch = [name for name in listed if not (EVIDENCE / name).is_file()]
    extras_outside_sums = sorted(files - set(sums) - {'MANIFEST.json', 'SHA256SUMS.txt'})
    report['evidence_integrity'] = {
        'manifest_sha256': sha(EVIDENCE / 'MANIFEST.json'),
        'sha256sums_entries': len(sums), 'files_on_disk': len(files),
        'sha256sums_mismatches': sums_mismatch,
        'manifest_listed_files_missing': listed_mismatch,
        'files_not_in_sha256sums': extras_outside_sums,
        'passed': not sums_mismatch and not listed_mismatch,
    }
    # Input provenance must be the same frozen inputs as the previous round.
    input_files = ['download-manifest.json', 'libraries-manifest.json',
                   'selected-members-manifest.json']
    input_compare = []
    for name in input_files:
        current = EVIDENCE / 'inputs' / name
        frozen_candidates = [SERVER_STATIC / name, L2_STATIC / 'inputs' / name]
        frozen = next((path for path in frozen_candidates if path.is_file()), None)
        input_compare.append({
            'file': name, 'current_sha256': sha(current),
            'frozen_path': str(frozen) if frozen else None,
            'frozen_sha256': sha(frozen) if frozen else None,
            'match': bool(frozen) and sha(current) == sha(frozen)})
    report['inputs'] = input_compare
    report['inputs_passed'] = all(row['match'] for row in input_compare)
    # Declared binary SHA256 values vs frozen server-static manifests.
    declared = {row['member']: row['sha256'] for row in manifest['inputs']['members']}
    server_download = load(SERVER_STATIC / 'download-manifest.json')
    frozen_spk_sha = server_download.get('sha256') or server_download.get('package', {}).get('sha256')
    members_frozen = {}
    for path in (SERVER_STATIC / 'selected-members-manifest.json',
                 SERVER_STATIC / 'libraries-manifest.json'):
        data = load(path)
        rows = data if isinstance(data, list) else (data.get('members') or data.get('files') or [])
        for row in rows:
            name = row.get('member') or row.get('path') or row.get('name')
            if name and row.get('sha256'):
                members_frozen.setdefault(name, row['sha256'])
    binary_compare = [{'member': member, 'declared': digest,
                       'frozen': members_frozen.get(member),
                       'match': members_frozen.get(member) == digest}
                      for member, digest in declared.items()]
    report['binaries'] = {'spk_declared': manifest['inputs']['spk']['sha256'],
                          'spk_frozen': frozen_spk_sha,
                          'spk_match': frozen_spk_sha == manifest['inputs']['spk']['sha256'],
                          'members': binary_compare}
    report['binaries_passed'] = report['binaries']['spk_match'] and all(
        row['match'] for row in binary_compare)
    # Frozen tracked evidence unchanged: commit tree vs working tree for old dirs.
    frozen_dirs = ['docs/validation/issue7-native-001/server-static',
                   'docs/validation/issue7-native-001/static',
                   'docs/validation/issue7-native-001/l2-static',
                   'docs/validation/issue7-native-001/server-capability',
                   'docs/validation/issue7-native-001/REPORT.md']
    status = git('status', '--porcelain', '--', *frozen_dirs)
    report['frozen_evidence_status'] = {'command': status.args,
                                        'exit_code': status.returncode,
                                        'porcelain': status.stdout.splitlines(),
                                        'clean': status.returncode == 0 and not status.stdout.strip()}
    # T2 report directory must not be part of the verified commit.
    in_commit = git('ls-tree', '-r', '--name-only', 'fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4',
                    '--', 'docs/validation/issue7-native-001/l2-source-002')
    report['t2_report_commit_status'] = {
        'command': in_commit.args, 'exit_code': in_commit.returncode,
        'paths_in_commit': in_commit.stdout.splitlines(),
        'tracked_in_commit': bool(in_commit.stdout.strip()),
        'note': ('T2 report directory is uncommitted in fefb8ee; hashed here from the working '
                 'tree as the reviewed artifact; it must be committed before packaging/freeze')}
    report['passed'] = (report['evidence_integrity']['passed'] and report['inputs_passed']
                        and report['binaries_passed'] and report['frozen_evidence_status']['clean'])
    output = BUNDLE / 'environment' / 't2-evidence-check.json'
    output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'passed': report['passed'],
                      'evidence_integrity': report['evidence_integrity']['passed'],
                      'inputs': report['inputs_passed'], 'binaries': report['binaries_passed'],
                      'frozen_clean': report['frozen_evidence_status']['clean'],
                      't2_tracked_in_commit': report['t2_report_commit_status']['tracked_in_commit'],
                      'output': str(output)}))


if __name__ == '__main__':
    main()