"""New immutable bundle for a later final G4 run; earlier bundles stay untouched."""
import argparse,hashlib,json,re,shutil,subprocess,sys
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument('--run-id',required=True)
parser.add_argument('--candidate-commit',required=True)
args=parser.parse_args()
assert re.fullmatch(r'g4-\d{3}',args.run_id) and re.fullmatch(r'[a-f0-9]{40}',args.candidate_commit)
here=Path(__file__).resolve().parent
base=Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
final=base/args.run_id
load=lambda path:json.loads(path.read_text())
main=load(final/'summary.json');supplement=load(final/'supplement/summary.json')
functional=load(base/('functional-'+args.run_id)/'result.json')
schema=load(base/('schema-'+args.run_id)/'results.json')
integrity=load(final/'final-integrity-post-history.json');inventory=load(final/'inventory-001.json')
history=load(final/'history-scale-verified/results.json')
assert main['candidate_commit']==args.candidate_commit
assert main['all_commands_passed'] and supplement['all_passed'] and functional['complete'] and schema['complete'] and integrity['passed']
assert inventory['all_reported_counts_reconciled']
assert history['passed']
bundle=here/('G4-evidence-bundle-'+args.run_id[-3:]);bundle.mkdir(exist_ok=False)
runs=[]
for number in range(1,int(args.run_id[-3:])+1):
    for prefix in ('g4-','schema-g4-','functional-g4-'):
        name=prefix+f'{number:03}'
        if (base/name).is_dir():runs.append(name)
subprocess.run([sys.executable,str(here/'export-g4-evidence.py'),'--destination',str(bundle/'evidence'),'--runs',*runs],check=True)
(bundle/'harness').mkdir()
for path in here.iterdir():
    if path.is_file() and (path.suffix in ('.py','.sh','.ps1','.rs') or path.name=='schema-harness-attempt001-failure.txt'):
        shutil.copy2(path,bundle/'harness'/path.name)
(bundle/'provenance').mkdir()
for number in range(1,int(args.run_id[-3:])+1):
    name=f'g4-{number:03}'
    preparation=here/('prepared-g4-002-final' if number==2 else 'prepared-'+name)
    if (preparation/'source.json').is_file():shutil.copy2(preparation/'source.json',bundle/'provenance'/(name+'-source.json'))
for source,destination in [('G4_REPORT.md','REPORT.md'),('G4-COVERAGE.md','COVERAGE.md')]:
    content=(here/source).read_text()
    content=re.sub(r'G4-evidence-bundle(?:-\d{3})?/', '', content).replace('evidence-g4-001-002/','evidence/')
    content=content.replace('(G4-STATUS.md)','(REPORT.md)').replace('(G4-COVERAGE.md)','(COVERAGE.md)')
    (bundle/destination).write_text(content)
kernel=next(row for row in inventory['commands'] if row['name']=='kernel-fs-debug-index1')
(bundle/'kernel-unique-tests.json').write_text(json.dumps({'count':len(kernel['tests']),'tests':kernel['tests'],
    'index0_not_applicable':main['index0_not_applicable']},indent=2)+'\n')
(bundle/'final-result.json').write_text(json.dumps({'candidate_commit':main['candidate_commit'],'g4_complete':True,
    'main_commands':main['command_counts'],'schema':{key:schema[key] for key in ('passed','failed','ignored')},
    'history_scale_timeout':{key:history[key] for key in ('passed','passed_count','failed_count','ignored_count')},
    'source_integrity':integrity,'unique_inventory_all_counts_reconciled':True,
    'scope':'WSL local synthetic tests; no NAS; G5 performance not included'},indent=2)+'\n')
(bundle/'README.md').write_text('# G4 独立验证证据\n\n'
    '- [验收结果、计数与原件入口](REPORT.md)\n'
    '- [规格逐条映射与未实测边界](COVERAGE.md)\n'
    '- [机器结论](final-result.json)\n'
    '- [FUSE唯一测试及INDEX=0不适用原因](kernel-unique-tests.json)\n'
    '- [全部原始证据](evidence/evidence-manifest.json)\n\n'
    'harness/ 保存独立验证脚本，provenance/ 保存每轮冻结提交和归档哈希。源码与target不复制；通过指定仓库提交重建。'
    '脚本中Linux绝对路径记录本次隔离环境，复跑需新run目录并调整环境路径。原WSL二进制和状态保留，报告内哈希绑定它们。此包不含真实清单文件名。\n')
manifest={'candidate_commit':main['candidate_commit'],'files':{}}
for path in bundle.rglob('*'):
    if path.is_file():
        with path.open('rb') as stream:manifest['files'][str(path.relative_to(bundle))]=hashlib.file_digest(stream,'sha256').hexdigest()
(bundle/'MANIFEST.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'bundle':str(bundle),'files':len(manifest['files'])}))
