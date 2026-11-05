# G4-014 独立验证报告（T11 / T9 韧性修复）

- 候选提交：`b303ac6bf01f6ff66f48d1e74cb94740561a841c`（`codex/fix-7`，父 `c1c1263`）
- 上一轮独立基线：`fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4`（G4-013 已接受）
- 验证者：T4 独立验证者（未修改产品源码、未复用作者构建产物、未连接 NAS、未读取凭据、未 push/发布）
- **验收结论（产品行为矩阵）：通过**。主矩阵 **59/59 命令 PASS**，补测 **28/28 命令 exit 0**；
  两个“命令绿但工件自报 false”的项目已显式列入 `STATUS.json` 的 `allowed_failed_checks`（见 §7），
  不作为产品失败，也不被隐藏。
- 本包唯一进入结论的运行：WSL `g4-014-rerun`（`drive-native-test` 用户，非 root）。被中断的 attempt 3
  与作废的 root attempt 4 只作环境原件保留，不进入任何计数。

> 答案来源声明：本报告所有计数来自本轮实际命令的退出码与测试输出解析（记录于 run `summary.json`、
> `logs/` 与 `inventory-001.json`），不把作者自测或 G4-013 旧计数当作本轮通过。唯一与作者数字的比对是
> T9 release ELF 的 SHA 一致性（记录为“可复现性检查”，不是测试通过）。完整入口与边界见
> [COVERAGE.md](COVERAGE.md)；机器可读状态见 [STATUS.json](STATUS.json)。

## 1. 不可变快照与身份

本轮从共享对象库 `git archive b303ac6` 建立独立源码树，并在解包前后按 `git ls-tree` 的 blob/路径/
执行位逐项校验：

| 项 | 值 |
|---|---|
| 候选提交 | `b303ac6bf01f6ff66f48d1e74cb94740561a841c` |
| 快照归档 `candidate.tar` sha256 | `c99a0215749f50468a4f43cd029e4049884b5dd2b220278abe30d6e2d5112218` |
| 校验条目 | 2803 项，前后两次校验 `passed=true` |
| 解包树 listing sha256 | `0ef3176a8a2d479efda0ac465f9dfa03cc524be546cf26034e894083a6841be0` |
| 运行前后源码变化 | `changed_source_files=[]`；`source_content_unchanged_after_mtime_reset=true` |
| 运行后新增文件 | 仅 `vendor/fuser/Cargo.lock`（构建产物，不改变源码 hash 判定） |

快照身份与产品输入记录见 [environment/snapshot-verification-before.json](environment/snapshot-verification-before.json)
与 [environment/environment-014.json](environment/environment-014.json)。

## 2. 隔离与 target 来源（无污染论证）

正式 run 使用**全新 run 根** `/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-014-rerun`，
以非 root 用户 `drive-native-test` 执行（FUSE/kernel 组要求非 root）。target 不是作者 target，而是：

1. attempt 2 的私有 target 先被保留为 `$BASE/g4-014-target-seed`；
2. attempt 3 从该 seed 复制 registry 依赖后生成自己的 target；attempt 3 被中断后，其 target 被移为
   `$BASE/g4-014-rerun-target-seed`；
3. 正式 run 以 `--registry-seed $BASE/g4-014-rerun-target-seed` 启动。

`run-g4-014.py` 的 seed 逻辑（源码核实）**只按快照 `Cargo.lock` 中 `source=registry+` 的包**，从 seed target
的 debug/release `deps/.fingerprint/build` 复制 `.rlib/.rmeta/.so/.d`；明确排除 workspace/path 包、测试 ELF、
incremental 与 hardlink。`--copy-target` 在 runner 内被断言禁止，且每次 run 必须是全新路径（assert 不存在）。
本轮 summary 记录的 `registry_seed`：`copied_files=2812`、`copied_bytes=1714266149`、
`selection=registry+ only`；`CARGO_INCREMENTAL=0`。因此 target 复用仅限**编译缓存种子**，不含作者构建产物
或作品二进制。注意：supplemental 的 `target_provenance` 字段写的是“fresh empty private target”，它与主 run
`registry_seed` 记录口径不同；**本报告以主 summary 的 `registry_seed` 记录为准**，两者不构成冲突。

宿主空间门禁（Windows C: `Get-PSDrive`，非 guest df）：正式 run 启动时 `free_bytes=334893826048`
（门限：新 run/target ≥ 42949672960，每条命令前 ≥ 10737418240）。主矩阵与补测期间未触门禁。

## 3. 正式主矩阵结果（59/59）

- run summary：[evidence/g4-014/summary.json](evidence/g4-014/summary.json)，sha256
  `b8c14b01ffc51fff74e15306c2e944e55872b1193708717c8aece03c4cc30131`
- `complete=true`、`all_commands_passed=true`、`commands=59`、`command_counts={passed:59, failed:0, not_run:0}`
- start `2026-11-05T11:25:41.944794+00:00`，end `2026-11-05T11:54:09.167519+00:00`（约 28.5 分钟）
- candidate=`b303ac6...`；harness sha256=`88fa9b573ac109d85b1c7af2fcf4b2c02e81a0be58527f8031c411b1a9c5271d`
  （与冻结 G4-012/G4-013 runner 同字节）；`residual_owned_mounts=[]`；ACL 截图 4 张齐全。

| 组 | 本轮结果（passed/failed/ignored） | 与 G4-013 对照 |
|---|---|---|
| workspace all-features | **992/0/42** | 985/0/42，**+7** = T9 retry 测试 |
| workspace default | **982/0/37** | 975/0/37，**+7** |
| FUSE debug INDEX1 / INDEX0 | 38/0/0（filtered 41） / 36/0/0（filtered 43） | 完全相同 |
| FUSE release INDEX1 / INDEX0 | 38/0/0（filtered 41） / 36/0/0（filtered 43） | 完全相同 |
| attr TTL=0 | 38/0/0（filtered 41） | 完全相同 |
| kernel runtime debug/release | 1/0/0 / 1/0/0 | 相同 |
| native loopback | **70/0/0/70** | G4-013 主矩阵为冻结 012 期望 64/2；013 校正补测为 70/0。本轮直接用 013 校正 harness，逐 case 与 013 基线一致（详见 §5） |
| GUI 18 模块 | 全部 PASS（5,5,24,3,4,5,6,6,1,7,11,5,10,27,3,4,17,5） | 相同 |
| permission-history labels / nemo / nautilus / unified render | 3 / 10 / 10 / PASS | 相同 |
| TLS worker 链（补测） | 10/0/0 | 10/0 |
| 210k 容量（补测） | 1/0/0（另有 ignored-list 前置） | 1/0 |
| 组件策略替换（补测） | 8/8 case PASS | 8/8 |
| T9 retry 定向（补测） | unit 4/0、seed 10/0、core-lib 415/0/1、runtime-lib 157/0/1 | 作者声明 411+4、157；见 §6 |
| T9 对抗 probe（补测） | 13/0/0（9 验证者 + 4 产品 retry 测试） | 新增 |
| T9 测量日志复算（补测） | 4/4 claims 复现；工件自报 `passed=false`（见 §7） | 新增 |
| T1 write-learning 定向（补测） | 8 条命令全 PASS（计数 8/2/1/1/1/1/1/1） | 沿用 G4-013 结论，本轮回归 |
| core-history probe（补测） | 5/0/0 | 相同 |
| doc/package 前置（补测） | `offline-documentation-staging` 2/0；package 三命令 exit 0，其中 metadata 自报 false（见 §7） | 013：delta 为空；本轮 changelog 有版本条目 |

逐命令名称、退出码、原始 stdout/stderr 在 [evidence/g4-014/logs/](evidence/g4-014/logs/) 与
[evidence/g4-014/inventory-001.json](evidence/g4-014/inventory-001.json)；补测命令在
[evidence/g4-014/supplemental-001/logs/](evidence/g4-014/supplemental-001/logs/) 与
[evidence/g4-014/supplemental-001/summary.json](evidence/g4-014/supplemental-001/summary.json)。

## 4. 与 G4-013 的差异（只看实际差异）

1. **workspace 计数 +7**：all-features 985→992、default 975→982，恰好等于 T9 新增的 7 个 retry/seed 测试；
   没有其它计数漂移。
2. **native loopback**：G4-013 主矩阵使用冻结 012 期望（64/2，2 例为 T1 已证明的预期差异），G4-013 补测
   用校正 harness 得到 70/0。本轮按 T4 的既定修正，主矩阵直接用校正 harness（70 例）以避免重复已知差异，
   并把本轮 70 例 observed 与 G4-013 校正基线**逐例比对**：`changed_observed_cases=[]`、
   `verdict=observed classification identical to G4-013 corrected baseline`。这是对“T9 未改变分类行为”的
   直接证据，而不是把 013 的通过数搬过来。基线见
   [evidence/g4-014/native-loopback-baseline-013.json](evidence/g4-014/native-loopback-baseline-013.json)，
   逐例结果见 [evidence/g4-014/native-loopback-results.json](evidence/g4-014/native-loopback-results.json)。
3. **FUSE/GUI/ACL 等**：四组 FUSE 与 attr0 计数与 013 相同；ACL 4 张截图 SHA 与 013 完全相同
   （user/space/directory/unknown 四值一致，`acl_screenshots_complete=true`）。
4. **包回归**：013 时 `9c28992..fefb8ee` 的 packaging delta 为空；本轮 `9c28992..b303ac6` 因 `d7d8b6a`
   修改了 `debian/changelog`，`package-metadata-equivalence` 自报 `passed=false`。这是**版本条目变化**，
   不是产品行为回归，见 §7。

## 5. 尝试历史（原件保留，不进入计数）

| attempt | 用户/入口 | 结果 | 原件位置 |
|---|---|---|---|
| 1 | root | 在**任何产品命令前**中止：旧 `g4-002` Nautilus 私有依赖已被外部清理，runner 启动断言失败 | `environment/attempt1-missing-nautilus-prefix/` |
| 2 | root | `native-loopback` 命令失败：runner 固定查找 `run-native-loopback.sh`，当时脚本名不符；target 被保留为 `g4-014-target-seed` | `environment/attempt2-native-loopback-runner-name/` |
| 3 | root | 运行到 28 条命令被用户中断（complete=false；26 pass；native-loopback FAIL 是脚本输出路径问题，探针实际 70/70；workspace-default 未完成）；其 target 成为正式 run 的 registry seed | `environment/attempt3-interrupted/`（[abort-record.json](environment/attempt3-interrupted/abort-record.json)） |
| 4 | root | `kernel-parent-actions-preflight` 确定性失败：`permissions_kernel.rs:28` 断言 `getuid != 0`；作废，target 不复用 | `environment/attempt4-root-invalid/`（[abort-record.json](environment/attempt4-root-invalid/abort-record.json)） |
| **5** | **drive-native-test** | **正式 run：主矩阵 59/59 PASS + 补测 28/28 exit 0；唯一进入结论** | `g4-014-rerun` |

attempt 3/4 的 `recorded_as_product_failure=false`；attempt 4 记录 `commands_recorded=25`、
failed=[kernel-parent-actions-preflight, core-history-probe]（后者是停止 run 时被中断，非产品失败）。
完整机器可读历史见 [environment/attempt-history-014.json](environment/attempt-history-014.json)。

## 6. T9 韧性修复：独立审阅与对抗

**实现审阅（静态）**：新增 `crates/drive-core/src/native_retry.rs`：

- `MAX_ATTEMPTS=5`；退避 `200/400/800/1600ms`（合计 3000ms，`MAX_BACKOFF_MS=2000` 为单次上界）；
- 可重试白名单仅：`Error::ConnectionClosed`、`Error::Timeout`、以及 `Error::Io` 的
  `UnexpectedEof/ConnectionReset/ConnectionAborted/BrokenPipe/NotConnected/TimedOut/WouldBlock`；
- 其余（Server/Decode/InvalidConfig/SessionInvalid/CursorReset/Tls/BadMagic/VersionMismatch/
  UnsupportedServer/DownloadPermissionDenied/Mutation*）**不重试**；
- `read/pull_events/list_dir` 包装 `read`，每次尝试重新构造 future，保持同 cursor/path，不推进页、不跳页；
- 全 crate grep 显示 retry 仅在 **8 处只读调用点**：`native_initialization.rs:82/:104`、
  `native_root_guard.rs:37/:73`、`native_snapshot.rs:132/:369`、`native_snapshot_watermark.rs:19/:22`；
  mutation/session 路径无引用。详细行号与判定见 [review/T9-WIRING-CHECK.md](review/T9-WIRING-CHECK.md)。

**独立对抗（13 项）**：把产品 `native_retry.rs`（仅去掉模块文档行，`product_file_sha256=156bf982...`）
`include!` 进外部 crate 运行：上限恰为 5、K 次断连后同操作成功、耗尽保留原错误、14 类非重试错误只尝试
一次、7 种可重试 `Io` 错误会重试、立即成功不 sleep、退避有界（约 3s 且非零）、100ms 取消后不再发起新
尝试、每次尝试新 future。**13/13 PASS**（9 验证者 + 4 产品测试）。

**测量日志复算（不是新 NAS 观测）**：4 条报告 claims 全部复现：
ROOT_MF pages=76/events=102014/elapsed=214.961s/disconnects=25/server_errors=0/completed=true；
HOME attempts5 pages=311/events=148281/elapsed=308.99s/disconnects=24/stop=retry_exhausted；
HOME attempts8 pages=289/events=126129/elapsed=422.684s/disconnects=7；
`/Drive` enumeration 5001 requests/159.145s/0 disconnects。
但工件自报 `passed=false`：HOME attempts5 的 retry phase 直方图推出 19 retries/298 requests，而字段记录
24 disconnects、pages=311；另三行自洽。**结论**：报告文字数字与原始字段一致；该行的断连/页/请求计数
不自洽是测量记账问题（作者应澄清或注记），不构成产品行为证据，也不据此声称 HOME 修复成功。
工件见 [evidence/g4-014/t9-measure-audit.json](evidence/g4-014/t9-measure-audit.json)。

**回归**：`t9-retry-unit-tests` 4/0、`t9-seed-retry-tests` 10/0、`drive-core` lib 415/0/1、`drive-runtime` lib
157/0/1；作者声明为 411/0/1 + retry 4/0，与本轮合并口径 415 一致，但这是本验证者独立运行，不是采信
作者结果。**release bins 可复现性**：本轮构建的 fsd/worker/setup 三 ELF SHA 与作者记录**完全一致**
（fsd `a3ed4ce5…`、worker `54e96e17…`、setup `dafc0249…`，见
[evidence/g4-014/release-bins-014.json](evidence/g4-014/release-bins-014.json)）；该比对只说明可复现，
不作为测试通过计数。

## 7. 失败 / 自报失败 / 未实测（不得混入通过）

### 7.1 allowed-failed（命令 exit 0，但工件自报 false；已写入 `STATUS.json`）

| 项 | 自报 | 事实与判定 |
|---|---|---|
| `package-metadata-equivalence` | `passed=false` | `9c28992..b303ac6` 的 packaging delta = `M debian/changelog`（新增 `0.1.0~dev20261105.2` 条目，提交 `d7d8b6a`）；frozen deb-001 的 MANIFEST/SHA256SUMS/包 SHA 三项仍全为 `true`，未构建新 deb。属发布元数据变更，**不是产品测试失败**；T5 打包必须确认版本条目并另建/核对 deb。 |
| `t9-measure-log-audit`（工件自报） | `passed=false` | 4/4 report claims 复现；仅 HOME attempts5 行 retry 记账不自洽（19 retries/298 requests vs 24 disconnects/pages=311）。属测量记账 caveat，**不是产品行为失败**；建议作者注记。 |

两条的完整理由、结果 SHA、follow-up 见 [STATUS.json](STATUS.json) 的 `allowed_failed_checks` 与
[evidence/g4-014/package-metadata-equivalence.json](evidence/g4-014/package-metadata-equivalence.json)、
[evidence/g4-014/t9-measure-audit.json](evidence/g4-014/t9-measure-audit.json)。

### 7.2 未实测 / 不在本包结论内

- 真实 NAS 上的 T9 修复后挂载复测（T3 负责）；真实断连端到端日志；HOME 超大根 900s 内可行性；
  `/Drive` 完整枚举；其它服务器版本/网络；断连根因。
- FUSE 写路径/初始化的 retry → 用户可见行为端到端；真实 worker upload IPC。
- 新候选 deb 的构建/解包/安装/升级/重装；G5 性能；SPK 未解包组件。
- 本包未验证 `work/issue7-deb-002-artifacts/` 的内容（仅观察到存在）。
- T1/T2 本轮无代码变化（除 `d7d8b6a` 的 debug-only 断言与文档）；T1 8 条定向回归全过，T2 结论沿用
  G4-013（无安全 L2 来源），本包不重复论证。

## 8. 证据包与核验

- 包目录：`work/issue7-validation/G4-evidence-bundle-014-final/`
- 关键路径：`REPORT.md`（本文件）、[COVERAGE.md](COVERAGE.md)、`MANIFEST.json`、`SHA256SUMS`、
  [STATUS.json](STATUS.json)、`harness/`（runner 与全部 helper 原件）、`environment/`、`evidence/g4-014/`、
  `review/`。
- MANIFEST 覆盖除自身外的全部文件；SHA256SUMS 覆盖除 MANIFEST.json 与自身外的全部文件；排除规则
  在 MANIFEST 的 `exclusions` 中有说明，核验脚本为 `harness/check-bundle-014.py`（自检结果记录在包外
  `work/issue7-validation/prepared-g4-014/bundle-self-check.json`，避免自引用环）。
- 复现实验的入口：`harness/prepare-g4-014.py`（快照）、`harness/start-g4-014.sh`、`harness/run-g4-014.py`、
  `harness/start-g4-014-supplements.sh`、`harness/run-g4-014-supplements.py`；快照来源与 SHA 见
  [evidence/g4-014/source.json](evidence/g4-014/source.json) 与 `prepared-g4-014/{candidate.tar,source.json}`。

## 9. 建议

1. **产品行为矩阵可进入 T10/真实 NAS 验收**：T9 修复在合成/本机夹具上未改变既有 013 行为（FUSE/GUI/
   loopback/ACL 完全一致），仅新增 +7 个 retry 测试；retry 语义与接线经独立对抗 13/13 通过。
2. **T5 打包前必须处理两项 allowed-failed**：确认 `debian/changelog` 的 `dev20261105.2` 版本条目是否
   本轮预期，并在打包管线中重建/核对 deb；`package-metadata-equivalence` 的“相对 9c28992 无 delta”
   断言与该版本条目天然冲突，需要打包任务给出最终口径（不是放宽产品断言）。
3. **T9 测量报告建议补注**：HOME attempts5 行的断连/页/请求记账需澄清；在真实 NAS 复测前，不应把
   T9 的 HOME 改善写成已验收结论（ROOT_MF 的 25 次断连恢复与 102014 events 仅是作者已提交测量，本包
   只复算，未重测）。
4. 建议真实 NAS 验收重点覆盖：初始化只读重试与用户可见行为（FUSE E2E）、断连根因与重试上限、以及
   HOME/`/Drive` 边界。
