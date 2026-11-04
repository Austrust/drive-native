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
