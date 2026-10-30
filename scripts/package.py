"""Create development artifacts after verify.sh; never builds an installer."""
from pathlib import Path
import hashlib
import io
import json
import tarfile
import zipfile

root=Path(__file__).resolve().parents[1]
delivery=root/'delivery'
delivery.mkdir(exist_ok=True)
binary=root/'target/release/drive-native-lab'
if binary.read_bytes()[:4]!=b'\x7fELF':raise SystemExit('expected a built Linux ELF')

binary_archive=delivery/'drive-native-lab-linux-x86_64.tar.gz'
note=b'''Drive Native development lab (NOT a sync client)
Run on Linux x86-64: ./drive-native-lab --simulate
Tested on Ubuntu 24.04 / WSL only; Mint desktop compatibility is not verified.
No credentials, NAS connection, FUSE mount or installation are performed.
passed=true reports an offline state-machine simulation only.
Source and protocol limitations are supplied in drive-native-source.zip.
'''
with tarfile.open(binary_archive,'w:gz') as tar:
    for name,data,mode in [('drive-native-lab',binary.read_bytes(),0o755),('README.txt',note,0o644)]:
        info=tarfile.TarInfo(name);info.size=len(data);info.mode=mode;info.mtime=0
        tar.addfile(info,io.BytesIO(data))

source_archive=delivery/'drive-native-source.zip'
manifest={}
with zipfile.ZipFile(source_archive,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
    for path in sorted(root.rglob('*')):
        rel=path.relative_to(root)
        if not path.is_file() or any(p in {'target','delivery','__pycache__','.git'} for p in rel.parts):continue
        if path.name=='core' or path.suffix=='.core':continue
        data=path.read_bytes();name=rel.as_posix()
        z.writestr('drive-native/'+name,data);manifest[name]=hashlib.sha256(data).hexdigest()
    # Distribution landing page has no recursive archive/checksum links inside the source ZIP.
    landing='''# 源码包内的交付说明

本目录是源码分发的占位入口。压缩包和 SHA256SUMS 在原下载位置单独提供，不嵌套进源码包。

从 [项目 README](../README.md) 或 [实现报告](../REPORT.md) 开始；构建流程见 scripts/verify.sh。
'''.encode('utf8')
    z.writestr('drive-native/delivery/README.md',landing)
    manifest['delivery/README.md']=hashlib.sha256(landing).hexdigest()
    z.writestr('drive-native/SOURCE-MANIFEST.json',json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')

hashes={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [source_archive,binary_archive]}
(delivery/'SHA256SUMS').write_text(''.join(f'{sha}  {name}\n' for name,sha in hashes.items()),encoding='ascii')
with zipfile.ZipFile(source_archive) as z:
    assert z.testzip() is None
    for name,sha in manifest.items():assert hashlib.sha256(z.read('drive-native/'+name)).hexdigest()==sha
with tarfile.open(binary_archive) as tar:
    assert tar.getmember('drive-native-lab').mode==0o755
    assert tar.extractfile('drive-native-lab').read()==binary.read_bytes()
summary={'source_files':len(manifest),'zip_crc_verified':True,'source_hashes_verified':True,
    'binary_roundtrip_verified':True,'binary_mode':'0755','hashes':hashes,
    'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}
(delivery/'package-checks.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf8')
print(json.dumps(summary,indent=2))
