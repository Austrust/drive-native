"""Publish only completed final G4 facts to the independent work-directory report."""
import json
from pathlib import Path

here=Path(__file__).resolve().parent
base=Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
run=base/'g4-005'
load=lambda p:json.loads(p.read_text())
main=load(run/'summary.json');history=load(run/'history-scale-verified/results.json')
functional=load(base/'functional-g4-005/result.json')
integrity=load(run/'final-integrity-post-history.json')
assert main['all_commands_passed'] and history['passed'] and functional['complete'] and integrity['passed']
assert load(run/'supplement/summary.json')['all_passed']
assert load(base/'schema-g4-005/results.json')['complete']
assert load(run/'inventory-001.json')['all_reported_counts_reconciled']
commit=main['candidate_commit']; assert commit=='bae1ef1ddc4061df586221557011f4e30f04d134'
text=(here/'G4_REPORT.md').read_text()
text=text.replace('c744729141ca8b23bde63c61ca8a644bdfdaee81',commit,1)
text=text.replace('32 条主','33 条主').replace('32条主','33条主')
text=text.replace('G4-evidence-bundle/','G4-evidence-bundle-005/')
text=text.replace('/g4-003/','/g4-005/').replace('/functional-g4-003/','/functional-g4-005/').replace('/schema-g4-003/','/schema-g4-005/')
text=text.replace('| 955 | 0 | 40 ignored；其中39项','| 955 | 0 | 41 ignored；其中40项')
text=text.replace('| 36 | 0 | 0 |','| 37 | 0 | 0 |').replace('| 34 | 0 | 两个','| 35 | 0 | 两个')
text=text.replace('36 个 FUSE 唯一内核用例为原 31 + NSI 2 + ACL 3','37 个 FUSE 唯一内核用例为原 31 + NSI 2 + ACL 4')
text=text.replace('| schema 独立旧/新 reader | 6 | 0 | 0 |','| schema 独立旧/新 reader | 6 | 0 | 0 |\n| 独立真实历史IPC/GUI容量与超时检查 | 6 | 0 | 0 |\n| mtime交叉用例单独preflight | 1 | 0 | 0；另计入FUSE37项 |')
cancel=functional['checks']['cancelled_active_original_scan']['observed_opendirs']
text=text.replace('已观察到8次 opendir',f'已观察到{cancel}次 opendir')
text=text.replace('main 实际 IndexHits=33033、IndexMisses=30、IndexFallback=32、IndexDegraded=1；other 命中114','main 实际 IndexHits=33132、IndexMisses=29、IndexFallback=32、IndexDegraded=2；other 命中120')
text=text.replace('3a639c94021960a8fe3a67f851d1b8f7116bdb7c60bb1d3371c9fc9a171110c7',functional['binaries']['fixture'])
text=text.replace('| g4-003 | c744729141ca8b23bde63c61ca8a644bdfdaee81 | 本报告最终通过结果 |',
'''| g4-003 | c744729141ca8b23bde63c61ca8a644bdfdaee81 | 原矩阵全部通过；后续静态复核发现缺少仅Setattr的mtime交叉场景，追加验收后被新候选取代 |
| g4-004 | b3dfff5992e2421c3898ff7e79203a2d23c2f5a3 | workspace955/0/41、GUI148/0/0；新增mtime用例五次执行在内容写拒绝断言得到EAGAIN11而非EACCES13，原件完整保留 |
| g4-005 | bae1ef1ddc4061df586221557011f4e30f04d134 | 本报告最终通过结果 |''')
text=text.replace('第二/三轮改为verbose，均完成对账','第二至五轮使用verbose，均完成对账')
text=text.replace('第三轮provider命令显式使用独立复制的prefix，10项通过','第三至五轮provider命令显式使用独立复制的prefix，各10项通过')
text=text.replace('f2a260fc4a179609da6a1f706e725d72fb348ae147ab04883eb054329747a8f1',
                  load(here/'prepared-g4-005/source.json')['candidate_archive_sha256'])
text=text.replace('/final-integrity.json)', '/final-integrity-post-history.json)')
text=text.replace('`start-g4-003.sh` → `run-g4.py`；补测为 `supplement-g4-003.sh`，依次执行210k、托盘、截图、原版FSearch功能、schema和最终完整性检查。harness及三轮冻结runner',
'''`start-g4-005.sh` → `run-g4.py`；补测为 `supplement-g4.sh g4-005 FULL_SHA`，依次执行210k、托盘、截图、原版FSearch功能、schema及独立history检查；历史最终补测另执行 `history-scale-pinned-g4.sh g4-005 FULL_SHA`，最后 `finalize-g4-integrity.py RUN final-integrity-post-history.json`。harness及五轮冻结runner''')
text=text.replace('vendor fuser wire ABI | 55','vendor fuser既有wire ABI回归 | 55')
text=text.replace('33 条主命令的逐名清单与测试框架计数全部核对一致。','33 条主命令的逐名清单与测试框架计数全部核对一致，包含1,209个唯一测试名称。vendor/fuser从基线到本轮没有代码差异；55项ABI测试也是基线既有，本轮重新执行。')

scale=next(c for c in history['checks'] if c['name']=='actual_history_page_100_pages_gui_responsive')
timeout=next(c for c in history['checks'] if c['name'].startswith('real_clientbus_15s'))
chapter=f'''## 大历史与真实超时补测

[独立结果](G4-evidence-bundle-005/evidence/g4-005/history-scale-verified/results.json) 为6项通过、0失败、0忽略，单列于产品原测试组之外。冻结产品 Desktop(Service) 在私有 D-Bus 服务中运行，从未调用 Service::start/Unlock，没有worker或挂载。直接SQL写入10,001条合成记录，默认30天/10,000条保留触发器恰留seq2..10001；这验证容量与读路径，写入生命周期由既有事务测试负责。10,000是默认容量，**不是可配置硬上限**。

真实 ClientBus 分20页读满10,000条，无漏项或重复，游标与顺序正确；mount/result/ASCII大小写且含字面`%_`名称的组合筛选返回精确500条。真实 HistoryPage 分100页加载10,000条，再实际筛选到500条；加载总墙钟{scale['wall_seconds']:.3f}s，5ms Qt心跳最大间隔{scale['qt_max_heartbeat_gap_ms']:.2f}ms、p99为{scale['qt_p99_heartbeat_gap_ms']:.2f}ms。250ms响应预算是独立harness功能标准，不是PLAN规定的数字，也不作为G5性能结论。

独立同协议peer延迟60秒，实际 ClientBus 在{timeout['timeout_seconds']:.3f}s返回`{timeout['error']}`，回调一次、pending清空、真实HistoryPage显示可刷新错误，另一正常服务在{timeout['healthy_peer_reply_seconds']:.3f}s内已回应。超时等待期间Qt心跳{timeout['qt_heartbeat_count']}次，最大间隔{timeout['qt_max_heartbeat_gap_ms']:.2f}ms。它验证真实15秒传输超时与界面响应，不把Store即时busy当作超时。

[锁定依赖证明](G4-evidence-bundle-005/evidence/g4-005/history-scale-verified/provenance.json) 核对所有registry依赖与候选Cargo.lock一致。首次独立crate按离线缓存解析的版本图与候选不同，且harness误将zbus Builder::session当作Result调用unwrap，编译失败；两次Builder API误用的原源码/构建日志分别保存在history-scale与history-scale-pinned目录，均没有运行测试。修正独立harness后最终结论只使用history-scale-verified。两次都不修改产品源码。实际容量/错误整窗截图位于该补测目录。安装包升级/重装后的历史保留仍 **未实测**；Store重开及schema迁移不替代安装包验收。

## mtime权限交叉回归

`setattr_only_persists_mtime_without_granting_content_writes_or_uploads` 使用真实FUSE验证Setattr允许/WriteContent拒绝时mtime=1234、本地原字节不变、正常卸载后Store重开仍保持mtime；内容写和truncate仍精确EACCES，上传仍被真实targets_denied门禁保留pending。远端协议使用完整内容上传，所以该组合不声称远端mtime已经同步。既有Setattr拒绝场景同时验证chmod拒绝。Traverse拒绝的missing子项也返回EACCES。

第四轮第一次新增用例暴露dispatcher Store争用时EAGAIN泄漏；[原preflight失败](G4-evidence-bundle-005/evidence/g4-004/logs/kernel-setattr-preflight.log)及其余四次相同失败原件保留。第五轮保留精确errno断言，在preflight、debug/release、INDEX=0/1及attrTTL=0全部通过。

'''
text=text.replace('## 原版 FSearch 功能结果',chapter+'## 原版 FSearch 功能结果')
(here/'G4_REPORT.md').write_text(text)
coverage=(here/'G4-COVERAGE.md').read_text()
coverage=coverage.replace('最终第三轮 `c744729` 全部通过','最终第五轮 `bae1ef1` 主矩阵及补测全部通过')
coverage=coverage.replace('G4-evidence-bundle/evidence/g4-003/','G4-evidence-bundle-005/evidence/g4-005/')
coverage += '''
## 追加容量、超时与mtime交叉验收

- 默认保留10,000条的实际IPC及GUI：独立harness `default_retention_10001_to_10000`、`actual_ipc_10000_pagination`、`actual_ipc_combined_mount_result_literal_name_filter`、`actual_history_page_100_pages_gui_responsive`、`actual_history_page_result_name_filter`。真实ClientBus/Desktop与HistoryPage、合成私有SQLite，250ms Qt心跳预算为harness自定；非硬配置上限、非真实NAS。
- 实际15s timeout：`real_clientbus_15s_timeout_page_error_healthy_peer_and_heartbeat`，同协议独立延迟peer、真实ClientBus和HistoryPage，另一正常服务仍可读。原件[history-scale-pinned](G4-evidence-bundle-005/evidence/g4-005/history-scale-verified/results.json)。
- 安装包升级/重装历史保留：**未实测**；schema升级和Store重开只证明各自范围。
- PLAN-7 §4.3的Setattr/WriteContent交叉：`setattr_only_persists_mtime_without_granting_content_writes_or_uploads`，实际utimes、bytes、写/截断拒绝、queued upload门禁及重开mtime持久化；远端全文件上传在WriteContent拒绝时保持pending，不声称远端mtime同步。原Setattr拒绝组合新增chmod实际拒绝。Traverse拒绝新增missing路径EACCES。
'''
(here/'G4-COVERAGE.md').write_text(coverage)
status=(here/'G4-STATUS.md').read_text().replace('补测正在执行。','补测全部通过，源码未改、无残留挂载。')
status=status.replace('当前第五轮补测完成前，不宣称最终G4完成。','第五轮最终G4主矩阵及全部补测已完成；[最终报告](G4-evidence-bundle-005/REPORT.md)与[证据包](G4-evidence-bundle-005/README.md)保留全部五轮原件。')
(here/'G4-STATUS.md').write_text(status)
readme=(here/'README.md').read_text()
readme=readme.replace('- G4 三轮证据', '- G4 最终五轮证据 — [最终报告](G4-evidence-bundle-005/REPORT.md)、[自足证据包](G4-evidence-bundle-005/README.md)。\n- G4 历史三轮证据')
(here/'README.md').write_text(readme)
print('updated independent final G4 report and navigation from closed passing results')
