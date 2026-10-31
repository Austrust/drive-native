# Store 提交边界与合并唤醒

本批为后续增量状态索引增加统一入口，不改变当前 desktop 状态算法，也不订阅 NAS。`drive-runtime`、`drive-fs` 的生产共享 Store、对应测试和示例使用 `Arc<SharedStore>`，不再直接暴露 `Arc<Mutex<Store>>`。

## 锁与语义水位

`SharedStore::lock/try_lock` 返回可解引用为 Store 的 guard，保持标准 Mutex 的阻塞、WouldBlock 和 PoisonError 语义。poison recovery 取出的仍是同一包装 guard，没有绕过提交检查的底层 Mutex 入口。只持有 Store 的独立 CLI、初始化或测试流程仍可使用普通 `Store`；它们没有并发派生索引。

guard 析构与外层返回值无关。SQLite `total_changes` 只是是否需要检查的快速提示；实际通知比较已提交 journal 的 epoch、head 与 overflow。事务和保存点回滚、不变 UPDATE、只读查询、消费自身 prune、未丢数据的预算修改不会产生语义唤醒。真实 overflow（包括缩小预算丢弃历史）会失效。显式 prune 虽不发图标通知，落后消费者重新读 journal 时仍得到 gap，不能假定旧 cursor 完整。

正常只读 guard 析构只读取 SQLite 的本地计数和 autocommit 标志，不执行水位 SQL。可变解引用额外检查边界，覆盖以 DerefMut 替换整个 Store、但新连接计数恰好相同的情况。多个内部事务在同一 guard 内提交，释放边界仅合并为一次本地 revision；这不是逐事务历史，具体变化仍由 journal 提供。

外层返回 Err 前已提交 dirty、节点或下载状态时，guard 仍会通知。guard 释放时尚有未提交事务、水位读取失败或发生 panic/poison，发布 `verified=false`，不把内部序号当已提交水位。此通知表示必须重新核对/回滚，不表示写入成功。后续正常 rollback/修复后重新读取有效水位可恢复边界；已 poison 的 Mutex 继续拒绝正常 lock，并保持未确认，不能通过读取 PoisonError 的内部 guard 自动恢复可信状态。初始化水位读取失败从非零未确认 revision 开始，不能伪造 ready 零状态。

## 消费与发布契约

- `change_state()` 返回单个原子字中的 `{revision, verified}`，Acquire 读取；revision 与有效性不会来自不同写入。
- guard 在仍持有 Store 锁时以 Release 写入新状态，使旧派生展示先失效；释放 Store 锁之后才向容量为 1 的通道 `try_send`。没有用户回调、flights 锁或阻塞发送。
- `take_notifications()` 仅允许领取一次。重复领取或接收器丢弃后的重新领取明确返回 `AlreadySubscribed`，不静默替换旧消费者。队列满只合并唤醒；消费者每次收取后读取最新 state，不能按消息条数计变更。receiver 不持有 Store 或 sender；SharedStore 的 owner 信号字段先于 SQLite 析构，先原子失效并发送终止唤醒，然后通道断开。因此即使消费者还未读到断开，state 也不能继续显示 verified。接收器断开也必须使派生索引失效。
- 索引在持 Store 锁的同一捕获边界读取 journal 和 `change_state`，投影/发布遵循[增量索引契约](INCREMENTAL_STATUS_INDEX.md)。清理日志不改变语义 revision。查询仅可使用与当前 verified revision 一致的发布代次，并在返回前复核；不因旧值暂时还可读就声称当前已同步，尤其不能继续暴露已撤销的 ACL 路径。
- 此 revision 只涵盖 Store 边界，不涵盖 flight 生命周期、外部内容丢失和独立可用性核验；完整索引仍需相应瞬态 stamp。原 flights→Store 锁顺序不变。

消费者对当前持锁批次内尚未释放的内部提交不应发布中间 atom；以完整 Store 串行边界作为可捕获代次。唯一所有者 `into_inner()` 取回 Store 时失效旧通知域；不得把脱离包装后的修改继续当成同一共享索引。

## 验证范围

专用合成回归覆盖外层失败前提交、多次写合并、只读、不变写、事务/保存点回滚、prune 和预算变化、overflow、未提交 guard、初始水位失败、try_lock、poison 恢复、满通道/断开、唯一所有者取回，以及真实草稿方法 dirty 已提交后文件重开失败。测试仅使用临时数据。

2026-10-31 独立 WSL 功能验证：core 206 项、runtime 105 项、drive-fs 20 项通过；runtime 另有一个既有 FUSE 取消用例仍按默认配置忽略。workspace 全 features / 全 targets 编译检查通过，包含所有示例。随后 owner 析构失效补丁再次通过 13 项 guard 专项及全部消费者编译检查。首轮独立构建快照遗漏已提交的 evidence 样本而导致 include_str 编译失败，补齐快照后通过；失败日志保留。测试只用临时状态、合成 IPC 和私有 FUSE，未访问 NAS，全部负载起止登记到 P0 时间轴。

```sh
cargo test --offline --locked --all-features -p drive-core
cargo check --offline --locked --all-features --workspace --all-targets
cargo test --offline --locked --all-features -p drive-runtime
umask 022
cargo test --offline --locked --all-features -p drive-fs --lib --tests -- --include-ignored --test-threads=1
cargo test --offline --locked --all-features -p drive-core --lib shared_store
```

仍需完整索引的原子发布、权限撤销时查询竞态、随机 oracle、瞬态订阅、性能与全量同步验收。事务 journal 的写入开销和 guard 边界开销须在独立性能窗口量化；本次编译/功能测试不能替代该比较。没有替换当前 desktop 算法或据此声明性能达标。
