"""Make a self-contained small evidence bundle; no source/target/database copies."""
import argparse,hashlib,json,pathlib,shutil,subprocess,sys
BASE=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
HERE=pathlib.Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--runs',nargs='+',required=True);p.add_argument('--final',required=True);p.add_argument('--output',type=pathlib.Path,required=True)
a=p.parse_args();assert a.final in a.runs and not a.output.exists()
def sha(file):
    with file.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
out=a.output;out.mkdir(parents=True);rows=[];private=[]
ALLOWED_SUFFIXES={'.json','.log','.py','.sh','.rs','.toml','.lock','.md','.png','.sha256','.txt','.in','.patch'}
def copy(source,destination):
    assert source.is_file() and not source.is_symlink(),source
    assert source.suffix in ALLOWED_SUFFIXES,source
    destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,destination)
def copy_directory(source,destination):
    assert source.is_dir() and not source.is_symlink(),source
    for file in source.rglob('*'):
        assert not file.is_symlink(),file
        if file.is_file():copy(file,destination/file.relative_to(source))
def export_run(path,relative,final=False):
    summary=json.loads((path/'summary.json').read_text())
    if final:
        assert summary['complete'] and summary['all_commands_passed'],'final matrix must pass completely'
        assert not summary['changed_source_files'] and not summary['residual_owned_mounts']
    inventory=path/'inventory-001.json'
    if inventory.exists():
        parsed=json.loads(inventory.read_text())
        assert parsed['source_summary_sha256']==sha(path/'summary.json') and parsed['all_reported_counts_reconciled']
        copy(inventory,out/relative/'inventory-001.json')
    else:
        inventory=out/relative/'inventory-001.json';inventory.parent.mkdir(parents=True,exist_ok=True)
        subprocess.run([sys.executable,str(HERE/'analyze-g4.py'),'--run',str(path),'--output',str(inventory)],check=True)
        parsed=json.loads(inventory.read_text())
        assert parsed['source_summary_sha256']==sha(path/'summary.json') and parsed['all_reported_counts_reconciled']
    for name in ['summary.json','source.json','source-hashes-before.json','failure.txt','coordinator-stop.json','coordinator-control.json','cache-provenance.json','final-integrity.json','native-loopback-results.json','native-loopback-elf.sha256','worker-versions.json','retry-preflight-clocks.json','retry-preflight-clock-audit.json']:
        if (path/name).is_file():copy(path/name,out/relative/name)
    for folder in ['logs','acl-screenshots','screenshots']:
        if (path/folder).exists():copy_directory(path/folder,out/relative/folder)
    for folder in ['native-loopback-crate','core-history-probe']:
        root=path/folder
        if not root.exists():continue
        for name in ['Cargo.toml','Cargo.lock','provenance.json','lock-verification.json','metadata.log','metadata.json','cargo-metadata.json','src/main.rs','src/lib.rs']:
            if (root/name).is_file():copy(root/name,out/relative/folder/name)
    rows.append((str(relative),summary))
    for file in path.glob('old-artifacts/*'):
        if file.is_file():private.append({'path':str(file),'sha256':sha(file),'scope':'saved original test ELF; deliberately excluded from portable bundle'})
    for name in ['native-loopback-elf.sha256','source.json','source-hashes-before.json']:
        if (path/name).is_file():private.append({'path':str(path/name),'sha256':sha(path/name)})
    # These are diagnostic artifacts, not additional candidate acceptance runs.
    diagnostic_files={
        'clock-probe-001':['results.json','clock-samples.json','case-1.log','case-2.log'],
        'clock-diagnostic-001':['summary.json','test-only-instrumentation.patch','clean-products.log','build-diagnostic.log',
            'two-original-cases-concurrent.log','runtime-full-original-parallelism.log','clock-samples.json','clock-outliers.json','diagnostic-audit.json']}
    for folder,names in diagnostic_files.items():
        for name in names:
            if (path/folder/name).is_file():copy(path/folder/name,out/relative/folder/name)
    probe=path/'clock-probe-001/results.json'
    if probe.exists():
        data=json.loads(probe.read_text())
        private.append({'path':data['copied_elf'],'sha256':data['copied_elf_sha256'],'scope':'unchanged original test ELF used only for clock probe'})
    diagnostic=path/'clock-diagnostic-001/summary.json'
    if diagnostic.exists():
        data=json.loads(diagnostic.read_text())
        private.append({**data['binary'],'scope':'test-only instrumented diagnostic ELF; no candidate acceptance claim'})
for name in a.runs:
    assert name.startswith('g4-') and '/' not in name and '..' not in name
    path=BASE/name;export_run(path,pathlib.Path('evidence')/name,name==a.final)
    for sub in sorted(path.glob('fresh-*')):
        if (sub/'summary.json').is_file():export_run(sub,pathlib.Path('evidence')/name/sub.name)
    archive=HERE/f'prepared-{name}'
    if archive.exists():
        for file in archive.iterdir():
            if file.is_file() and file.suffix!='.tar':copy(file,out/'harness'/archive.name/file.name)
        if (archive/'candidate.tar').is_file():private.append({'path':str(archive/'candidate.tar'),'sha256':sha(archive/'candidate.tar'),'scope':'immutable git archive excluded for size'})
for file in HERE.iterdir():
    if file.is_file() and file.suffix in {'.py','.sh','.rs'}:copy(file,out/'harness'/file.name)
for archive in sorted(HERE.glob('prepared-g4-*')):
    if (archive/'not-run.json').is_file():
        for file in archive.iterdir():
            if file.is_file() and file.suffix!='.tar':copy(file,out/'harness'/archive.name/file.name)
        private.append({'path':str(archive/'candidate.tar'),'sha256':sha(archive/'candidate.tar'),'scope':'prepared but never executed; original archive retained'})
copy_directory(HERE/'native-loopback',out/'harness/native-loopback')
copy(HERE/'native-loopback/COVERAGE.md',out/'COVERAGE.md')
# Rewrite only the portable mapping's direct design links to its bundled source.
coverage=(out/'COVERAGE.md').read_text().replace('(main.rs)','(harness/native-loopback/main.rs)').replace('(RUNBOOK.md)','(harness/native-loopback/RUNBOOK.md)')
(out/'COVERAGE.md').write_text(coverage)
(out/'PRIVATE_ORIGINALS.json').write_text(json.dumps(private,indent=2,ensure_ascii=False)+'\n')
final=next(summary for name,summary in rows if name==f'evidence/{a.final}')
prior=next(summary for name,summary in rows if name=='evidence/g4-007/fresh-001')
prior_commands={row['name']:row for row in prior['commands']}
for name,expected in [('runtime-lib-all-features',(144,0,1)),('workspace-all-features',(964,1,41)),('core-fs-runtime-default',(791,1,36))]:
    assert tuple(prior_commands[name]['counts'][key] for key in ('passed','failed','ignored'))==expected
report=['# #7 原生响应与权限失效独立验收','',
f"最终候选 `{final['candidate_commit']}`；最终主矩阵全部通过：`{final.get('all_commands_passed')}`。逐项结果以原始日志和 inventory 为准。不同命令的重复执行不相加为唯一覆盖。",'',
'全部输入为本地合成夹具或既有冻结测试数据。未连接 NAS、未读取真实凭据、未扫描生产树、未 push 或发布。未重跑 G5、大树性能及 10k 历史压力。', '',
'## 早期失败与构建来源','',
'G4-006 的 d39a597 迟到 Read 拒绝预检 0/1/0，Deny 与预期 Unknown 不符，是保留的产品失败。G4-007 的首次同断言来自复制 target 后旧 core 的构建缓存，不能当作 ca86a85 产品失败。其 workspace 阶段按协调者要求中断。fresh-001 保存旧 ELF hash 后，清理本轮私有 target 的产品包并仅更新隔离源码 mtime；源码内容 SHA 前后相同，重新编译的 runtime 全 lib 144/0/1 包含原断言通过。所有初始日志、协调中断和 fresh 日志分开保留。最终 run 统一使产品缓存失效。','',
'G4-007/fresh 的 workspace 964/1/41 与 core/fs/runtime default 791/1/36 均保留同一 v6 夹具错误：新 trigger 引用已被夹具删除的 kind。针对该 v6 失败的修复只为旧 v6/v23 夹具补 DROP 该 trigger，不改变迁移断言。G4-008 的 99aa587 归档已准备但在测试前停止；测试数为0，未启动记录独立保留。','',
'G4-009 的 workspace all-features 964/2/42 保留两项原 900ms 前台重试断言失败；default 956/0/37 及其他已执行组通过，三个 release 组按协调要求暂缓。原日志没有第二包类型和精确 elapsed，不能确定这两项失败的根因。后续原 ELF 单跑与 test-only 派生插桩运行均作为独立诊断保留，不能抵消原失败。', '',
'## 实测分组','']
for relative,summary in rows:
    report += [f"### {relative}",'',f"候选 `{summary['candidate_commit']}`；complete=`{summary.get('complete')}`；[summary]({relative}/summary.json)、[逐项 inventory]({relative}/inventory-001.json)。",'',
               '| 命令 | 通过 | 失败 | 忽略 | 执行结论 |','|---|---:|---:|---:|---|']
    for command in summary['commands']:
        counts=command.get('counts',{});reported=counts.get('reported',False)
        values=[str(counts.get(k,0)) if reported else '—' for k in ('passed','failed','ignored')]
        status='协调者暂缓，未运行' if command.get('not_run') else '通过' if command.get('passed') else '未通过/中断（见原件）'
        label=f"[{command['name']}]({relative}/{command['log']})" if command.get('log') else command['name']
        report.append('| '+ ' | '.join([label,*values,status])+' |')
    report.append('')
report += ['## 独立重试诊断','',
'[原 ELF 单跑](evidence/g4-009/clock-probe-001/results.json) 两项通过；797 个相邻双钟样本中最大间隔差约 11.139µs。[派生源码诊断](evidence/g4-009/clock-diagnostic-001/summary.json) 保留原断言，但诊断日志/Store 读锁位于断言前，故其通过结果不计候选验收。提前捕获的原断言点 elapsed 最短 1.186085949s，6 条第二包记录均为 Download，attempts=1，retry_at=failed_at+2。两个 case 并发 2/0/0、runtime 全 lib 144/0/1 只说明本次诊断没有捕获早到请求。', '',
'[双钟异常样本](evidence/g4-009/clock-diagnostic-001/clock-outliers.json) 显示：1434 个样本的第 1201 点（距首样本 mono 12.289246558s）realtime 相对 mono 后跳约 22.386938317s，第 1227 点（12.552712513s）前跳约 22.383726646s。各次读取均由相邻 mono 读数夹住。所有重试 helper 的第二包此前已完成；该观测不确定原 workspace 失败的根因，也不说明宿主暂停或校时来源。[插桩差异](evidence/g4-009/clock-diagnostic-001/test-only-instrumentation.patch) 与原始构建日志保留，派生源码和 ELF 在私有目录。', '',
'## 覆盖与边界','',
'[COVERAGE](COVERAGE.md) 列出实际原生回环用例及产品回归入口。66 项公开 Client 的真实 TCP/pstream 回环与 3 项真实 worker ELF 版本握手分别证明各自边界；权限应用链另由真实投影函数→UNIX IPC→Rpc→Bridge 回归覆盖，不合称单一全链 worker 网络测试。', '',
'下载响应仅 0x3002/0x3008 可形成 Read 拒绝；3004/8004/9005/a005 误分类排除与宽整数拒绝均有实测。mutation 的 action-scoped 权限提示保持 uncertain/cleanup_uncertain，不声明无部分提交，不修改 ACL，不因 recheck 盲重发。', '',
'独立 core 历史补测验证 upload/mkdir/replace 的确认或冲突更新错误提示，并保留 detail.permission_response；replace 保留的是最后一次清理响应。GUI 模型补测核对中文动作标签、完成后 tooltip 清除提示和结构化 detail 保留。', '',
'INDEX0 两项要求 NSI 的 continuation 测试明确 N/A，其他 FUSE 测试仍执行；常规 cargo 中 ignored 内核测试由专用组复跑。另单列文档示例忽略与本轮未运行的 core 大树功能用例 `status_snapshot::tests::default_budget_admits_210k_node_snapshot_without_holding_store`；后者的先前候选原件另存，不计最终候选通过，也不能用未跑性能基准替代其功能覆盖边界。准确逐名状态见 inventory。真实 NAS ACL、空间能力来源、服务端部分提交行为、成对安装/重装与真实系统桌面面板仍未实测。', '',
'## 原件与复现','',
'全部命令在各 summary，harness 在 [harness](harness/)。候选 tar、旧 ELF 与大 target 不打包；私有原件位置及摘要见 [PRIVATE_ORIGINALS](PRIVATE_ORIGINALS.json)。包内逐文件 hash 见 [MANIFEST](MANIFEST.json)。未包含数据库或真实 manifest 成员名称。','']
(out/'REPORT.md').write_text('\n'.join(report))
for file in out.rglob('*'):
    assert not file.is_symlink(),file
    assert not {'target','cache','old-artifacts','native-loopback-scratch'}.intersection(file.relative_to(out).parts),file
    if file.is_file():
        assert file.suffix in ALLOWED_SUFFIXES,file
        with file.open('rb') as stream:magic=stream.read(16)
        assert not magic.startswith((b'\x7fELF',b'SQLite format 3\x00',b'!<arch>\n',b'MZ')),file
files={str(file.relative_to(out)):sha(file) for file in out.rglob('*') if file.is_file()}
(out/'MANIFEST.json').write_text(json.dumps({'final_candidate':final['candidate_commit'],'files':files},indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'output':str(out),'files':len(files),'bytes':sum(f.stat().st_size for f in out.rglob('*') if f.is_file()),'manifest_sha256':sha(out/'MANIFEST.json')}))
