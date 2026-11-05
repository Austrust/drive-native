"""Remove only this project's reproducible Rust incremental caches."""
from pathlib import Path
import json
import os
import shutil
import subprocess

base = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
relative = [f'g4-{n:03d}/target/debug/incremental' for n in (1, 2, 3, 4, 5, 6, 7, 9)]
relative += ['g4-009/clock-diagnostic-001/target/debug/incremental']
targets = [base / item for item in relative]
processes = subprocess.check_output(['ps', '-eo', 'comm='], text=True).splitlines()
assert not any(p.strip() in ('cargo', 'rustc') for p in processes), 'Build is active'
mounts = [Path(line.split()[4]) for line in Path('/proc/self/mountinfo').read_text().splitlines()]
records = []
seen = set()
for target in targets:
    assert target.resolve(strict=True) == target and target.is_relative_to(base)
    assert not target.is_symlink() and target.name == 'incremental'
    assert not any(mount.is_relative_to(target) for mount in mounts)
    count = allocated = logical = 0
    for parent, directories, files in os.walk(target, followlinks=False):
        for name in directories + files:
            path = Path(parent) / name
            assert not path.is_symlink(), f'Unexpected cache symlink: {path}'
            stat = path.stat()
            identity = (stat.st_dev, stat.st_ino)
            if identity not in seen:
                seen.add(identity)
                allocated += stat.st_blocks * 512
                logical += stat.st_size
            if path.is_file():
                count += 1
    records.append({'path': str(target), 'files': count, 'allocated_bytes': allocated,
                    'logical_bytes': logical, 'removed': False})

record = Path(__file__).resolve().parent / 'incremental-cache-cleanup.json'
data = {'reason': 'Recover host disk for independent issue #7 verification; only reproducible incremental build caches.',
        'preserved': ['all test ELF files in deps/examples', 'sources', 'logs', 'manifests',
                      'old-artifacts', 'clock diagnostic evidence', 'registry dependencies'],
        'targets': records}
def save():
    record.write_text(json.dumps(data, indent=2) + '\n')
save()
for target, row in zip(targets, records):
    shutil.rmtree(target)
    row['removed'] = True
    save()
print(json.dumps({'removed_targets': len(records),
                  'allocated_bytes': sum(row['allocated_bytes'] for row in records)}))
