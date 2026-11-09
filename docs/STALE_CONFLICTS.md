# 陈旧目录冲突标志（#6 follow-up）

## 结论

已提交的目录在本地库里可能残留 `nodes.conflict=1`。它不是可恢复的命名空间冲突，而是安装远端快照时给尚未确认的 namespace 操作打的标记，在 mkdir/rename 确认提交后没有被清除。自这个修复起：

- 快照安装、mkdir 确认、目录 rename 确认会在**同一个写事务**里清除该标志，且只清除满足「远端身份一致、无未完成状态」的目录；
- 真实冲突（脏内容、未终态操作、草稿、本地创建、删除排空）保持 `conflict=1` 与 `GetIssues` 恢复入口不变；
- 清除后 `status()` 不再返回 `Conflict`，`release()` 不再以冲突拒绝，GC/候选用集不再被污染。

## 症状

- `GetStatus`/桌面托盘把已绑定目录报为冲突或长期「需要检查」，但该目录没有可用的恢复动作（`can_keep_both=false`、`can_keep_directory=false`）；
- 目录树下的待上传文件有一个"冲突祖先"，`release()` 之类按 `conflict=0` 过滤的路径被拒绝；
- 重启、重新安装快照都不消失，直到该目录参与一次内容/命名空间写操作或人工修库。

## 根因链

1. 本地 mkdir/rename 已入队（`namespace_ops` pending）且 `namespace_overlay` 有条目时，安装远端快照不会看到该对象：`install_remote_snapshot` 把 overlay 的 `remote` 置为 `"null"`，并在 `state.rs` 的安装事务里给成员目录置 `conflict=1`；
2. 随后 mkdir 的 `confirm_directory`（`crates/drive-core/src/directories.rs`）或 rename 的 `confirm_rename`（`crates/drive-core/src/namespace.rs`）成功提交，绑定 `remote_aliases`、更新 `remote_version`、删除 overlay，但历史实现对 `nodes.conflict` 没有任何写入；
3. 于是标志永久残留。`sync_issue` 对已绑定目录保持沉默（不报为不可恢复冲突），但 `status()`、GC、`release()` 的过滤仍然看到它。

## 自愈规则

三条路径共用同一判据：目录、有冲突标志、未删除、脏内容为空、有 `remote_aliases` 绑定、非 `local-create-v1`、不在 `local_creations`/`namespace_overlay`/`local_unlinks`、没有未终态 `namespace_ops`/`operations`/`drafts`。满足时 `conflict=0`。

| 路径 | 位置 | 触发 |
| --- | --- | --- |
| 快照安装 | `crates/drive-core/src/state.rs` 安装事务，upsert 之后 | 修复历史库中已绑定且存活目录 |
| mkdir 确认 | `crates/drive-core/src/directories.rs` `confirm_directory` | 最后一个 mkdir 操作提交、overlay 删除后 |
| rename 确认 | `crates/drive-core/src/namespace.rs` `confirm_rename` | 远端版本匹配并提交、overlay 删除后，且仅限目录 |

上述清理是 stale-flag 自愈，不是冲突解决：标志背后仍存在的真实冲突不会被清除（见下节）。

## 不误清真冲突

- 脏内容（`dirty_blob`）、草稿、未终态上传/命名空间操作保留标志；
- 本地未确认的 mkdir 树（在 `namespace_overlay` 或 `local_creations`）保留标志，恢复入口仍是 `KeepDirectoryBoth`；
- 已删除（tombstone）目录不做自愈清理；它们的恢复语义见下节「绑定目录的 keep-local / keep-remote」。

## 绑定目录的 keep-local / keep-remote（issue #54）

已绑定远端身份（存在 `remote_aliases` 行）的目录被服务器删除（`deleted=1, conflict=0`）、但其下仍有 `deleted=0` 子节点时，安装快照事务会在文件树中把它标记为 `conflict=1`，因此问题中心候选集无需新增 SQL 分支即可看到它。

- 问题条目 `reason="bound_directory_conflict"`，令牌额外绑定目录的 `deleted` 状态、远端别名与存活后代签名；子项增删会使旧令牌失效。
- `can_keep_local=true`（子树仍有 inflight/uncertain 上传时动作返回可解释错误）；`can_keep_remote` 仅当子树没有「仅本地内容」（本地创建且无远端别名）时为 true，避免误删本地数据。
- **保留本地目录**（`KeepDirectoryLocal`）：保留节点 id、子孙与本地内容（inode/路径不变），解除旧远端别名，以本地新建身份重新排队 mkdir；已绑定的子项保持原远端身份（混合子树只重排根），`confirm_directory` 后绑定新远端别名。
- **接受远端删除**（`KeepDirectoryRemote`）：仅清除 `conflict` 标志、保留 tombstone 与远端别名；仅在没有仅本地内容时可用。
- 两者均为 additive D-Bus 方法，经 `GetOverview.recovery_actions.bound_directory_recovery` 与条目中的 `can_keep_local`/`can_keep_remote` 通告；不做 schema 迁移。

## 验证

```bash
cargo test -p drive-core --all-features --locked --test conflicts -- stale
cargo test -p drive-core --all-features --locked --test conflicts -- bound_directory
cargo test -p drive-runtime --all-features --locked --test desktop bound_directory_recovery
```

覆盖：mkdir/rename 提交后的自愈、历史库安装快照自愈与幂等、未终态操作与脏文件反例；以及绑定目录 tombstone 的 issue 表达、keep-local/keep-remote、重启与令牌失效。全量回归：

```bash
cargo test -p drive-core -p drive-runtime --all-features --locked
```

## 相关

- 恢复入口语义：`desktop/ISSUE_RECOVERY.md`；
- 快照安装与状态日志：`docs/DURABILITY.md`。
