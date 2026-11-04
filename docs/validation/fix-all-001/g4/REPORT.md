# G4 独立验证报告

最终候选 `bae1ef1ddc4061df586221557011f4e30f04d134` 通过本轮本地 G4 验证。33 条主矩阵命令全部通过；补充功能、托盘、schema 旧版拒绝及最终源码/卸载检查全部通过。测试在 WSL2 Ubuntu 24.04 普通用户下运行，源码来自固定 Git 归档、target 独立复制，Cargo offline。验证者只写 harness 与证据，没有修改产品代码。

本报告不含 G5 性能结论。真实 NAS、File Station、原生 ACL 用户/组/继承解析、真实服务端拒绝码映射、真实网络离线重连和真实 GNOME/KDE 系统面板均 **未实测**。逐场景范围见 [覆盖映射](COVERAGE.md)。

## 最终计数

每行是独立执行组；重复模式不能相加为唯一测试覆盖数。完整命令、环境、原始 log 哈希及每个测试名称见 [summary](evidence/g4-005/summary.json) 和 [inventory](evidence/g4-005/inventory-001.json)。33 条主命令的逐名清单与测试框架计数全部核对一致，包含1,209个唯一测试名称。vendor/fuser从基线到本轮没有代码差异；55项ABI测试也是基线既有，本轮重新执行。

| 执行组 | 通过 | 失败 | 忽略/不适用 |
|---|---:|---:|---|
| workspace all features | 955 | 0 | 41 ignored；其中40项在下列专用组显式执行 |
| core/fs/runtime default | 786 | 0 | 36 ignored；重叠于专用组 |
| vendor fuser既有wire ABI回归 | 55 | 0 | 0 |
| FUSE debug INDEX=1 | 37 | 0 | 0 |
| FUSE release INDEX=1 | 37 | 0 | 0 |
| FUSE debug INDEX=0 | 35 | 0 | 两个 NSI 专用用例 N/A |
| FUSE release INDEX=0 | 35 | 0 | 两个 NSI 专用用例 N/A |
| FUSE release attr TTL=0 | 37 | 0 | 0 |
| runtime kernel debug / release，各组 | 1 | 0 | 0 |
| 专用本地真实 D-Bus | 1 | 0 | 0 |
| core 210k 状态快照显式功能负载 | 1 | 0 | 0 |
| GUI 18 套 | 148 | 0 | 0 |
| 独立合成托盘验收 | 4 | 0 | 0 |
| Nemo provider | 10 | 0 | 0 |
| Nautilus provider，私有依赖 prefix | 10 | 0 | 0 |
| schema 独立旧/新 reader | 6 | 0 | 0 |
| 独立真实历史IPC/GUI容量与超时检查 | 6 | 0 | 0 |
| mtime交叉用例单独preflight | 1 | 0 | 0；另计入FUSE37项 |

37 个 FUSE 唯一内核用例为原 31 + NSI 2 + ACL 4，全部实际执行。INDEX=0 仅 `indexed_plain_continuation_and_seek_do_not_wait_for_store`、`indexed_plus_continuation_and_seek_do_not_wait_for_store` 不适用，因为它们显式断言 NSI 启用；它们在 debug/release INDEX=1 及 attr TTL=0 组均通过。唯一最终未执行的主矩阵 ignored 项为既有 `sproto` 文档代码块 `pstream::pmap (line 130)`，源码显式标为 `ignore`；宏本身的单元测试已运行。

GUI 各套通过数：gui_unit 5、cache 5、conflict 24、hidden_dialog 3、lazy 4、multi_mount 5、trust 6、transfer 6、theme 1、selector 7、backend_upgrade 11、pause_handover 5、initialization 10、unified_manager 27、gui_instance 3、queue 4、history 17、permission 5。provider 与独立托盘四项单列，不计入148。

## 大历史与真实超时补测

[独立结果](evidence/g4-005/history-scale-verified/results.json) 为6项通过、0失败、0忽略，单列于产品原测试组之外。冻结产品 Desktop(Service) 在私有 D-Bus 服务中运行，从未调用 Service::start/Unlock，没有worker或挂载。直接SQL写入10,001条合成记录，默认30天/10,000条保留触发器恰留seq2..10001；这验证容量与读路径，写入生命周期由既有事务测试负责。10,000是默认容量，**不是可配置硬上限**。

真实 ClientBus 分20页读满10,000条，无漏项或重复，游标与顺序正确；mount/result/ASCII大小写且含字面`%_`名称的组合筛选返回精确500条。真实 HistoryPage 分100页加载10,000条，再实际筛选到500条；加载总墙钟3.396s，5ms Qt心跳最大间隔35.31ms、p99为32.97ms。250ms响应预算是独立harness功能标准，不是PLAN规定的数字，也不作为G5性能结论。

独立同协议peer延迟60秒，实际 ClientBus 配置15000ms超时，在本次运行中14.519s返回`org.freedesktop.DBus.Error.NoReply`，回调一次、pending清空、真实HistoryPage显示可刷新错误，另一正常服务在0.257s内已回应。超时等待期间Qt心跳2905次，最大间隔5.50ms。它验证真实传输超时与界面响应，不把Store即时busy当作超时；实际时间按原值报告。

[锁定依赖证明](evidence/g4-005/history-scale-verified/provenance.json) 核对159个registry依赖与候选Cargo.lock一致。首次独立crate按离线缓存解析的版本图与候选不同，且harness误将zbus Builder API当作Result调用unwrap，编译失败；两次Builder API误用的原源码/构建日志分别保存在history-scale与history-scale-pinned目录，均没有运行测试。修正独立harness后最终结论只使用history-scale-verified。全部尝试均未修改产品源码。实际HistoryPage容量/错误截图位于该补测目录。安装包升级/重装后的历史保留仍 **未实测**；Store重开及schema迁移不替代安装包验收。

## mtime权限交叉回归

`setattr_only_persists_mtime_without_granting_content_writes_or_uploads` 使用真实FUSE验证Setattr允许/WriteContent拒绝时mtime=1234、本地原字节不变、正常卸载后Store重开仍保持mtime；内容写和truncate仍精确EACCES，上传仍被真实targets_denied门禁保留pending。远端协议使用完整内容上传，所以该组合不声称远端mtime已经同步。既有Setattr拒绝场景同时验证chmod拒绝。Traverse拒绝的missing子项也返回EACCES。

第四轮第一次新增用例暴露dispatcher Store争用时EAGAIN泄漏；[原preflight失败](evidence/g4-004/logs/kernel-setattr-preflight.log)及其余四次相同失败原件保留。第五轮保留精确errno断言，在preflight、debug/release、INDEX=0/1及attrTTL=0全部通过。

## 原版 FSearch 功能结果

[完整结果](evidence/functional-g4-005/result.json) 使用原版 FSearch 0.3.1、固定 audit 工具及最终候选诊断 fixture。主树 11,630 项，另一个独立挂载138项；两轮保存数据库的 missing/extra/duplicates/invalid/size-or-mtime-mismatch 均为0。

- 在 FSearch 尚活跃、已观察到12次 opendir 时向自建扫描进程组发送 SIGTERM；这是进程取消验证，不是 GUI 取消按钮测试。取消后挂载可用，完整重扫真值正确。
- 扫描 A 时 B 的 stat/listdir 正确；49字节草稿写后立即读、size/mtime、rename/delete 后再次保存索引真值正确。
- 宽目录200次 opendir/releasedir 循环通过；两挂载 hydration=0；fixture均正常退出，没有强制清理或残留挂载。
- main 实际 IndexHits=33132、IndexMisses=29、IndexFallback=32、IndexDegraded=2；other 命中120、无降级。保留含本地修改的功能场景降级记录，不以这些计时或计数代替 G5 的干净树配对结果。EntryTtlNonZero 两挂载均为0。

原版 FSearch SHA256：`b6f5406a24e82e6263c9490f20db90b9939e9d627763e620cb2bc0f793d05d8c`；audit：`761d6f36315c552af68e2f651d65819cbd8c83550acd135ff075d8991ba063c2`。最终功能 fixture：`e752e8fc1a9adc32de5991fa36ba4b82e825e1478a14e26eb4d8d99a484ca4ea`。

## schema 与截图

[schema 原件](evidence/schema-g4-005/results.json) 实测创建23、23→24、旧23拒绝24、24→25、旧23拒绝25、旧24拒绝25；拒绝后数据库文件哈希不变，各库 integrity_check=ok。旧23/24 reader 使用首轮冻结 ELF 不重编；schema25 reader以最终候选重新编译。[来源记录](evidence/schema-g4-005/provenance.json) 与 ELF 哈希保留。reader是从旧提交编译的最小 Store::open 程序，并非历史发布安装包。带历史/dirty内容的迁移及回滚/备份另由core用例覆盖。

最终统一多挂载布局7张，ACL QLabel文案图4张，以及实际 MainWindow 四态浅/深色8张分别保留。MainWindow使用真实产品组件和回调、合成 IPC；8张与首轮已逐图检查的PNG逐字节一致，文案及设置/重新核对按钮可见。[整窗图及hash](evidence/g4-005/supplement/mainwindow-screenshots/evidence.json)、[QLabel四态hash](evidence/g4-005/summary.json)。

## 保留的历史失败

| 轮次 | 冻结提交 | 记录 |
|---|---|---|
| g4-001 | cb22500970c139f104dc8fa9b6760dc8235c34e2 | workspace395/3/1，GUI146/1/0；早期 Cargo fail-fast 导致后续target未执行，不能当完整矩阵；INDEX=0错跑两个NSI专用用例是harness问题 |
| g4-002 | 652eabe741396c8332a7182599aaf4e53d5af48b | workspace949/6/40；完整FUSE INDEX=1各34/2/0，INDEX=0各32/2/0；GUI148/0/0 |
| g4-003 | c744729141ca8b23bde63c61ca8a644bdfdaee81 | 原矩阵全部通过；后续静态复核发现缺少仅Setattr的mtime交叉场景，追加验收后被新候选取代 |
| g4-004 | b3dfff5992e2421c3898ff7e79203a2d23c2f5a3 | workspace955/0/41、GUI148/0/0；新增mtime用例五次执行在内容写拒绝断言得到EAGAIN11而非EACCES13，原件完整保留 |
| g4-005 | bae1ef1ddc4061df586221557011f4e30f04d134 | 本报告最终通过结果 |

前两轮 [首轮summary](evidence/g4-001/summary.json)、[第二轮summary](evidence/g4-002/summary.json) 及全部原始日志保留。首轮GUI非verbose，无法从日志恢复所有逐项名，首轮inventory明确标记未逐名对账；第二至五轮使用verbose，均完成对账。第二轮8个不同回归失败的最小原件在 [workspace日志](evidence/g4-002/logs/workspace-all-features.log)、[完整debug内核日志](evidence/g4-002/logs/kernel-fs-debug-index1.log)，包括v6夹具history触发器、NSI夹具inode分配、严格清理JSON、调度/计数夹具、ExplicitRO errno与preview errno；修复后原断言在第三轮通过。

Nautilus原系统环境在前两轮缺 `Nautilus` GI namespace，记录为环境失败，没有算通过或忽略。第二轮从 Ubuntu noble-updates/main 下载并仅解包到私有prefix，10项补测通过；第三至五轮provider命令显式使用独立复制的prefix，各10项通过。包版本均 `1:46.4-0ubuntu0.2`，`gir1.2-nautilus-4.0` SHA256 `1ad995582e1e4d80ebf9ee6e22f24fe71d497f77e66fee70efcd6f8b38e00d0d`，`libnautilus-extension4` 为 `e5e4c915b01c4eb398813315b63b9d7d31a4d899a302e6a962d20d678610836e`。[原下载日志](evidence/g4-002/nautilus-private/download.log)、[补测结果](evidence/g4-002/nautilus-private/result-corrected.json) 保留；未安装系统包。

独立schema harness首轮尝试曾因SQLite连接未显式关闭而产生copy时序错误；原失败与脚本保留，修复harness后新目录的6项通过。Nautilus首次派生计数漏掉被D-Bus日志打断的首项，原派生文件保留，修正结果与原 `Ran 10 tests` 一致。这两项没有修改产品测试断言。

## 源码、命令与复核

最终归档SHA256 `dbfb638140942e2e1013b32fff44aa35091753992642b27b73ae039e547ec329`。[最后校验](evidence/g4-005/final-integrity-post-history.json) 确认冻结源码无修改，仅vendor独立离线测试生成 `vendor/fuser/Cargo.lock`，本轮及功能挂载无残留。

主命令入口为 `start-g4-005.sh` → `run-g4.py`；补测为 `supplement-g4.sh g4-005 FULL_SHA`，依次执行210k、托盘、截图、原版FSearch功能、schema及独立history检查；历史最终补测另执行 `history-scale-pinned-g4.sh g4-005 FULL_SHA`，最后 `finalize-g4-integrity.py RUN final-integrity-post-history.json`。harness及五轮冻结runner、source.json、每条完整命令和环境均在证据包内。日志计数保留测试框架原值；功能场景的断言不折算成Rust/GUI测试条数。没有访问NAS/凭据/生产数据，没有push或发布。
