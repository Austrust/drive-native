"""After all validation stops, reclaim only inventoried old compilation caches."""
from pathlib import Path
import json
import stat
import subprocess

base = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
record = Path(__file__).resolve().parent / 'old-library-cache-cleanup.json'
assert not record.exists(), 'Do not overwrite a cleanup record'
assert not any(name.strip() in ('cargo', 'rustc') for name in
               subprocess.check_output(['ps', '-eo', 'comm='], text=True).splitlines())
for line in Path('/proc/self/mountinfo').read_text().splitlines():
    assert not Path(line.split()[4]).is_relative_to(base), 'Validation mount still active'

selected = []
kept = {}
groups = []
seen = set()
for number in (1, 2, 3, 4, 5, 6, 7, 9, 11):
    for mode in ('debug', 'release'):
        folder = base / f'g4-{number:03d}' / 'target' / mode / 'deps'
        if not folder.exists():
            continue
        assert folder.resolve(strict=True) == folder and not folder.is_symlink()
        count = allocated = 0
        for path in sorted(folder.iterdir()):
            item = path.lstat()
            if path.suffix not in ('.rlib', '.rmeta', '.d'):
                kept[str(path)] = (item.st_dev, item.st_ino, item.st_size, item.st_mtime_ns)
                continue
            assert stat.S_ISREG(item.st_mode) and not path.is_symlink()
            assert path.resolve(strict=True) == path and item.st_nlink == 1
            with path.open('rb') as stream:
                assert stream.read(4) != b'\x7fELF', 'Never remove an executable by extension'
            identity = (item.st_dev, item.st_ino)
            assert identity not in seen
            seen.add(identity)
            selected.append({'path': str(path), 'device': item.st_dev, 'inode': item.st_ino,
                             'bytes': item.st_size, 'allocated_bytes': item.st_blocks * 512,
                             'mtime_ns': item.st_mtime_ns})
            count += 1
            allocated += item.st_blocks * 512
        groups.append({'directory': str(folder), 'files': count, 'allocated_bytes': allocated})

# The preflight is metadata-only except for a four-byte ELF exclusion check.
# These expected totals are the independently recorded, quiescent inventories.
assert len(selected) == 11494 + 3261, 'Inventoried cache file count changed'
assert sum(row['allocated_bytes'] for row in selected) == 21376344064 + 6477135872
data = {'scope': 'Only old g4-001..007/009/011 deps .rlib/.rmeta/.d; excludes all ELF/.so and g4-012',
        'groups': groups, 'removed_files': 0, 'preserved_other_entries': len(kept),
        'files': selected, 'complete': False}
def save():
    record.write_text(json.dumps(data, indent=2) + '\n')
save()
try:
    for row in selected:
        path = Path(row['path'])
        item = path.lstat()
        assert (item.st_dev, item.st_ino, item.st_size, item.st_mtime_ns, item.st_nlink) == (
            row['device'], row['inode'], row['bytes'], row['mtime_ns'], 1)
        assert path.resolve(strict=True) == path and not path.is_symlink()
        path.unlink()
        data['removed_files'] += 1
    for name, expected in kept.items():
        item = Path(name).lstat()
        assert (item.st_dev, item.st_ino, item.st_size, item.st_mtime_ns) == expected
    data['preserved_entries_unchanged'] = True
    data['complete'] = True
finally:
    save()
print(json.dumps({'removed_files': data['removed_files'], 'complete': data['complete'],
                  'guest_allocated_bytes': sum(row['allocated_bytes'] for row in selected)}))
