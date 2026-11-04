# Issue #7 逐目录权限实现与验收报告

状态：G3 代码已实现并通过全量编译；G4 独立运行进行中。真实 NAS ACL 未实测。
基线是 #5/#2 集成 `ed1fabf`；工作分支 `codex/fix-7`，schema25 固定为权限迁移。
规格：项目 `work/plan-5-2-20261104/PLAN-7.md`；接口见 [CONTRACTS.md](CONTRACTS.md)。

## 行为与来源

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

当前解析的原生 NodeInfo/事件没有逐动作 ACL 字段，mode 不是 ACL 证据。
成功确认会产生 observed；精确 typed PermissionDenied 接收契约只接受本次 durable intent 中的 node/action。
未核实的服务器数字码仍保留既有 uncertain/reconciliation，不能猜成权限拒绝。
**当前原生 worker 尚无可验证的数字错误码→typed PermissionDenied 映射**，真实拒绝自动学习须外部核实。
File Station 可选探测未实现且不调用；空间禁止只有测试注入证据，原生判定来源未实测。

## 数据保留与恢复

确定拒绝保留本地 body、draft 和 intent；独立兄弟操作继续调度，相关依赖仍被保留意图约束。
uncertain commit 不回退为可盲目重发的 pending。重连/恢复/重新核对使 observed 回 unknown，
不撤销用户只读策略；已知读取拒绝与其终态下载重试门禁一并恢复。
正文回复和目录续读的有界发布与首次拒绝发布串行化；常规元数据工作仍不阻塞无拒绝的缓存正文回复。
存在已知拒绝时 NSI 回复回退到当前权威检查。

GUI 显示用户只读、服务禁止、当前目录权限不足、权限待验证，提供策略设置与既有问题中心重新核对入口。
历史目标及父目录按当前身份与权限解析；不会直接信任事件时路径。

## 迁移与阶段证据

[G3 实现记录](validation/fix-all-001/G3_IMPLEMENTATION.md)列出 M1–M5 commits、失败编译原件和冻结命令。
schema24→25 延续原子迁移、备份、拒绝未来 schema 的保护；新增迁移用例保留 history 与未确认上传内容。
旧 profile 与当前用户策略分别测试，不使用 schema 迁移提升权限。

G4 使用独立不可变源码与 target；通过/失败/忽略、命令和失败原件：待独立结果。

| 场景 | 注入/约束真值 | 实测结果 |
|---|---|---|
| 根拒绝 / A 可建 / B 拒绝 | CreateDir(root)=deny、A=allow、B=deny | 待 G4 |
| 独立挂载 A | 上级根拒绝不越过新挂载范围 | 待 G4 |
| 只建目录、不建文件 | A CreateDir=allow / CreateFile=deny | 待 G4 |
| 内容可写、禁止删除/改名/改属性 | WriteContent=allow，其余动作分别 deny | 待 G4 |
| 继承中断/显式组合 | 注入有效逐动作真值，不从 mode 推导 | 待 G4；真实用户/组计算未实测 |
| 只遍历不列父目录 | List(A)=deny，Traverse(A)=unknown | 待 G4 |
| 跨目录、覆盖目标 | RenameTarget(B) 与 Delete(target) 独立 | 待 G4 |
| 撤销/恢复、已有 fd、离线重连 | Read deny→unknown、observed reset、发布门闩 | 待 G4 |
| 服务禁止/用户只读 | L2 / L1 覆盖子节点 allow | 待 G4 |
| 一项拒绝与独立队列 | typed refusal，字节保留、兄弟确认 | 待 G4 |
| 旧 false/true profile | explicit_read_only / follow_remote | 待 G4 |
| 迁移、备份、降级拒绝 | schema24→25 和旧版本拒绝 | 待 G4 |
| unknown | 尝试不等于授权；确认后学习 | 待 G4 |
| GUI 四态与恢复 | 离屏 fixture / 截图，非真实 NAS | 待 G4 |

未移除 DefaultPermissions，因此不宣称实现了另一套用户态 POSIX 权限系统。全部真实 ACL 验收仍由用户/Lead
在专用测试共享执行；本轮未连接 NAS、未读凭据、未操作生产数据、未 push 或发布。
