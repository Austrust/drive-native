# 启动时的 sync_paused 语义（issue #61）

`sync_paused_v1` 是**会话级**控制，不是持久用户偏好。Pause 写入该 flag 后，如果进程
被 stop / 崩溃 / `kill -9`，flag 会留在 profile store 里。修复前，新进程启动只消费
`startup-pause-v1.json` one-shot intent，从不读取/清理/报告这个 flag，于是会**静默
paused**：没有连接、没有上传/namespace 工作、overview 只显示 `paused`，与用户主动
暂停无法区分。

## 策略（`Store::reconcile_startup_pause`，由 `Service::new` 在 unlock 前执行一次）

1. **新鲜的一次性 handover intent**（`startup-pause-v1.json`，`schema=1`、
   `task_id` 匹配、权限 0600、文件年龄 ≤ 1 小时）：消费并删除该文件，**本次进程保持
   paused**，overview `paused_by = "startup_intent"`。这是 GUI 在旧 FSD 仍持有数据库
   时发布、由新独占 owner 提交的维护暂停，语义不变、仍是一次性。
2. **否则持久 flag `sync_paused_v1=1` 属于上一个进程生命周期**（stop/crash/kill -9）：
   自动恢复运行（清除 flag），overview `paused_by = "persisted_flag_auto_resumed"`，
   `startup_pause.auto_resumed=true`。理由：停止/崩溃不是用户对**新会话**的暂停指令；
   保持 paused 会让客户端无限期不同步。用户在当前会话内仍然可以随时 Pause。
3. **过期 intent（> 1 小时）**：仍然被消费/删除（one-shot 语义），但**不应用**，
   `startup_pause.stale_intent_discarded=true`。它不能覆盖用户显式 Resume，也不能
   掩盖持久 flag 的自动恢复策略。
4. **当前会话内的 Pause/Resume** 不经过启动路径：Pause 设 `paused_by="user"`，
   Resume 清空 `paused_by`。显式 pause 不会被自动恢复。

## 可观察字段（additive，schema 仍 1，IPC 版本不变）

`GetOverview` 追加：

- `paused`（bool）：当前是否 paused。
- `paused_by`（string|null）：`"user"` / `"startup_intent"` /
  `"persisted_flag_auto_resumed"` / null（正常运行且本会话无 pause 事件）。
- `startup_pause`（object|null）：`{schema:1, maintenance_intent, stale_intent_discarded,
  persisted_flag_found, auto_resumed, paused, at}`，结构化启动决策事件。

`startup_pause` 是**启动诊断**而不是 flag：正常启动时各 bool 为 false（`at` 为启动
时刻）；没有新增"永远为真"的字段。旧 GUI 忽略这些键即可。

## 升级/迁移边界

- 升级（schema 迁移）与 pause flag 相互独立：迁移后再执行本策略，因此"升级后不
  同步"不会再因为旧的 paused flag 而静默发生。
- 本批未在升级 pre/post-flight 诊断中额外插入 pause 警告路径：`startup_pause` +
  `paused_by` 是升级重启后可见的显式报告；若后续有独立升级诊断入口，应引用同一
  字段而不是复制策略。
- 无 schema 变更、无 IPC 版本变更、无新增 D-Bus 方法。

## 验证

```sh
cargo test -p drive-core --lib task_control
cargo test -p drive-runtime --lib startup_pause_tests
cargo test -p drive-runtime --test desktop overview_reports_sync_state
```
