# NSI 构建优化（issue #5，面向 FSearch 大树）

分支 `codex/fix-5-nsi-build`（基线 367e33d）。只改索引**构建路径**：

- `crates/drive-core/src/namespace_index.rs`（`load()` 重写为三遍构建；`ProjectedEntry` 改为借用视图）
- `crates/drive-core/src/filesystem_posix.rs`（`Store::visit_namespace_index` 双路径投影）

未改动：`BUILD_BUDGET`、预算计数公式、超时整代丢弃、entry TTL=0、attr TTL、revision+stamp 守卫、FUSE 消费方、SQL 之外的任何写路径。索引构建仍然**只读**（不写库、不分配 inode）。

## 1. 背景与瓶颈

G5 基线（裸机）dense 319,679 条目 / 56,073 目录构建 1.04 s、real manifest 369,299 / 66,776 构建 1.10–1.11 s，均超过 `BUILD_BUDGET=1s` → 整代丢弃（degraded），FSearch 全部回退 SQL，扫描比 .9 基线更慢。

本地无 Rust 工具链，因此用 Python + sqlite3 复刻完全相同的 schema 与 G5 树形（dense 319,679 / real 369,299），单测 SQL 侧成本（fetch 口径，dense，ms）：

| 查询 | 2 MB cache（默认） | 64 MB cache |
| --- | ---: | ---: |
| 现状：17 列 + 5 个 record join + p/pi 父连接 + 每行相关 `NOT EXISTS` | 586 | 456 |
| 去掉相关子查询，保留 record join + 父连接 | 494 | 353 |
| 去掉空表（local_modes/local_atimes/local_unlinks/dirty drafts）连接与父连接（10 列） | 303 | 207 |
| 只扫 n.id,i.ino,n.name,n.parent（父行解析用） | 125 | 40 |

结论：SQL 大头是 **5 个 record join + 2 个父连接 + 每行相关子查询**；real manifest 更受页缓存约束（长 `manifest:{path}` id，2 MB cache 下 10 列扫描也要 1.33 s）。Rust 侧另有 ~470–500 ms 的行解码/分配/建表成本（1.04 s − 586 ms）。两项都要砍。

## 2. 改动

### 2.1 双路径投影（`filesystem_posix.rs::visit_namespace_index`）

先做 6 个**一次** `EXISTS` 探针：`access_denied`、`local_creations`、`local_modes`、`local_atimes`、`local_unlinks`、`drafts WHERE dirty=1`。

- **快速路径**（6 个全空，G5 三棵树都满足）：一条 10 列 SQL

  `SELECT n.id,n.directory,m.size,m.mtime,m.mode,n.blob,n.dirty_blob IS NOT NULL,i.ino,n.name,n.parent FROM nodes n LEFT JOIN node_metadata m ON m.node=n.id LEFT JOIN fs_inodes i ON i.node=n.id WHERE n.deleted=0`

  无相关子查询、无 p/pi 父连接、无 access_blocked 子查询（探针已证其空）。
- **慢速路径**（任一探针非空）：保留原 `record_columns()`/`record_joins()` 投影与 `Projection`、`Projection::finish`、`cacheable/unlinked/dirty` 过滤逻辑**原样**，只把 `n.parent` 直接取出、去掉 p/pi 连接与相关子查询（语义在 Rust 复现，见 2.2/2.3）。

`visible_child` 语义从 SQL 搬入 Rust：仅当 `local_creations` 为空时快速路径恒 `true`（与原 `NOT EXISTS` 等价）；慢速路径用一次小查询

`SELECT s.parent,s.name,s.id FROM nodes s JOIN local_creations c ON c.node=s.id WHERE s.deleted=0 AND s.parent IS NOT NULL`

建 `(parent,name) -> [id]` 两层的 `HashMap`，行 `id` 在该桶内不存在**其它** id 时可见——保留原 SQL 的 `s.id != n.id`（只有自己不算冲突）与 NULL parent 不参与匹配的语义；TEXT 均为 BINARY collation，Rust `str` 相等即字节相等。

### 2.2 父身份/可见性移到 Rust（`namespace_index.rs::load`）

扫描时把 `n.parent` 字符串 intern 进 `HashMap<Box<str>, u32>`（父 id 数量 ≈ 目录数，远小于行数），行内只存 `u32` 槽位。第二遍用每行自身 id 匹配父槽位，得到 `parent_visible: Vec<bool>`、`parent_ino: Vec<Option<u64>>`。

等价性：原 `LEFT JOIN nodes p` + `LEFT JOIN fs_inodes pi` 的 `parent_visible = p.id IS NOT NULL AND p.deleted=0 AND p NOT IN access_blocked`；而**父行出现在扫描结果中** ⟺ 父节点 live 且未被 ACL 屏蔽，`pi.ino` 即父行自己的 `i.ino`。因此不需要再对 `nodes`/`fs_inodes` 做两次随机探针。

### 2.3 解码/分配成本

- 文本列走 `ValueRef::Text` → `&str` → `Arc<str>`（`text_column`/`optional_text_column`），省掉每列一次 `String` 分配 + 一次 `Arc` 拷贝；不再为每行构造 `PosixRecord`（慢速路径除外）。
- `ProjectedEntry<'a>` 改为借用 `id/name/parent`，`attributes` 改成轻量 `EntryAttributes`（size/mtime/atime/atime_nsec/mode u16/blocks）。
- 行先 stage 到 `Vec<ScannedRow>`，再按 pass 2/3 构建 `by_ino`/`children`/`directories`，`HashMap::with_capacity(rows.len())`；每行 `Arc` 分配从 3 次降到 2 次（id、name），`DirEntryLite` 复用。

### 2.4 快速路径不 stat、不 finish

blob 为 NULL 的行不调用 `std::fs::metadata`（G5 大树本来就没有 blob，旧代码在这条路上也不 stat，但旧代码每行仍构造 `Projection` 并走 `finish` 的条件判断）；dirty 行属性直接置 `None`（旧逻辑算完也会因 `cacheable=false` 丢弃），缺元数据 → `None`（与旧一致触发 `partial=true`）。快速路径**绝不写库**。

### 2.5 扫描期临时提高 SQLite 页缓存

`CacheSizeGuard`：`PRAGMA cache_size=-65536`（64 MiB），`Drop` 时恢复读取到的原值（含错误/超时路径）。dense 实测 SQL 侧 586→456 ms（~22%）。Store 是单连接 + `StoreGuard` 独占访问，构建期间无并发读该 PRAGMA；全仓生产代码此前不使用 `cache_size`。

## 3. 语义不变性论证

| 语义 | 旧实现 | 新实现 | 等价性 |
| --- | --- | --- | --- |
| `parent_visible` | p/pi join | 父行是否在扫描集 | 扫描 WHERE 已排除 deleted/blocked，见 2.2 |
| `parent_ino` | `pi.ino` | 父行 `i.ino` | 同一 fs_inodes 行 |
| `visible_child` | 相关 `NOT EXISTS`（同 parent+name、其它 id、live） | 快速路径恒 true（探针证明无冲突）；慢速路径 Rust 同条件含 `s.id!=n.id` | 等价 |
| root（`n.parent IS NULL`） | parent=自身 ino | `NO_PARENT` → parent=ino | 等价；`entry_by_ino(1).parent==1` 测试保持 |
| 缺 ino 行 | `partial=true`，父目录进 `incomplete` | 相同（父 ino 可解析时） | 等价；`dir_block=None` 不分配 inode |
| ACL / `access_blocked` | 每行 `NOT IN` 子查询 + p 的可见性 | 6 探针任一非空即回退原投影（原 SQL 原样） | 逐字节等价 |
| 属性 | `Projection::finish`（含 stat/draft/uuid/atime 校验） | 快速路径仅在 6 表全空时启用：无 draft/unlink/local_mode/atime；cacheable = `!dirty`；缺失元数据 → None | 等价（见风险 R1） |
| 整代一致性 | revision+stamp 前后校验 | 未改 | 不变 |
| 预算/降级 | 超时整代丢弃 | 未改 | 不变 |
| 写路径 | 只有 ino None 时 `INSERT fs_inodes`（索引路径不会走到） | 不写 | 更严格只读 |

已知差异（不影响生产消费方）：

- `EntrySlot.parent`：父节点为 deleted/blocked 的行，旧值为 p/pi 拿到的 raw 父 ino，新值为 0。`EntrySlot.parent` 在 drive-fs 无消费方（grep 确认），仅 `namespace_index.rs` 测试断言 root 的 `.parent==1`。子/目录块的归属不受影响：这类父目录本身不会出现在扫描结果里。
- 快速路径 size 取 `node_metadata.size`，不再对 blob 文件做 `stat`（见 R1）。

## 4. 验证清单（远端）

```bash
export PATH="$HOME/.cargo/bin:$PATH"; cd ~/dn-fix5-opt
CARGO_TARGET_DIR=$HOME/dn-fix5-opt/build/target-opt cargo fmt --check
CARGO_TARGET_DIR=$HOME/dn-fix5-opt/build/target-opt cargo check --workspace --all-targets --all-features --locked -j 16
CARGO_TARGET_DIR=$HOME/dn-fix5-opt/build/target-opt cargo test -p drive-core namespace_index -j 16
CARGO_TARGET_DIR=$HOME/dn-fix5-opt/build/target-opt cargo build --release --locked -j 16 -p drive-fs --example fsearch_fixture
```

必过单测：`batch_projection_matches_sql_projection`、`budgets_discard_whole_generation`、`missing_inode_forces_directory_fallback_without_allocating`、`acl_overlay_hides_directory`、`local_creation_shadows_remote_child`、`explicit_atime/mode/dirty` 系列、`degraded_holder_recovers`。

基准验收（`fsearch_fixture` release）：dense/real 均 `index_degraded=false`、`index_build_us` ≤ 600,000（目标 dense ≤500,000）；scan/fsearch first/repeat 不劣于 .9 基线；correctness 计数全 0。

## 5. 风险

- **R1（属性来源）**：快速路径用 `node_metadata.size` 代表已下载文件长度。下载发布时写入的 blob 长度由远端 size 决定且校验 sha256，正常路径两者一致；若用户绕过客户端改动 blob 文件，旧实现会反映文件长度，新实现反映 DB。此类改动会产生 dirty draft 行（已排除在索引外），属于不支持场景。若验证阶段需要严格 `stat` 语义，可加一条 `EXISTS(SELECT 1 FROM nodes WHERE blob IS NOT NULL)` 探针让有 blob 的树走慢速路径（代价：此类树的构建回到 stat 成本）。
- **R2（缓存 PRAGMA）**：64 MiB 仅在构建窗口内生效并恢复；由 `Drop` 兜底，异常路径下最坏是连接后续沿用 64 MiB（只影响性能，不影响正确性）。
- **R3（慢速路径性能）**：含本地创建/覆盖的树仍走原投影 + Rust 冲突判定，构建成本接近旧值；大树 + 大量本地创建的极端场景可能仍超预算（与旧行为一致：整代丢弃回退 SQL）。
- 回滚：单次 revert 本提交即可，无 schema/数据迁移。

## 6. 预期收益

| 树 | 现状 | 预期 | 依据 |
| --- | ---: | ---: | --- |
| dense 319,679 | 1.04 s（丢弃） | ~0.42–0.55 s | SQL 586→~250–300 ms + Rust 500→~150–250 ms |
| real 369,299 | 1.10 s（丢弃） | ~0.55–0.75 s | SQL 受页缓存；64/128 MiB cache 视远端结果再调 |

（远端实测数字与两轮调整见提交后的验证记录。）
