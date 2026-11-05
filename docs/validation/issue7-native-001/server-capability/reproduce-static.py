"""Reproduce bounded static evidence. Does not execute or load Synology code."""
from pathlib import Path
import argparse
import json
import struct
import subprocess
import sys

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--syncd', type=Path, required=True)
p.add_argument('--core', type=Path, required=True)
p.add_argument('--python-libs', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
tool = Path(__file__).with_name('analyze-server-elf.py')
if a.out.exists():
    raise SystemExit('Output exists. Use a new directory; frozen evidence must not be overwritten.')
a.out.mkdir(parents=True)
groups = [
    ('identity', a.syncd, r'GetFileCapabilityHandler(6Handle|15CheckParameters)|PathConverterMiddleware21HandleAfterHeaderRead|PathConvertHelper(7Convert|22QueryNodeByPermanentId|9ProcessID|7Process)|Handler7Utility10IsSafePath'),
    ('acl', a.syncd, r'(DownloadNodeACLTest|CreateACLTest|RemoveACLTest)15PermissionCheck|Rename(Source|Dest)ACLTest7GetPath'),
    ('fields', a.core, r'DriveAcl10Capability9ToPObject'),
]
for label, binary, pattern in groups:
    subprocess.run([sys.executable, str(tool), str(binary), '--out', str(a.out / label), '--pattern', pattern, '--python-libs', str(a.python_libs)], check=True)

# Reuse only the reader/decoder initialization, before its output-writing loop.
saved = sys.argv
sys.argv = [str(tool), str(a.syncd), '--out', str(a.out), '--python-libs', str(a.python_libs)]
ns = {'__name__': 'static_reader'}
exec(tool.read_text(encoding='utf-8').split('args.out.mkdir')[0], ns)
sys.argv = saved
names = ns['names']
selected = {'_ZTV19RenameSourceACLTest', '_ZTV17RenameDestACLTest'}
lines = []
for s in ns['syms']:
    if s['name'] not in selected:
        continue
    lines.append(f"{s['name']} {s['address']:#x}")
    for i in range(0, s['size'], 8):
        value = struct.unpack('<Q', ns['read'](s['address'] + i, 8))[0]
        lines.append(f'  address-point-slot {i - 16:+#x}: {value:#x} {names.get(value, "")}')
(a.out / 'rename-vtables.txt').write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(json.dumps({'output': str(a.out), 'synology_code_executed': False}))
