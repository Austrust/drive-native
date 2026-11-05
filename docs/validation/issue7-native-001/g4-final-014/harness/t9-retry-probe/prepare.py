#!/usr/bin/env python3
"""Prepare the T9 retry adversarial probe crate against the frozen snapshot.

Generates an external crate whose retry module is the product file
crates/drive-core/src/native_retry.rs with ONLY the leading `//!` module-doc
lines removed (recorded: original SHA256, included SHA256, prefix bytes). All
other code is the verifier's adversarial test harness. No product source is
modified and no private product API is imported.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
PRODUCT_REL = 'crates/drive-core/src/native_retry.rs'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--candidate', required=True)
    args = parser.parse_args()
    source = args.source.resolve()
    output = args.output.resolve()
    assert len(args.candidate) == 40 and all(c in '0123456789abcdef' for c in args.candidate)
    assert not output.exists(), 'refusing to overwrite a prior probe'
    assert not output.is_relative_to(source)
    product = source / PRODUCT_REL
    assert product.is_file(), product
    original = product.read_bytes()
    lines = original.split(b'\n')
    index = 0
    while index < len(lines) and lines[index].startswith(b'//!'):
        index += 1
    # keep one trailing empty line removal only; the rest of the file is byte-identical
    stripped = b'\n'.join(lines[index:])
    assert stripped and not stripped.startswith(b'//!'), 'unexpected remaining module doc'
    output.mkdir(parents=True)
    (output / 'src').mkdir()
    (output / 'src' / 'native_retry_included.rs').write_bytes(stripped)
    shutil.copy2(HERE / 'main.rs', output / 'src' / 'main.rs')
    shutil.copy2(source / 'Cargo.lock', output / 'Cargo.lock')
    with (HERE / 'Cargo.toml.in').open() as stream:
        manifest = stream.read().replace('@SPROTO_PATH@', json.dumps(str(source / 'crates/sproto')))
    (output / 'Cargo.toml').write_text(manifest)
    metadata = subprocess.run(['cargo', 'metadata', '--offline', '--format-version', '1',
                               '--manifest-path', str(output / 'Cargo.toml')],
                              capture_output=True, text=True)
    (output / 'metadata.json').write_text(metadata.stdout)
    (output / 'metadata.log').write_text(metadata.stderr)
    assert metadata.returncode == 0, metadata.stderr
    frozen = tomllib.loads((source / 'Cargo.lock').read_text())['package']
    resolved = tomllib.loads((output / 'Cargo.lock').read_text())['package']

    def identity(row):
        return (row['name'], row['version'], row.get('source'), row.get('checksum'))

    frozen_registry = {identity(row) for row in frozen if row.get('source')}
    registry = [row for row in resolved if row.get('source')]
    unexpected = [identity(row) for row in registry if identity(row) not in frozen_registry]
    data = {
        'candidate_commit': args.candidate,
        'frozen_source': str(source),
        'product_file': PRODUCT_REL,
        'product_file_sha256': sha(product),
        'included_file_sha256': sha(output / 'src' / 'native_retry_included.rs'),
        'stripped_leading_line_count': index,
        'scope': ('external adversarial tests of the actual retry source file; real sproto::Error '
                  'type; no product source modification; no private crate API imported'),
        'source_cargo_lock_sha256': sha(source / 'Cargo.lock'),
        'probe_cargo_lock_sha256': sha(output / 'Cargo.lock'),
        'registry_dependencies_checked': len(registry),
        'unexpected_registry_dependencies': unexpected,
        'registry_dependencies_match_frozen_candidate': not unexpected,
        'harness_main_sha256': sha(HERE / 'main.rs'),
    }
    (output / 'provenance.json').write_text(json.dumps(data, indent=2) + '\n')
    print(json.dumps(data))
    return 0 if data['registry_dependencies_match_frozen_candidate'] else 1


if __name__ == '__main__':
    import sys
    sys.exit(main())