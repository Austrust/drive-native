# Issue #7 逐目录权限实现与验收报告

状态：产品冻结 `9c9aaba` 通过独立 G4-012 完整矩阵及补测。根目录权限不再扩展为整盘只读；明确 Read 拒绝后不会自动下载重试，取消最后一个读取需求时会清理其重试意图。all-features 976通过/0失败/42忽略，default 966/0/37；内核、GUI、210k容量、真实本机worker链和组件数据保留结果见下表。此前41365a6的两条真实worker反例原件保留，原断言在新候选通过。本轮本地开发版本为 `0.1.0~dev20261105.1`；本文记录源码验证，包生成与实际内容检查以同交付的 `REPORT.md` 为准。真实 NAS ACL、精确写拒绝自动学习与独立 L2 来源仍未闭合，#7完整验收保持开放。
基线是 #5/#2 集成 `ed1fabf`；工作分支 `codex/fix-7`，schema25 固定为权限迁移。
原集成分支 `codex/fix-all-5-2-7` 的历史保留，本轮已将指定的 `codex/fix-7` 快进后继续。原交付 `3d35d7b` 后新增 `10c5e6c`、`d39a597`、`ca86a85`、`99aa587`，`4e6aaf7` 补齐旧 schema 夹具，`41365a6` 补上运行中系统时间变化时的下载退避门限，`2dafd12` 阻止已知读取拒绝被后台重试覆盖。增量原件与边界见[原生反馈记录](validation/issue7-native-001/REPORT.md)。
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
写请求的直接权限相关响应保留数字码和请求类型，历史显示“服务器返回权限相关响应，提交状态待核对”。仍保留 uncertain/cleanup_uncertain，不改成可重发 pending、不猜改名哪端拒绝、不写精确目录 ACL。官方服务端静态调查已证明特定 ACL middleware 在业务处理器前拒绝；但同码也来自 profile 路径过滤，码值不提供精确动作/节点归属，复合写入各阶段也未全部证明。**精确写拒绝自动学习仍未完成。**
File Station 可选探测未实现且不调用；空间禁止只有测试注入证据，原生 L2 来源与更新接线未实现，当前生产初始化保持 unknown。
新增[空间只读来源复核](validation/issue7-native-001/l2-static/REPORT.md)确认 team `privilege="read-only"` 有 SDK 共享配置来源，但节点能力中的只读投影还受 ACL 输出路径条件控制。仅限定 build=27892、精确字符串和 view_id 仍不足以证明全子树不可写，因此不接入 L2；这也不能简化为“所有 privilege 都只是根 ACL”。
[typed 接收契约](../crates/drive-runtime/src/wire.rs)与 bridge/namespace 消费路径已实现；原生 Read 分类已接线，精确写拒绝的覆盖仍来自隔离 transport 夹具。静态码语义不等于真实 NAS 实测。
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

本轮的增量验证结果由[原生反馈记录](validation/issue7-native-001/REPORT.md)维护，最终命令、逐项名称和旧反例见[独立报告](validation/issue7-native-001/g4-final-012/REPORT.md)。各组存在重复，不能相加为唯一总数。

| G4-012 / 9c9aaba | 通过 | 失败 | 忽略/范围 |
|---|---:|---:|---|
| workspace all-features | 976 | 0 | 42忽略；显式内核和容量另列 |
| workspace default | 966 | 0 | 37忽略 |
| FUSE debug / release INDEX1 | 各38 | 0 | 0 |
| FUSE debug / release INDEX0 | 各36 | 0 | 各2项NSI专用不适用 |
| FUSE attr TTL=0 | 38 | 0 | 正目录项TTL仍为0 |
| runtime debug / release内核 | 各1 | 0 | 0 |
| GUI 18套 | 148 | 0 | 0；另历史标签补测3项通过 |
| 原生回环 / 实际worker版本 / 历史补测 | 66 / 3 / 5 | 0 | 0 |
| Nemo / Nautilus | 各10 | 0 | 合成夹具 |
| 文档打包回归 | 2 | 0 | 离线/原发布模式 |
| 210k节点容量 | 1 | 0 | 原默认预算、显式ignored；不是G5基准 |
| 本机TLS→真实worker→IPC | 10 | 0 | 6项Bridge链、4项直接RPC；非NAS |
| 策略切换与组件替换 | 8 | 0 | 真FSD/worker、私有D-Bus/FUSE；非dpkg安装 |

新增读取/取消5项、双钟/旧ticket5项、原退避失败2项及runtime152/0/1也通过。主矩阵59条执行命令的计数与逐项清单一致；1310个唯一测试名称是不同层次的去重口径，不能与分组计数相加。组件补测确认legacy false须显式切换策略并重新挂载，旧进程固定旧worker、新pair正常启动，移除/恢复组件后pending正文、history、pin和cursor保留。真实NAS、Mint与实际dpkg安装/升级/重装仍未实测；新包解包检查不替代这些项目。

历史失败完整保留：[G4-006–010](validation/issue7-native-001/g4-through-010/REPORT.md)含首次revision遗漏、旧schema夹具、原退避两项失败及010准备阶段环境中断；新独立报告保留G4-011读取拒绝后后台重试的2项失败。修复没有放宽原断言。当前候选没有重跑G5，旧性能数字不移作新版本实测。

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

未移除 DefaultPermissions，因此不宣称实现了另一套用户态 POSIX 权限系统。全部真实 ACL 验收仍由用户/Lead
在专用测试共享执行；本轮未连接 NAS、未读凭据、未操作生产数据、未 push 或发布。
