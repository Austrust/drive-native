# Issue #7 逐目录权限实现与验收报告

状态：产品冻结 `fefb8ee` 通过独立 G4-013 完整矩阵及补测。根目录权限不再扩展为整盘只读；明确 Read 拒绝后不会自动下载重试，取消最后一个读取需求时会清理其重试意图；upload 直接响应 `0x9003` 且目标为已存在节点时自动学习为 `(node, write_content)=denied(observed)`（可证窄子集，见 [写拒绝学习增量](validation/issue7-native-001/write-learning/REPORT.md)）。G4-013 all-features 985通过/0失败/42忽略，default 975/0/37（相对 G4-012 各 +9 = T1 新增测试）；内核、GUI、210k容量、真实本机worker链和组件数据保留结果见下表。G4-012 的旧计数、旧期望与失败原件保持冻结，013 只声明差异。本轮本地开发版本 `0.1.0~dev20261105.2`；本文记录源码验证，包生成与实际内容检查以同交付的 `REPORT.md` 为准。真实 NAS 只读挂载健康检查已执行，但 `initial_index` 被服务端/网络断连阻塞（.2，build 17892；见[真实 NAS 阻塞报告](validation/real-nas-initial-index-20261105/REPORT.md)）；真实 NAS ACL/0x9003 实际可达性、其余精确写拒绝码与独立 L2 来源仍未闭合，#7 完整验收保持开放，不因 G4-013 合成矩阵关闭。
基线是 #5/#2 集成 `ed1fabf`；工作分支 `codex/fix-7`，schema25 固定为权限迁移。
原集成分支 `codex/fix-all-5-2-7` 的历史保留，本轮已将指定的 `codex/fix-7` 快进后继续。原交付 `3d35d7b` 后新增 `10c5e6c`、`d39a597`、`ca86a85`、`99aa587`，`4e6aaf7` 补齐旧 schema 夹具，`41365a6` 补上运行中系统时间变化时的下载退避门限，`2dafd12` 阻止已知读取拒绝被后台重试覆盖，`9c9aaba` 清理取消与权限检查相遇时的早退意图，`fefb8ee` 实现 upload/0x9003 窄子集的提交前 ACL 拒绝自动学习。增量原件与边界见[原生反馈记录](validation/issue7-native-001/REPORT.md)与[G4-013 独立报告](validation/issue7-native-001/g4-final-013/REPORT.md)。
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
本轮新增的唯一可证窄子集：**upload 直接响应 `0x9003` 且请求目标为已存在节点**时，sproto 在直接回复边界产生 typed `MutationPermissionDenied`，worker 绑定本次请求节点与 `write_content`，经既有 `refuse_operation` 学习为 `denied(observed, 当前 revision)`：journal 由 `targets_denied` 门禁阻止重发、history 记 failed/permission_denied，profile 变化/重连/重新核对按既有 revision 失效。`0x9004`/`0xa004`（与 profile 只读过滤同码）、`0xa003`（无已证生产者）、rename 源/目标、mkdir/replace 创建与复合阶段、`0x3002/0x3004` 一律保持原 hint/uncertain，不学习、不推导父目录/兄弟路径、不升格 L2。依据、判定表、测试与未实测项见[写拒绝学习增量](validation/issue7-native-001/write-learning/REPORT.md)；真实 NAS 的 `0x9003` 可达性/归属仍未实测（待 T7）。
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

本轮的增量验证结果由[原生反馈记录](validation/issue7-native-001/REPORT.md)维护，产品 `fefb8ee` 的最终命令、逐项名称、T1/T2 审阅与旧反例见[G4-013 独立报告](validation/issue7-native-001/g4-final-013/REPORT.md)；`9c9aaba` 的 [G4-012 报告](validation/issue7-native-001/g4-final-012/REPORT.md)保持历史冻结，013 只声明差异。各组存在重复，不能相加为唯一总数。

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

主矩阵 58 条命令中 56 条通过、2 条非产品失败：`native-loopback`（上述冻结 012 期望 64/2）与 `core-history-probe-prepare`（证据包初版缺 `core-history-probe.rs`，验证者 harness 打包错误，补测重跑 5/0）；补测 22/22 命令通过。另有一次验证者自身 run 路径错误导致的主矩阵尝试1中止，原件保留且不记为产品失败。组件补测确认legacy false须显式切换策略并重新挂载，旧进程固定旧worker、新pair正常启动，移除/恢复组件后pending正文、history、pin和cursor保留。真实 NAS 已做只读挂载健康检查但 `initial_index` 阻塞（见下节）；Mint 与实际 dpkg 安装/升级/重装仍未实测；新包解包检查不替代这些项目。

历史失败完整保留：[G4-006–010](validation/issue7-native-001/g4-through-010/REPORT.md)含首次revision遗漏、旧schema夹具、原退避两项失败及010准备阶段环境中断；新独立报告保留G4-011读取拒绝后后台重试的2项失败。修复没有放宽原断言。当前候选没有重跑G5，旧性能数字不移作新版本实测。

## 真实 NAS 只读挂载健康检查（阻塞，2026-11-05）

新开发包 `0.1.0~dev20261105.2`（产品行为 `fefb8ee` + assert-only，无初始化代码改动）在真实 NAS（`[redacted-test-host]`、TLS `[redacted-test-tls-name]`、build 17892、非管理员测试账号）做只读挂载健康检查：

- selector 登录与列表正常：team root list 约 0.03s、folders=18；personal root folders=3。
- fsd `initial_index` **两次均失败**：错误类 `native_transport` / `native_response_timeout`，TLS `UnexpectedEof: peer closed connection without TLS close_notify`；`nodes=0`，无残留挂载。失败在初始化候选接收阶段，未提交。
- 隔离只读探针（Lead）：`list_dir` 小请求 200/200 通过；`pull_event` cursor=0 常成功（约 516 events，next=2000），紧接 cursor=2000 常被断连；子目录 watermark S0 可进入 enumerate（10s 约 948 目录/8351 节点），但约 1000 次 list 请求内被断连；根 `/` 走 legacy `collect_snapshot_seed`。
- 归因边界：只证明“长初始化序列中被服务端/网络中断 + 产品无每请求重试/断点续拉 → 初始化无法完成”；**未找到根因**，未证明版本差异、会话限制或协议字段语义；不是 T1 窄子集修复引入的已证回归。项目既有记录（[initialization-owner-20261101](../../../outputs/drive-native/initialization-owner-20261101/REPORT.md)、[development-20261101-3](../../../outputs/drive-native/delivery/development-20261101-3/RELEASE_NOTES.md)、[development-20261101-4](../../../outputs/drive-native/delivery/development-20261101-4/RELEASE_NOTES.md)、[personal-003](../../../work/initialization-pipeline-8/docs/validation/personal-003/REPORT.md)）早已标注真实 NAS 初始化/续扫未完成，本次是既有集成缺口在 #7 候选上的暴露。
- 后续（待下轮验证，不在本轮改产品代码）：P0 为 `list_dir`/`pull_event` 每请求有限重试 + 断点续拉；P1 根 scope watermark 路径可行性；P2 旧版本/账号/网络对照。

完整证据、已证/未证边界与建议见[真实 NAS initial_index 阻塞报告](validation/real-nas-initial-index-20261105/REPORT.md)。**真实 NAS ACL/e2e 验收因此未完成，Issue #7 保持开放。**

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

未移除 DefaultPermissions，因此不宣称实现了另一套用户态 POSIX 权限系统。全部真实 ACL 验收仍由用户/Lead 在专用测试共享执行；T1 未连接 NAS、未读凭据、未操作生产数据、未 push 或发布。2026-11-05 的真实 NAS 只读健康检查已执行但 `initial_index` 阻塞（见[真实 NAS 阻塞报告](validation/real-nas-initial-index-20261105/REPORT.md)），真实 NAS ACL/e2e 仍未完成，#7 保持开放。
