# 已提交状态投影基础层

2026-10-31。本层基于 `7806d62`，并包含 `ca30b0b` 的 journal 传播范围精化，把[事务变化日志](STATUS_CHANGE_JOURNAL.md)转换成可供[增量状态索引](INCREMENTAL_STATUS_INDEX.md)使用的当前节点事实。源码为 [status_projection.rs](../crates/drive-core/src/status_projection.rs)。它没有启动索引拥有者、桌面订阅或缓存核验，也没有改变现有 desktop 查询。

## 接口与一致性

- `snapshot_status(budget)` 返回整个 Store 的可见 `StatusSnapshot { through, atoms }`。不裁剪挂载根；消费者以配置的根身份约束查询。根的 `parent=None` 保留。当前是一次 Store 串行边界内的有界捕获，不是可跨锁分页的快照会话。
- `capture_status_delta(after, budget)` 返回 `ProjectionBatch { from, through, nodes, affected_buckets }`。每个 `ProjectedNode { id, atom }` 的 `None` 表示该身份当前不可见或不存在。旧新名称桶都保留，用于移除旧贡献和核对同名遮蔽。
- `Capture::Rebuild` 明确区分 journal epoch/gap/cursor 超前、预算不足和关系图无效。失败不返回可发布的半份数据，不清理 journal，也不推进消费者水位。

Store 的独占写入所有权和调用方串行边界是前提。开始时确认 autocommit，拒绝读取同连接未提交的事务；读取捕获水位内全部变化，再在同一次调用中投影当前事实。`max_changes` 装不下全部变化时返回重建，不把日志页面误当事务边界。回滚不产生变化；先提交后外层返回错误的结果仍可捕获。独立写连接绕过 Store 属于所有权违例，不在此契约内。

本层不负责消费次序。IndexOwner 必须比较已发布 epoch/from/through，幂等忽略已确认旧批次，拒绝倒序、重叠或 gap；不能重新执行迟到删除。详见[索引接入门禁](INCREMENTAL_STATUS_INDEX.md)。

## 投影内容与可见性

`StatusAtom` 包含身份、父、名称、目录类型、`own` 状态、继承固定、内容可用性、下载错误和 `ContentIdentity`。内容身份含远端/本地版本、完整与脏内容引用，以及草稿 token/base/generation/checkpoint/dirty。它们是内部元数据，不是文件正文；未来诊断导出不能直接输出真实账号的 atom。

状态按当前 core 规则从元数据导出：节点冲突、脏草稿基准冲突、本地待上传、祖先 namespace_overlay、继承固定、版本差异和当前版本下载错误均参与。没有调用会检查磁盘的 `Store::status`、`open_cached` 或正文读取接口。下载错误不覆盖冲突/待上传；目录本身不携带下载错误。

内容有 blob、dirty_blob 或脏草稿引用时标为 `Unverified`，没有这些引用时标为 `Absent`。**本层不会生成 `Complete`**：缓存外部丢失不能被 SQL 引用证明不存在。后续核验/内容发布订阅必须按完整内容身份关联结果，未知内容由聚合核心保持未确认。目录离线摘要仍由子项决定。

可见性检查本地祖先删除、祖先同名遮蔽及权限阻断。权限同时沿本地父关系和 namespace_overlay 保存的远端父关系传播；这是 `access_blocked` 的双父边语义。节点或祖先被拒绝时返回不可见，不把权限变化转换成删除操作。无父节点、过深或循环祖先图要求重建。相较旧批量 seeds，仅保留可从可见祖先到达的节点，避免遮蔽目录下的后代作为孤立可见项泄漏。

## 增量范围与预算

普通单文件状态变化只投影该文件、必要的祖先事实和同名桶，不枚举整树。元数据父链按主键读取；同名桶与本地子项使用现有父/名称索引。目录冲突/内容版本等普通状态变化遵循 journal 的精化范围，不扩展后代；父/名称/类型、固定、删除等继承与命名空间变化扩展受影响子树。名称桶继承输入的扩展标记，不能把仅节点变化意外升级成整目录变化。

ACL 或 namespace_overlay 变化可能影响本地父之外的后代。需要展开目录时，本次捕获至多读取一次有界 overlay 远端父边集合，再沿相关本地/远端边扩展；不反复为每个节点执行全局 `access_blocked` 递归视图。被遮蔽或重新可见的同名对象也扩展其受影响后代。完整快照只用于初始化/明确重建，不可作为每次状态查询的实现。

预算包含变化数、节点数、名称桶数、逻辑字符串/载荷字节、工作步数与软期限。节点上限也约束投影用的祖先缓存；宽树、深祖先或大量 overlay 边可能要求更大预算或后续分块重建。字符串读取前检查 SQLite 编码长度，容器与输出复制另留固定余量；字节计账不是 SQLite/分配器 RSS 的硬限制。软期限在循环/SQL 行边界检查，不能中断一条已执行的 SQLite 语句。

当前初始捕获会保持调用方 Store 边界直到完成或预算拒绝；[统一提交唤醒](STORE_COMMIT_NOTIFICATIONS.md)和[纯树原子批次](INCREMENTAL_STATUS_INDEX.md)已提供基础。大型树的分块 WAL 固定快照、瞬态 flights、核验结果及运行时发布仍待接入。本层通过测试不代表大目录启动或桌面性能已经改善。

已明确 deleted、同名遮蔽或无权的节点/本地祖先立即投影为 None；不再继续核验其不可见历史父链，避免遗留 tombstone 缺父使整个可见树被拒绝。远端 ACL 父链遇到明确无权同样短路。增量中仍输出该身份的 None，以移除旧展示贡献；可见节点缺父或循环仍要求重建，不静默省略。

## 验证入口

针对性测试见 [status_projection_tests.rs](../crates/drive-core/src/status_projection_tests.rs)：完整提交/预算拒绝、回滚/外层失败、同名遮蔽、旧新位置、ACL 远端父边、继承规则、关系错误、内容未核验与无正文 IO、epoch/gap 和只读无副作用。

2026-10-31 在 WSL Ubuntu 24.04 独立 `status-projection-004` 中，14 项投影测试及 core 全功能 209 项通过，0 失败、0 忽略。还覆盖实际 `replace_local` 的原子替换、目录跨父移动后继承固定，以及不适用下载错误不能覆盖冲突/目录状态。运行入口为 `cargo test --offline --locked --all-features -p drive-core --lib status_projection`，随后 `cargo test --offline --locked --all-features -p drive-core`。

独立 target、2 jobs 的负载已登记到现有 P0 时间轴。首次编译的 SQL 整数转换错误及新增替换测试缺少 fs_root 前置的失败均保留在本地记录，最终运行通过后才登记上述结论。未运行 NAS、桌面、FUSE、全 workspace 或性能验收；公开内容仅含通用结论。
