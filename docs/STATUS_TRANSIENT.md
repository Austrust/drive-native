# 瞬态状态：下载生命周期与正文核验

2026-10-31。本模块为增量展示提供独立于数据库的下载与正文可用性来源。[status_transient.rs](../crates/drive-core/src/status_transient.rs) 与 [IndexOwner](../crates/drive-core/src/status_index.rs) 已实现纯内存接口；没有启动线程、访问正文、连接 NAS 或替换桌面查询。真实 flight 生命周期、核验任务、提交屏障与桌面发布由后续运行层接入，不能将本批通过等同于性能目标完成。

## 来源、批次与确认

`TransientSource` 属于 FSD 生命周期。调用者为每个新来源分配唯一 epoch；网络重连、worker 替换和 RPC 会话更换不新建来源。下载以 `(id, version, token)` 记录；调用者须为每次实际下载生成不复用的生命周期 token，不能只拿文件版本充当 token。重复开始幂等，完成或取消只移除精确匹配的一项。共享同一下载的多个读取者不能各自结束该 flight，新旧版本的独立下载可以并存。

每次状态变化增加瞬态 sequence，并记录完整的单节点状态；不增加数据库序号。`read(after, limit)` 返回严格连续批次，或明确的 `SnapshotRequired`。日志受条数和字节预算约束；超限释放队列，保留权威集合，并要求落后消费者重新捕获。来源不得悄悄丢掉 gap。

消费者调用 `IndexOwner::apply_transient`。epoch、from、每条 sequence 和 through 必须连续；最近 32 个已确认批次保存有界收据，只允许内容完全相同的精确重复。其他迟到、篡改、重叠和缺口批次拒绝。批内相同身份只安装最终状态；预检失败不提交部分节点或水位。正常变更只处理相关节点与祖先；隐藏或已删除身份仍有 flight 时，保留瞬态项，不为它重建整树，也不将其作为可见文件展示。

成功发布后才调用 `source.acknowledge(published_cursor)` 回收已消费日志。确认不改变来源序号，不生成通知；确认快照时用实际安装的 `through`，不能确认捕获之后的新 head。没有确认会最终触发有界日志的恢复要求，即使消费者已经读过这些记录。连续确认后的长期运行不会仅因总事件数增加而周期重建。

## 实际正文身份与一次性核验票据

`ContentKey::from_content` 从实际本地正文提取身份，按以下优先级选择：

- dirty draft：`token + base + generation`；checkpoint 与远端期望版本不属于正文身份。
- dirty blob：不透明的 blob 存储标识（`blob_id`）。
- 完整 blob：不透明的 blob 存储标识与本地版本 `local_version`。

Store 写入时计算摘要，并在调用方提供 expected 时核对长度与摘要；blob 文件名始终是随机 UUID，并非正文 SHA-256。这里比较存储对象身份，不按内容摘要合并对象；字段名称也使用 `blob_id`，避免运行层把摘要误当缓存文件标识。

仅远端版本推进时，旧完整 blob 仍可离线使用。数据库重新投影同一实际 key 保留已有可用性；实际 key 改变则需要新证据。真实 `create_local_file` 使用非空的 `local-create-v1`，零字节 dirty blob 与该文件后续 draft 均能产生有效 key。

核验调用顺序为：

```text
bind_content(id, actual_key)
ticket = begin_verification(id, expected_actual_key)
在来源锁外检查该 ticket.content() 对应的正文
record_availability(ticket, Complete 或 Missing)
消费并发布瞬态批次，然后 acknowledge
```

`VerificationTicket` 只能由来源构造，包含私有的来源 epoch、单调探测 token、身份与正文 key。新探测替代旧探测；解绑/重新绑定也撤销旧票，即使后来恢复相同 key。结果只能提交一次：同值结果没有展示变化，也会消费票据，不能用同一票稍后将 Missing 改回 Complete。取消只作用于当前精确票据；过期或另一 epoch 的结果不能改变状态或消耗新票。序号耗尽与预算拒绝发生在变更前，提交失败保留当前票据供明确重试。

开始核验本身不撤销已有完整证据，也不生成展示 sequence；若实际释放、删除或 IO 错误已经发生，运行层必须及时解绑或提交 Missing，并设置 pending。未知和 Missing 都不会显示离线完整；此纯模块不会自行下载修复。单纯取消探测不等于证实正文缺失。

消费者再次比较当前 atom 的实际 key。旧正文结果不能覆盖新正文，显式解绑会撤销匹配旧 key 的已核验证据；同批随后绑定另一 key 时，折叠过程仍保留该撤销。gap 后的全量快照不依赖解绑历史，当前 key 没有匹配证明就恢复为未确认。上层仍必须在正文切换、数据库提交与来源绑定之间实行统一的生命周期顺序；不能靠稍后偶然到达的核验补救已经发布的不一致状态。

## 快照、发布与预算边界

首次安装、epoch 变化和 gap 使用 `install_transient_snapshot(expected_database, snapshot)`；它要求数据库 stamp 精确匹配，并在快照替换时将缺少证据的完整内容恢复为未确认。相同瞬态 stamp 的不同快照被拒绝。

数据库 stamp 相等只能检查关联，不能证明调用者同时捕获了两个来源。运行层必须按已有锁顺序捕获数据库水位和瞬态快照，在所有未消费变更期间遮蔽 ready 状态，补齐捕获后的连续变化后再发布。被拒绝的批次保留旧代用于恢复，不表示旧代仍然最新；特别是 ACL 撤销不能在 pending 窗口继续暴露旧路径。此职责见[增量状态契约](INCREMENTAL_STATUS_INDEX.md)和 [Store 提交通知](STORE_COMMIT_NOTIFICATIONS.md)。

来源限制节点、总 flight、每节点 flight、日志行数、日志字节和存活逻辑字节；探测表计入预算，日志释放同时回收 ring 容量。树拥有者同时计算旧/新存活状态与 staging 的峰值逻辑额度：删除节点也不能先扣除尚未释放的旧代，再声称可以容纳准备内存。所有校验完成后才安装树与瞬态水位。逻辑余量不是进程 RSS 或分配器保证，也不承诺 OOM 恢复。调用者持有的快照、票据和未交付批次仍需运行层的统一预算。

## 验证记录

本批在独立 WSL source/target、功能验证负载登记下进行，不修改运行基线。初次状态专项 60 通过、2 失败：旧测试将相同正文 key 从已确认直接改成 unknown，与本批“同正文保留核验结果”的契约冲突；修正为实际正文更换，并保留首次失败记录。随后完整 core 253 通过及真实旧 desktop oracle 1 项通过。oracle 直接调用旧 `desktop::states`，不是用同一个 owner 重建冒充旧算法。

增加 ack、实际零字节创建/draft 等用例后，状态专项 68 通过，随后旧 desktop oracle 1 项通过，负载记录确认正常退出。审查进一步发现同 key 探测乱序/ABA 风险，现已加入一次性票据和显式解绑处理，最终完整 core 回归 264 项通过，0 失败、0 忽略；包括 same-key ABA、乱序、跨 epoch、取消、失败重试、同批解绑折叠和 gap 快照恢复。随后最终旧 desktop oracle 1 项通过。测试进程均正常退出，使用 1 个编译 job；同时间其他隔离功能验证也在运行，本轮耗时不作性能证据。

[瞬态专项](../crates/drive-core/src/status_transient_tests.rs)覆盖多版本 flight、精确结束、重复批次、gap、epoch、内容替换、旧版缓存、缺失、draft、隐藏身份、预算、序号耗尽、确认回收与真实零字节创建。[owner 专项](../crates/drive-core/src/status_index_tests.rs)补充展示 generation 耗尽不半提交、缩小批次的旧代峰值预算。没有本批 NAS、Mint、实际后台接入或性能通过声明。

复现：使用隔离 `CARGO_TARGET_DIR` 执行 `cargo test --offline --locked --all-features -p drive-core`，然后执行 `cargo test --offline --locked --all-features -p drive-runtime --test status_index_oracle`。共享负载期间的耗时不作为性能比较证据。
