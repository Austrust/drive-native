# T9（初始化只读有界重试）独立审阅

被审提交：`b303ac6bf01f6ff66f48d1e74cb94740561a841c`（父 `c1c1263`；产品树相对上轮 `fefb8ee`
新增 `native_retry.rs` + 8 处只读调用点接线 + 3 项 seed retry 测试，另有上轮 T4 建议的
`debug_assert_eq!(code, 0x9003)`（debug-only，无 release 行为变化））。审阅者未修改产品源码与作者报告；
所有产品测试在本轮独立快照 + 全新 target 中执行。

## 1. 变更范围与不变项（独立 diff 核对）

`git diff fefb8ee b303ac6 -- crates` 仅含：
`native_retry.rs`（新增 166 行）、`native.rs`（模块声明）、`native_initialization.rs`、
`native_root_guard.rs`、`native_snapshot.rs`、`native_snapshot_seed_tests.rs`、
`native_snapshot_watermark.rs`、`worker.rs`（debug_assert 3 行 + 注释）。

- schema、IPC、协议版本：**无改动**（diff 中无 schema/IPC/version 常量；`spoto/error.rs`、
  `connection_diagnostics.rs`、`bridge.rs`、mutation 路径均未改）。
- seed 预算常量未改：`SNAPSHOT_NODE_LIMIT=500_000`、seed 循环 `0..4096`、事件上限 `2_000_000`、
  时间预算 `900s`（`native_snapshot.rs`/`native_snapshot_reconcile.rs`/`native_snapshot_watermark.rs`
  均不在预算修改范围；diff 仅把调用包进 retry）。
- 产品构建脚本 `scripts/build-linux.sh` 用 `--all-features`，`native`/`native-research` 路径在交付二进制中
  启用（已核对）；本轮 release 构建复验见 REPORT。

## 2. retry 语义审阅（`native_retry.rs`）

| 审阅点 | 实现事实 | 结论 |
|---|---|---|
| 尝试上限 | `MAX_ATTEMPTS=5`（首次 + 最多 4 次重试） | 有界；测试与对抗 probe 均验证持续失败 5 次后停止 |
| 退避 | 200 → 400 → 800 → 1600 ms，`min(MAX_BACKOFF_MS=2000)`；共 4 次 sleep = 3.0s 上限 | 有界；无 unbounded sleep |
| 可重试错误 | 仅 `ConnectionClosed`、`Timeout(_)`、`Io` 七种 transport kind（UnexpectedEof/ConnectionReset/ConnectionAborted/BrokenPipe/NotConnected/TimedOut/WouldBlock） | 与“transport 类”声明一致；其他一律 1 次 |
| 不重试错误 | `Server`（含 12290/0x2001）、`MutationPermissionDenied/Uncertain`、`DownloadPermissionDenied`、`Decode`、`InvalidConfig`、`SessionInvalid`、`CursorReset`、`Tls`、`BadMagic`、`VersionMismatch`、`UnsupportedServer`、非 transport `Io` | 对抗 probe 逐类注入验证仅 1 次尝试 |
| 幂等请求/同游标 | `read(|| client.pull_events(watch))` 每次用**同一个 watch/cursor**重新发请求；`list_dir(view, path)` 用同一 path；调用方只在 Ok 后推进 cursor/path | 不会跳页；seed 测试断言失败页 cursor 序列 `[0,2000,2000,2000,2000,4000]` |
| 只读路径 | 全部 8 个调用点均为 `pull_events` / `list_dir` / `current_sync_watermark` / `current_selected_node`；mutation（upload/remove/rename/replace/mkdir）代码不引用 retry 模块 | 只读；mutation 语义未变 |
| 预算/进度 | retry 发生在单页请求内部；页/事件/时间预算检查与 seed 持久化逻辑未改；`*seed = Some(completed)` 只在到达边界后执行（无部分 checkpoint） | 不越过预算/边界；不提前发布进度 |
| 错误分类/诊断 | `read()` 在耗尽后返回**原始错误变体**（不透传包装）；`connection_diagnostics.rs`/`sproto::error.rs` 未改；root_guard 对 12291 的映射仍在 retry 之后 | 诊断语义不被掩盖；延迟增加最多 3s/页 |
| panic/取消安全 | `read()` 无 unwrap/expect；算术用 `saturating_*` 且 shift ≤4；future 在 backoff sleep 中被 drop 时不会再发起尝试 | 无 panic；取消安全 |

## 3. 接线审阅（只读 + 同位置）

| 文件 | 位置 | 行为 |
|---|---|---|
| `native_root_guard.rs` | `scoped_events` 的 `pull_events`；`current_root_matches` 的父目录 list | seed/fence/pull 页、根身份核对走 retry；`Server 12291 → RootUnavailable` 映射保持在外层 |
| `native_snapshot.rs` | 枚举任务 `list_dir`；reconcile 局部重查 `list_dir` | 目录枚举与收敛重查使用同一 path |
| `native_snapshot_watermark.rs` | `current_sync_watermark`、`current_selected_node` | 非根 scope watermark/根读取走 retry；外层 `timeout_at` 截止仍生效 |
| `native_initialization.rs` | `Directory` list_dir、`Root` current_selected_node | 持久初始化 owner 的目录/根读取走 retry |

`pull_initialization_page` 复用 `scoped_events` 自动获得 retry；`native_retry` 模块为 `mod retry;`
+ `pub(super)`，**无公开 API/schema/IPC 变化**。

## 4. 独立对抗测试（外部 crate 运行产品源码文件）

`harness/t9-retry-probe/` 把快照中的 `crates/drive-core/src/native_retry.rs`（仅去掉 4 行 `//!` 模块文档，
其余字节一致，记录原始/包含 SHA）通过 `include!` 放入外部 crate，用真实 `sproto::Error` 类型运行
9 项验证者对抗测试 + 4 项该文件自带的产品测试（共 13 项）：

1. `MAX_ATTEMPTS` 常量 = 5；
2. K∈{1,2,4} 次断连后成功：尝试数 = K+1、同操作恢复；
3. 持续失败到上限：恰好 5 次尝试、**不再执行 scripted 成功**、返回原始 `Io(UnexpectedEof)`；
4. 非可重试错误集合（14 种：Server 12290/0x2001、两种 mutation typed、DownloadPermissionDenied、
   Decode、SessionInvalid、InvalidConfig、CursorReset、BadMagic、VersionMismatch、UnsupportedServer、
   两种非 transport Io）均只尝试 1 次并原样返回错误；
5. transport 可重试集合（Timeout/ConnectionClosed + 7 种 Io kind）均重试一次后成功；
6. 立即成功不 sleep（<150ms）；
7. 耗尽退避有界（实测 ~3.0s，边界 2.9–8s）；
8. 在第一次 200ms backoff 中取消 future 不会发起下一次尝试（calls=1）；
9. 每次尝试都构造新的 request future（created == calls）。

以上全部通过（13/0/0），是对作者测试的独立补充而非重复。

## 5. 真实 NAS 测量日志的独立复算

本包只复算作者已提交的 JSONL（不是新的 NAS 观测）：
- `ROOT_MF`：76 次页请求、102,014 事件、214.96s、25 次断连全部恢复、server_errors=0、
  `stop_reason=boundary_with_retry`；attempt histogram 48/10/2/1/2，派生 retries=25 与字段一致。
- `HOME attempts=5`：311 页请求/148,281 事件/308.99s，24 断连，`retry_exhausted`；
  `attempts=8`：289/126,129/422.68s，7 断连，探针时限到；两者均未证明 900s 内完成 → 报告保留残余、未放宽预算。
- `/Drive` 枚举：5,001 次 list_dir、159.1s、0 断连、5,000 目录/20,476 节点（探针 cap）。
- 内部一致性：所有 histogram 派生 requests/retries 与字段一致，retries == disconnects。

**测量语义观察（非产品缺陷）**：JSONL 的 `pages` 字段等于“no-retry 阶段页数 + retry 阶段请求尝试数”
（ROOT_MF 13+63=76；HOME 13+298=311、28+261=289），即包含重试尝试；ROOT_MF 的去重逻辑页数应为
13+(63−25)=51 页。报告表格把 `pages` 直接写成“页数”，语义上应读作“页请求次数”；这不影响
“远小于 4096 页/2M 事件/900s 预算”的结论，但建议冻结前在文档中注明口径。

## 6. 发现与残余

未发现需要修改产品代码的缺陷。非阻塞观察：
1. **测量口径**见 §5（建议文档注记）。
2. 重试把单页失败的最大延迟从立即失败提高到 ~3s（4 次退避）；错误变体不变。G4 矩阵未出现计数/行为回归
   （见 REPORT §主矩阵），但依赖“立即失败”时延的断言如果存在会变慢，不是语义变化。
3. `Io(WouldBlock)` 被列为可重试：对非阻塞 socket 信号会产生有界重试，无害但可考虑仅对连接类 kind。
4. 若服务器在响应丢失前推进了游标，同 cursor 重试可能得到 `CursorReset`（非重试 → seed 失败，不跳页）；
   真实 NAS 25/25 恢复、无 CursorReset，但其它版本/网络未测。
5. `debug_assert_eq!(code, 0x9003)`（上轮 T4 建议）只在 debug 构建断言；release 无行为变化；
   该不变量由 sproto typed 生产唯一性保证，G4 debug 测试通过。

## 7. 未实测（沿用报告声明并独立确认）

- 修复后真实 NAS 挂载复测（T3）；“真实失败后再成功”的端到端日志。
- HOME 超大根 900s 内可行性、根 watermark 路径、完整 `/Drive` 枚举、其它服务器版本/网络、断连根因。
- FUSE/初始化 retry 的用户可见端到端；新 deb 构建/安装（打包任务）。

**审阅结论：通过。** 有界、只读、同位置、fail-closed 的重试语义成立；未发现请求放大越界、
错误分类被掩盖、mutation 误重试或预算被绕过。计数见 [REPORT.md](../REPORT.md)。
