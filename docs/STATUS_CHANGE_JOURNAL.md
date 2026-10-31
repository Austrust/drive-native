# 提交状态变化日志：基础层

2026-10-31。基于 `2629ca6` 新增的独立实现，设计来源为当前 [P1 §C 契约](P1_ARCHITECTURE.md)及本地写入口审查；没有移植官方二进制或其他来源代码。本批只提供可相信的差分来源，**没有切换 desktop 查询、没有实现增量聚合树或瞬态传输订阅，也不声明性能改善**。

## 存储与提交契约

[status_changes.rs](../crates/drive-core/src/status_changes.rs) 在 Store 启动迁移和恢复之后，为唯一写连接安装 TEMP 表与 TEMP trigger。main 数据库 schema 版本仍为 18，外部只读连接看不到这些 TEMP 对象。Store 每次打开生成新 epoch；派生索引必须以该 epoch 的一致快照开始。重启后旧 cursor 返回明确 EpochChanged，不静默返回空差分。

trigger 与原数据的语句、事务和 savepoint 一同提交或回滚。公开查询只接受 autocommit 边界，禁止读取本连接尚未提交的临时行。某个公共方法先提交一次变更、随后文件 IO 或另一事务失败时，第一次已提交的变化仍可读取，不依赖最外层返回 Ok。

启用本写连接的 `recursive_triggers`，使 INSERT OR REPLACE 的隐式删除同样可见。没有增加 main-schema trigger。任何以后引入的主库 trigger、连接替换、关闭 recursive_triggers 或绕过 Store 的写入都须重新审查：本接口不捕捉其他连接的写入；现有唯一 writer 所有权仍是前提。

## API 与预算

- `status_change_watermark()`：epoch、最新行序号、已丢弃至何处、保留条目/计账字节、溢出次数和预算。
- `read_status_changes(after, limit)`：有限页的变化、next cursor、当前水位及 has_more；epoch 不同、历史丢弃、cursor 超前分别返回 Rebuild 原因。
- `prune_status_changes(through)`：显式确认已消费范围，落后消费者下次收到 gap。
- `set_status_journal_budget()`：条目与字节双预算，缩小预算无法保留现有缓冲时也显式推进 gap。

同预算设置和已确认范围的重复 prune 在 SQL 写入前早退，不增加旧 `local_revision`。真正 prune、预算修改和 TEMP 触发器仍会计入 SQLite `total_changes`；未来发布通知必须改用语义水位及瞬态序号，不能把清理日志造成的粗通知当作节点变化。纯聚合核心与后续提交接口见[增量目录状态](INCREMENTAL_STATUS_INDEX.md)。

默认保留 8192 条/4 MiB；配置上限 65536 条/16 MiB。字节是已记录 UTF-8 字段加每行固定余量，**不是 SQLite 内存或进程 RSS 的硬上限**。单行过大或条目/字节超限时，清空当前有界缓冲并将丢弃水位推进到本行，随后允许继续记录；没有“丢了一行却继续声称完整”的状态。溢出本身也跟随事务回滚。

每条写入使用元信息计数和主键水位检查，不扫描 COUNT/SUM 或排序整个缓冲。显式 prune 可汇总有限已消费前缀。序号是触发器产生的行差分序号，不是服务器游标，也不等于一个事务。

## 变化覆盖

每行包含来源表、INSERT/UPDATE/DELETE、旧新身份、旧新父和名称、传播范围。只保存状态失效所需字段，不保存文件正文、令牌或操作正文。值完全未改变的 UPDATE 不记录。

| 来源 | 原因与消费者责任 |
| --- | --- |
| nodes | 身份、位置、版本、内容引用、脏写、pin、删除、冲突；包含旧新位置。UPDATE 只有身份、父、名称、类型、pin 或 deleted 改变时传播后代；版本、内容和冲突改变只影响自身及祖先。INSERT/DELETE 保守传播后代 |
| drafts | 存在、base、generation、checkpoint、dirty，含写入先置 dirty 后 IO 失败情形 |
| transfers | 下载阶段、错误和清除错误，不能只监听 nodes |
| operations | 上传 pending/inflight/uncertain/conflict/confirmed 对应任务状态 |
| namespace_ops、namespace_overlay | 操作阶段、祖先 overlay、位置/权限继承；正文变化只发失效，不复制正文 |
| local_creations、local_unlinks | 同名遮蔽、解除遮蔽及本地删除；消费者核对旧新同名桶 |
| access_denied | 权限根变化；access_blocked 是递归视图，消费者需失效后代 |
| remote_aliases、content_version_aliases | 服务器身份绑定、先到事件镜像折叠、版本别名 |
| node_metadata、local_modes | 目录/属性投影；不是每次都改变图标，消费者应按类别处理 |
| cache_releases | 请求、确认和重试任务状态；真正缓存可用性仍取决于节点发布/释放 |
| engine_bridge_intents、engine_bridge_tombstones | 保留官方参照通道产生的状态来源，避免其节点变化绕过捕捉 |

排除 fs_inodes、cache_access、cache_published 的 inode/LRU/保留时间记账，metadata 设置和收件箱/seen 游标记账，upload_origins/upload_paths 以及 engine_upload_stages 的中间协议证据。它们本身不改变现有逐文件状态；实际提交、冲突、命名空间和内容变化由上述表捕捉。若以后桌面新增对这些字段的状态依赖，必须扩充覆盖，不能默认日志已经包含它们。

## 后续消费者必须遵守

本日志记录**失效事实，不是完整历史节点状态**。读取后在同一 Store 串行边界内投影当前 atom，合并所有旧新位置；处理至选定水位后才发布派生索引。页面边界不代表事务边界，不能发布半个事务的树。未知/丢弃历史必须重建，不能跳到最新 cursor 冒充补齐。

后续已补[Store guard 的提交失效及唤醒](STORE_COMMIT_NOTIFICATIONS.md)、[同边界投影](STATUS_PROJECTION.md)和[纯树同代路径/摘要](INCREMENTAL_STATUS_INDEX.md)。后台调度、重建与差分追赶、实际内容核验、跨线程发布及 flights epoch/seq 尚未闭环。SQLite trigger 不能捕捉运行中下载生命周期或外部删除缓存文件；现有 desktop batch 继续运行并作为对照，不能仅因新增基础模块就宣布 P1 完成。

## 验证

[单元测试](../crates/drive-core/src/status_changes_tests.rs)覆盖事务/保存点回滚、提交后外层失败、重命名/原子替换、身份绑定/同名遮蔽、权限、关系表、REPLACE 隐式删除、错误清除、分页、prune、预算溢出与缩容、重启 epoch、主 schema/外部只读隔离及排除 inode/LRU/未应用事件。

2026-10-31 在 WSL Ubuntu 24.04 的独立 `status-change-journal-004` 运行中：专项 13/13 通过；core 全功能回归 174 通过、0 失败、0 忽略（包含上述 13 项及既有 schema 12 升级测试）。新增 schema 清单测试要求未来新表/字段显式分类，避免遗漏写来源。主库 schema、外部只读连接、事务与恢复行为通过本批覆盖；未运行整个 workspace、真实 NAS、桌面、FUSE 或性能测试，不据此关闭 P1/发布门禁。

运行入口：`cargo test --offline --locked --all-features -p drive-core --lib status_changes`，随后运行 `cargo test --offline --locked --all-features -p drive-core`。使用独立 target、两个编译 job，资源干扰记录到既有基线的负载时间轴；并行开发负载存在，测试耗时不作性能比较。详细日志保留本地，公开内容仅为上述脱敏结论。
