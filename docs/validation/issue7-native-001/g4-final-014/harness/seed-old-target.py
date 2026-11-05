#!/usr/bin/env python3
"""Create the previous-candidate build target seeded only with registry
dependency artifacts from this candidate's independent target (Cargo.lock
registry names only; no workspace/path product artifacts, test ELFs or
incremental cache are copied)."""
import argparse
import hashlib
import json
import shutil
import tomllib
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, required=True, help='old-candidate source snapshot')
    parser.add_argument('--seed', type=Path, required=True, help='current candidate target dir')
    parser.add_argument('--target', type=Path, required=True, help='new old-candidate target dir')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source, seed, target = args.source.resolve(), args.seed.resolve(), args.target.resolve()
    assert not target.exists() and not args.output.exists()
    assert not target.is_relative_to(source)
    packages = tomllib.loads((source / 'Cargo.lock').read_text())['package']
    names = {row['name'] for row in packages if row.get('source', '').startswith('registry+')}
    normalized = {name.replace('-', '_') for name in names}
    result = {'source': str(source), 'seed': str(seed), 'target': str(target),
              'selection': 'Cargo.lock registry+ packages only; debug deps/fingerprint/build; no '
                           'workspace/path products, test ELF or incremental cache',
              'copied_files': 0, 'copied_bytes': 0, 'complete': False}
    output = args.output
    output.parent.mkdir(parents=True, exist_ok=True)

    def save():
        output.write_text(json.dumps(result, indent=2) + '\n')

    save()
    files = total = 0
    for folder in ('deps', '.fingerprint', 'build'):
        origin = seed / 'debug' / folder
        if not origin.is_dir():
            continue
        destination = target / 'debug' / folder
        destination.mkdir(parents=True, exist_ok=True)
        for item in origin.iterdir():
            if folder == 'deps':
                if not item.is_file() or item.suffix not in ('.rlib', '.rmeta', '.so', '.d'):
                    continue
                stem = item.stem.removeprefix('lib').rsplit('-', 1)[0]
                if stem not in normalized:
                    continue
                shutil.copy2(item, destination / item.name)
                files += 1
                total += item.stat().st_size
            elif item.is_dir() and item.name.rsplit('-', 1)[0] in names:
                shutil.copytree(item, destination / item.name, symlinks=False)
                for copied in (destination / item.name).rglob('*'):
                    if copied.is_file():
                        files += 1
                        total += copied.stat().st_size
    result.update(complete=True, copied_files=files, copied_bytes=total)
    save()
    print(json.dumps({'complete': True, 'copied_files': files, 'copied_bytes': total}))


if __name__ == '__main__':
    main()