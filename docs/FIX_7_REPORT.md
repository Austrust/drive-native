# Issue #7 逐目录权限实现与验收报告

状态：候选 `bae1ef1` 通过 G4 独立主矩阵及权限补测。真实 NAS ACL 未实测，原生权限来源仍有下述边界。
基线是 #5/#2 集成 `ed1fabf`；工作分支 `codex/fix-7`，schema25 固定为权限迁移。
最终本地交付分支 `codex/fix-all-5-2-7` 保持产品候选 `bae1ef1` 不变，追加报告与证据。
规格：项目 `work/plan-5-2-20261104/PLAN-7.md`；接口见 [CONTRACTS.md](CONTRACTS.md)。

## 行为与来源

逐文件变更见[候选改动清单](validation/fix-all-001/CHANGED_FILES.md)。主要模块为 core `write_permissions.rs`、schema25 与上传/下载调度，
runtime service/bridge/desktop/setup，FUSE `permissions.rs` 与实际操作入口，以及 GUI permissions/window/wizard/bus。

三层策略区分用户只读、明确服务空间禁止、逐节点逐动作判断。前两者写操作返回 EROFS，
目标已知拒绝返回 EACCES。unknown 允许尝试，由服务端最终确认；本地写入/排队不会把 unknown 持久化成 allowed。
根不可 create 不会让 A/B 全盘只读；祖先 Traverse 与目录 List、文件 Read、各类写动作分开。
本地移动未确认时同时检查保留的远端祖先，不借移动绕过可达性限制。

旧 profile `writable=false` 保留为 explicit_read_only/legacy；true 为 follow_remote/legacy。
新向导明确保存策略与 user 来源，不复制根 writable hint 到子目录。
策略设置用 task_id 和 revision 比较保存；重新核对不改用户只读选择。已挂载任务重新挂载后完整应用 RO/RW 选项。

保留 DefaultPermissions，继续让内核承担现有 uid/gid/mode/umask 等 POSIX 预检查。
unknown/allowed 使用原乐观 mode；混合 action 不能因为单项拒绝统一去掉写位而误拒另一项允许操作。
`access()` 因此是乐观可尝试值，不能当作服务器授权证明。精确门禁在各实际操作入口执行。
entry TTL 恒 0，attr 默认 1000 ms；权限变化没有扩大 TTL。

当前解析的原生 [NodeInfo](../crates/sproto/src/actions/list.rs) 只有身份、版本、类型、尺寸和时间等元数据，未解码逐动作 ACL；同文件的 ShareInfo.writable 是根级 hint，mode 不是 ACL 证据。
成功确认会产生 observed；精确 typed PermissionDenied 接收契约只接受本次 durable intent 中的 node/action。
未核实的服务器数字码仍保留既有 uncertain/reconciliation，不能猜成权限拒绝。
**当前原生 worker 尚无可验证的数字错误码→typed PermissionDenied 映射**，真实拒绝自动学习须外部核实。
File Station 可选探测未实现且不调用；空间禁止只有测试注入证据，原生 L2 来源与更新接线未实现，当前生产初始化保持 unknown。
[typed 接收契约](../crates/drive-runtime/src/wire.rs)与 bridge/namespace 消费路径已实现，实际构造拒绝回复的覆盖来自隔离 transport 夹具。
真实逐目录 ACL 预知不可得，列为外部验收；不把接收端与测试注入描述为原生 ACL 解析已经实现。

## 数据保留与恢复

确定拒绝保留本地 body、draft 和 intent；独立兄弟操作继续调度，相关依赖仍被保留意图约束。
uncertain commit 不回退为可盲目重发的 pending。重连/恢复/重新核对使 observed 回 unknown，
不撤销用户只读策略；已知读取拒绝与其终态下载重试门禁一并恢复。
正文回复和目录续读的有界发布与首次拒绝发布串行化；常规元数据工作仍不阻塞无拒绝的缓存正文回复。
存在已知拒绝时 NSI 回复回退到当前权威检查。

本地 mtime 修改按 Setattr 检查，普通写入/截断按 WriteContent 检查。远端 mtime 仍通过整文件上传：
Setattr 允许、WriteContent 拒绝时，mtime 本地持久化且正文不变，上传保留 pending；元数据专用远端写回未实现。
具体语义见[文件时间契约](FILE_TIMESTAMPS.md)。

GUI 显示用户只读、服务禁止、当前目录权限不足、权限待验证，提供策略设置与既有问题中心重新核对入口。
历史目标及父目录按当前身份与权限解析；不会直接信任事件时路径。

## 迁移与阶段证据

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
