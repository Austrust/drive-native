#!/usr/bin/env python3
"""Independent verification of this bundle's MANIFEST.json and SHA256SUMS.

Recomputes every hash from files on disk, detects missing/extra files under the
documented self-exclusion rule, and cross-checks that both inventories describe
the same set.
"""
import hashlib
import json
import sys
from pathlib import Path

BUNDLE = Path(__file__).resolve().parent.parent
EXCLUDE_MANIFEST = {'MANIFEST.json'}
EXCLUDE_SUMS = {'MANIFEST.json', 'SHA256SUMS'}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    all_files = {path.relative_to(BUNDLE).as_posix(): path
                 for path in BUNDLE.rglob('*') if path.is_file()}
    manifest = json.loads((BUNDLE / 'MANIFEST.json').read_text())
    listed = manifest['files']
    manifest_mismatch = [name for name, digest in listed.items()
                         if name not in all_files or sha(all_files[name]) != digest]
    manifest_missing = sorted(name for name in listed if name not in all_files)
    manifest_extra = sorted(name for name in all_files
                            if name not in listed and name not in EXCLUDE_MANIFEST)
    sums = {}
    sums_duplicates = []
    for line in (BUNDLE / 'SHA256SUMS').read_text().splitlines():
        if not line.strip():
            continue
        parts = line.split(None, 1)
        if len(parts) != 2:
            sums_duplicates.append(line)
            continue
        digest, name = parts[0], parts[1].strip()
        if name in sums:
            sums_duplicates.append(name)
        sums[name] = digest
    sums_mismatch = [name for name, digest in sums.items()
                     if name not in all_files or sha(all_files[name]) != digest]
    sums_extra = sorted(name for name in all_files
                        if name not in sums and name not in EXCLUDE_SUMS)
    sums_missing = sorted(name for name in sums if name not in all_files)
    # MANIFEST.json documents and hashes SHA256SUMS; SHA256SUMS excludes itself by
    # the documented rule, so this one name is the expected difference.
    set_mismatch = sorted((set(listed) ^ set(sums)) - {'SHA256SUMS'})
    elf = [name for name, path in all_files.items() if path.read_bytes()[:4] == b'\x7fELF']
    result = {
        'bundle': str(BUNDLE),
        'manifest_file_count': len(listed),
        'files_on_disk': len(all_files),
        'manifest_mismatch': manifest_mismatch,
        'manifest_missing': manifest_missing,
        'manifest_extra': manifest_extra,
        'sha256sums_entries': len(sums),
        'sha256sums_mismatch': sums_mismatch,
        'sha256sums_missing': sums_missing,
        'sha256sums_extra': sums_extra,
        'sha256sums_duplicates_or_malformed': sums_duplicates,
        'inventory_set_mismatch': set_mismatch,
        'elf_files_present': elf,
        'exclusion_rule': 'MANIFEST.json excludes itself; SHA256SUMS excludes MANIFEST.json and itself',
        'passed': not (manifest_mismatch or manifest_missing or manifest_extra
                       or sums_mismatch or sums_missing or sums_extra
                       or sums_duplicates or set_mismatch or elf),
    }
    print(json.dumps(result))
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())