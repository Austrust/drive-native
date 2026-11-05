# Issue #7 逐目录权限实现与验收报告

状态：产品冻结 `fefb8ee` 通过独立 G4-013 完整矩阵及补测。根目录权限不再扩展为整盘只读；明确 Read 拒绝后不会自动下载重试，取消最后一个读取需求时会清理其重试意图；upload 直接响应 `0x9003` 且目标为已存在节点时自动学习为 `(node, write_content)=denied(observed)`（可证窄子集，见 [写拒绝学习增量](validation/issue7-native-001/write-learning/REPORT.md)）。G4-013 all-features 985通过/0失败/42忽略，default 975/0/37（相对 G4-012 各 +9 = T1 新增测试）；内核、GUI、210k容量、真实本机worker链和组件数据保留结果见下表。G4-012 的旧计数、旧期望与失败原件保持冻结，013 只声明差异。本轮本地候选为 `0.1.0~dev20261105.3`（`b303ac6`，含 T9 只读初始化请求重试/同 cursor 续拉；独立 G4-014 已通过：主矩阵 59/59、补测 28/28，见「G4-014 增量」），上一开发包 `.2` 的源码验证见 G4-013；本文记录源码验证，包生成与实际内容检查以同交付的 `REPORT.md` 为准。真实 NAS 受限验收已取得两轮部分证据：T7（`.2`）个人空间子目录 scope 完整挂载、FUSE mkdir A、独立挂载、S02/S12 写矩阵 6/6 committed+真值；T10（候选 `.3`）ROOT_MF 根 scope 4/4 完整挂载（`preview_only=false`、nodes=91,081）、S01 根 mkdir/create 真实响应 `0x9004`、fail-closed、真值未变，T7 的根 `initial_index` 阻塞未复现。但完整真实 ACL/e2e 仍未通过：S01 仅完成 root deny 子项（同挂载 B 行不可用），S03–S11 blocked，缺口 2 为 partially observed、缺口 3 为 blocked_not_observed；真实 NAS ACL/`0x9003` 实际可达性、其余精确写拒绝码与独立 L2 来源仍未闭合，#7 完整验收保持开放，不因 G4-013 合成矩阵或 T7/T10 部分通过而关闭。
基线是 #5/#2 集成 `ed1fabf`；工作分支 `codex/fix-7`，schema25 固定为权限迁移。
原集成分支 `codex/fix-all-5-2-7` 的历史保留，本轮已将指定的 `codex/fix-7` 快进后继续。原交付 `3d35d7b` 后新增 `10c5e6c`、`d39a597`、`ca86a85`、`99aa587`，`4e6aaf7` 补齐旧 schema 夹具，`41365a6` 补上运行中系统时间变化时的下载退避门限，`2dafd12` 阻止已知读取拒绝被后台重试覆盖，`9c9aaba` 清理取消与权限检查相遇时的早退意图，`fefb8ee` 实现 upload/0x9003 窄子集的提交前 ACL 拒绝自动学习。此后为 `d7d8b6a`（G4-013 证据、T2、assert-only 断言）→ `d341628`（根 `initial_index` 阻塞记录）→ `d5f9490`（T7 部分通过）→ `c1c1263`（包内引用归位）→ `b303ac6`（T9 初始化只读请求重试与同 cursor 续拉；候选 `.3` 源码，独立 G4-014 已通过，见下）。增量原件与边界见[原生反馈记录](validation/issue7-native-001/REPORT.md)与[G4-013 独立报告](validation/issue7-native-001/g4-final-013/REPORT.md)。
`9c9aaba` 在权限拒绝早退前重新检查取消和独立下载需求，保留 pin、旧缓存与其他 reader；同时准备上述deb版本及显式离线文档模式。包内的未内置材料指向配套源码归档，不生成未发布提交的GitHub入口。
规格：项目 `work/plan-5-2-20261104/PLAN-7.md`；接口见 [CONTRACTS.md](CONTRACTS.md)。

## 行为与来源

此前整体实现的逐文件变更见[候选改动清单](validation/fix-all-001/CHANGED_FILES.md)，本轮增量按上方原生反馈记录的 M1–M4 追溯。主要模块为 core `write_permissions.rs`、schema25 与上传/下载调度，
runtime service/bridge/desktop/setup，FUSE `permissions.rs` 与实际操作入口，以及 GUI permissions/window/wizard/bus。

三层策略区分用户只读、明确服务空间禁止、逐节点逐动作判断。前两者写操作返回 EROFS，
目标已知拒绝返回 EACCES。unknown 允许尝试，由服务端最终确认；本地写入/排队不会把 unknown 持久化成 allowed。
根不可 create 不会让 A/B 全盘只读；祖先 Traverse 与目录 List、文件 Read、各类写动作分开。
本地移动未确认时同时检查保留的远端祖先，不借移动绕过可达性限制。

旧 profile `writable=false` 保留为 explicit_read_only/legacy；true 为 follow_remote/legacy。
新向导明确保存策略与 user 来源，不复制根 writable hint 到子目录。
策略设置用 task_id 和 revision 比较保存；重新核对不改用户只读选择。已挂载任务重新挂载后完整应用 RO/RW 选项。
旧任务如果迁移后仍显示“用户设为只读”，需要在“读写设置”中明确选择“按远端目录权限同步”，保存后停止并重新启动该挂载；单击“重新核对”不会替用户改变只读策略。这里说明新代码的操作入口，本轮没有修改现有任务配置或安装新包。

保留 DefaultPermissions，继续让内核承担现有 uid/gid/mode/umask 等 POSIX 预检查。
unknown/allowed 使用原乐观 mode；文件 WriteContent 与 Setattr 同时拒绝才去掉写位。目录保留原 POSIX 写位，因为目录自身及创建动作全部拒绝仍不能证明每个子项都不可删除/重命名。
`access()` 因此是乐观可尝试值，不能当作服务器授权证明。精确门禁在各实际操作入口执行。
entry TTL 恒 0，attr 默认 1000 ms；权限变化没有扩大 TTL。

当前解析的原生 [NodeInfo](../crates/sproto/src/actions/list.rs) 只有身份、版本、类型、尺寸和时间等元数据，未解码逐动作 ACL；同文件的 ShareInfo.writable 是根级 hint，mode 不是 ACL 证据。
成功确认会产生 observed；精确 typed PermissionDenied 接收契约只接受本次目标。
增量实现仅在原生 `recv_download` 直接回复边界将已核实的 `0x3002/0x3008` 学为目标 Read 拒绝，返回 EACCES；认证、连接和前置读取不会误入此分类。依据与排除项见[官方客户端静态证据](validation/issue7-native-001/static/REPORT.md)。
写请求的直接权限相关响应保留数字码和请求类型，历史显示“服务器返回权限相关响应，提交状态待核对”。除下述唯一可证窄子集外，仍保留 uncertain/cleanup_uncertain，不改成可重发 pending、不猜改名哪端拒绝、不写精确目录 ACL。官方服务端静态调查已证明特定 ACL middleware 在业务处理器前拒绝；但同码也来自 profile 路径过滤，码值不提供精确动作/节点归属，复合写入各阶段也未全部证明。
本轮新增的唯一可证窄子集：**upload 直接响应 `0x9003` 且请求目标为已存在节点**时，sproto 在直接回复边界产生 typed `MutationPermissionDenied`，worker 绑定本次请求节点与 `write_content`，经既有 `refuse_operation` 学习为 `denied(observed, 当前 revision)`：journal 由 `targets_denied` 门禁阻止重发、history 记 failed/permission_denied，profile 变化/重连/重新核对按既有 revision 失效。`0x9004`/`0xa004`（与 profile 只读过滤同码）、`0xa003`（无已证生产者）、rename 源/目标、mkdir/replace 创建与复合阶段、`0x3002/0x3004` 一律保持原 hint/uncertain，不学习、不推导父目录/兄弟路径、不升格 L2。依据、判定表、测试与未实测项见[写拒绝学习增量](validation/issue7-native-001/write-learning/REPORT.md)；真实 NAS 的 `0x9003` 可达性/归属仍未实测；T7/T10 均未观测到 upload `0x9003`（T10 只做根 mkdir/create 拒绝子集，未覆盖 upload，不能据此判定不可达，也不改变本窄子集）。
File Station 可选探测未实现且不调用；空间禁止只有测试注入证据，原生 L2 来源与更新接线未实现，当前生产初始化保持 unknown。
新增[空间只读来源复核](validation/issue7-native-001/l2-static/REPORT.md)确认 team `privilege="read-only"` 有 SDK 共享配置来源，但节点能力中的只读投影还受 ACL 输出路径条件控制。仅限定 build=27892、精确字符串和 view_id 仍不足以证明全子树不可写，因此不接入 L2；这也不能简化为“所有 privilege 都只是根 ACL”。
[typed 接收契约](../crates/drive-runtime/src/wire.rs)与 bridge/namespace 消费路径已实现；原生 Read 分类已接线，精确写拒绝的唯一可证窄子集（upload/0x9003/已存在节点）已按同一 typed 契约接线并学习，其余写码仍只作诊断。静态码语义不等于真实 NAS 实测。
官方服务端提供 `get_file_capability(view_id,path)`，成功响应为 `header.capabilities`，见[服务端静态证据](validation/issue7-native-001/server-static/REPORT.md)。[能力与身份补充证据](validation/issue7-native-001/server-capability/REPORT.md)确认响应不含节点身份/版本，按永久 ID 解析后也另行按路径查询，不能排除路径替换竞态；因此没有把该路径快照持久化为精确节点 ACL。字段名称也不等于当前同步操作的 ACL primitive，新建动作不能从目录 can_write 推导。这里记录的是已发现但尚不具备安全接线条件的来源，不再声称协议没有逐节点能力查询。

同一证据确认有效 Delete 已包括服务端父链 ACL；父目录自身 Delete 不是 delete_child。unlink/rmdir 现在检查父 Traverse 与目标 Delete；rename 检查源父 Traverse、源 RenameSource、目标父 RenameTarget 和覆盖目标 Delete。FUSE 与持久队列使用同一对象边界，父遍历或目标明确拒绝仍阻断。这是对 PLAN-7 操作表中“父目录删除规则”的具体化，不把父自身权限当作对子项的继承规则。

## 数据保留与恢复

确定拒绝保留本地 body、draft 和 intent；独立兄弟操作继续调度，相关依赖仍被保留意图约束。
uncertain commit 不回退为可盲目重发的 pending。重连/恢复/重新核对使 observed 回 unknown，
不撤销用户只读策略；已知读取拒绝与其终态下载重试门禁一并恢复。
IPC21 另外保留原生 profile_changed：无文件事件也使当前根全部逐节点证据回 unknown，保留 L1/L2 和写意图。旧 fsd/worker 的 IPC20 混用被拒绝，须成对更新。下载开始时捕获权限 revision；期间更新后的迟到结果不重新写入旧 allow/deny，迟到拒绝仅影响该次调用，后续只读尝试可重新判定。
正文回复和目录续读的有界发布与首次拒绝发布串行化；常规元数据工作仍不阻塞无拒绝的缓存正文回复。
存在已知拒绝时 NSI 回复回退到当前权威检查。
下载退避同时检查持久墙钟截止时间和进程内单调门限，前台、后台、新 flight 与重连共用；运行中系统时间前跳不能缩短已发布的退避时长。进程重启按持久剩余时间恢复一次，显式重试/权限重置清除失败记录后释放门限。旧版本 ticket 不读取或覆盖当前版本的门限。这只影响下载，不改变未知写入的核对规则。
明确的原生 Read 拒绝保存 `retry_at=null`；后台补全和唤醒、Bridge 的缓存/新请求入口都检查 Read 与祖先 Traverse。单独重试传输不会覆盖已知拒绝；重新核对、重连或 profile 失效仍按 revision 回到 unknown。最终独立测试和真实本机worker链均通过，不把作者编译视为验收。

本地 mtime 修改按 Setattr 检查，普通写入/截断按 WriteContent 检查。远端 mtime 仍通过整文件上传：
Setattr 允许、WriteContent 拒绝时，mtime 本地持久化且正文不变，上传保留 pending；元数据专用远端写回未实现。
具体语义见[文件时间契约](FILE_TIMESTAMPS.md)。

GUI 显示用户只读、服务禁止、当前目录权限不足、权限待验证，提供策略设置与既有问题中心重新核对入口。
历史目标及父目录按当前身份与权限解析；不会直接信任事件时路径。

## 迁移与阶段证据

本轮的增量验证结果由[原生反馈记录](validation/issue7-native-001/REPORT.md)维护，产品 `fefb8ee` 的最终命令、逐项名称、T1/T2 审阅与旧反例见[G4-013 独立报告](validation/issue7-native-001/g4-final-013/REPORT.md)；`9c9aaba` 的 [G4-012 报告](validation/issue7-native-001/g4-final-012/REPORT.md)保持历史冻结，013 只声明差异。候选 `b303ac6` 的独立矩阵、T9 审阅/对抗与两项 allowed-failed 见 [G4-014 独立报告](validation/issue7-native-001/g4-final-014/REPORT.md)，其增量只相对 G4-013 声明。各组存在重复，不能相加为唯一总数。

| G4-013 / fefb8ee | 通过 | 失败 | 忽略/范围 |
|---|---:|---:|---|
| workspace all-features | 985 | 0 | 42忽略；相对 G4-012 的 976 为 T1 新增 9 项（显式内核和容量另列） |
| workspace default | 975 | 0 | 37忽略；+9 |
| FUSE debug / release INDEX1 | 各38 | 0 | 0 |
| FUSE debug / release INDEX0 | 各36 | 0 | 各2项NSI专用不适用 |
| FUSE attr TTL=0 | 38 | 0 | 正目录项TTL仍为0 |
| runtime debug / release内核 | 各1 | 0 | 0 |
| GUI 18套 | 148 | 0 | 0；另历史标签补测3项、Nemo/Nautilus各10通过 |
| native loopback（冻结012 / 校正013期望） | 64 / 70 | 2 / 0 | 64/2为012旧期望原样运行；校正只改两处期望并加4个对抗例（delta见下） |
| 文档打包回归 | 2 | 0 | 主矩阵离线/原发布模式；补测3条亦通过 |
| 210k节点容量 | 1 | 0 | 显式ignored；功能容量，不是G5基准 |
| 本机TLS→真实worker→IPC | 10 | 0 | 6项Bridge链、4项直接RPC；非NAS |
| 策略切换与组件替换 | 8 | 0 | 真FSD/worker、私有D-Bus/FUSE；非dpkg安装 |
| T1 write-learning 定向测试 | 16项测试执行 | 0 | 8条命令单独计数：sproto 8、worker 2、bridge 3、core 3；不与workspace相加 |

**012→013 native-loopback delta（冻结保留 + 校正）**：T1 把 upload 直接回复 `0x9003` 分类为 typed `MutationPermissionDenied`，因此冻结的 012 loopback harness 中 `upload_reply_9003`、`mkdir_reply_9003` 两例由 `mutation_permission_uncertain` 期望变为 typed denied：旧期望原样运行 **64/2（原件保留）**；只改这两处期望并增加 4 个对抗用例的校正 harness 为 **70/0**。逐例比对只有这 2 例变化，其余 **64 例 observed 与 012 完全一致**；新增 4 例（download 0x9003、download 0xa004、auth/connect 阶段 0x9003）全部保持 generic `server`。012 的旧期望与旧报告不修改。

主矩阵 58 条命令中 56 条通过、2 条非产品失败：`native-loopback`（上述冻结 012 期望 64/2）与 `core-history-probe-prepare`（证据包初版缺 `core-history-probe.rs`，验证者 harness 打包错误，补测重跑 5/0）；补测 22/22 命令通过。另有一次验证者自身 run 路径错误导致的主矩阵尝试1中止，原件保留且不记为产品失败。组件补测确认legacy false须显式切换策略并重新挂载，旧进程固定旧worker、新pair正常启动，移除/恢复组件后pending正文、history、pin和cursor保留。真实 NAS 已完成只读健康检查（`.2` 首轮阻塞）、T7（`.2`）与 T10（候选 `.3`）受限复测（见下节）；Mint 与实际 dpkg 安装/升级/重装仍未实测；新包解包检查不替代这些项目。

历史失败完整保留：[G4-006–010](validation/issue7-native-001/g4-through-010/REPORT.md)含首次revision遗漏、旧schema夹具、原退避两项失败及010准备阶段环境中断；新独立报告保留G4-011读取拒绝后后台重试的2项失败。修复没有放宽原断言。当前候选没有重跑G5，旧性能数字不移作新版本实测。

### G4-014（候选 `b303ac6`，T9 初始化只读重试）增量

T4 对候选 `b303ac6` 的独立验证通过：主矩阵 **59/59 命令 PASS**、补测 **28/28 exit 0**；正式 run `g4-014-rerun` 为非 root 运行，`STATUS.json` 记录主/补测均 complete。下表只声明相对 G4-013 的差异，各组不可相加为唯一总数。

| G4-014 / b303ac6 | 通过 | 失败 | 忽略/相对 G4-013 |
|---|---:|---:|---|
| workspace all-features | 992 | 0 | 42；+7 = T9 retry 测试（retry unit 4 + seed 3） |
| workspace default | 982 | 0 | 37；+7 |
| FUSE debug / release INDEX1 | 各38 | 0 | 与 G4-013 完全相同 |
| FUSE debug / release INDEX0 | 各36 | 0 | 与 G4-013 完全相同（各2项NSI专用不适用） |
| FUSE attr TTL=0 | 38 | 0 | 与 G4-013 相同 |
| native loopback（013 校正 harness） | 70 | 0 | 逐 case 与 013 基线一致（含 2 例 typed denied 与 4 个对抗例） |
| GUI 18 套 | 148 | 0 | 与 G4-013 相同 |
| 本机TLS→真实worker→IPC | 10 | 0 | 与 G4-013 相同；非NAS |
| 策略切换与组件替换 | 8 | 0 | 8/8 case；非dpkg安装 |
| 210k节点容量 | 1 | 0 | 与 G4-013 相同 |
| T9 定向：retry unit / seed / core-lib / runtime-lib / 对抗 probe | 4 / 10 / 415 / 157 / 13 | 0 | core-lib +4；runtime-lib 持平；probe 13/0 |
| T1 write-learning 定向 | 8 条命令 | 0 | 源码零改动，结论沿用 G4-013 |

**审阅与重试语义**：重试仅接入 8 处只读调用点（`native_initialization.rs:82/:104`、`native_root_guard.rs:37/:73`、`native_snapshot.rs:132/:369`、`native_snapshot_watermark.rs:19/:22`）；`MAX_ATTEMPTS=5`、退避 200/400/800/1600ms，只有 `ConnectionClosed`/`Timeout`/transport `Io` 可重试；server/decode/config/session/cursor/TLS/mutation typed 错误均不重试；不跳页、不推进 cursor，mutation/session 路径无引用。对抗 probe 13/13 通过；release 三 ELF SHA 与作者记录一致。

**`STATUS.json` 的 2 项 allowed-failed（如实保留，不得当作通过项）**：

- `package-metadata-equivalence`：`9c28992..b303ac6` 的 packaging delta 仅有 `debian/changelog` 的 `.2` 条目变更（提交 `d7d8b6a`）；冻结 deb-001 的 MANIFEST/SHA256SUMS/包 SHA 三项仍为 true。这是版本条目口径差异，不是产品失败；T5 的 `.3` 最终构建/打包核验负责处理。
- `t9-measure-log-audit`：T9 的 4/4 测量结论可复现（ROOT_MF 76 页/102,014 事件/214.961s/25 次断连/0 服务端错误；HOME attempts5 311 页/148,281 事件/308.99s；attempts8 289 页/126,129 事件/422.684s；/Drive 5,001 请求/159.145s/0 断连），但 HOME attempts5 行自报记账不自洽（直方图 19 retries/298 requests 对字段 24 disconnects/pages=311）。这是**测量记账 caveat，不是产品行为证据**，不能据此声称 HOME 根修复成功；HOME 仍按残余处理，真实 NAS 复测由 T3/T10 负责。

G4-014 是合成/本机矩阵的独立验证，不替代真实 NAS 的完整 ACL 验收；#7 仍保持开放。

## 真实 NAS 受限验收（T7 + T10，部分通过，2026-11-05）

新开发包 `0.1.0~dev20261105.2`（产品行为 `fefb8ee` + assert-only）与候选 `0.1.0~dev20261105.3`（`b303ac6`，含 T9 初始化只读请求重试/同 cursor 续拉）在真实 NAS（`[redacted-test-host]`、TLS `[redacted-test-tls-name]`、build 17892、非管理员测试账号）完成三轮受限验收。下列 A 轮“只读健康检查”阻塞事实保留，不因 T7/T10 部分通过而删除。

**A. 只读挂载健康检查（`.2` 首轮，阻塞事实，保留）**

- selector 登录与列表正常：team root list 约 0.03s、folders=18；personal root folders=3。
- fsd `initial_index` **两次均失败**：`native_transport` / `native_response_timeout`，TLS `UnexpectedEof: peer closed connection without TLS close_notify`；`nodes=0`，无残留挂载。
- 隔离只读探针（Lead）：`list_dir` 小请求 200/200；`pull_event` cursor=0 常成功（约 516 events，next=2000），紧接 cursor=2000 常被断连；子目录 watermark S0 可进入 enumerate（10s 约 948 目录/8351 节点），约 1000 次 list 请求内被断连；根 `/` 走 legacy `collect_snapshot_seed`。
- 该轮**未找到根因**，仅证明长初始化序列断连 + 产品无每请求重试/断点续拉；不是 T1 窄子集修复引入的已证回归（既有记录：`outputs/drive-native/initialization-owner-20261101/REPORT.md`、`outputs/drive-native/delivery/development-20261101-3/RELEASE_NOTES.md`、`outputs/drive-native/delivery/development-20261101-4/RELEASE_NOTES.md`、`work/initialization-pipeline-8/docs/validation/personal-003/REPORT.md`，均为项目工作区路径，未入本包）。测量与最小修复见 [R2 韧性报告](validation/real-nas-initial-index-20261105/REPORT-R2-RESILIENCE-20261105.md)。

**B. T7 受限真实 NAS 验收（`.2`，T3，部分通过）**

| 观测 | 结果 |
|---|---|
| 个人空间**子目录 scope 完整挂载** | **成功**：90.7s，`preview_only=false`，catching_up→enumerating→importing，FUSE rw |
| FUSE `mkdir` 创建 A（批准测试目录） | **成功**；File Station 真值 A：append/write/read/exec/del=true，posix 777，ACL=true |
| A 独立挂载 M_A（scope=A） | **成功**：1.2s，`preview_only=false`，rw=True |
| S02（单独挂载 A） | **pass**：A 内 mkdir 异步终态 committed，File Station 真值确认 |
| S12（补充写矩阵） | **pass**：create/write/truncate/rename/delete **6/6 committed**，server truth 逐项确认 |
| 清理 | A 已删除并核验：parent listing 中 A=0、`list(A)`=file_not_found；无残留挂载/进程 |
| 根 scope（ROOT_MF/HOME_ROOT） | **当时仍阻塞**：ROOT_MF 300s 超时（`initial_index` `native_transport`(6)/`native_response_timeout`(2)，nodes=0）；HOME 仅 preview（555.6s 后 `preview_only=true`，随后 `native_server_rejected` code 8193/0x2001、recovery=needs_attention）。该根阻塞在 T10 的 `.3` 候选上未复现（见 C），但 HOME 仍无正式完整初始化判定 |
| S01/S03–S11、缺口 2 真实拒绝观测、缺口 3 管理员实验 | **blocked**（根/B/夹具/管理员条件不满足）；缺口 2 的真实 `0x9003` 可达性与 history 落库仍未观测，不能写“已验证” |

统计：Issue #7 的 11 场景 = **pass 1（S02）/ blocked 10**；补充 S12 = pass。没有任何场景以“通过”掩盖阻塞。

**C. T10 根 scope 回归与受限验收（候选 `.3`，T3/T10，部分通过）**

- **候选包**：`0.1.0~dev20261105.3`，源码 `b303ac6`（T9 重试修复；候选包唯一构建差异是私有提取树中的本地 changelog 条目），deb SHA256 `c7b8a5dd…fb83`；由 `.2` 升级安装（`19:09:09` upgrade、`19:09:11` configured），`dpkg --verify` 0 行；fsd `12f7a868…38f6`、worker `e6ad3bfb…656f`、setup `afc48e47…68e`、GUI wrapper `037b1847…9586`（完整值见 [T10-RESULT](validation/real-nas-t10-20261105/T10-RESULT.md)）。
- **ROOT_MF 根 scope（T9 修复目标）**：**4/4 挂载成功**，`ready_seconds` 206.233 / 121.049 / 124.670 / 171.954，`preview_only=false`、`writable=true`，phases 含 enumerating→catching_up→importing、`scope_complete=true`；root state DB `nodes=91,081`；FUSE probe `list_ok=true`、`stat_ok=true`。T7 在 `.2` 上的根 `initial_index` 阻塞**未复现**。
- **S01 根拒绝**（仅根 mkdir/create 预期失败，不写真实数据）：3/3 本地先成功、异步终态 `refused_remote_uncertain`/`permission_commit_uncertain`，真实响应码均为 `0x9004`，`fail_closed=true`、`non_commit_verified=true`、`local_retained=true`；ROOT_MF 真值前后 `append=false/write=false`、POSIX 555、fingerprint 一致，根无实际变更。`0x9004` 与本地 profile 只读过滤同码，`code_certainty=ambiguous`，**不构成 T1 可学习候选**；本次 `mkdir/create_file` 子集**未观测 `0x9003`**，只说明子集未覆盖 upload，不证明 upload 路径不可达，也不改变现有学习逻辑（upload `0x9003` + 已存在节点才学习）。rename 拒绝码仍未进入 history，保持不猜测。
- **写矩阵**：A（`P032-acceptance-<YYYYMMDD-HHMMSS>`，仅授权测试目录）经 FUSE mkdir 创建（本地即时 `[ -d ]` 为 false，File Station 真值已确认），M_A 独立挂载 ready `1.7s`；S02 pass，补充 S12 create/write/truncate/rename/create/delete **6/6 committed**；矩阵 `pass=2/fail=0/blocked=0`。GUI 四态 `observable=none`，不把未观测当通过。
- **清理与场景计数**：A 已删除，File Station 父目录列表连续核验 absent、truth `perm_missing`；getinfo 仍 stale `exists=true`（见产品观察）。11 个正式场景 = **S02 pass / 10 blocked**（S01 仅 root deny 子项、无同挂载 B 行；S03–S11 缺夹具/管理员条件）；补充 S12 pass；**缺口 2 = partially_observed**（`0x9004` 非唯一 ACL 码，`0x9003` 未观测），**缺口 3 = blocked_not_observed**（无管理员 revoke/restore 或共享级只读实验）。HOME 根正式判定跳过（residual），B 的 deny 与同挂载 root/A/B 拓扑未提供。
- **产品观察（本轮未改代码）**：1) FUSE 新建目录本地即时不可见；2) fresh parent mount 不暴露已由服务端确认的 A，导致无法走本地清理；3) File Station getinfo 在删除后仍 stale `exists=true`（与 T7 现象一致），不以 stale getinfo 判成功/失败；4) `0x9004` 与 profile 只读过滤同码，直接影响缺口 2/T1 学习与 fail-closed 粒度判定。
- **环境**：所有产品挂载已 stop，无残留 fsd/worker/mount_ctl/dbus；候选 `.3` 保持安装未卸载；未对 ROOT_MF 根或其它真实数据目录产生写入，S01 仅执行拒绝尝试并验证无提交。

**D. 结论与后续**

- T10 在真实 NAS 上**首次取得根 scope 完整挂载**与根拒建失败关闭证据，T7 的根 `initial_index` 阻塞在 `.3` 上未复现；S02/S12 正向写路径继续通过。
- 但真实 NAS **完整 ACL 验收仍未通过**：S01 只有 root deny 部分、缺同挂载 B 行，S03–S11 blocked，缺口 2 仅 partially observed、缺口 3 blocked_not_observed，HOME 根未正式判定；**不得写成“#7 通过”**。T10 的 `0x9004` 观测既不是 upload `0x9003` 的替代证据，也不改变 T1 唯一可学习子集。
- 后续（待授权，不在本轮改产品代码）：同挂载 root/A/B 或等价受控拓扑与 Drive 可见只读 B（S01/S07/S09/S10）；C/D/E/G 夹具或受控 ACL 配置窗口（S03–S06）；管理员 revoke/restore 与共享级 L2（S08/S09/缺口 3）；offline/full relay 与 GUI 四态；HOME 根正式判定、根 watermark/超大个人树；旧版本/账号/网络对照与“断连后再成功”端到端；产品侧确认 FUSE 新目录可见性、fresh mount 可见性与 getinfo stale，并评估 `0x9004` 去歧义。
- 证据：[T10-RESULT](validation/real-nas-t10-20261105/T10-RESULT.md)（逐字节复制，SHA256 `fa54f3a8…bba7`）与[复制核验](validation/real-nas-t10-20261105/COPY-VERIFY.md)；外部证据包 `work/issue7-acceptance/evidence/t10-20261105/`（MANIFEST `20106384…dd252`，40/40 SHA 通过，不复制入仓）。T7 证据：[T7-RESULT](validation/real-nas-t7-20261105/T7-RESULT.md)（SHA256 `1393fa9d…e3bf`）与[复制核验](validation/real-nas-t7-20261105/COPY-VERIFY.md)，外部包 `work/issue7-acceptance/evidence/t7-limited-20261105/`（MANIFEST `372e1c82…f53e`，23/23，不复制入仓）；根初始化韧性测量/修复见 [R2 报告](validation/real-nas-initial-index-20261105/REPORT-R2-RESILIENCE-20261105.md)；旧阻塞证据见[真实 NAS initial_index 阻塞报告](validation/real-nas-initial-index-20261105/REPORT.md)。

以下计数和场景表对应此前冻结 `bae1ef1` 的完整G4，作为历史证据保留，不能当作新候选的通过计数。

[G3 实现记录](validation/fix-all-001/G3_IMPLEMENTATION.md)列出 M1–M5 commits、失败编译原件和冻结命令。
schema24→25 延续原子迁移、备份、拒绝未来 schema 的保护；新增迁移用例保留 history 与未确认上传内容。
旧 profile 与当前用户策略分别测试，不使用 schema 迁移提升权限。

G4 使用独立不可变源码与 target，候选 `bae1ef1`：全量 Rust 955 通过 / 0 失败 / 41 忽略；
FUSE debug/release 各 37 / 0 / 0，其中含 4 个 ACL 内核用例；INDEX=0 各 35 / 0 / 0 加 2 个 NSI 专用不适用；
GUI 148 / 0 / 0 中含权限 5 项、selector 7 项。迁移/旧 reader 独立探针 6 / 0 / 0。
完整分组计数见 [#5/#2 报告](FIX_5_2_REPORT.md)，重复组不相加。第一、二、四轮失败及修复依据保留在 [G4 反馈记录](validation/fix-all-001/G3_IMPLEMENTATION.md)。
逐项测试、注入真值与截图入口见[独立覆盖矩阵](validation/fix-all-001/g4/COVERAGE.md)和[原件报告](validation/fix-all-001/g4/REPORT.md)。

| 场景 | 注入/约束真值 | 实测结果 |
|---|---|---|
| 根拒绝 / A 可建 / B 拒绝 | CreateDir(root)=deny、A=allow、B=deny | 注入 + 实际 FUSE 通过 |
| 独立挂载 A | 上级根拒绝不越过新挂载范围 | 通过；动态用户只读返回 EROFS |
| 只建目录、不建文件 | A CreateDir=allow / CreateFile=deny | 实际 FUSE 通过 |
| 内容可写、禁止删除/改名/改属性 | WriteContent=allow，其余动作分别 deny | 通过；实际写/截断/删除/改名与动作模型分别覆盖 |
| 可改 mtime、禁止改正文 | Setattr=allow，WriteContent=deny | 实际 FUSE 通过：mtime 持久化、正文不变、写/截断 EACCES、完整上传继续 pending |
| 继承中断/显式组合 | 注入有效逐动作真值，不从 mode 推导 | 有效动作注入通过；真实用户/组与继承解析未实测 |
| 只遍历不列父目录 | List(A)=deny，Traverse(A)=unknown | 实际 FUSE 通过 |
| 跨目录、覆盖目标 | RenameTarget(B) 与 Delete(target) 独立 | 实际拒绝→恢复→改名通过 |
| 撤销/恢复、已有 fd、离线重连 | Read deny→unknown、observed reset、发布门闩 | 本地句柄/重开/reset 通过；真实网络重连未实测 |
| 服务禁止/用户只读 | L2 / L1 覆盖子节点 allow | 注入/用户策略通过；真实空间能力未实测 |
| 一项拒绝与独立队列 | typed refusal，字节保留、兄弟确认 | 本地 transport + core 通过 |
| 旧 false/true profile | explicit_read_only / follow_remote | 通过 |
| 迁移、备份、降级拒绝 | schema24→25 和旧版本拒绝 | 通过 |
| unknown | 尝试不等于授权；确认后学习 | 注入/typed receipt 通过；原生拒绝码映射未实测 |
| GUI 四态与恢复 | 离屏 fixture / 截图，非真实 NAS | 通过；完整窗口浅/深四态共 8 图 |

未移除 DefaultPermissions，因此不宣称实现了另一套用户态 POSIX 权限系统。全部真实 ACL 验收仍由用户/Lead 在专用测试共享执行；T1 未连接 NAS、未读凭据、未操作生产数据、未 push 或发布。2026-11-05 的真实 NAS 已完成只读健康检查（`.2` 首轮 `initial_index` 阻塞）及 T7（`.2`）/T10（候选 `.3`）受限复测；T10 取得根 scope 4/4 挂载与根拒建 `0x9004` fail-closed 证据；候选 `.3` 的 G4-014 独立矩阵 59/59、补测 28/28 已通过（`STATUS.json` 保留 2 项 allowed-failed，不替代真实 NAS），但完整真实 NAS ACL/e2e 仍未完成（S01 仅部分、S03–S11 blocked、缺口 2/3 未闭合），#7 保持开放，不写成通过。
