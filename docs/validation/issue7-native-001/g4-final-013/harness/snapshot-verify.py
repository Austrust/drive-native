#!/usr/bin/env python3
"""Verify that an extracted tree is byte-identical to a git commit.

Independent of the submission author: compares every blob path and executable
bit and recomputes git blob object ids against `git ls-tree -r`, and rejects
extra files (optionally allowing explicitly named build-generated extras).

The repository may be reached either with `-C <repo>` (normal checkout) or with
an explicit `--git-dir` (needed here because the git worktree .git file holds a
Windows-form path that Linux git cannot resolve; the shared object database is
reached through the main repository git dir).
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path


def _git(repo, git_dir, *args):
    command = ['git', '-c', 'safe.directory=*']
    if git_dir:
        command.append('--git-dir=' + str(git_dir))
    else:
        command += ['-C', str(repo)]
    return subprocess.run(command + list(args), capture_output=True, check=True).stdout


def _blob_sha1(data):
    digest = hashlib.sha1()
    digest.update(b'blob ' + str(len(data)).encode() + b'\0')
    digest.update(data)
    return digest.hexdigest()


def verify_tree(repo, commit, source, git_dir=None, allow_extra=()):
    repo = Path(repo).resolve() if repo is not None else None
    source = Path(source).resolve()
    head = _git(repo, git_dir, 'rev-parse', '--verify', commit + '^{commit}').decode().strip()
    tree = _git(repo, git_dir, 'rev-parse', '--verify', head + '^{tree}').decode().strip()
    raw = _git(repo, git_dir, 'ls-tree', '-r', '-z', '--full-tree', head)
    entries = []
    for chunk in raw.split(b'\0'):
        if not chunk:
            continue
        meta, path = chunk.split(b'\t', 1)
        mode, kind, obj = meta.decode().split(' ')
        entries.append({'mode': mode, 'kind': kind, 'object': obj, 'path': os.fsdecode(path)})
    entries.sort(key=lambda row: row['path'])
    problems = []
    checked = 0
    for row in entries:
        target = source / row['path']
        if row['kind'] != 'blob':
            problems.append({'path': row['path'], 'problem': 'unsupported object kind ' + row['kind']})
            continue
        if row['mode'] == '120000':
            if not target.is_symlink():
                problems.append({'path': row['path'], 'problem': 'expected symlink'})
                continue
            if _blob_sha1(os.fsencode(os.readlink(target))) != row['object']:
                problems.append({'path': row['path'], 'problem': 'symlink target blob mismatch'})
            checked += 1
            continue
        if target.is_symlink() or not target.is_file():
            problems.append({'path': row['path'], 'problem': 'missing or not a regular file'})
            continue
        data = target.read_bytes()
        actual = _blob_sha1(data)
        if actual != row['object']:
            problems.append({'path': row['path'], 'problem': 'blob mismatch',
                             'expected_sha1': row['object'], 'actual_sha1': actual})
            continue
        if bool(target.stat().st_mode & 0o111) != (row['mode'] == '100755'):
            problems.append({'path': row['path'], 'problem': 'executable bit mismatch'})
            continue
        checked += 1
    listed = {row['path'] for row in entries}
    actual = set()
    for path in source.rglob('*'):
        if path.is_file() or path.is_symlink():
            actual.add(path.relative_to(source).as_posix())
    allow_extra = set(allow_extra)
    allowed_extra = sorted(allow_extra & (actual - listed))
    extra = sorted(actual - listed - allow_extra)
    missing = sorted(listed - actual)
    for name in extra:
        problems.append({'path': name, 'problem': 'extra file not in commit'})
    for name in missing:
        problems.append({'path': name, 'problem': 'missing file'})
    listing_sha = hashlib.sha256('\n'.join(
        row['mode'] + ' ' + row['kind'] + ' ' + row['object'] + '\t' + row['path']
        for row in entries).encode()).hexdigest()
    return {
        'repo': str(repo), 'git_dir': str(git_dir) if git_dir else None,
        'commit': head, 'root_tree': tree, 'source': str(source),
        'entries': len(entries), 'verified_blobs': checked,
        'extra_files': extra, 'allowed_extra_files': allowed_extra,
        'missing_files': missing,
        'problems': problems[:50], 'problem_count': len(problems),
        'tree_listing_sha256': listing_sha,
        'passed': not problems and len(entries) == checked and not extra and not missing,
        'method': 'git ls-tree -r blob object ids + path set + executable bit; no product source writes',
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo', type=Path)
    parser.add_argument('--git-dir', type=Path)
    parser.add_argument('--commit', required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--allow-extra', action='append', default=[])
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert args.repo or args.git_dir
    result = verify_tree(args.repo, args.commit, args.source, args.git_dir,
                         allow_extra=args.allow_extra)
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'passed': result['passed'], 'entries': result['entries'],
                      'problem_count': result['problem_count'],
                      'allowed_extra_files': result['allowed_extra_files'],
                      'tree_listing_sha256': result['tree_listing_sha256']}))
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())