# Issue #5：FSearch 兼容与按需挂载元数据性能修复报告（裸机基线）

- 日期：2026-11-08
- 分支：`codex/fix-5`（基线 `367e33d`，即 codex/fix-7 系列最新稳定集成点）
- 提交：`94f738d perf(core): cut namespace index build cost on large trees (#5)`、`144d73d fix(core): keep body-size precedence in the lean namespace projection (#5)`
- 远端验证机：`workstation202611`（Ubuntu Server 24.04.5，kernel 6.8.0-147-generic，i9-14900K 24C/32T，125 GiB RAM，rustc/cargo 1.99.1）；本机只存放源码，编译/测试/基准全部在远端执行
- 状态：代码与基准完成；Rust 全量验证与 GUI 验证见 §6；**未 push、未开 PR、未评论 issue**

## 1. 摘要

前一版 #5 实现（NamespaceIndex，下称 NSI）在 WSL2 上测得两棵大树反而慢于 .9 基线。本轮在**裸机 Linux**上复现并定位到唯一根因：NSI 构建在大树上超过 1 s 构建预算 → 整代索引被丢弃（`degraded=true`、`index_bytes=0`）→ 全部请求回退 SQL，且回退路径比 .9 多一次逐目录 ACL 查询，因此比 .9 更慢。

修复不放松任何语义守卫（entry TTL 仍为 0、attr TTL 仍为 1000 ms、整代 revision+stamp 守卫、不可信/超限仍整代回退 SQL、索引只读不分配 inode、预算仍为 1 s），而是把索引构建本身降到预算内：

- 100k 树构建 73–76 ms（修复前同样 ≤1 s，本来已生效）
- dense 树（319,679 条目）构建 280.8–281.5 ms（修复前命中 1 s 上限被整代丢弃；完整构建需 1.51 s）
- real 树（369,298 条目）构建 499.8–512.9 ms（修复前 1.10 s 被丢弃；完整构建 1.01 s）

三套树与原版 FSearch 0.3.1 端到端配对（3 轮中位，首扫/重扫）：

| 树（条目/目录） | 本轮首/重 | .9 首/重 | local 首/重 | 目标首/重 | 判定 |
|---|---|---|---|---|---|
| 102,130 / 2,066 | **0.665 / 0.614 s** | 1.616 / 1.565 s | 0.264 / 0.264 s | ≤2.0 / ≤1.6 s | 达成 |
| 319,679 / 56,073 | **2.868 / 2.518 s** | 8.777 / 8.374 s | 0.815 / 0.765 s | ≤13 / ≤12 s | 达成 |
| 369,298 / 66,776 | **5.472 / 5.121 s** | 17.484 / 17.096 s | 1.065 / 1.066 s | ≤15 s | 达成 |

相对 .9 提速 2.43×/2.55×、3.06×/3.33×、3.20×/3.34×；相对本地 ext4 为 2.52×/2.33×、3.52×/3.29×、5.14×/4.80×。全部扫描正确性核对 missing/extra/duplicates/invalid/attribute_mismatches = 0，hydration = 0，无残留挂载。

## 2. 方法与工具链

- 夹具：仓库自带 `crates/drive-fs/examples/fsearch_fixture.rs`。合成树形状 `entries = groups×(files+1) + wide + 130`、`dirs = groups + 66`：100k = `2000×40 + 20000 + 130`；dense = `56007×4 + 39514 + 130`。真实布局用本地真实目录树导出的 manifest（4 字段：相对路径、d|f、size、mtime 秒；`. = root`），上传 `manifest-personal-sync.tsv`（369,298 条目/66,776 目录，53,263,047 B）。
- 被测客户端：fixture 以 FUSE 挂载（`fuse-manifest`/`fuse` 模式），元数据由 SQLite + NSI 提供，下载路径使用 `NoNetwork` hydrator（记录 hydration 计数并返回 ENETDOWN，证明扫描不触发正文下载）。
- 扫描端：原版 FSearch 0.3.1（`--update-database`，每次独立 XDG_CONFIG/DATA/CACHE/RUNTIME、`GSETTINGS_BACKEND=memory`、`NO_AT_BRIDGE=1`、`dbus-run-session -- xvfb-run -a`）。另用独立 C 代理 `scan.c`（opendir/readdir/fstatat）给出纯元数据遍历的对照口径。
- 正确性：`scripts/validation/fsearch-index-audit.c` 链接上游 `libfsearch.a`，导出 FSearch 实际索引 TSV，与夹具 `truth.json` 逐项比对 path/type/size/mtime，并核对 indexed 条目数、重复、非法行。
- 配对：每轮按 final → .9 → local 顺序交替；每轮新建夹具实例与 FSearch 数据库目录；首扫从 FSearch 启动开始计时（不含夹具构造/索引构建，构建时间在夹具 ready 状态中单独报告），重扫在同一实例上第二次执行；每树 3 轮取中位。
- 版本与哈希（构建记录见 `docs/validation/FSEARCH.md` 与 `~/dn-fix5/logs/fsearch-build.log`）：
  - 上游源码：fsearch 0.3.1，sha256 `b16ab75556d841bf858633710d71c92f35d34362614b8584b0a5b71690a72c39`
  - `fsearch` 二进制 sha256 `bdc92e5b67e1f1e09d9a3428d0fd91cd86aeab6ba339bfd13b356d8ad8558145`
  - `fsearch-index-audit` sha256 `468cd2202eb1fafaa811cfbed201e7151a1510470f92916ba73b2f550db2a0de`
  - 被测 fixture `fixture-final` sha256 `145a553175aec5c86a2d939feabd137c4390a316895c80340ad49a1028e2f12e`（从 submit `144d73d` 构建）；诊断版 `fixture-final-diag` sha256 `d94e82dbddb2188a9eca988f5542d56aaf58462b505339eaff707bd3edc2ef2e`
  - 对照 `.9` 树：`d555944bfe3058cb223bf7d8ebc71244f1e4d7a3`（tag dev20261103.9 对应源码）
- 基准编排：`work/fix-5/remote/bench.py`（本地保存，远端 `~/dn-fix5/bench.py`），产物 `summary.json` 与本地 `work/fix-5/run/final-3r-*`；辅助 `stats.py`/`stats2.py`/`stats3.py`。

## 3. 结果

### 3.1 端到端 FSearch 3 轮中位（秒）

| 树 | 扫描 | 本轮 | .9 | local | 目标 | 本轮/.9 | 本轮/local |
|---|---|---|---|---|---|---|---|
| 100k | 首 | 0.665 | 1.616 | 0.264 | ≤2.0 | 2.43× | 2.52× |
| 100k | 重 | 0.614 | 1.565 | 0.264 | ≤1.6 | 2.55× | 2.33× |
| dense | 首 | 2.868 | 8.777 | 0.815 | ≤13 | 3.06× | 3.52× |
| dense | 重 | 2.518 | 8.374 | 0.765 | ≤12 | 3.33× | 3.29× |
| real | 首 | 5.472 | 17.484 | 1.065 | ≤15 | 3.20× | 5.14× |
| real | 重 | 5.121 | 17.096 | 1.066 | 无独立目标 | 3.34× | 4.80× |

每目录成本（首/重，ms/目录）：100k 0.322/0.297；dense 0.051/0.045；real 0.082/0.077。
每条目成本（首/重，µs/条目）：100k 6.5/6.0；dense 9.0/7.9；real 14.8/13.9（local 同口径 2.6/2.5/2.9 µs）。
每目录成本口径含 FSearch 客户端固定开销（约 0.3 s 用户态：启动、排序、保存数据库），不与该口径的 G5 WSL2 数字（≤0.15 ms/目录）混用；本报告以总耗时与同机同数据配对为主。

纯元数据遍历代理（无 FSearch 客户端开销，首/重秒）：

| 树 | 本轮 | .9 | local |
|---|---|---|---|
| 100k | 0.503 / 0.440 | 1.452 / 1.394 | 0.074 / 0.070 |
| dense | 2.354 / 2.093 | 8.313 / 7.827 | 0.378 / 0.336 |
| real | 4.925 / 4.582 | 16.627 / 16.263 | 0.429 / 0.402 |

### 3.2 索引构建与降级

| 树 | 修复前 build | 本轮 build | index_bytes | degraded/partial |
|---|---|---|---|---|
| 100k | 73–76 ms（未降级） | 73.1–75.8 ms | 43,527,984 | false / false |
| dense | 1,040,496 µs（命中 1 s 上限→整代丢弃） | 280,778 / 281,527 / 281,059 µs | 134,349,910 | false / false |
| real | 1,104,720 µs（命中 1 s 上限→整代丢弃） | 512,896 / 503,470 / 499,779 µs | 212,191,075 | false / false |

修复前大树全部回退 SQL 的端到端结果（同一 harness）：dense 11.53/11.28 s、real 24.54/24.14 s，均慢于 .9（8.78/8.37、17.48/17.10）；根因是候选在 `crates/drive-fs/src/directory.rs` 的 `Views::open` 权威视图路径对每个目录多做一次 `permissions::inode(..., Action::List)` ACL 查询（#7 的逐目录权限语义），该成本只在 SQL 回退时出现。

对照实验（只把 `BUILD_BUDGET` 1 s→5 s 的临时二进制，不修改提交）：完整构建 dense 1.5099/1.5326 s、real 1.0095/1.0198 s；索引生效后端到端 dense 2.72/2.47 s、real 5.27/5.22 s——与本轮最终结果一致，证明「构建进入预算 → 索引生效」即达成全部目标。

### 3.3 正确性

- 三套树 × 全部配置的 audit 比对：indexed = truth 条目数（102,130 / 319,679 / 369,298），missing = extra = duplicates = invalid = attribute_mismatches = 0。
- hydration 次数 = 0（扫描不触发正文下载）；fixture 均正常停止，无残留挂载。
- 100k 诊断运行（`fixture-final-diag`，索引路径）：`index_build_us=73,075`、`bytes=43,527,984`、`degraded=false`、`partial=false`；stage spans `LookupQueue count=5`、`LookupProjection count=10`（相对约 10.6 万次 LOOKUP，SQL 回退 <0.01%），首扫 `ReaddirReply 2,705`、`Releasedir 2,092`——索引路径确实承载了扫描流量。

## 4. 根因与修改

### 4.1 根因

`NamespaceIndex::load` 在 1 s 预算内完成构建，超时则整代丢弃并留空快照（`Snapshot::empty`，`degraded=true`）。旧实现每条目成本约 3.2–3.3 µs，导致 32 万/37 万条目树构建 1.01–1.51 s → 超限 → 所有索引查询降级回 SQL。降级后 `views.open` 权威路径每目录一次 ACL SQL，加上每目录多次 Store 串行等待，观测到每目录 ~0.1 ms 级成本，整树放大到 11–24 s。

### 4.2 修改（3 个文件，+504/−81）

1. `crates/drive-core/src/filesystem_posix.rs`
   - `visit_namespace_index` 双路径：先用 6 个一次性 `EXISTS` 探针（`access_denied`、`local_creations`、`local_modes`、`local_atimes`、`local_unlinks`、`drafts dirty=1`）判断树是否干净；干净树走 `NAMESPACE_LEAN_SQL`（10 列、无 5 个 record join、无父连接、无每行相关子查询），否则完全走旧投影 SQL（`Projection`/`finish`/`cacheable` 过滤原样）。
   - 父身份与可见性移到 Rust：扫描时把 `n.parent` intern 成 `u32` 槽位，第二遍按 `row.id` 匹配得 `parent_visible`/`parent_ino`（`NO_PARENT=u32::MAX`）；父行出现在扫描结果 ⟺ 父 live 且未被 `access_blocked`，与原 `p/pi` join 等价。
   - `visible_child`：lean 路径由 `local_creations` 为空探针证明恒真；慢路径一次查询建 Rust 冲突表（保留 `s.id != n.id` 语义）。
   - lean 属性解码：`ValueRef::Text` → `Arc<str>` 直接构造；借用式 `ProjectedEntry<'a>`；`EntryAttributes{size,mtime,atime,atime_nsec,mode:u16,blocks}`；行先 stage 到 `Vec<ScannedRow>`；`HashMap::with_capacity`。
   - body 语义保持：body 列用 `COALESCE(n.dirty_blob,n.blob)`（与旧 `record_columns()` 一致）；body 非 NULL 时仍 `std::fs::metadata(root/blobs/<body>)` 取长度作为 size/blocks，stat 失败则该行属性缺失（父目录 `partial`），block 数仍 `ceil(size/512)`。
   - 扫描期 `PRAGMA cache_size=-65536`（64 MiB）由 RAII 守卫恢复原值。
2. `crates/drive-core/src/namespace_index.rs`
   - `load()` 改为三遍构建：先 stage 行，再解析父槽位，最后建 `by_ino`/children/directories/incomplete 与排序后的 `DirBlock`；预算检查、超时整代丢弃、revision+stamp 校验均不变。
   - 新增测试：`lean_tree_clean_blob_size_uses_posix_precedence`（lean 路径下 size 取 blob 实际长度、与 `posix_record_by_inode` 一致）、`lean_tree_missing_blob_is_a_lean_attribute_miss`（blob 缺失 → `partial=true`、`entry_by_ino=None`、名字仍在父目录块）。
3. `docs/fix-5/nsi-build-optimization.md`：改动说明、逐项语义等价论证、远端验证记录、残余风险（新增/更新）。

### 4.3 未改变的行为（红线）

entry TTL 恒 0；attr TTL 默认 1000 ms（`DRIVE_NATIVE_FS_TTL_MS` 可调，本轮未调大）；整代 revision+stamp 守卫；dirty/draft/未链接/预览/索引 miss/不可信代数仍回退 SQL；容量与构建预算超限仍整代降级；索引构建仍只读、不分配 inode、不写库；FUSE 协议 TTL 参数顺序经核对正确，无产品改动。

## 5. 验证矩阵（远端 workstation202611）

见 `work/fix-5/verify/logs/<RunId>/`。Rust 验证入口 `work/fix-5/remote/verify/run-verify.ps1 -Stage rust`（上传源码到 `~/dn-fix5/src-verify`，隔离 `CARGO_TARGET_DIR=~/dn-fix5/build/target-verify`）。

- `cargo check --workspace --all-targets --all-features --locked`：exit 0（仅既有 warning）
- `cargo test --workspace --all-features --locked`：**1014 passed / 0 failed / 42 ignored**（基线 1012 + 本树新增 2 个 lean 测试）
- default-features：**840 passed / 0 failed / 36 ignored**
- 210k core 显式负载：1 passed；vendored fuser ABI：55 passed；runtime D-Bus：1 passed
- FUSE 内核显式组（`--ignored --test-threads=1`，debug/release × INDEX=1/0、attr TTL=0）：5 个 stage 最终全部通过，计数 38 / 36 / 38 / 36 / 38；取得通过所用尝试次数：debug-index1 1 次、debug-index0 2 次、release-index1 1 次、release-index0 3 次、release-attr0 1 次（详见下）
- GUI（等价 `scripts/test-gui.sh` 的 19 模块矩阵，offscreen + 独立 XDG + 每模块 `dbus-run-session`）：**19/19 模块、158 tests、0 failed**，`VERIFY_GUI_OK tests=158 failed=0 modules=19 dbus=1 warns=0`，source manifest 前后一致（`38e81a24…`）；日志 `work/fix-5/verify/logs/20261108-022636/`。GUI 验证复用此前上传的 `144d73d` 源码树（`-NoUpload`）；venv 复用 `~/dn/.venv-build`（Qt 6.12.0）而非按本树 lock 重装，属环境复用警告，不影响用例结论。

**既有 flake 说明（不掩盖）**：
1. FUSE 内核组中 `cached_close_tests` 的两个测试在本机裸机上是既有 flaky：`cached_close_tests.rs:269:48` 的 `TrySendError::Full`（测试辅助代码在 256 槽队列非空时 `try_send().unwrap()`）与 `cached_close_tests.rs:139` 的 barrier 超时。对照实验（同一远端、lib ignored 7 测试、5 次）：当前树 3/5 失败、基线 `367e33d` 树 1/5 失败，失败用例、行号、错误文本完全一致，且 `cached_close_tests.rs` 在两棵树完全相同。
2. 一次 `release-index0` 重试在 `readdirplus_continuation_does_not_wait_for_a_busy_store` 上挂起超过 5 分钟（该测试持 Store 锁验证 continuation 不阻塞），手动终止后的下一次尝试通过（36/0/0）；该 stage 在 run3 中亦整体通过。事后单独重跑该测试二进制（同样的 `INDEX=0`、诊断环境）在当前树与基线 `367e33d` 树上各 3 次，均 3/3 通过、未复现挂起——该挂起只在整组 `cargo test` 连续跑多个测试二进制的上下文出现一次。该配置 `DRIVE_NATIVE_FS_INDEX=0`，本次改动的 NSI 构建代码完全未启用、测试文件未改动，因此不归因于本次修改；按既有测试环境不稳定处理并如实记录。
3. 结论：验证阶段按「单 stage 重试至通过并记录尝试次数」取得绿色证据，不修改、不弱化任何产品测试；对应产品代码路径未被改动（本次只改 drive-core 的 NSI 构建）。

## 6. 未测项与边界

- 未接触真实 Synology NAS、未读取任何凭据、未在真实用户文件上执行写操作；real 树仅使用元数据 manifest 在夹具上重建布局。
- 未在真实 Mint/Cinnamon 面板、Nemo/Nautilus 鼠标交互、HiDPI/主题下实测（这些属于桌面功能，需真机）。
- `.7` 基线未能纳入配对：`c6f0a80`（dev20261102.7 源码）没有 `NotifyHub`/`publication`/NSI 等 API，仓库夹具无法直接编译进 .7；本报告以 `.9`（`d555944`）作为更保守的旧基线。issue 原描述中的「.7 对比」由 .9 对比替代，不宣称已覆盖 .7。
- 未安装/对比群晖官方 Linux 客户端（离线环境无官方包），因此不宣称「不超过官方 2 倍」这一讨论性目标。
- 基准为本机裸机 Linux 6.8 口径，不能与 G5 的 WSL2 数字直接混用；G5 旧候选数字已被本树结果取代。
- 生产下载树（含 blob）的 lean 路径会为每个干净 body 行做一次 `stat` 取真实长度；估算 30 万已下载文件约 0.2–0.3 s，仍在 1 s 预算内，但未在真实全量下载树上实测。
- 基准窗口虽确认无其他 cargo/test 进程、load 0.1–2.3，但主机为共享服务器，未做硬隔离；3 轮中位与 min/max 已保存在 `summary.json`。

## 7. 复现

```bash
# 远端（workstation202611）
source ~/.cargo/env
cd ~/dn-fix5
# 1) 构建 FSearch 0.3.1 + 审计器（见 docs/validation/FSEARCH.md）
# 2) 构建被测夹具（从提交 144d73d 源码）
CARGO_TARGET_DIR=~/dn-fix5/build/target-final cargo build --release --locked -j16 \
  -p drive-fs --example fsearch_fixture
# 3) 配对基准（每树 3 轮）
for tree in 100k dense real; do
  python3 bench.py --run ~/dn-fix5/run/final-3r-$tree-scan --kind scan --tree $tree \
    --manifest ~/dn-fix5/run/manifest-personal-sync.tsv --configs final,baseline9,local --rounds 3 \
    --fsearch ~/dn-fix5/fsearch/upstream-build/src/fsearch \
    --audit ~/dn-fix5/fsearch/fsearch-index-audit \
    --scan-bin ~/dn-fix5/bin/scan --fixture-dir ~/dn-fix5/bin
  python3 bench.py --run ~/dn-fix5/run/final-3r-$tree-fs --kind fsearch --tree $tree \
    --manifest ~/dn-fix5/run/manifest-personal-sync.tsv --configs final,baseline9,local --rounds 3 \
    --fsearch ~/dn-fix5/fsearch/upstream-build/src/fsearch \
    --audit ~/dn-fix5/fsearch/fsearch-index-audit \
    --scan-bin ~/dn-fix5/bin/scan --fixture-dir ~/dn-fix5/bin
done
python3 ~/dn-fix5/summary_view.py ~/dn-fix5/run/final-3r-100k-fs/summary.json
# 4) Rust 全量验证
pwsh -File work/fix-5/remote/verify/run-verify.ps1 -Stage rust
```

## 8. 证据清单

- `work/fix-5/run/final-3r-{100k,dense,real}-{scan,fs}/summary.json`、`work/fix-5/run/final-diag-100k/summary.json`（3 轮全部结果、min/max/中位、fixture CPU、loadavg、ready index 状态、audit 比对计数）
- `work/fix-5/logs/final-*.log`（FSearch/audit 原始输出）
- `work/fix-5/run/final-evidence.tar.gz`（远端证据包原件）
- `work/fix-5/verify/logs/<RunId>/{rust-stages.tsv,rust-run.log,rust-summary.txt,rust-kernel-*.log}`（Rust 验证）
- `work/fix-5/remote/`（`bench.py`、`scan.c`、`build-fixture.sh`、`build-fsearch.sh`、`build_manifest.py`、`verify/`）
- 远端 `~/dn-fix5/logs/`（FSearch 构建、夹具构建、基准日志）
