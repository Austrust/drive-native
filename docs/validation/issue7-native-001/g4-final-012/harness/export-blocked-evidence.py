"""Estimate then export only small, existing issue7 evidence. No test/build execution."""
import argparse, hashlib, importlib.util, json, pathlib, re, shutil

HERE = pathlib.Path(__file__).resolve().parent
BASE = pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=pathlib.Path, required=True)
parser.add_argument('--export-under-100mib', action='store_true')
args = parser.parse_args()
assert not args.output.exists(), 'never overwrite an evidence bundle'
spec = importlib.util.spec_from_file_location('inventory', HERE/'analyze-g4.py')
inventory = importlib.util.module_from_spec(spec); spec.loader.exec_module(inventory)
files = {}; generated = {}; runs = []; private = []; prior_checks = []
suffixes = {'.json','.log','.py','.sh','.rs','.toml','.lock','.md','.png','.sha256','.txt','.in','.patch'}
def sha(path):
    with path.open('rb') as stream: return hashlib.file_digest(stream,'sha256').hexdigest()
def verify_prior(path, expected, authority):
    actual=sha(path) if path.is_file() else None
    prior_checks.append({'path':str(path),'authority':str(authority),'expected_sha256':expected,
        'actual_sha256':actual,'matched':actual==expected})
def add(source, destination):
    assert source.is_file() and not source.is_symlink(), source
    assert source.suffix in suffixes, source
    with source.open('rb') as stream: magic = stream.read(16)
    assert not magic.startswith((b'\x7fELF',b'SQLite format 3\x00',b'!<arch>\n',b'MZ')), source
    name = str(destination)
    assert name not in files and name not in generated, name
    files[name] = source
def tree(source, destination):
    for file in source.rglob('*'):
        if file.is_file(): add(file, destination/file.relative_to(source))
def json_bytes(value): return (json.dumps(value,indent=2,ensure_ascii=False)+'\n').encode()
def derive_inventory(run, summary):
    result = {'candidate_commit':summary['candidate_commit'], 'source_summary_sha256':sha(run/'summary.json'),
        'main_matrix_complete':summary['complete'],'commands':[],
        'note':'Counts are executions per command; repeated modes are not additive unique coverage.'}
    unique = {}
    for command in summary['commands']:
        if 'exit_code' not in command: continue
        log = run/command['log']; text = log.read_text(errors='replace')
        rows = inventory.rust_tests(text) if command['kind']=='rust' else inventory.gui_tests(text) if command['kind']=='gui' else []
        if command['kind']=='json' and command.get('result_path'):
            cases=json.loads((run/command['result_path']).read_text())['cases']
            rows=[{'suite':command['name'],'name':case['name'],'status':'ok' if case['passed'] else 'FAILED','reason':None} for case in cases]
        totals={status:sum(row['status']==status for row in rows) for status in ('ok','FAILED','ignored','unparsed')}
        expected=command['counts']
        reconciled=not expected.get('reported') or all(totals[status]==expected.get(key,0) for status,key in [('ok','passed'),('FAILED','failed'),('ignored','ignored')])
        result['commands'].append({'name':command['name'],'exit_code':command['exit_code'],'reported_counts':expected,
            'inventory_counts':totals,'counts_reconciled':reconciled,'log':command['log'],'log_sha256':sha(log),'tests':rows})
        for row in rows:
            key=row['suite']+'::'+row['name']
            unique.setdefault(key,{'suite':row['suite'],'name':row['name'],'observations':[]})['observations'].append(
                {'command':command['name'],'status':row['status'],'reason':row['reason']})
    result['unique_tests']=[unique[key] for key in sorted(unique)]
    result['all_reported_counts_reconciled']=all(row['counts_reconciled'] for row in result['commands'])
    assert result['all_reported_counts_reconciled'],run
    return result

for name in ('g4-006','g4-007','g4-007/fresh-001','g4-009'):
    run=BASE/name; rel=pathlib.Path('evidence')/name
    summary=json.loads((run/'summary.json').read_text()); runs.append((name,rel,summary))
    for command in summary['commands']:
        if command.get('log') and command.get('log_sha256'):
            verify_prior(run/command['log'],command['log_sha256'],run/'summary.json')
    for state,expected in summary.get('acl_screenshots',{}).items():
        if expected:verify_prior(run/'acl-screenshots'/f'acl-{state}.png',expected,run/'summary.json')
    for filename in ('summary.json','source.json','source-hashes-before.json','failure.txt','coordinator-stop.json',
        'coordinator-control.json','cache-provenance.json','final-integrity.json','native-loopback-results.json',
        'native-loopback-elf.sha256','worker-versions.json'):
        if (run/filename).is_file():add(run/filename,rel/filename)
    existing=run/'inventory-001.json'
    if existing.is_file():
        parsed=json.loads(existing.read_text())
        verify_prior(run/'summary.json',parsed['source_summary_sha256'],existing)
        for command in parsed['commands']:
            verify_prior(run/command['log'],command['log_sha256'],existing)
        assert parsed['source_summary_sha256']==sha(run/'summary.json') and parsed['all_reported_counts_reconciled']
        add(existing,rel/'inventory-001.json')
    else:generated[str(rel/'inventory-001.json')]=json_bytes(derive_inventory(run,summary))
    for folder in ('logs','acl-screenshots','screenshots'):
        if (run/folder).exists():tree(run/folder,rel/folder)
    for folder in ('native-loopback-crate','core-history-probe'):
        for filename in ('Cargo.toml','Cargo.lock','provenance.json','lock-verification.json','metadata.log','metadata.json',
            'cargo-metadata.json','src/main.rs','src/lib.rs'):
            if (run/folder/filename).is_file():add(run/folder/filename,rel/folder/filename)
    private.append({'path':str(run),'candidate':summary['candidate_commit'],
        'scope':'Original private run retained; target, binary, source and databases excluded from this portable bundle.'})

diagnostic_files={
    'clock-probe-001':('results.json','clock-samples.json','case-1.log','case-2.log'),
    'clock-diagnostic-001':('summary.json','test-only-instrumentation.patch','clean-products.log','build-diagnostic.log',
        'two-original-cases-concurrent.log','runtime-full-original-parallelism.log','clock-samples.json','clock-outliers.json','diagnostic-audit.json')}
for folder,names in diagnostic_files.items():
    for name in names:add(BASE/'g4-009'/folder/name,pathlib.Path('evidence/g4-009')/folder/name)
probe=json.loads((BASE/'g4-009/clock-probe-001/results.json').read_text())
for row in probe['cases']:
    verify_prior(BASE/'g4-009/clock-probe-001'/row['log'],row['log_sha256'],BASE/'g4-009/clock-probe-001/results.json')
private.append({'path':probe['copied_elf'],'sha256':probe['copied_elf_sha256'],'scope':'Unchanged original test ELF used only for clock probe.'})
derived=json.loads((BASE/'g4-009/clock-diagnostic-001/summary.json').read_text())
for row in derived['commands']:
    verify_prior(BASE/'g4-009/clock-diagnostic-001'/row['log'],row['log_sha256'],BASE/'g4-009/clock-diagnostic-001/summary.json')
audit=json.loads((BASE/'g4-009/clock-diagnostic-001/diagnostic-audit.json').read_text())
verify_prior(BASE/'g4-009/clock-diagnostic-001/summary.json',audit['summary_sha256'],BASE/'g4-009/clock-diagnostic-001/diagnostic-audit.json')
private.append({**derived['binary'],'scope':'Derived test-only diagnostic ELF; original candidate unchanged; not acceptance evidence.'})

cleanup=json.loads((HERE/'archive-cache-cleanup.json').read_text())
for number in range(6,11):
    name=f'g4-{number:03}'; prepared=HERE/f'prepared-{name}'
    metadata=json.loads((prepared/'source.json').read_text())
    archived=next(row for row in cleanup['archives'] if row['commit']==metadata['candidate_commit'])
    assert archived['sha256']==metadata['candidate_archive_sha256']
    private.append({'path':archived['path'],'sha256':archived['sha256'],'commit':archived['commit'],
        'scope':'Reproducible Git archive removed after hash verification to recover host space; raw evidence retained.', 'rebuild':archived['rebuild']})
    for file in prepared.iterdir():
        if file.is_file() and file.suffix!='.tar':add(file,pathlib.Path('harness')/prepared.name/file.name)
    if (prepared/'environment-failure').exists():tree(prepared/'environment-failure',pathlib.Path('evidence')/name/'environment-failure')
    if name in ('g4-008','g4-010'):add(prepared/'source.json',pathlib.Path('evidence')/name/'source.json')
    if (prepared/'not-run.json').exists():add(prepared/'not-run.json',pathlib.Path('evidence')/name/'not-run.json')
for file in HERE.iterdir():
    if file.is_file() and file.suffix in ('.py','.sh','.rs'):add(file,pathlib.Path('harness')/file.name)
tree(HERE/'native-loopback',pathlib.Path('harness/native-loopback'))
for name in ('archive-cache-cleanup.json','partial-target-cleanup.json'):
    add(HERE/name,pathlib.Path('environment')/name)
add(HERE/'cleanup-partial-target.py',pathlib.Path('environment/cleanup-partial-target.py'))
generated['PRIVATE_ORIGINALS.json']=json_bytes(private)
generated['PRIOR_HASH_AUDIT.json']=json_bytes({'scope':'Compare existing log/screenshot/summary bytes against hashes already recorded before the environment fault; excluded binaries are not re-read.',
    'checks':prior_checks,'matched':sum(row['matched'] for row in prior_checks),
    'mismatches':[row for row in prior_checks if not row['matched']],
    'limitation':'Files without a pre-existing hash receive only current copy/manifest integrity, not a historical integrity claim.'})
coverage=(HERE/'native-loopback/COVERAGE.md').read_text().replace('(main.rs)','(harness/native-loopback/main.rs)').replace('(RUNBOOK.md)','(harness/native-loopback/RUNBOOK.md)')
generated['COVERAGE.md']=coverage.encode()
report=['# #7 增量独立验证：环境阻塞交接','',
'待验证候选 `41365a650eb7d7df696046e21b7671697231f851` 的 G4-010 在复制构建缓存时发生 I/O/只读文件系统错误，尚未启动清理、构建或测试，测试数为 0。本包不宣称该候选通过；已执行的最后候选为 G4-009 / `4e6aaf7166fd370076adf43a0dad3e6ecdca36d6`。','',
'仅收录 G4-006–010 本轮小型原件；不重新导出 G4-005 及更早历史。无 NAS、真实凭据或生产树输入，无 push/发布。未运行新的 G5、大树性能或 10k 历史压力。','',
f"故障后核对原件与故障前已有 SHA 记录：[PRIOR_HASH_AUDIT](PRIOR_HASH_AUDIT.json) 共 {len(prior_checks)} 项，匹配 {sum(row['matched'] for row in prior_checks)} 项，不一致/缺失 {sum(not row['matched'] for row in prior_checks)} 项。没有历史 SHA 的文件仅具备本次复制和 manifest 完整性，未冒称故障前后已比对。",'',
'## 失败归属与未执行项','',
'- G4-006：迟到 Read 拒绝预检 0/1/0，真实产品失败，Deny 与预期 Unknown 不符。','- G4-007 首次：复制 target 后没有重编 core，保留为缓存污染，不能当 ca86a85 产品失败；workspace 编译按协调要求中断。',
'- G4-007/fresh-001：强制重编后 runtime 144/0/1，原迟到 Read 通过；workspace 964/1/41 与三包 default 791/1/36 保留同一旧 v6 夹具 trigger 错误。',
'- G4-008：候选已归档但未开始测试，被后续夹具修复候选替代，测试 0 项。','- G4-009：workspace all-features 964/2/42，两个原 900ms 前台重试断言失败；default 956/0/37。release INDEX1 已启动且通过，release INDEX0、runtime release、attr0 按协调要求暂缓，未计通过。',
'- G4-010：环境准备失败，测试 0 项；未执行新时钟用例、原失败复验或最终完整矩阵。','',
'## 逐组实际结果','',
'重复执行组不能相加为唯一测试总数。每轮 inventory 保存精确 suite/name、各组状态与原日志 hash；全部已报告计数与逐名解析一致。','']
for name,rel,summary in runs:
    report += [f'### {name}','',f"候选 `{summary['candidate_commit']}`；[summary]({rel}/summary.json)、[inventory]({rel}/inventory-001.json)。",'',
        '| 命令 | 通过 | 失败 | 忽略 | 结果 |','|---|---:|---:|---:|---|']
    for row in summary['commands']:
        counts=row.get('counts',{}); values=[str(counts.get(k,0)) if counts.get('reported') else '—' for k in ('passed','failed','ignored')]
        label=f"[{row['name']}]({rel}/{row['log']})" if row.get('log') else row['name']
        status='未运行：协调暂缓' if row.get('not_run') else '通过' if row.get('passed') else '失败/中断，见原件'
        report.append('| '+' | '.join([label,*values,status])+' |')
    report.append('')
report += ['## 重试与双钟诊断','',
'G4-009 原失败日志未给出精确 elapsed 或第二包类型，根因尚不能确定。[原 ELF 单跑](evidence/g4-009/clock-probe-001/results.json) 两项通过，797 个双钟样本最大间隔差约 11.139µs，不能抵消原失败。','',
'[派生诊断](evidence/g4-009/clock-diagnostic-001/summary.json) 两项并发 2/0/0、完整 runtime 默认并行 144/0/1。原 900ms 断言保留，但前置插桩读锁/输出可能影响断言，因此仅看提前捕获的原断言点 elapsed：6 条最短 1.186085949s，全部第二包为 Download，attempts=1、retry_at=failed_at+2。本次未捕获早到请求，诊断通过不计候选验收。[测试差异](evidence/g4-009/clock-diagnostic-001/test-only-instrumentation.patch) 与[局限审计](evidence/g4-009/clock-diagnostic-001/diagnostic-audit.json)保留。','',
'[1434 个采样点](evidence/g4-009/clock-diagnostic-001/clock-samples.json)的[异常区间](evidence/g4-009/clock-diagnostic-001/clock-outliers.json)：第1201点距首样本 mono 12.289246558s，realtime 相对 mono 后跳约22.386938317s；第1227点 12.552712513s 前跳约22.383726646s。重试 helper 的第二包此前已完成，不能用此次分歧确定原失败根因，也不能确定宿主暂停、校时或其他来源。','',
'## 环境与清理','',
'[环境失败记录](evidence/g4-010/environment-failure/summary.json)记录复制首个 Errno5、随后 Errno30，以及 WSL 启动 E_UNEXPECTED。[宿主空间原件](evidence/g4-010/environment-failure/windows-filesystem-drives.json)显示 C: 当时仅余54,185,984 bytes；此前 guest df 的可用空间不代表宿主空间。复制完整 stderr 未落盘且工具输出被截断，本包明确保留的是故障观察记录，未伪造完整 trace。','',
'根协调者先核对 SHA 后删除5个可再生 Git tar，共215,050,240 bytes，见[archive 清理原件](environment/archive-cache-cleanup.json)；各 source.json/hash/重建命令保留。标准 WSL df 后可访问且根挂载恢复 rw，未执行 fsck/remount/服务重启。其后只删除从未用于构建或测试的010半成品 target，见[清理记录](environment/partial-target-cleanup.json)及[执行源码](environment/cleanup-partial-target.py)。010 source/logs 与009及更早原件保留。宿主空间仍不足，未重启验证。','',
'## 覆盖边界与复现','',
'[COVERAGE](COVERAGE.md)映射原生 pstream 回环66项、实际 worker ELF 版本握手3项、实际投影→UNIX IPC→Rpc→Bridge 回归。它们不是单一真实 worker 网络全链测试，更不是 NAS 实测。独立历史5项与标签模型3项验证 uncertain/cleanup_uncertain 不盲重发及完成后提示清除而 detail 保留。','',
'INDEX0 两项必须有 NSI 的 continuation 用例明确 N/A；doc-ignore 与本轮未跑 core 210k 功能 ignored 单列，见 inventory。本次未声称所有 ignored 均已运行。未实测边界仍包括真实 NAS ACL、整空间能力来源、服务端部分提交真值、成对安装/重装及真实系统桌面面板。','',
'命令见各 summary，[harness](harness/) 保留冻结 runner、入口与辅助源码；[PRIVATE_ORIGINALS](PRIVATE_ORIGINALS.json)记录排除的大产物位置与已保存 hash。原始日志、诊断、失败、截图不覆盖；本包没有 ELF、target、源代码大归档、数据库或真实 manifest 名称。[MANIFEST](MANIFEST.json)覆盖全部小文件。','']
generated['REPORT.md']='\n'.join(report).encode()
generated['STATUS.json']=json_bytes({'acceptance_status':'blocked_environment','pending_candidate':'41365a650eb7d7df696046e21b7671697231f851',
    'latest_tested_candidate':'4e6aaf7166fd370076adf43a0dad3e6ecdca36d6','g4_010_test_commands_started':0,
    'scope':'G4-006 through G4-010 only; no G5, no older G4 export'})
estimate=sum(p.stat().st_size for p in files.values())+sum(map(len,generated.values()))
# The final manifest is small; reserve 256 bytes per pathname for conservative estimate.
upper=estimate+sum(len(name.encode())+256 for name in [*files,*generated])+4096
print(json.dumps({'planned_files':len(files)+len(generated),'known_bytes':estimate,'upper_estimated_bytes':upper,
    'under_100MiB':upper<100*1024*1024,'export_requested':args.export_under_100mib}),flush=True)
if not args.export_under_100mib:raise SystemExit(0)
assert upper<100*1024*1024,'report size before any export above limit'
assert shutil.disk_usage(HERE).free>upper+32*1024*1024,'preserve host free-space margin'
args.output.mkdir()
for name,source in files.items():
    destination=args.output/name;destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,destination)
    assert sha(destination)==sha(source),'copy differs: '+name
for name,content in generated.items():
    destination=args.output/name;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(content)
manifest={'acceptance_status':'blocked_environment','pending_candidate':'41365a650eb7d7df696046e21b7671697231f851',
    'files':{name:sha(args.output/name) for name in sorted([*files,*generated])}}
(args.output/'MANIFEST.json').write_bytes(json_bytes(manifest))
print(json.dumps({'output':str(args.output),'files':len(manifest['files']),'bytes':sum(p.stat().st_size for p in args.output.rglob('*') if p.is_file()),
    'manifest_sha256':sha(args.output/'MANIFEST.json')}),flush=True)
