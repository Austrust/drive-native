# G3 实现记录

基线：G2 `ed1fabf91ec54bc1ba09d3537128b7d229a12936`，分支 `codex/fix-7`。
ACL 专用代理因额度错误未产生产品改动；协调者接手单线实现。G1 各流已冻结。

## M1：schema 25 与策略模型

- 新增 `write_policy` / `directory_permissions`，保留 schema 24 历史。
- Profile 缺少新字段时按旧 writable 迁移；用户只读不能被重连覆盖。
- unknown 只允许尝试，未提交成功不会记录 allowed；仅精确 action 的拒绝生效。
- 保留 DefaultPermissions，混合 action 不统一移除写位，准确门禁由操作入口执行。
- `g3-check-001.log`：失败，SQLite revision 使用了不支持的 u64。
- `g3-check-002.log`：失败，两处 Store guard 缺少 mut。
- `g3-check-003.log`：全 workspace / all-targets / all-features cargo check 通过；后续接线仍须最终检查。
- 未验证点：全部测试、内核、GUI、迁移、运行时撤销、真实 NAS 均未实测。

## 权限来源边界

现有 NodeInfo / 事件解析没有逐操作 ACL；合成 mode 和共享根 writable 不作为逐节点证据。
当前未核实任何 NAS 数字错误码可无歧义表示某个 action 的确定拒绝，所以不新增猜测映射。
typed PermissionDenied 接收契约用于明确拒绝和隔离夹具；普通未知服务器错误仍保留 uncertain / reconciliation。
File Station 辅助探测未实现、默认不调用；真实 ACL、空间禁止来源仍需外部验收。

## 验证门禁

G3 期间仅 cargo check；G4 由不写产品代码的独立验证代理运行。
本文不是测试通过报告。原始日志保留在本目录，最终结果见 G4 报告。

## M2–M5 与 G3 冻结

| 里程碑 | commit | 内容 |
|---|---|---|
| M1 | ce22057 | schema25、三层模型、legacy profile、挂载策略 |
| M2 | 442824f | confirmed/typed-refusal 接收、队列隔离、重连、祖先与回复边界、核心用例 |
| M3 | 85b906d | setup/wizard、overview、目录权限与 revision 策略 IPC、历史目标保护 |
| M4 | bcaaf75 | FUSE 逐动作检查、目录与正文回复保护、3 个显式内核夹具 |
| M5 | 812ef09 | GUI 四态、读写设置、重新核对入口、5 个 GUI 夹具 |

最终产品候选 `812ef09`。`g3-check-007.log` 对此候选执行
`cargo check --workspace --all-targets --all-features --locked --offline -j 2`，退出 0。
此前 `004/005/006` 也通过编译；这些均不是运行测试。
独立 target：`/home/drive-native-test/.cache/ultra-acl-target`。

新增核心用例包含 schema24→25、旧 profile、unknown 不持久化允许、根/A/B、只遍历不列目录、
远端 overlay 祖先、精确拒绝的数据保留与兄弟队列、旧回复阻挡首次撤销发布、普通元数据不阻挡缓存读取。
所有用例截至 G3 冻结均**未实测**。保留 DefaultPermissions；未实施替代内核 POSIX 权限系统。

G3 后只由独立验证代理执行 G4。真实 NAS、凭据、生产数据、push、发布均未操作。

## G4 反馈后的修复候选

首轮独立归档为 `cb22500`，保留原有候选与失败日志。以下修改由协调者完成，作者仍只运行 cargo check：

| commit | 修改依据 | 未验证点（提交时） |
|---|---|---|
| 8ceda14 | 调度先验证 durable intent 再解码 ACL 目标；重置 observed Read 拒绝时恢复下载重试门禁 | 新增回归和完整矩阵未实测 |
| 55189ba | 显式分类三张新表；确定非提交后重试释放旧 upload_paths 捕获；新草稿重查父 CreateFile；修正缺元数据及独立任务范围的夹具 | 新候选由独立 G4 复跑，不能沿用旧候选通过数 |

`g3-check-009.log` 和 `g3-check-010.log` 的全 workspace/all-targets/all-features 编译完成；010 退出 0。
新候选未删除或放宽原断言。独立挂载 A 的夹具按 native::project 的选定子树模型令 A.parent=None；
产品 `ensure_fs_root` 的 inode 1 身份保护保持不变。原失败为该夹具错误地在整树 Store 上改挂载根。

最终 G4/G5 结果见 [#5/#2 报告](../../FIX_5_2_REPORT.md)和 [#7 报告](../../FIX_7_REPORT.md)，本节保留修复提交时的验证边界。
