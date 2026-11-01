# 持久初始化候选基础

本开发分支提供私有候选存储API和持久写回门禁，SQLite schema19、内部IPC16。本轮变更待统一验证；它尚未成为生产初始化调度器，不代表逐目录提前显示、跨重启自动续扫或减少整树重试已经交付。

## 存储与事务

`initialization_run` 绑定任务、所选根范围、服务器恢复身份、起点和预期的权威游标；`initialization_entries`、`initialization_directories` 保存候选节点与直接子目录覆盖，`initialization_events` 保存按序接收的事件批次。它们不向 FUSE 提供可见元数据，也不把部分列表写入权威节点表。

每项候选变更在 SQLite 事务中核验身份、代次及预算，成功提交后才可读取新结果。节点数、序列化字节数、事件批次数和事件字节数均有上限；超预算返回错误，不悄悄丢弃事件或把未完成目录标成完成。初始化失败仍保留持久门禁与原权威内容。

| API族 | 约束 |
| --- | --- |
| `begin_initialization` / `resume_initialization` / `restart_initialization` | 不隐式覆盖活动候选。恢复前由调用方重新认证、核对根和服务器恢复身份；重启私有候选不能改变任务、所选范围或权威基准。 |
| `begin_initialization_directory` / `store_initialization_directory` / `store_initialization_denied` | 只接收完整、已核验的直接子项列表。相同回执与正文重复为幂等；复用回执但正文不同则拒绝。所选根不可被标为无权限子树。 |
| `record_initialization_events` / `pending_initialization_events` / `apply_initialization_events` | 接收与应用游标及序列分别保存。接收不等于应用；事件引起的候选结构变化使相关目录覆盖或根核对失效。 |
| `begin_initialization_root_refresh` / `refresh_initialization_root` | 根当前元数据回应须对应当前请求票据，不能由迟到响应满足新请求。 |
| `begin_initialization_fence` / `confirm_initialization_fence` / `complete_initialization` | fence 必须对应候选的当前边界；完整覆盖、事件全部应用及根核对满足后，才可原子安装权威快照。 |

## 票据与重启

票据绑定 `run_id`、进程代次 `epoch` 和请求 `generation`。目录请求使用 run 内单调分配的代次，避免同一目录身份删除后重建使旧回应重新有效。根核对与 fence 使用各自一次性票据；后续相关变更使旧回应失效。

重新打开 Store 会推进未完成候选的 epoch，撤销旧根／fence 票据；未完成的 loading／failed 目录回到待查状态，已完整接收的目录与事件回执保留。失败候选保持阻止写回，须显式恢复。当前协议没有已验证的跨会话分页快照承诺，因此不从中间页 offset 继续；持久覆盖单位是完整目录列表。

`complete_initialization` 再次验证图结构、覆盖、预算和游标一致性，随后走既有快照安装事务；权威节点、游标和候选 ready 状态一起提交。普通快照安装不能绕过活动候选清除门禁。完成阶段仍读取整份候选做验证和安装，尚未消除整树内存与提交成本。

同一服务器恢复身份下，起点可以早于原权威游标，但最终提交必须追平或超过它。恢复身份已验证改变时才允许进入新的低计数域。此候选API尚未提供同一恢复身份下的已验证服务器游标重置授权入口；既有普通恢复路径的行为保持独立。

## 已接入的安全边界与缺失的调度器

源码已在普通事件接收／应用、快照安装及 runtime 上传、改名、删除等远端写入路径检查持久候选门禁；绑定新的连接也重新读取门禁。旧本地权威元数据、完整缓存与未上传内容由原有路径保留，不能用清缓存解除初始化状态。

这些检查是防止未完成候选期间误写的边界，**不是生产初始化调度器**。当前生产恢复仍请求完整 `RecoverySnapshot`，收到并核验整份内容后调用原安装路径。尚需实现的调度器负责：

- 建立和恢复候选，重新验证会话／根／恢复身份，派发目录票据并处理取消。
- 接收真实原生事件、推进候选、重查失效目录，取得独立 fence 后触发原子完成。
- 为 FUSE 设计可见性及权限规则，并验证已完成目录提前可见是否满足数据安全要求。
- 接入故障恢复、资源预算和阶段进度，完成真实 NAS、升级及性能验证。

因此不要手工向用户任务写入活动候选并期待当前生产 worker 自动续扫。也不能将本API的重开语义当成跨重启端到端续扫已经可用。

## 版本与验收入口

存储迁移、配套进程切换及旧版本拒绝打开新库的边界见[升级与恢复](UPGRADE_RECOVERY.md)。可观察的类型化重试原因见[初始化进度](INITIALIZATION_PROGRESS.md)。测试与交付状态只维护在[唯一验收入口](../DELIVERY_GATES.md)，不由本文接口说明替代。

源码入口：[候选模型与恢复](../crates/drive-core/src/initialization_store.rs)、[事务与安装操作](../crates/drive-core/src/initialization_store_ops.rs)、[Store迁移](../crates/drive-core/src/state.rs)、[runtime恢复和上传门禁](../crates/drive-runtime/src/bridge.rs)、[命名空间门禁](../crates/drive-runtime/src/namespace.rs)。
