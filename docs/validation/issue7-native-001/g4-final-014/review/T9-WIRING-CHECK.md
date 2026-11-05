# T9 native_retry 接线与语义独立核查

- 验证者：T4 verifier（独立于作者）
- 候选提交：`b303ac6bf01f6ff66f48d1e74cb94740561a841c`
- 产品文件：`crates/drive-core/src/native_retry.rs`（166 行，产品树内 SHA-256 `156bf982dde47fff85b7c08fe27a2305570d3d5cc691379d1094e38dc1e701eb`）
- 方法：对不可变快照（`prepared-g4-014/candidate.tar`，archive sha256 `c99a0215749f50468a4f43cd029e4049884b5dd2b220278abe30d6e2d5112218`）做静态接线核查；用外部探针 crate 原样 include 产品 `native_retry.rs`（仅去掉 4 行 `//!` 文档行）执行对抗测试；不修改产品源码。
- 结论：**静态语义与接线通过**；未发现 mutation / session / 分页推进路径经过或受重试影响。残留边界见末节。

## 1. 机制事实（源码级）

- 常量：`MAX_ATTEMPTS = 5`；`BASE_BACKOFF_MS = 200`；`MAX_BACKOFF_MS = 2000`。
- 退避序列：第 1→2 次重试前 200 ms、2→3 400 ms、3→4 800 ms、4→5 1600 ms，单次调用累计最多 3000 ms；上限常量 2000 ms 仅作单次封顶。
- `retryable` 精确白名单：
  - `Error::ConnectionClosed`
  - `Error::Timeout`
  - `Error::Io(kind)` 仅 `UnexpectedEof`、`ConnectionReset`、`ConnectionAborted`、`BrokenPipe`、`NotConnected`、`TimedOut`、`WouldBlock`
  - 其余全部 `false`：`Server`、`Decode`、`InvalidConfig`、`SessionInvalid`、`CursorReset`、`Tls`、`BadMagic`、`VersionMismatch`、`UnsupportedServer`、`DownloadPermissionDenied`、`Mutation*` 等。
- `read()` 循环：成功立即返回；失败在 `attempt < 5 && retryable` 时 `sleep(backoff(attempt))` 后重试；否则原错误（含原 attempt 的错误值）返回，不包装、不转换。
- `pull_events` / `list_dir` 是 `read` 的薄包装，`FnMut` 每次调用重新构造 future；同一 cursor / path / directory id 在重试间保持不变，不使用上次失败的部分结果。
- 只读 API 均为 `pub(super)`，crate 外不可直接调用。

## 2. 接线面（精确 8 处非测试调用）

| 文件 | 行 | 操作 | 读/写性质 |
|---|---|---|---|
| `crates/drive-core/src/native_initialization.rs` | 82 | `read` | 只读（初始化握手响应） |
| `crates/drive-core/src/native_initialization.rs` | 104 | `list_dir` | 只读（初始化枚举） |
| `crates/drive-core/src/native_root_guard.rs` | 37 | `pull_events` | 只读（事件拉取） |
| `crates/drive-core/src/native_root_guard.rs` | 73 | `list_dir` | 只读 |
| `crates/drive-core/src/native_snapshot.rs` | 132 | `list_dir` | 只读（快照枚举） |
| `crates/drive-core/src/native_snapshot.rs` | 369 | `list_dir` | 只读（快照续拉） |
| `crates/drive-core/src/native_snapshot_watermark.rs` | 19 | `read` | 只读（watermark） |
| `crates/drive-core/src/native_snapshot_watermark.rs` | 22 | `read` | 只读（selected node） |

反查（`grep -R "retry::"` / `native_retry` 引用）除上述 4 个文件、`native.rs` 的 `mod retry;` 及测试外无其他引用；mutation（upload/mkdir/rename/delete 等）与服务会话路径不经过 `native_retry`。因此重试不会改变写请求语义，也不会把一次失败写操作变成两次执行。

## 3. 独立对抗探针（`harness/t9-retry-probe`）

探针把产品 `native_retry.rs` 原样 include 进一个父模块（提供与产品相同的 `super::*` 名字：`sproto`、`Client`、`EventBatch`），从而对**真实产品源码 + 真实 `sproto::Error` 类型**做行为测试，而不是对复制品或作者测试做测试。

9 个验证者测试 + 4 个产品自带测试 = 13 项：

1. `max_attempts_constant_is_five` — 上限为 5。
2. `transport_disconnects_then_success_retries_same_operation` — K 次断连后成功，恢复且操作被重新执行。
3. `persistent_transport_failure_stops_at_the_attempt_cap` — 持续失败恰好 5 次尝试后停止（不无限重试）。
4. `non_retryable_errors_are_attempted_once` — 14 类非重试错误只尝试一次。
5. `retryable_transport_errors_are_retried` — 7 种 `Io` kind + `ConnectionClosed`/`Timeout` 可重试。
6. `immediate_success_has_no_backoff` — 首次成功不 sleep。
7. `exhausted_backoff_is_bounded_and_nonzero` — 退避总量约 3.0 s，非零且有界。
8. `cancelled_future_during_backoff_does_not_start_another_attempt` — 100 ms 取消后不再发起新尝试。
9. `operation_constructs_a_fresh_future_for_each_attempt` — 每次尝试重新构造 future（不存在复用已 poll 的 future 的问题）。
10–13. 产品自带 4 项（作者测试，独立重跑，不作为独立证据唯一来源）。

结果：**13/13 通过**（本轮 supplement 重跑确认；见 `supplement-014.json` / `logs/t9-retry-probe.log`）。

## 4. 与既有断言的相容性

- T1 的 typed 权限分类（`MutationPermissionDenied` 等）不受影响：这些错误不在 `retryable` 白名单，仍然一次失败即返回，`0x9003` 唯一归属与精确写拒绝自动学习逻辑均不经过重试层。
- `CursorReset`、`SessionInvalid` 等不重试：避免在协议状态失效时继续用同一 cursor 放大请求。
- 事件分页：`pull_events`/`list_dir` 的 cursor/path 只在成功返回值后由调用方推进；重试使用同一输入，不会跳页。
- 既有 `runtime-*` / `core-*` 预检（retry-clock、original-retry-pair、permission、v6-migration 等）在 G4-014 主矩阵中全部通过，说明启动/重试时钟与既有语义未出现回归。

## 5. 观察与残留边界

1. 作者测量报告的 `pages` 是**请求尝试数**（含重试），不是不同页数；本轮 `analyze-t9-measure.py` 已从 JSONL 独立复算并写 `t9-measure-audit.json`，报告中应明确该语义，避免把 76 次请求误解为 76 个不同页。
2. 退避为固定指数封顶，没有抖动（jitter）；多实例同时断连时存在同步重试尖峰的理论风险，但本修复范围与真实 NAS 观测（单实例）未证明该风险，不构成拒绝理由。
3. 探针验证的是单次调用语义；真实 socket/TLS 断开时序、服务端限流下的长期行为仍以主矩阵 TLS 链 10/10 与 T3 的真实 NAS 观测作为补充，而非本探针覆盖。
4. 未实测：真实 NAS 上初始化只读调用在重试耗尽时的 UI/诊断文案端到端（属 T3/真实验收范围）。

## 6. 判定

- 接线只覆盖 8 处只读调用；机制有界（最多 5 次尝试、最多 ~3 s 退避）、可取消、错误原样返回；对抗测试 13/13 通过。
- **建议接受 T9 的 retry 语义与接线**，前提是 G4-014 主矩阵与其余补测按验收口径全过（见 REPORT.md 汇总）。
