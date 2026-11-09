# Windows / CFAPI 移植路线（#11 设计骨架）

本文是 issue #11 的落地摘要，不是完整设计论证。目标是固定三个平台接口、给出去 POSIX 的最小改动顺序，并让「核心可在无 POSIX 依赖下编译」变成可执行的回归门禁。完整论证、代码事实与逐行清单见 #11 讨论与评审记录。

- 基线：`integration/fixes3`（#9 TB1 之后，schema **v29**：`nodes.in_sync`、`platform_state_changes`、`commit_outcome` 已存在）
- 本阶段（step 0）只改脚本与文档：新增 `scripts/check-core-posix.sh`、`scripts/core-posix-allowlist.txt`、本文，并在 `scripts/verify.sh` 接入一行调用；**不改 Rust 代码、不改 schema、不改 Linux 行为**。
- 证据等级：本文的仓库路径与行号均为当前树实测（P0）；CFAPI 调用面、Windows 文件 ID 生命周期、watcher 自事件竞争为待原型验证的推断（P1，见 §6）。

## 0. 结论摘要

1. 三个接口现在就可以固定，不依赖 #9 CommitOutcome 的落地；顺序是 **FileIdentity → StatePush → LocalChangeSource**。
2. 真正的工作量不在「抽 trait」，而在 **身份通道与错误语义**：core 的 `ino` 同时是 SQLite 主键、目录项投影键、预览保留 inode 与 20+ 公开方法的入参，必须先降为平台实现细节。
3. **StatePush 不需要新 schema**：v29 的 `platform_state_changes` + `nodes.in_sync` 就是持久层，缺的是「core 在事务内写队列」+「runtime 平台驱动 drain + ack」。
4. FileIdentity 需要 **v30**：新增 `platform_file_ids` 增量表（本设计唯一建议的 schema 变更）。
5. Windows 的真实验收必须在 **Windows 原生 runner** 上做；Linux 交叉 `cargo check` 只能覆盖纯 Rust 的 `drive-platform` 骨架（`rusqlite bundled` / `fuser` / `nix` 使 core 无法在 Linux 上做 Windows target 的完整 check）。
6. 明确不做见 §5；首轮不重写 FUSE 回调层，不把 Windows 运行时依赖引入 core。

## 1. 现状与边界

| 层 | 现状 | 移植结论 |
| --- | --- | --- |
| `sproto` 远端协议 | 纯 Rust（bytes/macon/rustls/serde/tokio），无平台 API | 两平台复用，不需要接口 |
| `drive-core` 业务引擎 | SQLite 唯一持久权威；但身份用合成 inode、错误用 `libc::E*` 构造 `io::Error` | 语义可移植，**结构不可移植**（P0 障碍） |
| `drive-fs` | 直接 `impl fuser::Filesystem`；`errno()`/`attr()`/`statfs`/名称 UTF-8 检查全是 POSIX | Linux 专用；Windows 由 CFAPI 占位符 + 回调节点承担 |
| `drive-runtime` | 单写 fsd + worker、`Invalidation` 内核通知、`/proc/self/mountinfo`、`setrlimit`、Unix socket IPC | Linux 专用；Windows 需要 PlatformSession（本设计不展开） |
| `desktop` GUI | Qt + D-Bus，通知/托盘已平台化 | 本轮不涉及 |

**三块 P0 缺口**

- **FileIdentity**：`FileRecord.ino: u64`（`crates/drive-core/src/filesystem.rs:18`）与 `fs_inodes(ino INTEGER PRIMARY KEY AUTOINCREMENT, node TEXT UNIQUE NOT NULL)`（`state.rs:543`）让 core 拥有 inode 分配权；根固定 ino=1（`filesystem_inodes.rs:31-48`、`filesystem.rs:408-435`），预览又复用保留 inode（`preview.rs:82-91,308-334`）。Windows 的 128 位真实文件 ID 塞不进这个契约。
- **StatePush**：语义变更日志目前是 `TEMP` 表（`status_changes.rs:214-232`），消费端是 `StatusChanged2` JSON（`status_events.rs`）；v29 的 `platform_state_changes`（`state.rs:377-389`）与 `nodes.in_sync`（`state.rs:341-345`）**全仓无生产者、无消费者**。Linux 平台通知仍是 `Invalidation::Inode/Entry/All`（`bridge.rs:71-77`，生产调用点 19 处）。
- **LocalChangeSource**：全仓无 inotify / `ReadDirectoryChangesW` / CFAPI 调用；本地变更完全靠 FUSE 拦截后写 core journal（`drive-fs/src/writable.rs`、`namespace.rs`、`lib.rs`）。Windows 没有 FUSE 拦截层，这个输入通道必须新建。

**不可破坏的既有约束**（`docs/P1_ARCHITECTURE.md`）

- SQLite 是唯一持久权威，平台状态是派生镜像，不得反向作为上传/删除依据（`:45`）。
- 停止/卸载期间旧句柄的已确认保存契约不变（`:71`）。
- 性能门禁不得放宽：2000 项目录 p95 ≤ 300 ms、21 万节点 GetStates ≤ 500 ms、48 路混合 ≤ 1 s 且零超时、约 20 万节点 RSS ≤ 300 MB（`:86`）。FileIdentity 不允许在 getattr 热路径增加一次额外 SQL 往返。

## 2. 三个 trait 签名草案

共同约定（`crates/drive-platform`，workspace 第 5 个成员，edition 2024）：

- 默认不依赖 tokio/rusqlite/zbus；Windows 专属依赖只出现在 `cfg(windows)` + `feature = "cfapi"`。
- 全部 `Send + Sync`、**同步 API**；node 身份继续用 `String`（与 `nodes.id` 一致），不发明新 ID 类型。
- 平台错误用 `PlatformError`，core 负责映射进现有 `Error`/`CommitCategory`；**libc errno 类型不得进入 core 公共错误**。
- `pub const PLATFORM_API_VERSION: u16 = 1;`，与 IPC/wire 契约（`wire.rs:3`，当前 21）联动升版。

### 2.1 FileIdentity（第一步，契约影响最大）

```rust
pub struct PlatformFileId([u8; 16]);      // 不透明；Linux 用低 8 字节放合成 inode
pub struct LocalHandle(pub u64);          // Linux: FUSE fh/inode；Windows: HANDLE
pub enum Access { Read, Write, ReadWrite }

pub trait FileIdentity: Send + Sync {
    fn reserve_id(&self) -> PlatformResult<PlatformFileId>;
    fn id_for_handle(&self, handle: LocalHandle) -> PlatformResult<PlatformFileId>;
    fn open_by_id(&self, id: &PlatformFileId, access: Access) -> PlatformResult<LocalHandle>;
    fn native_inode(&self, id: &PlatformFileId) -> PlatformResult<u64>; // FUSE 投影；Windows Unsupported
    fn release_id(&self, id: &PlatformFileId) -> PlatformResult<()>;
}
```

- core 只持久化 node ↔ platform id 映射（`bind_platform_id` / `platform_id_for_node` / `node_for_platform_id`）；Linux 适配器继续用 `fs_inodes` 分配编号，**升级前后 inode 编号不变**（FUSE 旧句柄/目录 cookie 语义保持）。
- v30 迁移（增量）：
  ```sql
  CREATE TABLE platform_file_ids(
      node TEXT PRIMARY KEY,
      platform_id BLOB NOT NULL,
      generation INTEGER NOT NULL DEFAULT 0,
      backend TEXT NOT NULL CHECK(backend IN ('inode','cfapi')),
      UNIQUE(backend, platform_id)
  );
  INSERT INTO platform_file_ids SELECT node, ino, 0, 'inode' FROM fs_inodes;
  ```
  `fs_inodes` 表保留给 Linux 适配器；`record_impl` 对 `platform_file_ids` 做 `LEFT JOIN`（仍是一条 SQL），首次分配才写入。`FileRecord.ino` 分两步废弃（先加 `platform_id`，再移除入参）。
- 风险：重启/升级后 inode 稳定性；预览保留 inode 与 CFAPI 占位符身份不兼容（首轮 `cfg(unix)` 门控）；Windows `open_by_id` 在卷不支持 FILE_ID_128 时需回退。

### 2.2 StatePush（第二步）

```rust
pub enum PlatformStateKind { Placeholder, Hydrate, HydrateProgress, Dehydrate, InSync, Conflict, Delete }
pub struct QueuedPlatformState { pub seq: i64, pub node: String, pub kind: PlatformStateKind, pub payload: String }
pub enum PushOutcome { Delivered, Retry { retry_after_ms: u64, reason: &'static str } }

pub trait StatePush: Send + Sync {
    fn capabilities(&self) -> StateCapabilities;          // placeholders/hydration_progress/in_sync
    fn push_batch(&self, batch: &[QueuedPlatformState]) -> PlatformResult<Vec<PushOutcome>>;
}
```

- **写入是 core 的 SQL，trait 只做 drain + ack**：平台 I/O 不能放进 core 的 SQLite 事务边界。core 在业务变更的同一事务内 `enqueue_platform_state`（回滚则队列行一起回滚）；runtime 新增 `platform_state_pump` 轮询 `pending_platform_states(bounded)` → `push_batch` → `Delivered` 写 `delivered_at`、`Retry` 写 `attempts+1`/`last_error` 并按上限退避（复用 `commit_outcome::retry_delay` 风格）。
- Linux 适配器 `capabilities()` 全 false、`push_batch` 全 `Delivered`（继续走现有 invalidation），**Linux 行为零变化**；Windows 侧 `CfSetInSyncState`/`CfUpdatePlaceholder`/`CfReportProviderProgress` 在 Step 5 原型里接。
- `nodes.in_sync` 仍由 core 在提交确认后置 1、脏写清 0（语义权威）；平台镜像允许滞后但必须可重放。首轮不加 `platform_epoch` 列。
- 风险：队列积压需上限与丢弃策略（参考 status journal 的 8192 条 / 4 MiB 预算）；同一 node 的 Hydrate→InSync 必须保序；CFAPI in-sync 是每占位符状态，与 `nodes.in_sync` 语义不能互相替代。

### 2.3 LocalChangeSource（第三步）

```rust
pub struct LocalChange { pub path: PathBuf, pub kind: LocalChangeKind }
pub enum LocalChangeKind { Created, Modified, Renamed { from: PathBuf }, Removed, AttrChanged }
pub type ChangeSink = Box<dyn Fn(LocalChange) + Send + Sync>;

pub trait LocalChangeSource: Send {
    fn start(&mut self, roots: &[PathBuf], sink: ChangeSink) -> PlatformResult<ChangeSourceGuard>;
    fn reconcile_all(&mut self, roots: &[PathBuf]) -> PlatformResult<Vec<LocalChange>>;
}
```

- Linux：`start` 只是把 sink 注册到现有 journal 写入边界（`drive-fs/src/writable.rs`、`namespace.rs`、`lib.rs`），不新增 inotify；`reconcile_all` 返回空。**行为零变化是硬验收**。
- Windows：每同步根一个 `ReadDirectoryChangesW`（64 KiB 缓冲、`OVERLAPPED` + 事件线程），溢出→`reconcile_all`；启动顺序为「snapshot 起点 → attach watcher → diff 补差」；与 journal 的竞争用 fencing：watcher 事件先查 `namespace_overlay`/`local_creations` 待确认记录，本进程已登记的不重复写 journal。
- **最大未知（P1）**：CFAPI 水合是提供方自己写文件数据，`ReadDirectoryChangesW` 无法区分来源，会把水合写入当外部修改。候选对策：按 path 注册 in-flight 区间抑制、水合完成后对账；需要 Windows spike。这是把 LocalChangeSource 放最后的主因。

## 3. drive-core 去 POSIX：清单、最小改动集、门禁

### 3.1 分类清单（生产代码；测试模块另计）

| 类别 | 位置（当前树实测） | 处理 |
| --- | --- | --- |
| 模块级 `std::os::unix` | `drafts.rs:7`（`FileExt::read_at`）、`task_control.rs:10`（`MetadataExt/OpenOptionsExt`；`:14` O_NOFOLLOW\|O_NONBLOCK、`:23` uid/geteuid、`:46` 目录 fsync）、`preview.rs:401-425`（`O_DIRECTORY\|O_NOFOLLOW`、AsRawFd、errno 分支） | 抽 `SecureIo`（read_at / open_private_no_follow / fsync_dir）+ 预览模块首轮 `cfg(unix)` 门控 |
| 已 `#[cfg(unix)]` 门控 | `state.rs:178-183`（`ReadHandle::read_at`）、`state.rs:1192-1193`、`:1332-1333`、`:1375-1376`（目录 fsync） | 保留门控，Step 3 由 `SecureIo` 替换后删除 |
| 目录 fsync | `File::open(<目录>)?.sync_all()` 共 6 处：`drafts.rs:172`、`download_publication.rs:131`、`state.rs:1193/1333/1376`、`task_control.rs:46`（`download_publication.rs:49` 也是目录） | `SecureIo::fsync_dir`（Windows 打开目录会失败） |
| `libc::E*` 错误构造 | 18 个生产文件（`filesystem.rs`、`filesystem_posix.rs`、`namespace.rs`、`namespace_index.rs`、`removals.rs`、`replacements.rs`、`creations.rs`、`directories.rs`、`conflicts.rs`、`download_jobs.rs`、`drafts.rs`、`preview.rs`、`state.rs`、`timestamps.rs`、`write_permissions.rs` 等） | 换成语义错误变体（Busy/NotEmpty/IsDir/Unsupported/Stale/CrossDevice），`drive-fs/errno()` 反向映射 |
| 身份面 | `filesystem.rs:6-9,18,49-66,408-435,452-549`、`filesystem_inodes.rs:31-48`、`filesystem_posix.rs:21,57,597`、`preview.rs:82-91,308-334`、`namespace_index.rs:26-181`、`native_snapshot.rs:475` | FileIdentity Step 1；`ino` 只作 drive-fs 数字投影 |
| 名称策略 | 现只有 FUSE 边界拒绝非 UTF-8（`drive-fs/src/lib.rs:374-375,482-483`、`namespace.rs:20-23,83-84`） | `NamePolicy`（大小写折叠/保留名/非法字符/长度/尾点空格；不可表示名持久映射），Windows 侧在服务器→本地命名处应用 |
| 平台依赖 | `crates/drive-core/Cargo.toml:21 libc = "0.2"` | Step 3 后 libc 只留在 `drive-platform` |

不在 core 但同一移植批次的平台面：`drive-fs` 的 fuser 类型/`errno()`/`attr()`/statfs、`drive-runtime` 的 `transport.rs` SCM_RIGHTS + `SO_PEERCRED`、`bridge.rs` worker stdin fd、`mount_registry.rs`、`setrlimit`、`tokio::signal::unix`（见 §4 的 WorkerTransport/PlatformSession 边界）。

### 3.2 最小改动集（Step 0 只做第 0 项，其余按序实施）

1. 新增 `crates/drive-platform`：`PlatformError`/`PlatformFileId`/`LocalHandle` + 三 trait + `SecureIo`/`NamePolicy`/`PlatformSession` 骨架（Linux 逐步搬、Windows 返回 `Unsupported`），workspace 全绿。
2. FileIdentity：v30 + Store helper + Linux inode 适配 + `record_impl` 单查询 JOIN + drive-fs 数字映射；`FileRecord.ino` 分两步废弃。
3. `SecureIo`：read_at、open_private_no_follow + owner/mode 检查、`fsync_dir`（6 处目录 fsync）、`WriterLock(File)`（`state.rs:115-116,144,497`）→ Windows `LockFileEx`。
4. errno → 语义错误（18 文件机械替换；新增错误变体控制在 6–7 个；映射表放 `drive-platform/src/error_map.rs`）。
5. `NamePolicy`（双平台真实冲突点；不可表示名持久映射，不修改远端名）。
6. StatePush 队列：core 同事务入队 + runtime pump + Linux no-op + Windows 骨架。
7. `preview.rs` 整模块 `cfg(unix)` 门控（Windows 初始化预览另案）。
8. `WorkerTransport`：`spawn_worker`/`send`/`recv` trait 化（UnixStream vs 命名管道 + DuplicateHandle），wire VERSION 升 22；Windows 实现后置。
9. `HydrationRequest`：range/priority/transfer_id/cancel/progress 从 FUSE 请求生命周期拆出，作为 CFAPI 水合输入契约。

### 3.3 门禁脚本 `scripts/check-core-posix.sh`

- 扫描 `crates/drive-core/src/**/*.rs`（跳 `*_tests.rs`）+ `Cargo.toml`（+ 有则 `build.rs`），命中规则：`os::unix` 导入/API、`libc::`、`nix::`、`#[cfg(unix)]`、目录 fsync、Cargo 里的 `libc`/`nix` 依赖。
- 三模式：默认只报未 allowlist 违例并 `exit 1`；`--list` 打印全部命中及状态（不判失败）；`--strict` 忽略 allowlist 全量报并 `exit 1`。
- allowlist：`scripts/core-posix-allowlist.txt`，每行 `仓库相对路径:grep -E 正则`，注释写明「为何暂时允许 + 迁移步骤」。当前条目录入现状（Step 0 基线）：模块级 unix 三处、已 gated 的 state.rs 块、6 处目录 fsync、errno 构造 15 个文件（语义化后删除）、Cargo 的 libc 依赖（Step 3 后删除）。
- 设计意图：新增 POSIX 用法必须显式登记理由，避免「Windows 适配」时才发现 core 里又长出一块平台代码；`--strict` 用于评审时看全量面。

### 3.4 CI 现实与建议

- 仓库当前**没有 `.github`**，唯一门禁是 `scripts/verify.sh`（Linux：workspace 测试 + `drive-native-lab --simulate`）。
- Linux 上可立即执行：`cargo test --workspace --all-features --locked`、`cargo check -p drive-platform`、`cargo check -p drive-platform --features cfapi`（验证 feature gating）、`bash scripts/check-core-posix.sh`。
- Linux 交叉 check Windows target 只能做到 `cargo check -p drive-platform --target x86_64-pc-windows-msvc`（纯 Rust、无链接；需 `rustup target add`）。`drive-core` 因 `rusqlite bundled`（C 工具链）、`fuser`/`nix`（Linux only）无法在 Linux 上完整 check。
- **唯一真验收**是 Windows 原生 runner：`cargo check --workspace --all-features --locked`（后续再加 `cargo test -p drive-core -p drive-platform`，前提是测试模块先做 `#[cfg(unix)]` 门控）。建议本阶段之后单独提交 `.github/workflows/ci.yml`（ubuntu + windows 双 matrix）。
- 本分支不新增 `.github`；门禁已接入 `scripts/verify.sh`：`cargo test` 之后、`--tests-only` 早退之前执行 `bash scripts/check-core-posix.sh 2>&1 | tee evidence/core-posix.log`，默认模式失败即阻断整条门禁；`--strict` 仅供收紧基线时评审用，不接入 CI。

## 4. CFAPI 骨架：可编译范围

- 新 crate `crates/drive-platform`：
  ```
  Cargo.toml            # 默认无平台依赖；[features] cfapi = ["dep:windows-sys"]
  src/lib.rs            # 三 trait + PlatformError + PlatformFileId/LocalHandle + 常量
  src/error.rs  src/identity.rs  src/state_push.rs  src/local_change.rs
  src/secure_io.rs      # fsync_dir / read_at / open_no_follow / writer lock 抽象
  src/name_policy.rs
  src/linux/mod.rs      # cfg(unix)：基于现有 core/fs 代码的适配（首轮可返回 Unsupported）
  src/windows/mod.rs    # cfg(windows)：骨架 impl，全部 PlatformError::Unsupported
  src/windows/cfapi.rs  # cfg(all(windows, feature="cfapi"))：FFI 声明 + 类型，不运行
  ```
- 「可编译」的三层定义：Linux 默认 `cargo check -p drive-platform`；Linux 交叉 `--features cfapi --target x86_64-pc-windows-msvc`；Windows 原生 `cargo check --workspace --all-features`。
- 骨架只写签名/类型与 `Unsupported` 返回：不调用 `CfConnectSyncRoot`/`CfCreatePlaceholders`/`CfSetInSyncState`/`CfUpdatePlaceholder`/`CfReportProviderProgress`，不做 Shell/服务/注册表/同步根 UI，不把 `windows-sys` 带进 `drive-core` 依赖树。

## 5. 实施顺序、验收与回归风险

| 步 | 内容 | 验收 | 主要回归风险 |
| --- | --- | --- | --- |
| 0 | `drive-platform` 骨架 + 本文 + `check-core-posix.sh`（本阶段） | workspace 全绿；三模式脚本结果符合预期；文档完整 | 与 #9/#31 并行分支的 Cargo/workspace 冲突 |
| 1 | **FileIdentity**：v30 + Store helper + Linux inode 适配 + JOIN + drive-fs 数字映射 | workspace 测试；`metadata_performance`；mounted 内核测试；v29→v30 回填/迁移测试；`status_latency_210k --ignored` | inode 稳定性（重启/旧句柄/目录 cookie）；预览保留 inode；每请求 SQL 次数；`FileRecord.ino` 大规模改动漏改 |
| 2 | **StatePush**：core 同事务入队 + runtime pump + Linux no-op + Windows 骨架 | 队列回滚/重放/顺序/上限测试；Linux invalidation 行为零变化；崩溃重启重发 | 双通道重复通知；`nodes.in_sync` 与 CFAPI 镜像语义混淆；队列增长 |
| 3 | **LocalChangeSource**：Windows watcher + 启动对账 + 自事件抑制；Linux 注册到现有 journal 边界 | Windows 原型（spike）+ fake FS 对账/溢出/fencing 单测；Linux 行为零变化；mounted 写路径回归 | **水合自事件**（最大未知）；watcher 溢出丢事件；启动窗口竞争；与 `namespace_overlay`/`local_creations` 的遮蔽规则冲突 |
| 4 | NamePolicy / SecureIo / errno 语义化（可与 1–3 交错，schema/身份先不动） | core 测试 + drive-fs errno 映射；Windows skeleton 单测 | 18 文件替换面大；Linux 错误文案/类别变化导致 GUI/测试断言变化 |
| 5 | Windows 运行时（PlatformSession/服务/IPC/CFAPI 真实现） | 后续 issue，不属 #11 骨架 | — |

**FUSE / mounted 回归矩阵（Step 1–3 每步都跑）**

- `cargo test --workspace --all-features --locked`（存量全量，基线 `evidence/tests.log`）。
- 内核测试（普通 Linux 用户 + `/dev/fuse`；**ignored 不能当通过**，按 `docs/ACCEPTANCE_COMMANDS.md` 逐二进制串行、`--test-threads=1`、`DRIVE_NATIVE_FS_DIAGNOSTICS=1`）：`drive-fs/tests/mounted.rs`、`permissions_kernel.rs`、`namespace_index_kernel.rs`、`directory_views`/`readdirplus`/`entry_revalidation`/`interrupted`/`deadlock_regression`/`preview_tests`/`preview_plus_revoke`。
- 性能不回归（Step 1 必跑）：`status_latency_210k`、`namespace_index_budget`、`metadata_performance`、`drive-fs bench_tests`；门禁值见 §1。
- GUI 不回归（触及 schema/身份时）：`GUI_PYTHON=$HOME/dn/.venv-build/bin/python bash scripts/test-gui.sh`。
- 迁移：新增 `crates/drive-core/tests/schema_v30.rs`（或扩展 `schema_v29.rs` 模式）覆盖回填/重复迁移/空 `fs_inodes`/预览 inode 冲突。

**明确不做（本周期边界）**

1. 不重写 `drive-fs` FUSE 回调层：`writable.rs`/`namespace.rs`/`lib.rs` 入口结构不变，只接 sink 与错误映射；`mounted.rs` 的 saved-handle/fsync 契约不动。
2. 不把 Windows 运行时依赖引入 core：`drive-core/Cargo.toml` 不加 `windows-sys`/CFAPI/命名管道依赖；`cfg(windows)` 只出现在 `drive-platform`/`drive-runtime` 骨架。
3. 不做完整 CFAPI：不实现占位符创建/水合/脱水/进度/同步根注册/Shell 集成。
4. 不动 `DriveBackend`/`Commit` 死接口，不做 CommitOutcome 接线（#9 TB2/TB3）。
5. 不重写 IPC：`transport.rs`/`bridge.rs` 的 Unix 形状先保留，`WorkerTransport` 只设计接口与版本常量。
6. 不改变 Linux 行为/性能：Linux 适配器必须等价替换（编号、invalidation、错误分类、fsync 顺序）。
7. 不做预览的 Windows 等价实现（首轮 `cfg(unix)` 门控，另案）。
8. 不做 GUI/桌面平台层（#11 范围外）。

## 6. 需要在 Windows 上原型验证的推断项（P1）

1. CFAPI 占位符文件 ID 生命周期：创建后是否稳定（`FILE_ID_INFO` vs `CfGetPlaceholderInfo`），重命名/替换后是否变化。
2. `open_by_id`（`OpenFileById`）对 cloud placeholder 的可用性，以及离线/在线状态下的 HRESULT。
3. `ReadDirectoryChangesW` 与 CFAPI 水合的自事件竞争（抑制方案可行性）。
4. `CfSetInSyncState` 的调用前提（连接同步根后、文件句柄要求）与批量性能。
5. NTFS 大小写不敏感 vs 服务器大小写敏感的命名双向映射（不可表示名持久化）对同步语义的影响。

---

相关文档：[P1 架构约束](P1_ARCHITECTURE.md) · [状态变更日志](STATUS_CHANGE_JOURNAL.md) · [验收命令](ACCEPTANCE_COMMANDS.md) · [交付门禁](../DELIVERY_GATES.md) · 门禁脚本 `scripts/check-core-posix.sh`
