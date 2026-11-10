# FUSE 请求 deadline、有界 Store 等待与挂载队列调优（#59）

2026-11-11，分支 `fix/59-fuse-request-isolation`（起点 `a9b97f3`）。本批只做数据面可观测性与可控制等待问题的最小修复，不实现完整的 metadata/data lane 隔离架构，也不替代 #23/#24 的根因修复。

## 修改

**挂载队列参数。** `CacheFilesystem::init` 在 FUSE_INIT 握手中应用 `max_background=64`、`congestion_threshold=48`，不再沿用 vendored fuser 的 16/12 默认；`congestion_threshold` 会被归一化到 `1..=max_background`。可用环境变量覆盖：`DRIVE_NATIVE_FUSE_MAX_BACKGROUND`、`DRIVE_NATIVE_FUSE_CONGESTION_THRESHOLD`（非法或 0 回落到默认值，不会 panic）。生效值经 `drive_fs::request::snapshot()` 暴露。

**Per-opcode 服务端 deadline。** 每个 FUSE handler 在分发入口创建 `RequestDeadline`，并随 reply 一起移动到真正回答的工作线程。预算默认 metadata 5 s、read 30 s、write 60 s、readdir 10 s（`DRIVE_NATIVE_FUSE_DEADLINE_{METADATA,READ,WRITE,READDIR}_MS` 可覆盖）。超过预算时记录 `fuse.request_timeout` 计数并向 stderr 写一行结构化事件（systemd journald 可采集）：

```
drive-native fuse.request_timeout op=getattr elapsed_ms=... budget_ms=... detail=...
```

**有界 Store 等待。** metadata 读路径（`lookup`/`getattr` 的排队分支）把无界的 `store.lock()` 换成 `SharedStore::lock_timeout(50ms)`；超时返回可重试的 `EAGAIN`，记录 `fuse.degraded_reason=store_lock_timeout` 事件、`retryable`/`lock_wait_timeout` 计数和 lock-wait 直方图。仅改 metadata/cache-hit 路径，不动 Store 结构与 #23/#24 的持有期。

**GetOverview 指标。** `GetOverview` 增加 additive 的 `fuse` 对象（`schema` 仍为 1，IPC 版本不变）：`configured_tuning`/`effective_tuning`、`request_budget_ms`、每 opcode `count/total_us/max_us/p50_us/p95_us/p99_us/timeouts/retryable`（分位数为 2 的幂桶上界）、`request_timeouts`、`retryable_replies`、`lock_wait` 直方图、`lock_wait_timeouts`、`degraded_events` 和 `deadline_capability`。

## 明确未实现（不要当作已通过）

- **没有内核级抢占/取消**：FUSE 没有可用的请求取消原语，一个已经进入阻塞 `hydrate`/worker RPC 的线程无法被 deadline 打断。实现是"检测 + 上报 + 对可控等待设上限"：`read` 的 cache-miss 路径在 worker 返回后若已超预算，返回 `EIO` 并计一次 timeout；慢 worker 未返回期间请求仍会占用它所在的队列线程。`fuse.deadline_capability` 字段显式标记为 `detect-and-report+bounded-store-wait`。
- **没有完整 lane 隔离**：metadata 与 data 仍共用既有队列结构，只通过更大的内核 background 配额和有界等待降低相互拖垮的概率；"16 个慢下载时 metadata p95<100 ms" 的产品级保证未实现。
- **未逐项改造的路径**：write/目录投影路径中仍有持有 Store 的执行段（见 #23/#24），未改成有界等待；`opendir`/`readdir` 的 Store 持有期保持不变（本 issue 只限制 metadata 读的排队等待）。
- 分位数是直方图桶上界（不是精确 HDR 分位），launch 后无背景采样线程。

## 验证

离线合成夹具（无真实 NAS）。内核串行执行：

```sh
DRIVE_NATIVE_FS_DIAGNOSTICS=1 cargo test -p drive-fs --all-features --locked \
  --test mounted -- --ignored --nocapture --test-threads=1 \
  kernel_mount_applies_configured_fuse_queue_tuning
DRIVE_NATIVE_FS_DIAGNOSTICS=1 cargo test -p drive-fs --all-features --locked \
  --test mounted -- --ignored --nocapture --test-threads=1 \
  kernel_metadata_store_busy_returns_retryable_eagain_within_bounded_wait
```

- 调优测试对照 `/sys/fs/fuse/connections/<id>/max_background` 与 `congestion_threshold`：修复前是 vendored 默认 16/12，修复后应为 64/48。
- 有界等待测试在测试线程持有 Store 的情况下发起 metadata 请求：修复前会一直阻塞，修复后 <1 s 返回 `EAGAIN`，并增加 `retryable_replies`/`lock_wait_timeouts`/`getattr.retryable` 计数。
- `GetOverview` 的 additive 字段由 `drive-runtime` 既有 overview 单元测试断言（configured/effective 64/48、预算、分位数字段形状）。

## 回滚

`git revert <commit>` 即可；不涉及 schema/IPC/数据库迁移。取消 `DRIVE_NATIVE_FUSE_*` 环境变量即回到 64/48 与默认预算，只是不再有条目化超时事件。
