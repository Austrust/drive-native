"""Build an auditable development source snapshot, never a candidate installer."""
from pathlib import Path
import hashlib
import json
import zipfile

root = Path(__file__).resolve().parents[1]
delivery = root / 'delivery'
delivery.mkdir(exist_ok=True)
archive = delivery / 'drive-native-integration-source.zip'
manifest = {}
excluded = {'target', 'delivery', '__pycache__', '.git', '.venv'}

with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    def add(name, data):
        info = zipfile.ZipInfo('drive-native/' + name, (2026, 10, 28, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o100644 << 16
        z.writestr(info, data)
        manifest[name] = hashlib.sha256(data).hexdigest()

    for path in sorted(root.rglob('*')):
        rel = path.relative_to(root)
        if not path.is_file() or any(part in excluded for part in rel.parts):
            continue
        if path.suffix in {'.sqlite', '.db', '.core', '.pyc'} or path.name == 'core':
            raise RuntimeError(f'unexpected runtime artifact in source tree: {rel}')
        add(rel.as_posix(), path.read_bytes())
    add('delivery/README.md', ('# 当前源码快照\n\n这是开发源码和真实联调证据，尚无候选 deb。'
        '压缩包与校验值在下载位置提供，不嵌套在源码包中。\n\n'
        '从 [项目 README](../README.md) 和 [LIVE_REPORT](../LIVE_REPORT.md) 开始。\n').encode())
    z.writestr('drive-native/SOURCE-MANIFEST.json', json.dumps(manifest, indent=2) + '\n')

with zipfile.ZipFile(archive) as z:
    assert z.testzip() is None
    for name, digest in manifest.items():
        assert hashlib.sha256(z.read('drive-native/' + name)).hexdigest() == digest
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
(delivery / 'INTEGRATION-SHA256SUMS').write_text(f'{digest}  {archive.name}\n', encoding='ascii')
result = {'artifact_kind': 'development-source-not-installer', 'source_files': len(manifest),
          'zip_crc_verified': True, 'source_hashes_verified': True, 'sha256': digest}
(delivery / 'integration-package-checks.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
