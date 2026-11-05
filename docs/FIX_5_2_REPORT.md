# Issue #5 / #2 实现与验收报告

状态：候选 `bae1ef1` 通过 G4 独立验证与补测，G5 三套树配对及诊断已完成。全部正确性核对通过，五个总耗时目标及六个每目录成本目标均未达；两套大树慢于基线，#5 性能验收未关闭。

基线 `codex/fs-recovery-safe-9 @ d555944`；#5/#2 集成分支 `codex/fix-5-2 @ ed1fabf`。
随后 #7 在 `codex/fix-7` 单线实现；共同候选及结果以本报告最终验证章节为准。
本报告 G4/G5 对应产品冻结 `bae1ef1`，原交付记录为 `3d35d7b`。随后原集成历史在 `codex/fix-7` 继续补 #7 原生权限反馈，产品冻结 `9c9aaba` 的独立G4与本地开发版本 `0.1.0~dev20261105.1` 见 [#7 当前报告](FIX_7_REPORT.md)；这些增量没有重跑 G5，本文耗时不能移作新候选实测值。
规格为项目 `work/plan-5-2-20261104/PLAN.md` 和 `PLAN-ULTRA.md`。

## 实现范围

逐文件变更见[候选改动清单](validation/fix-all-001/CHANGED_FILES.md)。核心接线集中在
`drive-core/namespace_index.rs`、`filesystem_posix.rs`、`shared_store.rs`，
`drive-fs` 的索引/目录/回复路径；历史在 `sync_history.rs`、既有事务写入点和 runtime desktop IPC；桌面在 manager/history/tray/window/bus。

NamespaceIndex 是 Store 的只读元数据投影。LOOKUP、GETATTR、OPENDIR 和目录续读可从内存快照回答；
每个索引回复持有 projection 读租约，与 Store guard 发布的版本绑定。atime、inode 分配、未提交边界、
Store 替换均进入独立版本域，通知丢失不能把旧索引变成当前权威。
脏草稿、未链接对象、预览、索引 miss 或不可信代数沿原 SQL 路径处理。
索引容量或构建时间超限时整代降级 SQL；有界目录 LRU 懒加载未实现，契约允许安全全量回退。
当前失效和刷新以整代为单位，逐节点/逐目录精准刷新未实现；持续变动负载的收益不能从静态树基准推定。

正目录项 entry TTL 恒 0；attr TTL 默认 1000 ms、可设 0。基线 fuser 的两种 TTL 参数传递顺序及直接协议字段回归已存在，本轮重新运行验证，vendor/fuser 无产品改动，
不以空文件镜像、放宽 TTL、排除挂载或减少正确性核对取得性能数字。
详细接口、限制、计数与 G3 权限叠加见 [冻结契约](CONTRACTS.md)。

同步历史使用 schema24 的 `sync_history`，以 durable operation/transfer 身份合并重试，
记录上传确认、下载发布、命名空间确认与远端事件；元数据查询不会制造传输成功。
四个 JSON IPC 支持分页、挂载/结果/名称筛选、近期活动、身份解析和独立清理。
默认保留 30 天 / 10,000 条；历史清理不删除缓存、草稿或队列。

桌面使用同一主窗口中的挂载树与页面；历史及托盘活动按所属服务保存记录身份。
从历史打开文件会重新解析当前身份和路径；在线未缓存内容显示下载过程，离线无内容不假装已打开。
托盘提供汇总、近期历史、全体暂停/恢复和两种退出方式；全体操作保留单挂载手动暂停意图。

## 阶段与提交

- G0：协调者亲自冻结契约 `f730025`；后续补充共享投影租约和跨流接线。
- G1：A/B/C/D 各自 worktree、文件所有权和独立 target，仅 cargo check。
- G2：固定 A→B→C→D 合入 `ed1fabf`。作者里程碑、原件与首次编译失败见 [G2 记录](validation/fix-all-001/G2_REPORT.md)。
- G3：不与 G1 并行，schema25 权限；见 [#7 报告](FIX_7_REPORT.md)。
- G4：不写产品代码的独立代理，在不可变源码归档和独立 target 上运行矩阵。
- G5：G4 通过后构建独立基准程序，再进入独占窗口，原版 FSearch 与三套相同命名空间配对。

原有 bus.py 未提交改动的 stash `5dbece9e64d201a800df5332faf6ce99cc61829f` 保留；对应功能已比较整合。
没有 push、发布、安装到生产、连接 NAS 或读取凭据。

## 正确性与性能证据

G2 全量 `cargo check --workspace --all-targets --all-features --locked --offline -j2` 通过。
G3 冻结编译及失败原件见 [G3 记录](validation/fix-all-001/G3_IMPLEMENTATION.md)。编译通过不算测试通过。

G4 候选 `bae1ef1ddc4061df586221557011f4e30f04d134`，第五轮源码归档 SHA256：
`dbfb638140942e2e1013b32fff44aa35091753992642b27b73ae039e547ec329`。

| 独立执行组 | 通过 | 失败 | 忽略 / 不适用 |
|---|---:|---:|---|
| Rust workspace 全 features | 955 | 0 | 41 |
| core/fs/runtime 默认 features | 786 | 0 | 36 |
| vendored fuser ABI/单元 | 55 | 0 | 0 |
| FUSE INDEX=1，debug / release 各 | 37 | 0 | 0 |
| FUSE INDEX=0，debug / release 各 | 35 | 0 | 2 个必须启用 NSI 的专用用例不适用 |
| FUSE release，attr TTL=0 | 37 | 0 | 0 |
| runtime 内核，debug / release 各 | 1 | 0 | 0 |
| 私有真实 D-Bus | 1 | 0 | 0 |
| GUI 18 套 | 148 | 0 | 0 |
| Nemo / Nautilus provider 各 | 10 | 0 | 0 |
| 210k core 显式负载 / 独立托盘补测 | 1 / 4 | 0 | 0 |
| schema 23→24→25 与旧 reader 拒绝 | 6 | 0 | 0 |
| 大历史真实 IPC / GUI / 超时补测 | 6 | 0 | 0 |

执行组不相加为独立测试总数。普通 Rust 运行的 ignored 包括另行显式执行的 FUSE、runtime、D-Bus、210k 用例；
仅 sproto 的示意 pmap doc-test 保留原有 ignore。Nautilus 依赖解包到私有 prefix，未修改系统安装。
第一、二、四轮失败原件保留，第三轮为旧候选通过记录；修复依据见 [G4 反馈记录](validation/fix-all-001/G3_IMPLEMENTATION.md)。
完整命令、计数、原件哈希见[独立 G4 报告](validation/fix-all-001/g4/REPORT.md)，实际测试名称与规格对应见[覆盖矩阵](validation/fix-all-001/g4/COVERAGE.md)。

独立合成历史补测确认默认保留 10,000 条（配置可调），实际 IPC 20 页无漏重，组合筛选得到预期 500 条。
真实 HistoryPage 连续加载 100 页 / 10,000 条耗时 3.396 秒，5 ms Qt 心跳最大间隔 35.31 ms。
配置 15,000 ms 的真实 ClientBus 请求在 14.519 秒返回 NoReply；同期另一服务 0.257 秒回应，等待期间 Qt 心跳最大间隔 5.50 ms。
这是本地隔离 IPC 与离屏 GUI 证据，不代表 Mint 实机或安装包重装验收。

原版 FSearch 的 11,630 条目功能补测通过：active 扫描取消后两次保存索引的路径/类型/size/mtime 错误均为 0；
49 字节 dirty 内容立即可读，rename/delete、200 次目录释放循环、双挂载通过；hydration=0、两夹具正常退出。
该功能负载不是下表的配对性能样本。最终源码哈希未漂移，仅 vendor 测试生成 Cargo.lock，残留挂载为 0。

三轮配对中位数如下，单位为秒，使用 `time.monotonic()` 记录的实测耗时。首扫从夹具 ready 后启动 FSearch 开始计时，不包含构造合成状态及预先装载索引；不宣称冷宿主页缓存。

| 树（条目 / 目录） | 扫描 | `.9` | local | 候选 | 候选目标 | 结果 |
|---|---|---:|---:|---:|---:|---|
| 102,130 / 2,066 | 首扫 | 9.637 | 0.521 | 6.482 | ≤2.0 | 未达 |
| 同上 | 重扫 | 9.286 | 0.521 | 6.432 | ≤1.6 | 未达 |
| 319,679 / 56,073 | 首扫 | 56.018 | 1.725 | 63.028 | ≤13 | 未达，慢于基线 |
| 同上 | 重扫 | 53.512 | 1.574 | 61.326 | ≤12 | 未达，慢于基线 |
| 327,750 / 56,554（既有真实清单） | 首扫 | 96.183 | 1.774 | 112.513 | ≤15 | 未达，慢于基线 |
| 同上 | 重扫 | 97.939 | 1.625 | 115.875 | 无独立秒数目标 | 记录实测，慢于基线 |

候选单调时钟总耗时除以目录数，首扫/重扫分别为：100k 3.137/3.113 ms，密集树 1.124/1.094 ms，真实树 1.989/2.049 ms；六项均未达到 ≤0.15 ms/目录。
这个口径包含客户端启动、排序、保存与文件属性请求，不能称为纯目录 FUSE 开销；本地 100k 参考的同口径成本也约 0.252 ms/目录。
100k 完整索引下仍未达标；密集树三轮构建耗时约 1.060–1.061 s，真实树约 1.135–1.141 s，均触发 1 s 预算，bytes=0、partial/degraded=true，实际沿 SQL 回退扫描。
没有为获得更好样本放宽预算或 TTL。完整逐轮值、命令、请求数、构建信息与负载原件见[独立 G5 报告](validation/fix-all-001/g5/REPORT.md)。

独占窗口为 2026-11-04 15:50:59 至 17:15:57 UTC（北京时间 11-04 23:50:59 至 11-05 01:15:57，含关闭检查）。
本团队停止其他构建、测试及扫描。Windows CIM 负载采集 21/21 次失败，宿主 CPU/内存**未实测**；WSL guest 负载另行保存，不能替代 Windows 数据，不能据此声称整个宿主已证明空闲。
计时边界复核发现：UTC 起止跨度与单调时钟耗时最大相差约 11.8 秒。UTC start 在前置负载/资源采样之前；单调时钟覆盖日志打开、进程启动、wait 及日志关闭；随后读取资源用量再取 UTC end，之后才采集 load_after、SHA 和 audit，并非同一测量窗口。现有证据不能分离区间外开销与时钟因素，原因未确认，不归因于宿主暂停或校时。G5 证据保留两组原值及独立审计；主表不将 UTC 跨度混入单调时钟中位数。
54 次主扫描及 18 次独立诊断保存索引的 missing/extra/duplicate/invalid/属性不符均为 0，hydration=0；36 个夹具正常停止，无强制清理，关闭检查未发现本基准残留挂载。
真实清单仅复用既有 manifest，不重扫生产目录；local 用保留尺寸和时间的稀疏镜像。

以下为另一次诊断首扫的实测值，**不并入三轮主表**。CPU 是各进程累计 user+system，client 进程树包含 dbus/Xvfb 包装；各进程可以并行，不能相加后当作总耗时分割。fixture CPU 的采样范围还包含随后的本地索引 audit。

| 树 | LOOKUP 请求 | 索引命中 / 回退 | fixture CPU（s） | client user / system（s） | IndexReadUs 路径 elapsed（s） |
|---|---:|---:|---:|---:|---:|
| 100k | 106,210 | 112,457 / 0 | 2.97 | 0.628 / 2.827 | 1.857 |
| 密集 319k | 377,766 | 0 / 433,901 | 49.40 | 2.931 / 13.962 | 0.001299 |
| 真实清单 | 928,781 | 0 / 985,447 | 87.86 | 4.204 / 25.658 | 0.002596 |

100k 的 LookupProjection / DirectoryProjection elapsed 均为 0，证实这次诊断未走 SQL 投影；大树分别为 6.543/3.109 s 和 18.193/3.539 s。
IndexReadUs 覆盖索引路径直到 reply 调用返回，包含回复与租约范围，**不是纯 HashMap 读取或 CPU 时间**；降级时只记录快速判定。
另测的串行目录 scandir 在途时间也包含用户态、内核和调度；100k / 密集树分别约 0.319 / 0.731 ms/目录，与主表总耗时/目录分列。
这些 span 可以嵌套或重叠，纯内核传输、调度及纯索引读取未单独实测，不能通过相减伪造互斥分布或硬性能下限。
下一步性能工作仍需改善大树装载/有界目录回退，并定位完整索引下的请求处理成本；本轮不将这些未实现改进算作 #5 已解决。

## 验收对应与边界

| 验收项 | 实现/验证入口 | 当前证据 |
|---|---|---|
| 名称重验、脏属性、通知丢失与降级 | entry_revalidation、namespace_index、publication、目录夹具 | G4 通过 |
| INDEX=0 / attr TTL=0 兼容 | 独立矩阵 | G4 通过；NSI 专用两项在 INDEX=1 实测 |
| 三树真值、hydration、卸载 | G5 配对 harness | 54 次主扫描 + 18 次诊断全部通过 |
| 三树总耗时 / 每目录成本 | G5 完整配对 | 0/5 总耗时目标、0/6 目录成本目标通过，#5 保留未达项 |
| 同窗切换、单实例、托盘、混合暂停 | unified_manager / history / pause GUI suites、独立托盘补测 | G4 合成/本地通过 |
| 历史重启、重试、分页/筛选/清理、身份解析 | core history、runtime desktop、GUI history | G4 通过 |
| 新历史的安装包升级/重装保留 | 新 schema 包生命周期 | 未实测；Store 重开及 schema 迁移不能替代包重装 |
| 大量历史界面响应及真实 15 秒超时 | 10k 记录、真实 ClientBus 与 HistoryPage | 独立 6 项通过，实际延迟与心跳见上文 |
| 真实 Mint 面板/Nemo/HiDPI | 外部桌面验收 | 未实测 |

FOPEN_CACHE_DIR / io_uring 等可选实验未启用。真实 NAS、真实用户队列、生产初始化、裸机性能及长期运行均未实测。
本轮不扩展普通目录全量同步或既有 P0–P6 门槛，也不把历史 `.9` 结果算作新实现通过。
