"""Independent read-only oracle for quiescent, dedicated fault-test fixtures.

Expected manifests must be written by the test driver before the tested action,
never reconstructed from the client state being judged. No client code imported.
"""
import argparse
import hashlib
import json
import pathlib
import sqlite3


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def safe(root, name):
    rel = pathlib.PurePosixPath(name)
    if rel.is_absolute() or not rel.parts or '..' in rel.parts:
        raise ValueError('invalid fixture-relative path')
    p = root
    for part in rel.parts:
        p = p / part
        if p.is_symlink():
            raise ValueError('symlink in fixture path')
    if not p.resolve().is_relative_to(root.resolve()):
        raise ValueError('fixture path escapes root')
    return p


def verify(root, expected, state=None):
    errors = []
    if expected.get('schema') != 1:
        raise ValueError('unsupported oracle manifest')
    if root.is_symlink() or not root.is_dir():
        return {'passed': False, 'errors': [{'kind': 'invalid_fixture_root'}]}
    wanted = dict(expected['files'])
    directories = set(expected.get('directories', []))
    actual_files, actual_dirs = set(), set()
    for p in root.rglob('*'):
        rel = p.relative_to(root).as_posix()
        if p.is_symlink():
            errors.append({'kind': 'unexpected_symlink', 'path': rel})
        elif p.is_file():
            actual_files.add(rel)
        elif p.is_dir():
            actual_dirs.add(rel)
    if actual_files != set(wanted):
        errors.append({'kind': 'file_namespace_mismatch', 'missing': sorted(set(wanted)-actual_files), 'extra': sorted(actual_files-set(wanted))})
    if actual_dirs != directories:
        errors.append({'kind': 'directory_namespace_mismatch'})
    for name, sha in wanted.items():
        try:
            if digest(safe(root, name)) != sha:
                errors.append({'kind': 'content_mismatch', 'path': name})
        except (OSError, ValueError):
            errors.append({'kind': 'content_unavailable', 'path': name})
    if state is not None:
        db = None
        try:
            db = sqlite3.connect((state/'state.sqlite').resolve().as_uri()+'?mode=ro', uri=True)
            db.execute('BEGIN')
            if db.execute('PRAGMA quick_check').fetchone() != ('ok',):
                errors.append({'kind': 'database_integrity_failed'})
            for draft in expected.get('drafts', []):
                row = db.execute('SELECT token,base,dirty FROM drafts WHERE node=?', (draft['node'],)).fetchone()
                if row != (draft['token'], draft['base'], draft['dirty']):
                    errors.append({'kind': 'draft_identity_mismatch', 'node': draft['node']})
                try:
                    if digest(safe(state/'drafts', draft['token'])) != draft['sha256']:
                        errors.append({'kind': 'draft_content_mismatch', 'node': draft['node']})
                except (OSError, ValueError):
                    errors.append({'kind': 'draft_content_unavailable', 'node': draft['node']})
            for op in expected.get('operations', []):
                row = db.execute('SELECT node,kind,base,state FROM operations WHERE id=?', (op['id'],)).fetchone()
                if row != (op['node'], op['kind'], op['base'], op['state']):
                    errors.append({'kind': 'operation_mismatch', 'id': op['id']})
            if expected.get('exact_operation_ids') is not None:
                ids = {r[0] for r in db.execute('SELECT id FROM operations')}
                if ids != set(expected['exact_operation_ids']):
                    errors.append({'kind': 'operation_set_mismatch'})
        except sqlite3.DatabaseError:
            errors.append({'kind': 'database_unreadable'})
        finally:
            if db is not None:
                db.close()
    elif expected.get('drafts') or expected.get('operations') or 'exact_operation_ids' in expected:
        errors.append({'kind': 'state_required'})
    return {'passed': not errors, 'errors': errors, 'scope': 'quiescent fixture files and specified local records; not server identity or power-loss proof'}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root', required=True, type=pathlib.Path)
    p.add_argument('--manifest', required=True, type=pathlib.Path)
    p.add_argument('--state', type=pathlib.Path)
    a = p.parse_args()
    result = verify(a.root, json.loads(a.manifest.read_text()), a.state)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    raise SystemExit(0 if result['passed'] else 1)
