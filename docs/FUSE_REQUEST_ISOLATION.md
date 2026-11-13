# FUSE 请求 deadline、有界 Store 等待与挂载队列调优（#59）

2026-11-11，分支 `fix/59-fuse-request-isolation`（起点 `a9b97f3`）。本批只做数据面可观测性与可控制等待问题的最小修复，不实现完整的 metadata/data lane 隔离架构，也不替代 #23/#24 的根因修复。

> **2026-11-13 复核修正（issue #67 §3.4）**：本文件初版把 `max_background=64` 当作 #59 metadata 隔离效果的来源，与实测不符。两棵代码树 + `max_background=1` 决定性对照表明：`max_background` 只约束**内核下发的后台请求**（readahead/writeback，`FR_BACKGROUND`），不限制前台 `read()/getattr()`；16/64 个前台慢 cache-miss 读都**没有**抬升 metadata p95。因此 `16→64` 不能用于验收「16 个慢下载时 metadata p95 目标」。本文件已按实测改正；**本次不调参、不改变默认 64/48，也不把调参当收益**。实测证据见下文「#67 §3.4 复核」。

## 修改

**挂载队列参数（可配置项，不是隔离收益来源）。** `CacheFilesystem::init` 在 FUSE_INIT 握手中应用 `max_background=64`、`congestion_threshold=48`，不再沿用 vendored fuser 的 16/12 默认；`congestion_threshold` 会被归一化到 `1..=max_background`。可用环境变量覆盖：`DRIVE_NATIVE_FUSE_MAX_BACKGROUND`、`DRIVE_NATIVE_FUSE_CONGESTION_THRESHOLD`（非法或 0 回落到默认值，不会 panic）。生效值经 `drive_fs::request::snapshot()` 暴露。实测该值只影响内核 background 请求配额（`FR_BACKGROUND`：readahead/writeback），前台 `read()`/`stat()` 的并发与 metadata 延迟不受它约束（见复核 H1）。

**Per-opcode 服务端 deadline（detect-and-report）。** 每个 FUSE handler 在分发入口创建 `RequestDeadline`，并随 reply 一起移动到真正回答的工作线程。预算默认 metadata 5 s、read 30 s、write 60 s、readdir 10 s（`DRIVE_NATIVE_FUSE_DEADLINE_{METADATA,READ,WRITE,READDIR}_MS` 可覆盖）。超过预算时记录 `fuse.request_timeout` 计数并向 stderr 写一行结构化事件（systemd journald 可采集）：

```
drive-native fuse.request_timeout op=getattr elapsed_ms=... budget_ms=... detail=...
```

预算只对该 opcode 内**可控的 Store 等待**设上限；handler 之外的排队（内核侧）不在预算约束内，实测最坏尾仍 ~4 s（见复核 H3）。`fuse.deadline_capability` 字段显式标记为 `detect-and-report+bounded-store-wait`。

**有界 Store 等待（两级）。** metadata 读路径（`lookup`/`getattr` 的排队分支）把无界的 `store.lock()` 换成两级等待：先等 50 ms（`SharedStore::lock_timeout`，指数退避轮询）——这只是**软阈值**，超过时记录 `fuse.degraded_reason=store_lock_timeout`、`lock_wait_timeout` 计数和 lock-wait 直方图，**不返回错误、继续等待**；随后继续等到该 opcode 的预算（metadata 默认 5 s，可用 `DRIVE_NATIVE_FUSE_DEADLINE_METADATA_MS` 调整）——只有**整个预算耗尽**才返回可重试 `EAGAIN` 并记一次 `fuse.request_timeout`。原因：`std::fs`/`stat`/`ls` 不重试 `EAGAIN`，把 50 ms 瞬时写者竞争漏成 `EAGAIN` 会让 cache-hit 路径在普通应用里表现为失败；语义应为"瞬时竞争透明等待、真正长期阻塞才降级报错"。仅改 metadata/cache-hit 路径，不动 Store 结构与 #23/#24 的持有期。实测该机制把病态 Store 争用下的**中位**延迟从无界阻塞压到 300 ms 级可重试失败，但不缩短最坏尾（见复核 H3）。

**GetOverview 指标。** `GetOverview` 增加 additive 的 `fuse` 对象（`schema` 仍为 1，IPC 版本不变）：`configured_tuning`/`effective_tuning`、`request_budget_ms`、每 opcode `count/total_us/max_us/p50_us/p95_us/p99_us/timeouts/retryable`（分位数为 2 的幂桶上界）、`request_timeouts`、`retryable_replies`、`lock_wait` 直方图、`lock_wait_timeouts`、`degraded_events` 和 `deadline_capability`。

## #67 §3.4 复核（2026-11-13，lab FUSE 实测）

### 装置与口径

- 合成夹具（无真实 NAS、无用户目录）：root + `d000` + 64 个 256 KiB cached 文件 + 96 个慢 cache-miss 文件；test-only `SlowHydrator` 每次 hydrate 睡 1500 ms 后返回 EIO，使内核 READ 在 sleep 期间保持 in-flight；`with_metadata_ttl_ms(0)` 强制每次 stat/readdir 走 FUSE。
- 用例一次进程、一个挂载：`control → slow1 → slow16 → slow64 → store_hold`，每 case 10 s；`store_hold` 人为持共享 Store 4 s，metadata 预算 300 ms。
- 表中为每 rep 中位数；integration（本分支，`acd32bf`）n=3、base（`a9b97f3`）n=2；`max_background=1` 为决定性对照（`DRIVE_NATIVE_FUSE_MAX_BACKGROUND=1`，并经 sysfs 核对生效）。
- 证据 harness：`crates/drive-fs/tests/perf_isolation_probe.rs`（test-only `#[ignore]`，随本分支提交）。

metadata p95（µs）：

| 场景 | base（16/12） | integration（64/48） | `max_background=1` |
|---|---|---|---|
| control stat / p99 | 51 / 171 | 47 / 171 | —（未测） |
| control readdir | 131 | 117 | — |
| control 4 KiB cache-hit READ | 15 | 12 | — |
| slow16 stat | 37 | 45 | — |
| slow64 stat | 30 | 62 | 30 |
| slow64 内核 `waiting` 峰值 | **66/67**（16 配额仍允许 64 前台慢读同时在飞） | 67 | **67** |

### 结论

- **H1（否证）**：16/64 并发前台慢 cache-miss 读**不抬升** metadata p95。把 integration 强制设为 `max_background=1` 后，slow64 仍有 67 个请求在飞、stat p95 仅 30 µs。→ `max_background` 只约束后台请求（`FR_BACKGROUND`；vendored fuser `vendor/fuser/src/lib.rs` 的 `set_max_background` 文档亦只写 readahead 一类），前台 `read()/getattr()` 不受限；`16→64` 对 metadata 目标无效。
- **H2（数据 lane 内无残余队头）**：单/16/64 个慢数据请求都不拖慢 cache-hit 小读（4 KiB READ p95 12–15 µs 平坦），userspace 的 `read_queue`/`network_queue` 分离有效。
- **H3（残余串行点 = 共享 Store 写锁；EAGAIN 改善中位、不缩短最坏尾）**：持 Store 4 s、预算 300 ms 时，integration **36/36 返回可重试 `EAGAIN`、p50≈300.4 ms**；base（无预算）**无界阻塞 4.001 s、0 EAGAIN**。但两棵树**最坏尾都 ~4 s**（integration 每 rep 一个 3.905 s 样本；`max_background=1` 时 2.404 s），而 integration 服务端 `lookup` 直方图 max 仅 300.371 ms → **尾巴在 handler 之外（内核侧），预算约束不到**。同时 integration 会把慢路径转成 stat/ls 可见的 `EAGAIN` 失败。
- **H4（稳态无回归）**：control 稳态 base vs integration：stat p95/p99 51/171 vs 47/171 µs、readdir 131 vs 117 µs、4 KiB READ 15 vs 12 µs（µs 级噪声内）；CPU 相当；degraded 日志 28 行（<1 KB）；每请求成本增加 ≈0.1–0.2 µs CPU（1 次 `Instant::now` + 1 次 atomic bucket）。

### 对 #59 AC 的修正

- **AC1（cache-hit metadata 有界）**：在病态 Store 争用下两棵树最坏尾都 ~4 s，「cache-hit metadata <1 s」不能按尾延迟表述。本批能保证的是「多数请求在预算内可重试失败 + 结构化可见性」，不是最坏尾收敛。
- **AC2（16 慢下载时 metadata p95 目标）**：机制不对应——慢前台读不抬升 metadata p95，该 AC 不应由 `max_background` 调参验收（不是回归，是验收机制与实现不对应）。
- **残余串行点**：共享 Store 写锁。4 个 metadata worker（`crates/drive-fs/src/lib.rs` 的 `LocalQueue`）在 Store 长持有下可全部阻塞在 `SharedStore::lock_timeout`（`crates/drive-core/src/shared_store.rs`），无等待者去重/优先级。下一步应针对 Store 持有期（#23/#24），而不是继续调 `max_background`。

### 复现

```sh
# integration（本分支）证据 harness：需要普通用户 + /dev/fuse
DRIVE_NATIVE_FS_DIAGNOSTICS=1 \
  cargo test -p drive-fs --release --all-features --locked \
  --test perf_isolation_probe -- --ignored --nocapture --test-threads=1

# 决定性对照：强制 max_background=1（其余不变；进程内会打印 sysfs 生效值）
DRIVE_NATIVE_FS_DIAGNOSTICS=1 DRIVE_NATIVE_FUSE_MAX_BACKGROUND=1 \
DRIVE_NATIVE_FUSE_CONGESTION_THRESHOLD=1 \
  cargo test -p drive-fs --release --all-features --locked \
  --test perf_isolation_probe -- --ignored --nocapture --test-threads=1
```

base（`a9b97f3`）树没有 #59 的 `drive_fs::request` 模块，需使用同 harness 的 `perf_isolation_probe_base.rs` 变体（仅把 `drive_fs::request::snapshot()` 打桩，其余逐字节相同）；本分支提交的是 integration 版。输出为 `PERF case=... probe=... p50_us/p95_us/p99_us/eagain/...` 与 `PERF case=... fuse waiting_*` 行。

### 限制

- lab 合成夹具 + 单一挂载，不代表真实 NAS/桌面端到端；base 只有 n=2（integration n=3），两树差异以 µs 级噪声内为主。
- 最坏 ~4 s 尾巴只在 handler 之外（内核侧）观察到，尚未用内核 tracepoint 定位；WILLNEED 后台 readahead 压力实验（`perf_stream_probe.rs`）已编译但**未运行**，列为 follow-up。
- `store_hold` 的人为 4 s 持有比真实写者更病态，只用于暴露语义边界，不是现场 SLO 估计。
- 分位来自探针进程内直方图/采样，不是内核 eBPF 全量统计。

## 既有内核回归（本批）

离线合成夹具（无真实 NAS）。内核串行执行：

```sh
DRIVE_NATIVE_FS_DIAGNOSTICS=1 cargo test -p drive-fs --all-features --locked \
  --test mounted -- --ignored --nocapture --test-threads=1 \
  kernel_mount_applies_configured_fuse_queue_tuning
DRIVE_NATIVE_FS_DIAGNOSTICS=1 cargo test -p drive-fs --all-features --locked \
  --test mounted -- --ignored --nocapture --test-threads=1 \
  kernel_metadata_store_busy_returns_retryable_eagain_within_bounded_wait
```

- 调优测试对照 `/sys/fs/fuse/connections/<id>/max_background` 与 `congestion_threshold`：修复前是 vendored 默认 16/12，修复后应为 64/48。**注意：该测试只验证配置生效，不证明 metadata 隔离效果**（见复核 H1）。
- 有界等待测试在测试线程持有 Store 的情况下发起 metadata 请求：修复前会一直阻塞，修复后 <1 s 返回 `EAGAIN`，并增加 `retryable_replies`/`lock_wait_timeouts`/`getattr.retryable` 计数。
- `GetOverview` 的 additive 字段由 `drive-runtime` 既有 overview 单元测试断言（configured/effective 64/48、预算、分位数字段形状）。

## 明确未实现（不要当作已通过）

- **没有内核级抢占/取消**：FUSE 没有可用的请求取消原语，一个已经进入阻塞 `hydrate`/worker RPC 的线程无法被 deadline 打断。实现是"检测 + 上报 + 对可控等待设上限"：`read` 的 cache-miss 路径在 worker 返回后若已超预算，返回 `EIO` 并计一次 timeout；慢 worker 未返回期间请求仍会占用它所在的队列线程。
- **没有完整 lane 隔离**：metadata 与 data 仍共用既有队列结构。复核表明残余串行点是**共享 Store 写锁**，不是内核 background 队列；"16 个慢下载时 metadata p95<100 ms"的产品级保证未实现，且不能由 `max_background` 调参验收。
- **未逐项改造的路径**：write/目录投影路径中仍有持有 Store 的执行段（见 #23/#24），未改成有界等待；`opendir`/`readdir` 的 Store 持有期保持不变（本 issue 只限制 metadata 读的排队等待）。
- **EAGAIN 的可观测性已具备，透明重试未做**：`fuse.request_timeout` 事件 + `retryable_replies`/`lock_wait_timeouts`/每 opcode `retryable` 计数已覆盖降级可见性；不在 handler 内重试 `EAGAIN`——那只会延长内核队列占用，且 `std::fs` 侧语义仍是不重试。
- 分位数是直方图桶上界（不是精确 HDR 分位），launch 后无背景采样线程。

## 回滚

`git revert <commit>` 即可；不涉及 schema/IPC/数据库迁移。取消 `DRIVE_NATIVE_FUSE_*` 环境变量即回到 64/48 与默认预算，只是不再有条目化超时事件。`max_background` 默认值本次未改。
