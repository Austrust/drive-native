# #7 G4-013 独立验证证据（T4）

候选提交：`fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4`（分支 `codex/fix-7`，父提交 `9c28992`）。
验证者：T4 独立验证者（与作者独立；未修改产品源码、未改作者报告、未复用作者 target/构建产物、
不连接 NAS、不读取凭据、不 push/发布）。

**验收状态：产品主矩阵与补测通过，附 3 个验证者 harness 侧差异/更正（均有原件，见 §8）。**
主矩阵 58 条命令：56 通过、2 失败；两项失败均**不是产品缺陷**且已按更严格方式在补测重跑：
`native-loopback`（冻结 012 harness 期望早于本轮预期行为变化，校正后 70/0，且逐例证明只有 2 例预期变化）、
`core-history-probe-prepare`（本证据包初版缺 `core-history-probe.rs`，补测重跑 5/0）。
补测 22/22 命令通过（含真实 worker TLS 链 10/10、组件替换 8/8、T1 定向 16 项测试、210k 1/1、包前置回归）。
T1 增量审阅：**通过**（无线协议拒绝自动学习窄子集成立，fail-closed/归属/不学歧义码成立）。
T2 结论审阅：**成立**（静态证据内没有可安全接线的 L2 来源；证据完整、旧冻结未改），但 T2 报告目录
**未进入 fefb8ee 提交**，冻结前必须处理（见 §7）。

## 1. 独立性与不可变快照

| 项 | 值 |
|---|---|
| 候选提交 | `fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4`（`git cat-file -t` = commit） |
| 源码快照 | 私有 WSL 中由 `git archive fefb8ee`（共享对象库）生成并解包；archive SHA256 `816d9cc06c6486e54dc87a21a4db11eab7519537b1b30b3e27cc2270fde01765`，79,165,440 bytes |
| 提交树一致性 | 2,483 个 blob 的 path/可执行位/git 对象 ID 逐项复算一致（root tree `72ab91c1eb290f0f642aa416e9e1c84c7c8dba2e`，tree-listing SHA256 `8524b9164280d9c077e2d632222489970bda854a3ca06fea779cbe3586452c10`）；前后两次验证均 `passed=true`、`problem_count=0`（`environment/snapshot-verification-before.json`、`evidence/g4-013/snapshot-verification-after.json`） |
| 构建独立性 | 全新空私有 target（`g4-013/target`）；仅从**验证者自己**尝试 1 的 target 按 `Cargo.lock` 复制 registry 依赖种，未复制作者 target/产品产物；`CARGO_INCREMENTAL=0`（start 脚本导出）、`CARGO_BUILD_JOBS=2`、`--locked --offline` |
| 不可变性 | 主矩阵 `source-hashes-before.json` 逐文件对比结束态：`changed_source_files=[]`；补测结束再次全树对提交验证一致、archive SHA 未变（`archive_integrity.unchanged=true`）；唯一新增文件是 harness 自己生成的 `vendor/fuser/Cargo.lock`（主矩阵 `new_source_files` 明示，已作为允许项记录）；无 ELF 备份/产品 DB 导出 |

## 2. 宿主空间门禁（项目规则）

| 门禁点 | 观测 |
|---|---|
| 新建 run/target 前 ≥40 GiB | 准备时 Windows C: `81,838,788,608` bytes（76.2 GiB）通过；主矩阵启动时 C: `52,614,295,552` bytes（49.0 GiB）通过 |
| 每条命令前 ≥10 GiB | 主矩阵 58 条命令最小值 **27.81 GiB**；补测 22 条命令最小值 **20.84 GiB**；无 `not_run`（主/补均 0 条） |
| 空间不足即停 | 未触发；guest `df` 只记录，不作为宿主门禁 |

说明：验证期间另一并行任务（packaging 候选构建）也在使用宿主空间，因此 C: 可用量在验证过程中
从 ~76 GiB 降到 ~28 GiB；门禁始终满足，最终环境记录见 `environment/environment-013.json`。

## 3. 环境

WSL Ubuntu 24.04（内核/版本见 `environment/environment-013.json`），12 核；rustc/cargo 1.99.1
（`work/fsearch-perf-runs/env.sh`）；GUI venv `PySide6_Essentials 6.12.0` + `pyinstaller 6.22.3`；
`dbus-run-session`、`fusermount3` 可用；候选产品构建脚本以 `--all-features` 构建（`scripts/build-linux.sh`），
因此 T1 的 runtime 分类路径在交付二进制中启用（已核对，非默认关闭）。

## 4. G4 主矩阵（按 012 分组）

冻结源码、独立 target 上运行；命令、argv、退出码、逐命令日志哈希与逐名 inventory 在
`evidence/g4-013/inventory-001.json`，原始日志在 `evidence/g4-013/logs/`。重复组不累加为唯一测试数。

| 组 | 命令（本包名） | 通过/失败/忽略 | 结论 |
|---|---|---|---|
| workspace all-features | `workspace-all-features` | 985 / 0 / 42 | 通过（相对 012 的 976/0/42 **+9**，正好等于 T1 新增测试数） |
| workspace default | `workspace-default` | 975 / 0 / 37 | 通过（相对 012 的 966/0/37 **+9**） |
| 定向预检（权限/重试/时钟/版本/迁移/历史） | 见附录 A（除 workspace/FUSE/GUI/loopback 外的定向命令） | 全部通过 | 通过；计数与 012 一致（例如 core-permission 23→26 为 T1 新增 3 项，runtime-permission 4→7 为新增 3 项，runtime lib 152→157 为新增 5 项，sproto 3 不变） |
| FUSE 内核 debug/release INDEX1/INDEX0 | 6 条命令 | debug 38/36 + release 38/36，runtime 各 1 | 通过；INDEX0 中 2 个显式断言 NSI 启用的用例按既有设计 `--skip`，记为**不适用**，不进入通过计数 |
| attr TTL=0 | `kernel-fs-release-attr0` | 38 / 0 / 0 | 通过 |
| GUI 全套 | 18 个模块 + labels + 2 provider + render | 148 / 0 / 0 + labels 3 + nemo 10 + nautilus 10；render 通过 | 通过；计数与 012 一致 |
| native loopback（冻结 012 期望） | `native-loopback` | 64 / 2 / 0 | **失败（预期内）**：2 例为 T1 预期分类变化；原件保留，补测用校正期望重跑 70/0（§5.1） |
| 210k 容量（ignored） | 补测 `capacity-ignored-list`→`capacity-210k` | 1 / 0 / 0 | 通过（功能容量，不是性能基准） |
| doc/package 回归 | `offline-documentation-staging`（主）+ 补测 3 条 | 主 2/0/0；补测全部通过 | 通过（仅前置；新候选 deb 未构建，见 §9） |
| 完整性 | 快照 hash、source hash、ELF witness、mount 审计 | 0 变化 / 0 残留 | 通过 |

## 5. 补测（与主矩阵分开计数）

补测在**同一候选的同一私有 target** 上顺序执行（主矩阵完成后），22 条命令全部通过；逐命令日志与
退出码见 `evidence/g4-013/supplemental-001/`。

### 5.1 校正后的 native loopback（70 例）

- 冻结 012 harness 运行：66 例，64/2；失败 `upload_reply_9003`、`mkdir_reply_9003`——观测为
  `MutationPermissionDenied`，而冻结期望是 `mutation_permission_uncertain`。
- 校正 harness（仅改这两处 expected + 加 classify 分支 + 4 个对抗新增，见
  `harness/native-loopback-013-patch.json`）：70/0。
- 逐例比对：**恰好 2 例** observed/expected 变化（即上述两例），其余 64 例 observed 与 012 完全一致；
  新增 4 例（download 0x9003、download 0xa004、auth 阶段 0x9003、connect 阶段 0x9003）全部保持
  generic `server`，证明 typed 拒绝未泄漏到下载/握手分类。
- 记录：`environment/native-loopback-expectation-delta.json`。

### 5.2 真实 worker TLS→IPC→Bridge 链（10/10）

`tls-harness-build` + `tls-actual-worker-all-ten`：本机合成 TLS 服务 + **真实 `drive-native-worker`
ELF**（SHA256 `25da52bd11e61794820f7eb31c3e8356a163a98d60a8fac5ed692cf3f468b258`）→ IPC → Bridge。
10 例 = 6 例 Bridge 链（下载拒绝 3002/3008、profile 三态、迟到读）+ 4 例直接 Worker RPC
（3004/8004/9005/a005）；按既有口径**不相加称作完整 Bridge 链**。构建记录：
`evidence/g4-013/native-worker-chain-build-001/build.json`（SHA256 `086ff8e2…`）。

### 5.3 组件策略替换（8/8）

私有目录真实 fsd/worker 配对替换（旧侧为旧候选 `41365a6` 的保留构建 fixture，新侧由本快照在
本 target 构建）；8 个 case 全部通过。**不是** Debian 安装/升级/重装，不是跨版本兼容结论。
旧 probe 身份在补测前后未变（`68c6593…`）。结果：`evidence/g4-013/component-policy-handover-001/result.json`。

### 5.4 210k 容量 ignored（1/1）

`capacity-ignored-list` 精确列出 `default_budget_admits_210k_node_snapshot_without_holding_store`
后 `--ignored --exact` 执行 1 例通过；仅功能容量，不替代 G5 墙钟/每目录门槛。

### 5.5 T1 write-learning 定向测试（单独计数，16 项测试执行）

| 命令 | 通过/失败/忽略 |
|---|---|
| `t1-sproto-error-tests`（含 1 项新增分类 + 7 项既有错误分类） | 8 / 0 / 0 |
| `t1-worker-mutation-permission-tests` | 2 / 0 / 0 |
| `t1-bridge-learned-upload-denial` | 1 / 0 / 0 |
| `t1-bridge-ambiguous-upload-hint` | 1 / 0 / 0 |
| `t1-bridge-transport-loss-no-learn` | 1 / 0 / 0 |
| `t1-core-learned-upload-denial-revision` | 1 / 0 / 0 |
| `t1-core-definite-refusal-target-guard` | 1 / 0 / 0 |
| `t1-core-rename-remove-hints-no-rows` | 1 / 0 / 0 |

9 项新增测试全通过；计数另见 `environment/t1-write-learning-counts.json`。

### 5.6 包前置回归（不构建 deb）

`package-version`：`0.1.0~dev20261105.1`（打包任务将按最终提交更新版本，本包不作版本结论）；
`package-documentation-staging`：deb 用的 `scripts/stage-docs.py` 以 `DRIVE_NATIVE_SOURCE_COMMIT=CANDIDATE`、
`DRIVE_NATIVE_OFFLINE_DOCS=1` 预检通过；`package-metadata-equivalence`：打包输入
（`debian/ scripts/ Cargo.toml Cargo.lock vendor/`）相对产品冻结 `9c28992` **无差异**，旧冻结 deb 交付
（MANIFEST/SHA256SUMS/deb）与其 freeze 记录逐字节一致（未被本轮改动）。

### 5.7 标准编译检查

`standard-check-all-targets`：`cargo check --workspace --all-targets --all-features --locked --offline -j2`
通过（T1 作者报告中的标准检查项，本轮在独立 target 复跑）。

## 6. T1 独立审阅（摘要；全文见 review/T1-REVIEW.md）

- **非提交 + 唯一归属成立**：对照 server-static 冻结原件复核 ACL middleware 路径（InitRouteMap→
  TestPhase→ACLTest::Test→HandleError，Router/SyncTask 在 Commit 前返回并 DiscardBody）；复核
  `CreateACLTest::PermissionCheck` 的 `-3→0x9003` 与 `HandleError` 映射。
- **独立重扫**（capstone 线性扫描四个公开 ELF 全部可执行段，独立于作者工具/范围）：复现作者 syncd
  计数 `0x9003×1, 0x9004×7, 0xa003×1, 0xa004×4 = 13`；扩大到四个 ELF 后 **0x9003 写立即数产生点仍唯一**
  （`0x22329b`），另见的 `sub esi,0x9003` 是错误码分派消费者；`0x9004` 共 8 处立即数引用（syncd 7 + core `FilterChecker` 1），其中写立即数 6 处，
  支持“0x9004 同码异源不能学”。记录：`environment/server-immediates-013.json`。
- **代码边界**：typed `MutationPermissionDenied` 只在 `mutation_reply_error` 的
  `action=="upload" && code==0x9003` 产生；worker 只在 `kind=="upload"` 且
  `node.version != CREATE_VERSION` 时绑成 `PermissionDenied{node, write_content}`；创建/mkdir/replace/
  remove/rename/缺身份全部回 hint。bridge 只在 `!reconcile` 学，且 `refuse_operation` 校验
  `(node, action)` ∈ 请求 target 集合，不匹配即报错（fail-closed）。
- **对抗测试**：见 §5.1、§5.5；另：0x9004/0xa003/0xa004/0x3002/0x3004/0x9005/0xa005/宽整数码
  在 44 个 mutation wire 用例中无一被 typed；download/setup 阶段不泄漏 typed。
- **发现（不阻塞）**：① 冻结 012 loopback harness 期望需随预期行为变化更新（本包已记录 delta）；
  ② T1 REPORT §4.1 一处笔误（写“0x9004 从 hint 集合移除”，实际移出的是 0x9003，0x9004 仍在 hint 集合）；
  ③ 建议在 `mutation_permission_response` 增加 `code==0x9003` 断言作为纵深防御（当前唯一生产者已由
  sproto 层保证，不可利用）。
- **未实测**：真实 NAS `0x9003` 响应可达性、真实 worker upload IPC 端到端、FUSE 写路径 EACCES 端到端、
  其它 Drive Server 版本/未解包组件。

## 7. T2 独立审阅（摘要；全文见 review/T2-REVIEW.md）

- 证据包完整性独立复算通过（MANIFEST/SHA256SUMS 逐文件一致；四个 ELF 与报告 SHA 一致；输入与上一轮
  冻结包同 SHA）；旧冻结证据（server-static/static/l2-static/server-capability）git 工作树干净。
- 论证审阅：共享只读被物化为根 ACL 条目（继承/覆盖语义未证）、普通用户 share privilege 本地推导、
  profile/session read-only 过滤写侧不在包内、SDK 只读接口无调用者；**未发现被遗漏的可证明 L2 来源**。
  结论“不能安全接线、保持 unknown”成立；T3 真实 NAS 实验设计可执行。
- **流程问题**：T2 报告目录 `docs/validation/issue7-native-001/l2-source-002/` 与
  `work/issue7-native-evidence/l2-source-002/` **不在 fefb8ee 提交内**（前者工作树未跟踪，后者在仓库外）。
  冻结/打包前需提交该报告目录或明确记录其存放与 SHA，否则最终提交不自包含 T2 结论。

## 8. 失败/更正原件（不隐藏）

| 事件 | 原件 | 性质 |
|---|---|---|
| 主矩阵 `native-loopback` 64/2 | `evidence/g4-013/logs/native-loopback.log`、`evidence/g4-013/native-loopback-results.json` | 冻结 012 期望早于本轮预期行为变化；校正运行见 §5.1 |
| 主矩阵 `core-history-probe-prepare` 失败 | `evidence/g4-013/logs/core-history-probe-prepare.log` | 本证据包初版缺 `core-history-probe.rs`（验证者 harness 打包错误）；补测重跑 5/0 |
| 尝试 1 主矩阵中止（未产生产品结果） | `environment/attempt1-harness-path-failure/`（summary、failure、全部日志、launcher） | run-g4.py 从 `prepared-g4-013/` 启动导致 helper 脚本路径错误；**不记为产品失败**，修正后从头重跑 |
| 补测尝试 1（stale probe binary） | `environment/supplement-attempt1-stale-probe-binary/` | 校正 harness 与主矩阵 harness 同名同版本复用 cargo target 旧二进制；已通过重命名 probe 包修正 |
| 补测尝试 2（旧 driver） | `environment/supplement-attempt2-old-driver/` | TLS 构建沿用了 frozen build.py 的 40 GiB 新 target 门禁、docs staging 缺 env；用适配版 build wrapper + 显式 env 修正 |

所有原始 stdout/stderr、退出码、时间与 SHA 均保留；失败原件未被重写为通过。

## 9. 未实测 / 不适用（不进入通过计数）

- 真实 NAS：真实 ACL/共享高级权限、真实 `0x9003` 响应、服务端提交真值、profile 写侧、真实桌面面板。
- 新候选 deb 的 `dpkg-buildpackage`、解包 payload、安装/升级/重装（属打包任务 task-5；本包只做前置回归）。
- FUSE 写路径学到 deny → EACCES 的端到端；真实 worker upload IPC 链（0x9003 穿过真实 worker 到 Bridge）。
- G5 性能（既有 G5 原件与未达标结论不因本包改变）；其他 Drive Server 版本；SPK 未解包组件（web API/后台）。
- INDEX0 组中 2 个显式断言 NSI 启用的内核用例按既有设计跳过（不适用）；210k 只作功能容量；
  组件替换不是 deb 生命周期。

## 10. 与 012 的交叉核对

除 T1 预期变化外，主矩阵各组计数与 012 一致（例如：all-features 976→985、default 966→975 即 +9；
FUSE debug/release 38/36 不变；GUI 148、labels 3、providers 10/10、fuser 55、retry-clock 4、
core-v6 1、kernel-setattr 1、worker-elf-versions 3、ACL 截图 4 张 SHA 与 012 完全相同）。
`0x9003/0x9004` 相关行为差异仅限 §5.1 的两例。

## 11. 本包自核验方式

- 逐文件 SHA：`MANIFEST.json`（列出除自身外全部文件）与 `SHA256SUMS`（列出除自身和 MANIFEST 外全部文件）。
- 独立复核：`python3 harness/check-bundle-013.py`（重算全部 hash、检测缺失/多余/ELF/规则不符）；
  或 `sha256sum -c SHA256SUMS`。
- 覆盖与边界：[COVERAGE.md](COVERAGE.md)；T1/T2 全文：[review/T1-REVIEW.md](review/T1-REVIEW.md)、
  [review/T2-REVIEW.md](review/T2-REVIEW.md)；机器汇总：[STATUS.json](STATUS.json)。
- 检索入口：`evidence/g4-013/inventory-001.json`（主/补逐命令 argv、计数、日志 SHA）、
  `evidence/g4-013/logs/` 与 `evidence/g4-013/supplemental-001/logs/`（原始 stdout/stderr）。

## 12. 建议

1. **T1 可冻结**：建议作者在冻结前修正 REPORT §4.1 笔误（不影响产品行为），并把本包 `native-loopback`
   预期 delta 记入验收说明；可选加 `code==0x9003` 断言作为纵深防御。
2. **T2 无产品改动**：保持 `server_capability=unknown`，按报告等 T3 真实 NAS 实验；但 T2 报告/证据
   目前不在提交内，冻结前必须提交或明确记录（见 §7）。
3. **可进入打包（task-5）前置条件已满足**：包前置回归通过、旧冻结 deb 交付未被改动；但新候选 deb
   构建/解包/安装与真实 NAS 验收仍应作为后续独立步骤执行，不能用本包代替。
4. **真实 NAS 验收仍需执行**（T2 已给最小实验设计）；本包不新增真实 NAS 结论。

## 附录 A：主矩阵逐命令（58 条）

| 命令 | 通过 | 失败 | 忽略 | 退出码 | 状态 | 日志 |
|---|---:|---:|---:|---:|---|---|
| `offline-documentation-staging` | 2 | 0 | 0 | 0 | 通过 | [logs/offline-documentation-staging.log](evidence/g4-013/logs/offline-documentation-staging.log) |
| `clean-private-product-packages` | — | — | — | 0 | 通过 | [logs/clean-private-product-packages.log](evidence/g4-013/logs/clean-private-product-packages.log) |
| `core-definite_read_refusal_survives_restart_without_automatic_retry` | 1 | 0 | 0 | 0 | 通过 | [logs/core-definite_read_refusal_survives_restart_without_automatic_retry.log](evidence/g4-013/logs/core-definite_read_refusal_survives_restart_without_automatic_retry.log) |
| `core-revoked_traversal_suppresses_existing_transport_retry_wakes` | 1 | 0 | 0 | 0 | 通过 | [logs/core-revoked_traversal_suppresses_existing_transport_retry_wakes.log](evidence/g4-013/logs/core-revoked_traversal_suppresses_existing_transport_retry_wakes.log) |
| `runtime-bridge_respects_read_denial_before_cached_body_or_new_flight` | 1 | 0 | 0 | 0 | 通过 | [logs/runtime-bridge_respects_read_denial_before_cached_body_or_new_flight.log](evidence/g4-013/logs/runtime-bridge_respects_read_denial_before_cached_body_or_new_flight.log) |
| `runtime-cancelled_flight_clears_retry_even_when_read_or_traverse_is_denied` | 1 | 0 | 0 | 0 | 通过 | [logs/runtime-cancelled_flight_clears_retry_even_when_read_or_traverse_is_denied.log](evidence/g4-013/logs/runtime-cancelled_flight_clears_retry_even_when_read_or_traverse_is_denied.log) |
| `runtime-concurrent_read_revocation_and_last_reader_cancel_leave_no_retry_intent` | 1 | 0 | 0 | 0 | 通过 | [logs/runtime-concurrent_read_revocation_and_last_reader_cancel_leave_no_retry_intent.log](evidence/g4-013/logs/runtime-concurrent_read_revocation_and_last_reader_cancel_leave_no_retry_intent.log) |
| `retry-clock-preflight` | 4 | 0 | 0 | 0 | 通过 | [logs/retry-clock-preflight.log](evidence/g4-013/logs/retry-clock-preflight.log) |
| `obsolete-ticket-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/obsolete-ticket-preflight.log](evidence/g4-013/logs/obsolete-ticket-preflight.log) |
| `original-retry-pair-preflight` | 2 | 0 | 0 | 0 | 通过 | [logs/original-retry-pair-preflight.log](evidence/g4-013/logs/original-retry-pair-preflight.log) |
| `runtime-lib-original-parallelism` | 157 | 0 | 1 | 0 | 通过 | [logs/runtime-lib-original-parallelism.log](evidence/g4-013/logs/runtime-lib-original-parallelism.log) |
| `native-read-stale-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/native-read-stale-preflight.log](evidence/g4-013/logs/native-read-stale-preflight.log) |
| `runtime-permission-preflight` | 7 | 0 | 0 | 0 | 通过 | [logs/runtime-permission-preflight.log](evidence/g4-013/logs/runtime-permission-preflight.log) |
| `runtime-profile-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/runtime-profile-preflight.log](evidence/g4-013/logs/runtime-profile-preflight.log) |
| `runtime-version-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/runtime-version-preflight.log](evidence/g4-013/logs/runtime-version-preflight.log) |
| `core-permission-preflight` | 26 | 0 | 0 | 0 | 通过 | [logs/core-permission-preflight.log](evidence/g4-013/logs/core-permission-preflight.log) |
| `core-v6-migration-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/core-v6-migration-preflight.log](evidence/g4-013/logs/core-v6-migration-preflight.log) |
| `kernel-parent-actions-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/kernel-parent-actions-preflight.log](evidence/g4-013/logs/kernel-parent-actions-preflight.log) |
| `sproto-permission-preflight` | 3 | 0 | 0 | 0 | 通过 | [logs/sproto-permission-preflight.log](evidence/g4-013/logs/sproto-permission-preflight.log) |
| `sproto-profile-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/sproto-profile-preflight.log](evidence/g4-013/logs/sproto-profile-preflight.log) |
| `native-loopback` | 64 | 2 | 0 | 1 | 失败 | [logs/native-loopback.log](evidence/g4-013/logs/native-loopback.log) |
| `worker-trust-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/worker-trust-preflight.log](evidence/g4-013/logs/worker-trust-preflight.log) |
| `worker-elf-versions` | 3 | 0 | 0 | 0 | 通过 | [logs/worker-elf-versions.log](evidence/g4-013/logs/worker-elf-versions.log) |
| `core-history-probe-prepare` | — | — | — | 1 | 失败 | [logs/core-history-probe-prepare.log](evidence/g4-013/logs/core-history-probe-prepare.log) |
| `kernel-setattr-preflight` | 1 | 0 | 0 | 0 | 通过 | [logs/kernel-setattr-preflight.log](evidence/g4-013/logs/kernel-setattr-preflight.log) |
| `workspace-all-features` | 985 | 0 | 42 | 0 | 通过 | [logs/workspace-all-features.log](evidence/g4-013/logs/workspace-all-features.log) |
| `workspace-default` | 975 | 0 | 37 | 0 | 通过 | [logs/workspace-default.log](evidence/g4-013/logs/workspace-default.log) |
| `vendored-fuser-wire` | 55 | 0 | 0 | 0 | 通过 | [logs/vendored-fuser-wire.log](evidence/g4-013/logs/vendored-fuser-wire.log) |
| `kernel-fs-debug-index1` | 38 | 0 | 0 | 0 | 通过 | [logs/kernel-fs-debug-index1.log](evidence/g4-013/logs/kernel-fs-debug-index1.log) |
| `kernel-fs-debug-index0` | 36 | 0 | 0 | 0 | 通过 | [logs/kernel-fs-debug-index0.log](evidence/g4-013/logs/kernel-fs-debug-index0.log) |
| `kernel-runtime-debug` | 1 | 0 | 0 | 0 | 通过 | [logs/kernel-runtime-debug.log](evidence/g4-013/logs/kernel-runtime-debug.log) |
| `kernel-fs-release-index1` | 38 | 0 | 0 | 0 | 通过 | [logs/kernel-fs-release-index1.log](evidence/g4-013/logs/kernel-fs-release-index1.log) |
| `kernel-fs-release-index0` | 36 | 0 | 0 | 0 | 通过 | [logs/kernel-fs-release-index0.log](evidence/g4-013/logs/kernel-fs-release-index0.log) |
| `kernel-runtime-release` | 1 | 0 | 0 | 0 | 通过 | [logs/kernel-runtime-release.log](evidence/g4-013/logs/kernel-runtime-release.log) |
| `kernel-fs-release-attr0` | 38 | 0 | 0 | 0 | 通过 | [logs/kernel-fs-release-attr0.log](evidence/g4-013/logs/kernel-fs-release-attr0.log) |
| `runtime-private-dbus` | 1 | 0 | 0 | 0 | 通过 | [logs/runtime-private-dbus.log](evidence/g4-013/logs/runtime-private-dbus.log) |
| `gui-gui_unit_tests` | 5 | 0 | 0 | 0 | 通过 | [logs/gui-gui_unit_tests.log](evidence/g4-013/logs/gui-gui_unit_tests.log) |
| `gui-cache_gui_tests` | 5 | 0 | 0 | 0 | 通过 | [logs/gui-cache_gui_tests.log](evidence/g4-013/logs/gui-cache_gui_tests.log) |
| `gui-conflict_gui_tests` | 24 | 0 | 0 | 0 | 通过 | [logs/gui-conflict_gui_tests.log](evidence/g4-013/logs/gui-conflict_gui_tests.log) |
| `gui-hidden_dialog_tests` | 3 | 0 | 0 | 0 | 通过 | [logs/gui-hidden_dialog_tests.log](evidence/g4-013/logs/gui-hidden_dialog_tests.log) |
| `gui-lazy_gui_tests` | 4 | 0 | 0 | 0 | 通过 | [logs/gui-lazy_gui_tests.log](evidence/g4-013/logs/gui-lazy_gui_tests.log) |
| `gui-multi_mount_tests` | 5 | 0 | 0 | 0 | 通过 | [logs/gui-multi_mount_tests.log](evidence/g4-013/logs/gui-multi_mount_tests.log) |
| `gui-trust_gui_tests` | 6 | 0 | 0 | 0 | 通过 | [logs/gui-trust_gui_tests.log](evidence/g4-013/logs/gui-trust_gui_tests.log) |
| `gui-transfer_gui_tests` | 6 | 0 | 0 | 0 | 通过 | [logs/gui-transfer_gui_tests.log](evidence/g4-013/logs/gui-transfer_gui_tests.log) |
| `gui-theme_tests` | 1 | 0 | 0 | 0 | 通过 | [logs/gui-theme_tests.log](evidence/g4-013/logs/gui-theme_tests.log) |
| `gui-selector_gui_tests` | 7 | 0 | 0 | 0 | 通过 | [logs/gui-selector_gui_tests.log](evidence/g4-013/logs/gui-selector_gui_tests.log) |
| `gui-backend_upgrade_test` | 11 | 0 | 0 | 0 | 通过 | [logs/gui-backend_upgrade_test.log](evidence/g4-013/logs/gui-backend_upgrade_test.log) |
| `gui-pause_handover_test` | 5 | 0 | 0 | 0 | 通过 | [logs/gui-pause_handover_test.log](evidence/g4-013/logs/gui-pause_handover_test.log) |
| `gui-initialization_gui_tests` | 10 | 0 | 0 | 0 | 通过 | [logs/gui-initialization_gui_tests.log](evidence/g4-013/logs/gui-initialization_gui_tests.log) |
| `gui-unified_manager_tests` | 27 | 0 | 0 | 0 | 通过 | [logs/gui-unified_manager_tests.log](evidence/g4-013/logs/gui-unified_manager_tests.log) |
| `gui-gui_instance_test` | 3 | 0 | 0 | 0 | 通过 | [logs/gui-gui_instance_test.log](evidence/g4-013/logs/gui-gui_instance_test.log) |
| `gui-queue_gui_tests` | 4 | 0 | 0 | 0 | 通过 | [logs/gui-queue_gui_tests.log](evidence/g4-013/logs/gui-queue_gui_tests.log) |
| `gui-history_gui_tests` | 17 | 0 | 0 | 0 | 通过 | [logs/gui-history_gui_tests.log](evidence/g4-013/logs/gui-history_gui_tests.log) |
| `gui-permission_gui_tests` | 5 | 0 | 0 | 0 | 通过 | [logs/gui-permission_gui_tests.log](evidence/g4-013/logs/gui-permission_gui_tests.log) |
| `gui-permission-history-labels` | 3 | 0 | 0 | 0 | 通过 | [logs/gui-permission-history-labels.log](evidence/g4-013/logs/gui-permission-history-labels.log) |
| `provider-nemo` | 10 | 0 | 0 | 0 | 通过 | [logs/provider-nemo.log](evidence/g4-013/logs/provider-nemo.log) |
| `provider-nautilus` | 10 | 0 | 0 | 0 | 通过 | [logs/provider-nautilus.log](evidence/g4-013/logs/provider-nautilus.log) |
| `unified-ui-render` | — | — | — | 0 | 通过 | [logs/unified-ui-render.log](evidence/g4-013/logs/unified-ui-render.log) |

## 附录 B：补测逐命令（22 条）

| 命令 | 通过 | 失败 | 忽略 | 退出码 | 状态 | 日志 |
|---|---:|---:|---:|---:|---|---|
| `native-loopback-updated` | 70 | 0 | 0 | 0 | 通过 | [logs/native-loopback-updated.log](evidence/g4-013/supplemental-001/logs/native-loopback-updated.log) |
| `t1-sproto-error-tests` | 8 | 0 | 0 | 0 | 通过 | [logs/t1-sproto-error-tests.log](evidence/g4-013/supplemental-001/logs/t1-sproto-error-tests.log) |
| `t1-worker-mutation-permission-tests` | 2 | 0 | 0 | 0 | 通过 | [logs/t1-worker-mutation-permission-tests.log](evidence/g4-013/supplemental-001/logs/t1-worker-mutation-permission-tests.log) |
| `t1-bridge-learned-upload-denial` | 1 | 0 | 0 | 0 | 通过 | [logs/t1-bridge-learned-upload-denial.log](evidence/g4-013/supplemental-001/logs/t1-bridge-learned-upload-denial.log) |
| `t1-bridge-ambiguous-upload-hint` | 1 | 0 | 0 | 0 | 通过 | [logs/t1-bridge-ambiguous-upload-hint.log](evidence/g4-013/supplemental-001/logs/t1-bridge-ambiguous-upload-hint.log) |
| `t1-bridge-transport-loss-no-learn` | 1 | 0 | 0 | 0 | 通过 | [logs/t1-bridge-transport-loss-no-learn.log](evidence/g4-013/supplemental-001/logs/t1-bridge-transport-loss-no-learn.log) |
| `t1-core-learned-upload-denial-revision` | 1 | 0 | 0 | 0 | 通过 | [logs/t1-core-learned-upload-denial-revision.log](evidence/g4-013/supplemental-001/logs/t1-core-learned-upload-denial-revision.log) |
| `t1-core-definite-refusal-target-guard` | 1 | 0 | 0 | 0 | 通过 | [logs/t1-core-definite-refusal-target-guard.log](evidence/g4-013/supplemental-001/logs/t1-core-definite-refusal-target-guard.log) |
| `t1-core-rename-remove-hints-no-rows` | 1 | 0 | 0 | 0 | 通过 | [logs/t1-core-rename-remove-hints-no-rows.log](evidence/g4-013/supplemental-001/logs/t1-core-rename-remove-hints-no-rows.log) |
| `core-history-probe-prepare` | — | — | — | 0 | 通过 | [logs/core-history-probe-prepare.log](evidence/g4-013/supplemental-001/logs/core-history-probe-prepare.log) |
| `core-history-probe` | 5 | 0 | 0 | 0 | 通过 | [logs/core-history-probe.log](evidence/g4-013/supplemental-001/logs/core-history-probe.log) |
| `standard-check-all-targets` | — | — | — | 0 | 通过 | [logs/standard-check-all-targets.log](evidence/g4-013/supplemental-001/logs/standard-check-all-targets.log) |
| `capacity-ignored-list` | — | — | — | 0 | 通过 | [logs/capacity-ignored-list.log](evidence/g4-013/supplemental-001/logs/capacity-ignored-list.log) |
| `capacity-210k` | 1 | 0 | 0 | 0 | 通过 | [logs/capacity-210k.log](evidence/g4-013/supplemental-001/logs/capacity-210k.log) |
| `tls-harness-build` | — | — | — | 0 | 通过 | [logs/tls-harness-build.log](evidence/g4-013/supplemental-001/logs/tls-harness-build.log) |
| `tls-actual-worker-all-ten` | — | — | — | 0 | 通过 | [logs/tls-actual-worker-all-ten.log](evidence/g4-013/supplemental-001/logs/tls-actual-worker-all-ten.log) |
| `old-component-build` | — | — | — | 0 | 通过 | [logs/old-component-build.log](evidence/g4-013/supplemental-001/logs/old-component-build.log) |
| `new-component-build` | — | — | — | 0 | 通过 | [logs/new-component-build.log](evidence/g4-013/supplemental-001/logs/new-component-build.log) |
| `private-component-policy-handover` | — | — | — | 0 | 通过 | [logs/private-component-policy-handover.log](evidence/g4-013/supplemental-001/logs/private-component-policy-handover.log) |
| `package-version` | — | — | — | 0 | 通过 | [logs/package-version.log](evidence/g4-013/supplemental-001/logs/package-version.log) |
| `package-documentation-staging` | — | — | — | 0 | 通过 | [logs/package-documentation-staging.log](evidence/g4-013/supplemental-001/logs/package-documentation-staging.log) |
| `package-metadata-equivalence` | 1 | 0 | 0 | 0 | 通过 | [logs/package-metadata-equivalence.log](evidence/g4-013/supplemental-001/logs/package-metadata-equivalence.log) |
