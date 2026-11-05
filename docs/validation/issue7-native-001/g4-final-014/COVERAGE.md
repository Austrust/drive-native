# G4-014 覆盖映射（T11 独立验证，T9 韧性修复）

候选：`b303ac6bf01f6ff66f48d1e74cb94740561a841c`（`codex/fix-7`，父 `c1c1263`；产品行为 = 上轮
`fefb8ee` + 只读初始化有界每请求重试 + 一处 debug-only 断言）。本文件按 G4-013 的分组定义声明本轮
**实际入口与边界**；设计/历史映射见 [G4-013 COVERAGE](../G4-evidence-bundle-013-final/COVERAGE.md)，
不继承其通过计数。

所有命令在 WSL 本机合成夹具上运行，**不连接 NAS、不读取凭据、不 push/发布**；T9 的“真实 NAS 测量”
在本包中只做**已提交 JSONL 的独立复算**（不是新的 NAS 观测）。

| 组 | 实际入口（本包命令名） | 计数口径 | 边界 |
|---|---|---|---|
| workspace all-features | `workspace-all-features` | Rust harness `test result` 累加；逐名见 inventory | 全新私有 target 离线编译；仅从本验证者 attempt-3 的 target 经 `run-g4.py` Cargo.lock registry 过滤器复制 registry+ 依赖（`CARGO_INCREMENTAL=0`，不复制 workspace/path 包、测试 ELF 或 incremental）；正式 run 以非 root 用户 `drive-native-test` 执行 |
| workspace default | `workspace-default` | 同上 | 同上 |
| 定向预检（权限/重试/时钟/版本/迁移/历史） | `runtime-*`、`core-*`、`sproto-*`、`retry-clock-*`、`obsolete-ticket-*`、`original-retry-pair-*`、`native-read-stale-*`、`worker-trust-*` 等（附录 A） | 各命令独立计数 | 与本机 worker/Store 边界；不是真实 NAS 语义 |
| FUSE 内核 debug/release INDEX1/INDEX0 | `kernel-fs-{debug,release}-index{1,0}`、`kernel-runtime-{debug,release}` | 显式 `--ignored`；INDEX0 中 2 个显式断言 NSI 的用例按既有设计跳过=**不适用** | 需要真实 `/dev/fuse`；不含 G5 性能 |
| attr TTL=0 | `kernel-fs-release-attr0` | 显式 `--ignored` | 内核用例组的 TTL 边界 |
| GUI 全套 | `gui-*` 18 模块、`gui-permission-history-labels`、`provider-nemo`、`provider-nautilus`、`unified-ui-render` | Python `Ran N tests`/fail/skip | offscreen + 私有 D-Bus；`provider-nautilus` 使用**继承自 G4-013 输出的私有 Ubuntu 包解包依赖**（原 g4-002 前缀已被外部清理删除，不是产品产物） |
| native loopback | `native-loopback`（G4-013 校正 harness，70 例；主矩阵内） | 70 = 66 冻结 + 4 对抗 | 本机合成 pstream + 公开 `sproto::Client`；不执行 worker ELF |
| 真实 worker TLS 链 | `tls-harness-build` + `tls-actual-worker-all-ten`（补测） | 10 = 6 Bridge 链 + 4 直接 Worker RPC | 合成 TLS 服务 + 真实 worker ELF；不证明真实 NAS 错误码来源 |
| 210k 容量（ignored） | `capacity-ignored-list` → `capacity-210k`（补测） | 1 个显式 ignored 功能用例 | 功能容量，不是性能基准 |
| doc/package 回归 | `offline-documentation-staging`（主）、`package-documentation-staging`、`package-version`、`package-metadata-equivalence`（补测） | 见 inventory | **新候选 deb 未构建**；本包仅前置回归；观察到 `work/issue7-deb-002-artifacts/` 存在但**不验证**其内容。`package-metadata-equivalence` 命令本身 exit 0，但其**自报 `passed=false`**：`debian/changelog` 相对 `9c28992` 新增 `0.1.0~dev20261105.2` 条目（提交 `d7d8b6a`），frozen deb-001 清单/包 SHA 三项校验仍全为 true；该自报失败列入 `STATUS.json` 的 `allowed_failed_checks`，**不是产品测试失败**，但 T5 打包必须确认版本条目并另行构建/核对 |
| 组件策略替换 | `old-target-seed`、`old-component-build`、`new-component-build`、`private-component-policy-handover`（补测） | 8 个 case | 旧侧由本验证者从 `41365a6` 不可变快照重建（原 g4-011 保留 fixture 已被外部清理删除）；不是 dpkg 安装/升级 |
| **T9 retry 定向** | `t9-retry-unit-tests`（4）、`t9-seed-retry-tests`（10）、`t9-drive-core-lib`、`t9-drive-runtime-lib`（补测） | 与作者声明逐项对比 | 只证明隔离测试与 wrapper 语义；真实 NAS 行为未在本包重测 |
| **T9 对抗（独立）** | `t9-retry-probe-prepare` → `t9-retry-probe-tests`（补测） | 13 = 9 验证者对抗 + 4 产品 retry 测试（同一 crate 内 include 产品源码文件） | 实际 `native_retry.rs` 源码（去模块文档行）在外部 crate 中运行；不是产品二进制端到端 |
| **T9 测量日志复算** | `t9-measure-log-audit`（补测） | 4 组 claim 复核 + histogram 自洽 | 只复算已提交 JSONL，不是新 NAS 观测；JSONL `pages` 字段语义按“请求尝试数”复核（见 REPORT §T9 审阅） |
| **T9 release 产物** | `t9-release-bins-build`（补测） | 构建 exit code + 三个 ELF SHA 与作者记录比对 | 本地 release 构建，不是 deb；hash 差异若出现按环境/非确定记录，不作失败 |
| T1 write-learning 定向 | 8 条 `t1-*` 命令（补测） | 单独计数 | 只证明隔离夹具/纯函数（沿用 G4-013 结论） |
| core-history probe | `core-history-probe-prepare` + `core-history-probe`（补测） | 5 项 | 历史/失效边界 |
| 完整性/环境 | 快照前后 hash、source hashes、ELF witness、mount 审计、宿主门禁 | 见 `environment/` | 宿主 C 门禁 40/10 GiB；guest `df` 不替代 |

## 与 G4-013 的口径差异（必须显式记录）

1. 主矩阵直接使用 **G4-013 校正后的 native-loopback harness**（70 例）。G4-013 已独立证明冻结 012
   期望与 T1 预期行为的 2 例差异；本轮不重复冻结期望运行，而是把 014 的逐 case observed 与 G4-013
   校正基线**逐例比对**，要求 70 例完全一致（证明 T9 未改变分类行为）。
2. 尝试 1 主矩阵在**任何产品命令执行前**中止：环境清理删除了旧 `g4-002` Nautilus 私有依赖，
   run-g4 断言失败。该尝试原件保留在 `environment/attempt1-missing-nautilus-prefix/`，**不记为产品失败**；
   修正后从头重跑。
3. 组件替换的旧侧由本验证者从 `41365a6` 提交的不可变快照重建（旧 target 也被清理删除），不再复用
   作者旧构建；旧侧只是兼容性 fixture，不是新候选验收。
4. 包回归只做前置静态/文档检查；`work/issue7-deb-002-artifacts/` 的存在仅记录，不验证。
5. 尝试 4 以 root 启动，`kernel-parent-actions-preflight`（`permissions_kernel` 断言 `getuid != 0`）确定性失败；该 run 作废并原样保留在 `environment/attempt4-root-invalid/`，**不进入任何计数**。正式 run（attempt 5）以 `drive-native-test` 用户重跑，run 根 `g4-014-rerun`。
6. 尝试 3 在 workspace-default 被用户中断（`g4-014-aborted-partial`，complete=false，28 命令）；其 target 只作 registry 依赖种子。REPORT 中 aborted/invalid 与正式 rerun 分开记录。

## 正式 run 计数（唯一进入结论的运行，attempt 5）

- 主矩阵 run：`/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-014-rerun`
  （`summary.json` sha256 `b8c14b01ffc51fff74e15306c2e944e55872b1193708717c8aece03c4cc30131`）：
  `complete=true`、`all_commands_passed=true`、**59/59 命令 PASS**、`failed=0`、`not_run=0`。
- 补测：同一 run 的 `supplemental-001/summary.json`：`complete=true`、`all_passed=true`、**28/28 命令 exit 0**；
  其中 `package-metadata-equivalence` 自报 `passed=false`（原因见上表），是本包唯一 allowed-failed。
- 关键计数：workspace all-features **992/0/42**、default **982/0/37**（相对 G4-013 各 +7 = T9 新增 retry 测试）；
  FUSE debug/release INDEX1/INDEX0/attr0 **38/36/38/36/38**（与 013 相同）；native-loopback **70/0**（与 013 校正基线逐例一致）；
  GUI 18 模块 + labels(3) + nemo(10) + nautilus(10) + unified render 全 PASS；TLS 链 10/0；组件替换 8 case；
  210k 1/0；T9 probe 13/0；T9 measurement audit 4/0；release bins 三 ELF SHA 与作者记录一致。

## 未实测边界（不进入任何通过计数）

- T9 修复后的真实 NAS 挂载复测（T3 负责）；重试在真实断连上的端到端日志；HOME 超大根 900s 内可行性；
  `/Drive` 完整枚举；其他服务器版本/网络；断连根因。
- FUSE 写路径/初始化的 retry → 用户可见行为端到端；真实 worker upload IPC。
- 新候选 deb 构建/解包/安装/升级/重装；G5 性能；SPK 未解包组件。
