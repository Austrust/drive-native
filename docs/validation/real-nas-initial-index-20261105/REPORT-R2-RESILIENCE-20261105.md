# 根因收敛与最小修复：真实 NAS 根系初始化韧性（Round 2，2026-11-05）

状态：**团队根 scope 的 legacy seed 在真实 NAS 上被证明可通过有界每请求重试完成；已按方案 A 实现只读初始化的 per-request 重试+断点续拉并通过确定性测试。** 个人空间超大根（HOME）未证明能在 900s 预算内完成，作为残余保留；真实 NAS 挂载复测由 T3 在 release 产物上执行。未放宽产品 seed 预算（4096 页 / 2M 事件 / 900s）。

本轮在 `codex/fix-7` 上工作（起点 `c1c1263`），不 push、不改交付包与冻结证据；真实 NAS 仅只读（凭据由 Lead 放在临时文件，未复制/打印/入库，未做 NAS 写操作）。

## 1. 阶段 1：真实 NAS 只读测量（build 17892）

探针为私有源码副本中的只读测量程序（`Client::connect` + `pull_events`/`list_dir`，不发送 mutation）；在 6690 端口连接（Lead 提供的凭据文件里 `port=5001` 是 File Station Web 端口，不是 Drive 同步端口；探针用 `P032_PROBE_PORT=6690` 覆盖，凭据原文未打印）。原始 JSON 记录见同目录：

- `measure-root-scopes-20261105.jsonl`（ROOT_MF seed + HOME seed + 早期/Drive 枚举）
- `measure-home-attempts5-20261105.jsonl`、`measure-home-attempts8-20261105.jsonl`
- `measure-drive-enum-20261105.jsonl`

### 1.1 团队根 ROOT_MF（view 14 / node 1 / path "/"）

| 阶段 | 页数 | 事件 | 耗时 | 断连/重试 | 结果 |
|---|---:|---:|---:|---|---|
| 无重试 | 13 | 9,113 | 11.6s | 1 次 | 在 cursor 26000 被断（`Io UnexpectedEof`），未到边界 |
| 每请求重试（≤5 次，200ms→2s，同 cursor 续拉） | **76** | **102,014** | **215.0s** | **25 次断连全部恢复**，retries=25，server_errors=0 | **完成边界**（stationary empty batch） |

每请求尝试次数分布：1 次 48 页、2 次 10 页、3 次 2 页、4 次 1 页、5 次 2 页。总页数/事件/耗时与产品 `collect_snapshot_seed` 的预算（4096 页 / 2M 事件 / 900s）相容 → **团队根 legacy seed 可行**。

### 1.2 个人空间根 HOME（view 1 / node 1 / path "/"）

| 运行 | 进程内无重试阶段 | 重试阶段 | 结果 |
|---|---|---|---|
| attempts=5 | 13 页 / 9,113 事件 / 11.6s，cursor 26000 断 | 311 页 / 148,281 事件 / 309.0s，断连 24 | 某页 5 次耗尽后停止（`retry_exhausted`），**未到边界** |
| attempts=8 | 28 页 / 4,509 事件 / 7.6s，cursor 56000 断 | 289 页 / 126,129 事件 / 422.7s，断连 7，未出现重试耗尽 | 到达探针 420s 时限仍未到边界（deadline），**未证明 900s 内可行** |

两次运行的事件流/页数不一致（服务端分页与连接中断位置非确定），但均远大于团队根；**不为 HOME 放宽 seed 预算**，记为残余：超大个人树需要另做根 watermark/分页优化（根当前元数据契约未验证）。

### 1.3 子目录 `/Drive` 枚举

重试包装下 5,001 次 `list_dir` 请求、159.1s、**0 断连 / 0 重试**，推进 5,000 目录 / 20,476 节点仍未完成（探针目录上限）；该规模下未观察到 list 可靠性问题，但完整 `/Drive` 树远大于 5,000 目录，未测完。

## 2. 阶段 2：最小修复（方案 A）

### 2.1 实现

新增 `crates/drive-core/src/native_retry.rs`：

- 每个逻辑读请求最多 **5 次尝试**（首次 + 4 次重试），指数退避 **200ms → 400 → 800 → 1600ms**（上限 2s）。
- 只重试 transport 类错误：`Timeout`、`ConnectionClosed`、`Io` 的 `UnexpectedEof/ConnectionReset/ConnectionAborted/BrokenPipe/NotConnected/TimedOut/WouldBlock`。
- **不重试**：任何 `Server`（含 12290/0x2001 等服务端拒绝）、`MutationPermissionDenied`/`MutationPermissionUncertain`、`Decode`、`InvalidConfig`、`SessionInvalid`、`CursorReset`、`Tls`。
- `read()` 每次重试都重新发起同一个只读请求；调用方不推进 cursor/路径，故 seed 失败页从**同一 cursor** 续拉、目录 list 从**同一 path** 重试。

接线（只读初始化路径）：

| 文件 | 位置 | 行为 |
|---|---|---|
| `native_root_guard.rs` | `scoped_events` 的 `pull_events`；`current_root_matches` 的父目录 list | seed/fence/pull 页与根身份核对走重试 |
| `native_snapshot.rs` | 枚举任务中的 `list_dir`；reconcile 局部重查的 `list_dir` | 目录枚举与扫描期收敛走重试 |
| `native_snapshot_watermark.rs` | 当前水位 `current_sync_watermark`、当前根 `current_selected_node` | 非根 scope 的 watermark 读取走重试 |
| `native_initialization.rs` | `Directory` 的 list_dir、`Root` 的 current_selected_node | 持久初始化 owner 的目录/根读取走重试 |
| `native.rs` | `#[path="native_retry.rs"] mod retry;` | 模块声明 |

`pull_initialization_page` 复用 `scoped_events`，自动获得重试；mutation 路径（upload/remove/rename/replace/mkdir）不经过该模块，保持原语义。schema 25、IPC 21、错误分类与 seed 预算均未改。

### 2.2 确定性测试

| 测试 | 覆盖 |
|---|---|
| `native::retry::tests::transport_disconnects_are_retried_with_same_read` | 2 次断连后成功，同一读操作 |
| `native::retry::tests::persistent_transport_failure_stops_at_the_attempt_cap` | 持续失败 5 次后停止，不无限重试 |
| `native::retry::tests::server_protocol_and_mutation_errors_are_never_retried` | Server 12290/0x2001、两种 mutation typed 错误、Decode/InvalidConfig/SessionInvalid/CursorReset 均只尝试 1 次 |
| `native::retry::tests::timeout_and_connection_closed_are_retryable` | Timeout/ConnectionClosed 重试 |
| `native::native_snapshot::seed_tests::seed_retries_transport_failures_from_the_same_cursor_then_completes` | seed 页失败 3 次后成功；请求 cursor 序列 `[0,2000,2000,2000,2000,4000]`，总尝试 6 |
| `native::native_snapshot::seed_tests::seed_stops_after_bounded_retries_without_skipping_a_page` | seed 持续失败按上限停止，未越过失败页 cursor |
| `native::native_snapshot::seed_tests::seed_does_not_retry_a_server_refusal` | seed 页 server 拒绝只请求 1 次，整树失败不掩盖 |

计数（WSL，独立 target `/home/drive-native-test/.cache/p032-writelearn-target`，`CARGO_INCREMENTAL=0 -j2 --locked --offline`）：

- `cargo check --workspace --all-targets --all-features`：**exit 0**（4m20s；日志 `/home/drive-native-test/.local/state/p032-r2/verify-183447.log`）
- `cargo test -p drive-core --features native --lib retry::tests`：**4 passed / 0 failed / 0 ignored**
- `cargo test -p drive-core --features native --lib seed_tests`：**10 passed / 0 failed / 0 ignored**（原 7 项 + 新增 3 项）
- `cargo test -p drive-core --features native --lib`（全 lib）：**411 passed / 0 failed / 1 ignored**（52.00s）
- `cargo test -p drive-runtime --lib`：**157 passed / 0 failed / 1 ignored**（16.30s）

### 2.3 release 产物（给 T3 真实 NAS 快速验证，非正式 deb）

- 命令：`cargo build -p drive-runtime --all-features --release --bins --locked --offline -j2`
- 独立 target：`/home/drive-native-test/.cache/p032-r2-release-target`
- 构建时间：457（7m36s）s；exit 0
- 产物目录（WSL 本地，未入仓）：`/home/drive-native-test/.local/state/p032-r2/bin-release/`
  - `drive-native-fsd` SHA256 `a3ed4ce5b5a83754c14b3257130f6da2a528408e9151edf329f3e647073d3d75`（11,483,336 B）
  - `drive-native-worker` SHA256 `54e96e175ee4af6d84fdf47528c812f47cdefed1c5d9aaa2fb8f7c2329587f59`（6,965,288 B）
  - `drive-native-setup` SHA256 `dafc0249aefc254594a82e5e799c20cde7e5f1373fdd7d9c68c4df52200abe09`（5,075,904 B）

## 3. 结论与边界

- **team 根**：真实 NAS 上 legacy seed 在每请求重试下 76 页/102,014 事件/215s 完成边界，25 次断连全部恢复 → 方案 A 可行性已证实（合成重试测试亦通过）。
- **HOME 根**：超大个人树在 420s 探针时限内未到边界，900s 内未证明；不放宽预算，不阻塞 team 根修复 → 残余：根 watermark/分页优化。
- **未实测**：真实 NAS 上用本 release 产物的挂载复测（T3 下一步）；重试对真实失败后再成功的端到端日志；其他服务器版本/网络；HOME 根预算与水位路径；完整 `/Drive` 枚举；mutation 路径行为（未改，但本轮未在真实 NAS 重跑写矩阵）。
- 没有把“最多 5 次重试”当作对服务端断连根因的解释：断连原因仍未定（无服务端错误码），本修复只是让只读初始化对已观测的传输中断有界恢复。
