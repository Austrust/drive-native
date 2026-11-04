"""After END only: package numeric G5 evidence without real names or large artifacts."""
import argparse,datetime,hashlib,json,re,shutil,statistics
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument('--exclusive-window-ended',action='store_true')
parser.add_argument('--bundle-number',default='003')
args=parser.parse_args();assert args.exclusive_window_ended
assert re.fullmatch(r'\d{3}',args.bundle_number)
here=Path(__file__).resolve().parent
base=Path('/home/drive-native-test/.local/state/drive-native-ultra-benchmark')
load=lambda p:json.loads(p.read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
primary=load(base/'primary-001/summary.json');diag=load(base/'diagnostic-001/summary.json')
build=load(base/'build-002/build.json')
assert primary['complete'] and diag['complete'] and (base/'window-001/end.txt').exists()
assert primary['rounds']==3 and diag['rounds']==1
assert primary['build']['candidate_commit']==diag['build']['candidate_commit']==build['candidate_commit']
assert primary['build']==diag['build']==build
assert sha(here/'run-paired.py')==primary['harness_sha256']==diag['harness_sha256']
for summary in (primary,diag):
    assert not summary['residual_owned_mounts']
    for case in summary['cases']:
        assert case['shutdown']['normal'] and not case['shutdown']['forced_cleanup'] and not case['shutdown']['residual_mount']
        if 'directory_probe' in case:
            assert case['directory_probe']['diagnostics_delta'].get('EntryTtlNonZero',{}).get('count',0)==0
        for scan in case['scans']:
            assert scan['fresh_index_save_verified'] and scan['exit_code']==0
            assert not any(scan[k] for k in ('missing','extra','duplicates','invalid','attribute_mismatches','hydration_before','hydration_after'))
bundle=here/('G5-evidence-bundle-'+args.bundle_number);bundle.mkdir(exist_ok=False)
(bundle/'harness').mkdir();(bundle/'build').mkdir();(bundle/'load').mkdir();(bundle/'window').mkdir()
for name in ('run-paired.py','g5-window-001.sh','build-candidate-reuse.py','build-fixtures.sh','build-g5-002.sh','prepare-source.ps1',
             'start-windows-load.ps1','sample-windows-load.ps1','bundle-g5.py','check-bundle.py'):
    shutil.copy2(here/name,bundle/'harness'/name)
shutil.copy2(base/'build-002/build.json',bundle/'build/build.json')
shutil.copytree(base/'build-002/logs',bundle/'build/logs')
for p in (base/'window-001').iterdir():
    if p.is_file():shutil.copy2(p,bundle/'window'/p.name)
shutil.copy2(here/'G5-window-authorization.json',bundle/'window/authorization.json')
shutil.copy2(here/'G5-window-end.json',bundle/'window/end-verification.json')
for name in ('windows-load-001','windows-load-002'):
    if (here/name).exists():shutil.copytree(here/name,bundle/'load'/name)

private={};scan_count=0
for name,summary in [('primary-001',primary),('diagnostic-001',diag)]:
    target=bundle/name;target.mkdir()
    shutil.copy2(base/name/'summary.json',target/'summary.json')
    for case in summary['cases']:
        relative=Path(case['tree'])/f"{case['round']}-{case['role']}"
        original=base/name/relative;out=target/relative;out.mkdir(parents=True)
        for state in ('first','repeat'):
            path=original/(state+'.result.json');shutil.copy2(path,out/path.name);scan_count+=1
        # Never copy real-manifest logs, SQLite, exported paths, truth or mirrors.
        # Hash only fixed artifact filenames, not any user-derived tree name.
        for artifact in ('first.log','repeat.log','first.audit.log','repeat.audit.log','first.index.tsv','repeat.index.tsv',
                         'fixture.stderr','fixture/truth.json','profile/data/fsearch/fsearch.db'):
            p=original/artifact
            if p.is_file():
                private[str(p)]={'sha256':sha(p),'bytes':p.stat().st_size}
(bundle/'private-originals.json').write_text(json.dumps({'scope':'originals retained privately; paths are harness artifact names, not manifest member names',
    'roots':[str(base/'primary-001'),str(base/'diagnostic-001')],'artifacts':private},indent=2)+'\n')

trees={'100k':(102130,2066),'dense-319k':(319679,56073),'manifest-327750':(327750,56554)}
clock_records=[]
for mode,summary in [('primary',primary),('diagnostic',diag)]:
    for case in summary['cases']:
        for scan in case['scans']:
            span=(datetime.datetime.fromisoformat(scan['end']['utc'])-datetime.datetime.fromisoformat(scan['start']['utc'])).total_seconds()
            clock_records.append({'mode':mode,'tree':case['tree'],'role':case['role'],'round':case['round'],
                'state':scan['cache_state'],'monotonic_elapsed_seconds':scan['elapsed_seconds'],
                'utc_span_seconds':span,'utc_minus_monotonic_seconds':span-scan['elapsed_seconds'],
                'utc_over_monotonic_ratio':span/scan['elapsed_seconds']})
clock_groups=[]
for mode in ('primary','diagnostic'):
    for tree in trees:
        for role in ('baseline','candidate','local'):
            for state in ('first','repeat'):
                records=[r for r in clock_records if (r['mode'],r['tree'],r['role'],r['state'])==(mode,tree,role,state)]
                clock_groups.append({'mode':mode,'tree':tree,'role':role,'state':state,'n':len(records),
                    'median_monotonic_elapsed_seconds':statistics.median(r['monotonic_elapsed_seconds'] for r in records),
                    'median_utc_span_seconds':statistics.median(r['utc_span_seconds'] for r in records)})
clock_audit={'records':clock_records,'groups':clock_groups,
    'ordered_scan_events':['UTC start=stamp()', 'load_before=load_sample()', 'argv construction',
        'usage_before=resource.getrusage(RUSAGE_CHILDREN)', 'monotonic start',
        'scan log.open', 'Popen/wait', 'scan log.close', 'monotonic stop',
        'usage_after=resource.getrusage(RUSAGE_CHILDREN)', 'UTC end=stamp()',
        'load_after=load_sample()', 'scan log SHA and saved-index checks', 'audit and truth validation'],
    'monotonic_boundary':'after usage_before; before scan log.open/Popen; ends after process.wait and log.close, before usage_after; no load_after/SHA/audit',
    'utc_boundary':'start precedes load_before/argv/getrusage/monotonic start; end follows monotonic stop/getrusage but precedes load_after/SHA/audit; endpoints differ',
    'fixture_cpu_boundary':'fixture /proc CPU read before pre-scan sample command; scan() includes audit/validation; second /proc CPU read after scan() returns and before post-scan sample command; therefore includes audit interval',
    'client_cpu_boundary':'RUSAGE_CHILDREN before monotonic start and after monotonic stop; FSearch and dbus/Xvfb helpers, excluding later audit subprocess',
    'attribution':'difference is preserved; sampling-boundary overhead and any time-source behavior were not isolated; no claim about host suspension or clock adjustment'}
(bundle/'clock-audit.json').write_text(json.dumps(clock_audit,indent=2)+'\n')
clock_rows=[f'| {r["tree"]} | {r["state"]} | {r["median_monotonic_elapsed_seconds"]:.6f} | {r["median_utc_span_seconds"]:.6f} |'
    for r in clock_groups if r['mode']=='primary' and r['role']=='candidate']
max_clock_delta={mode:max(abs(r['utc_minus_monotonic_seconds']) for r in clock_records if r['mode']==mode) for mode in ('primary','diagnostic')}
ag={(r['tree'],r['role'],r['state']):r for r in primary['aggregates']}
num=lambda x:f'{x:.3f}'
yes=lambda v:'通过' if v else '未达'
rows=[];targets=[]
for tree,(entries,dirs) in trees.items():
    for state in ('first','repeat'):
        b,c,l=[ag[tree,role,state] for role in ('baseline','candidate','local')]
        goal=c['target_seconds'];target_text='—' if goal is None else f'{goal:g}s：{yes(c["target_met"])}'
        rows.append(f'| {tree} | {state} | {num(b["median_seconds"])} | {num(c["median_seconds"])} | {num(l["median_seconds"])} | {c["median_seconds"]/l["median_seconds"]:.2f}× | {target_text} | {c["median_ms_per_directory"]:.4f}：{yes(c["directory_target_met"])} |')
        targets.append({k:c[k] for k in ('tree','state','median_seconds','target_seconds','target_met','median_ms_per_directory','directory_target_ms','directory_target_met')})

index_rows=[];cpu_rows=[];request_rows=[];elapsed_rows=[];probe_rows=[];diagnostic_cpu_rows=[]
for tree in trees:
    cases=[c for c in primary['cases'] if c['tree']==tree and c['role']=='candidate']
    for case in cases:
        n=case['ready']['namespace_index']
        index_rows.append(f'| {tree} | {case["round"]} | {n.get("index_build_us",0)/1e6:.6f} | {n.get("index_bytes")} | {n.get("index_partial")} | {n.get("index_degraded")} |')
    for role in ('baseline','candidate','local'):
        for state in ('first','repeat'):
            c=ag[tree,role,state]
            cpu_rows.append(f'| {tree} | {role} | {state} | {c["median_seconds"]:.3f} | {c["median_fixture_cpu_seconds"]:.3f} | {c["median_client_cpu_seconds"]:.3f} |')
for case in diag['cases']:
    if case['role']!='candidate':continue
    for scan in case['scans']:
        d=scan['diagnostics_delta'];count=lambda k:d.get(k,{}).get('count',0)
        diagnostic_cpu_rows.append(f'| {case["tree"]} | {scan["cache_state"]} | {scan["elapsed_seconds"]:.3f} | {scan["fixture_cpu_seconds"]:.3f} | {scan["client_tree_user_seconds"]:.3f} | {scan["client_tree_system_seconds"]:.3f} |')
        request_rows.append(f'| {case["tree"]} | {scan["cache_state"]} | {count("LookupRequests")} | {count("GetattrRequests")} | {count("OpendirRequests")} | {count("ReaddirRequests")} | {count("ReaddirPlusRequests")} | {count("Releasedir")} | {count("IndexHits")} | {count("IndexFallback")} | {count("IndexDegraded")} |')
        for stage in ('IndexReadUs','LookupQueue','LookupStoreWait','LookupProjection','DirectoryStoreWait','DirectoryProjection','ReaddirReply','ReaddirPlusProjection'):
            item=d.get(stage,{})
            elapsed_rows.append(f'| {case["tree"]} | {scan["cache_state"]} | {stage} | {item.get("count",0)} | {item.get("total_us",0)/1e6:.6f} |')
    p=case['directory_probe']
    probe_rows.append(f'| {case["tree"]} | {p["directories"]} | {p["elapsed_seconds"]:.3f} | {p["scandir_inflight_seconds"]:.3f} | {p["scandir_inflight_seconds"]*1000/p["directories"]:.4f} | {p["fixture_cpu_seconds"]:.3f} |')

def guest_load(s):
    a=s['start_load']['cpu_ticks'];b=s['end_load']['cpu_ticks'];delta=[y-x for x,y in zip(a,b)]
    return {'samples':len(s['wsl_guest_samples']),'average_guest_busy_percent':100*(1-(delta[3]+delta[4])/sum(delta[:8])),
            'start_loadavg':s['start_load']['loadavg'],'end_loadavg':s['end_load']['loadavg']}
load_summary={'windows_host':'未实测；CIM资源无法访问；错误及失败启动原件保留',
              'wsl_guest':{'primary':guest_load(primary),'diagnostic':guest_load(diag)}}
machine={'candidate_commit':build['candidate_commit'],'baseline_commit':build['baseline_commit'],
         'timing_basis':'time.monotonic elapsed; wider UTC spans separately audited, not strict host wall clock',
         'correctness_all_passed':True,'scan_count':scan_count,'primary_scans':54,'diagnostic_scans':18,
         'tree_and_directory_targets':targets,'index_ready_without_degradation':primary['index_ready_without_degradation'],
         'diagnostic_index_hits_all_scans':diag['index_hits_all_scans'],'load':load_summary,
         'all_mounts_stopped_normally':True,'residual_mounts':[],
         'performance_complete':True,'all_specified_performance_targets_passed':all(t['directory_target_met'] and t['target_met'] is not False for t in targets)}
machine['performance_target_counts']={
    'tree_passed':sum(t['target_met'] is True for t in targets),
    'tree_failed':sum(t['target_met'] is False for t in targets),
    'tree_not_specified':sum(t['target_met'] is None for t in targets),
    'directory_passed':sum(t['directory_target_met'] is True for t in targets),
    'directory_failed':sum(t['directory_target_met'] is False for t in targets)}
(bundle/'result.json').write_text(json.dumps(machine,indent=2,ensure_ascii=False)+'\n')
report=f'''# G5 固定样本配对基准

固定候选 `{build['candidate_commit']}` 与基线 `{build['baseline_commit']}`。三树各3轮first/repeat完成，另3树各1轮诊断，合计72次保存索引扫描。missing/extra/duplicate/invalid/属性不符均为0，hydration均为0，全部自建fixture正常停止、无残留挂载。性能是否达到规格必须分别看树级秒数与≤0.15ms/目录列；不能把局部改善称为近本地达标。

原始数字见 [primary summary](primary-001/summary.json)、[diagnostic summary](diagnostic-001/summary.json)、[机器结论](result.json)。窗口primary为{primary['start']['utc']}至{primary['end']['utc']}，diagnostic为{diag['start']['utc']}至{diag['end']['utc']}；START授权和完整启动命令记录于window/及harness/。环境为WSL2 Ubuntu24.04 ext4普通用户，非Mint实机。全程未连接NAS/凭据/生产文件、未push或发布。

## 主结果

单位为**monotonic实测elapsed秒**；每格为3个固定样本中位数，完整逐轮原值保留。每目录成本为FSearch总monotonic elapsed毫秒/含root目录数，包含client/helper启动、遍历、排序及保存，不是FUSE净成本，也不冒充严格宿主墙钟。树规模100k=102,130项/2,066目录；dense=319,679项/56,073目录；既有真实manifest=327,750项/56,554目录。后者只读取事先已保存清单，未扫描用户源目录。

| 树 | 状态 | .9 | 候选 | 本地 | 候选/本地 | 树级目标 | 候选ms/目录：≤0.15 |
|---|---|---:|---:|---:|---:|---|---|
{chr(10).join(rows)}

同口径本地first每目录成本：100k={ag['100k','local','first']['median_ms_per_directory']:.4f}ms、dense={ag['dense-319k','local','first']['median_ms_per_directory']:.4f}ms、manifest={ag['manifest-327750','local','first']['median_ms_per_directory']:.4f}ms。所有角色逐轮成本在原JSON中；不因本地自身的启动成本而改写规格门槛。

首扫为新fixture与新FSearch数据库，不清宿主页缓存。单调钟计时在fixture ready/索引构建完成后、打开scan log及Popen之前开始，覆盖dbus-run-session、xvfb-run与原版FSearch，至process.wait完成；随后audit不在此elapsed内。本地参考由同清单真实size/mtime的稀疏文件构造，非空文件镜像；FUSE两列始终扫描真实挂载。正entry TTL始终0，attr TTL固定1000ms，无放宽TTL、过滤挂载或调大预算。

## UTC跨度与单调钟边界

[逐扫描clock-audit](clock-audit.json)保存全部72组原值和分组中位数。实际顺序为UTC start→load_before→argv/getrusage→monotonic start→log.open/Popen/wait/log.close→monotonic stop→getrusage→UTC end→load_after/SHA→audit。UTC end**不包含load_after、SHA与audit**；UTC跨度与monotonic elapsed**不是完全同一采样区间**。最大绝对差为主样本{max_clock_delta['primary']:.6f}s、诊断{max_clock_delta['diagnostic']:.6f}s。边界开销与时间源差异没有独立测量，原因未定位，不能归因于宿主暂停或校时。阈值与上述主表均保留原monotonic口径，未用UTC差替换或修正数据。fixture CPU另由外层采样包围scan/audit，不能误作仅FSearch计时区间内的CPU。

| 候选树 | 状态 | monotonic中位数秒 | UTC跨度中位数秒 |
|---|---|---:|---:|
{chr(10).join(clock_rows)}

## 索引准备成本和实际命中

预装成本未计入扫描elapsed，以下单独保留候选每轮ready原值；bytes为0且degraded为true的样本仍按实际SQL回退测量，未重跑挑样。普通ELF未启用诊断，ready不能单独证明每次请求实际命中；命中计数来自下方另跑的诊断ELF。

| 树 | 轮 | 构建秒 | bytes | partial | degraded |
|---|---:|---:|---:|---|---|
{chr(10).join(index_rows)}

## 可测时间分解

以下主样本的fixture CPU与client进程树CPU是实测CPU秒，client含启动辅助进程；不能从elapsed相减当作内核耗时。fixture采样围住scan/audit调用，audit只访问已保存本地索引，未读取挂载正文。表内中位数分别计算，不强行凑和。

| 树 | 角色 | 状态 | 主monotonic秒 | fixture CPU秒 | client/helpers CPU秒 |
|---|---|---|---:|---:|---:|
{chr(10).join(cpu_rows)}

另1轮诊断实际请求计数如下，**不与主3轮混算**：

| 树 | 状态 | lookup | getattr | opendir | readdir | readdirplus | releasedir | IndexHits | IndexFallback | IndexDegraded |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
{chr(10).join(request_rows)}

同一候选诊断扫描的CPU范围：client user/system均覆盖FSearch及dbus/Xvfb辅助进程，system CPU不是独立的纯FUSE内核往返计时。

| 树 | 状态 | 诊断monotonic秒 | fixture CPU秒 | client user秒 | client system秒 |
|---|---|---:|---:|---:|---:|
{chr(10).join(diagnostic_cpu_rows)}

诊断elapsed阶段允许重叠、嵌套。IndexReadUs的`_index_time`作用域在**命中时**覆盖索引路径直到reply调用返回，含reply/lease范围；**回退时**仅覆盖lease/索引尝试，退出该块后才走SQL与reply，不含后续SQL路径。它**不是纯HashMap读取，也不是CPU归因**；以下列项不能相加或互减成互斥分解。100k的106,210次LOOKUP及SQL projection计数0是实测请求路径事实，不能据此宣称未经独立跟踪的硬性能下限。DirectoryInodeWrites与IndexBytes的total_us字段分别装行数/bytes，不当作微秒；完整计数与直方图保留在原JSON。所有诊断EntryTtlNonZero=0。

| 树 | 状态 | 阶段 | 次数 | 聚合elapsed秒 |
|---|---|---|---:|---:|
{chr(10).join(elapsed_rows)}

诊断另测串行目录scandir：每目录在途是monotonic计时的scandir调用总时间/实际调用数，包含Python、FUSE处理、内核与调度；与上文总FSearch elapsed/目录分列，不混称。

| 树 | 目录数 | probe monotonic秒 | scandir在途秒 | 在途ms/目录 | fixture CPU秒 |
|---|---:|---:|---:|---:|---:|
{chr(10).join(probe_rows)}

纯内核往返/传输成本 **未单独实测**。这些计数、CPU及elapsed保留了差距证据，不把剩余elapsed武断归给某一未测阶段。

## 负载与证据边界

Windows2s CIM采样初次隐藏启动未产出样本；受控session尝试得到“无法从客户端中访问CIM资源”，保存错误后停止。Windows宿主CPU/内存 **未实测**，未安装或修改系统设置。错误原件见load/，不能把WSL guest数据冒充宿主安静证明。WSL逐秒/proc/stat与loadavg、开始/结束进程名快照均在两个summary中；主窗口guest平均busy={load_summary['wsl_guest']['primary']['average_guest_busy_percent']:.2f}%，诊断={load_summary['wsl_guest']['diagnostic']['average_guest_busy_percent']:.2f}%，这是包含树准备/audit在内的整体窗口观察值，不是每个扫描独占CPU使用率。

[build.json](build/build.json)保存四ELF、同源fixture、归档与toolchain哈希；879个基线归档文件（同源fixture除外）逐一验证仍为d555944，基线ELF复用证明与构建原log保留。普通与诊断分开，无产品RUSTFLAGS，基线example仅用drive_fs_baseline去除新NSI API。FSearch与audit为预存的原版工具，hash已逐次核对。

本包保留numeric summary、逐scan结果、负载、完整命令/harness/hash；真实清单成员名、audit TSV、索引数据库、truth、稀疏镜像、大型ELF不入包。完整私有原件保留于[位置与hash清单](private-originals.json)，其路径仅使用harness固定artifact名。失败或未达结果均未删除或覆盖。
'''
(bundle/'REPORT.md').write_text(report)
(bundle/'README.md').write_text('# G5证据\n\n- [报告](REPORT.md)\n- [机器结果](result.json)\n- [完整私有原件hash](private-originals.json)\n')
manifest={'candidate_commit':build['candidate_commit'],'files':{}}
for p in bundle.rglob('*'):
    if p.is_file():manifest['files'][str(p.relative_to(bundle))]=sha(p)
(bundle/'MANIFEST.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'bundle':str(bundle),'files':len(manifest['files']),'manifest_sha256':sha(bundle/'MANIFEST.json')}))
