"""Assemble self-contained G4 evidence (no source/target trees or production paths)."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

here = Path(__file__).resolve().parent
base = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
final = base / 'g4-003'
main = json.loads((final / 'summary.json').read_text())
supplement = json.loads((final / 'supplement/summary.json').read_text())
functional = json.loads((base / 'functional-g4-003/result.json').read_text())
schema = json.loads((base / 'schema-g4-003/results.json').read_text())
integrity = json.loads((final / 'final-integrity.json').read_text())
inventory = json.loads((final / 'inventory-001.json').read_text())
assert main['candidate_commit'] == 'c744729141ca8b23bde63c61ca8a644bdfdaee81'
assert main['all_commands_passed'] and supplement['all_passed'] and functional['complete'] and schema['complete'] and integrity['passed']
assert inventory['all_reported_counts_reconciled']
bundle = here / 'G4-evidence-bundle'
bundle.mkdir(exist_ok=False)
runs = ('g4-001','g4-002','g4-003','schema-g4-001','schema-g4-003','functional-g4-002','functional-g4-003')
subprocess.run([sys.executable, str(here / 'export-g4-evidence.py'), '--destination', str(bundle / 'evidence'), '--runs', *runs], check=True)
(bundle / 'harness').mkdir()
files = (
    'g4-001-runner.py','g4-002-runner.py','g4-003-runner.py','run-g4.py','run-g4.sh',
    'start-g4-002.sh','start-g4-003.sh','prepare-source.ps1','analyze-g4.py',
    'run-g4-supplement.py','supplement-g4-002.sh','supplement-g4-003.sh',
    'verify-tray-supplement.py','render-mainwindow-permissions.py','render-g4-001.sh',
    'verify-fsearch-functional.py','functional-g4-002.sh','functional-g4-003.sh','run-paired.py','summarize-functional.py',
    'schema-probe.rs','run-schema-g4-001.sh','schema-g4-003.sh','verify-schema.py',
    'verify-schema-attempt001.py','verify-schema-g4-001-success.py','schema-harness-attempt001-failure.txt',
    'prepare-nautilus-private.sh','verify-nautilus-private.sh','correct-nautilus-count.py',
    'finalize-g4-integrity.py','export-g4-evidence.py','bundle-g4.py',
)
for name in files:
    shutil.copy2(here / name, bundle / 'harness' / name)
(bundle / 'provenance').mkdir()
for source, name in [('prepared-g4-001','g4-001'),('prepared-g4-002-final','g4-002'),('prepared-g4-003','g4-003')]:
    shutil.copy2(here / source / 'source.json', bundle / 'provenance' / (name + '-source.json'))
for name, destination in (('G4_REPORT.md','REPORT.md'),('G4-COVERAGE.md','COVERAGE.md')):
    content = (here / name).read_text().replace('evidence-g4-001-002/', 'evidence/').replace('G4-evidence-bundle/', '')
    content = content.replace('(G4-STATUS.md)', '(REPORT.md)').replace('(G4-COVERAGE.md)', '(COVERAGE.md)')
    (bundle / destination).write_text(content)
kernel = next(row for row in inventory['commands'] if row['name'] == 'kernel-fs-debug-index1')
(bundle / 'kernel-unique-tests.json').write_text(json.dumps({'count': len(kernel['tests']), 'tests': kernel['tests'],
    'index0_not_applicable': main['index0_not_applicable']}, indent=2) + '\n')
(bundle / 'final-result.json').write_text(json.dumps({'candidate_commit':main['candidate_commit'],
    'g4_complete':True, 'main_commands':main['command_counts'], 'schema':{k:schema[k] for k in ('passed','failed','ignored')},
    'source_integrity':integrity, 'unique_inventory_all_counts_reconciled':True,
    'scope':'WSL local synthetic tests; no NAS; G5 performance not included'}, indent=2) + '\n')
(bundle / 'README.md').write_text('# G4 独立验证证据\n\n'
    '- [验收结果、计数与原件入口](REPORT.md)\n'
    '- [规格逐条映射与未实测边界](COVERAGE.md)\n'
    '- [机器结论](final-result.json)\n'
    '- [36 个 FUSE 唯一测试及 INDEX=0 不适用原因](kernel-unique-tests.json)\n'
    '- [全部原始证据](evidence/evidence-manifest.json)\n\n'
    'harness/ 保存独立验证脚本，provenance/ 保存冻结提交和归档哈希。源码与 target 不复制；通过仓库中指定提交重建。'
    '脚本中的 Linux 绝对路径记录本次隔离环境；复跑须使用新 run 目录并调整该环境路径。'
    '原 WSL 二进制与状态仍保留，报告中的 reader/fixture 哈希绑定它们。此包没有真实清单文件名。\n')
manifest = {'candidate_commit':main['candidate_commit'], 'files':{}}
for path in bundle.rglob('*'):
    if path.is_file():
        with path.open('rb') as stream:
            manifest['files'][str(path.relative_to(bundle))] = hashlib.file_digest(stream, 'sha256').hexdigest()
(bundle / 'MANIFEST.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps({'bundle':str(bundle), 'files':len(manifest['files'])}))
