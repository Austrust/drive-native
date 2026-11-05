# #7 增量验证设计映射

本文是设计和源码入口映射，**不表示任何用例已经通过**。候选及执行状态以协调者的冻结归档、命令日志和机器结果为准。本文不导入先前候选的通过结论。

[外部 harness](main.rs) 设计 66 项：下载 7、连接阶段 2、变更动作 44、profile 通知 13。它使用真实本机 TCP 回环收发、独立 pstream 对端和公开 `sproto::Client` API；没有调用产品私有解析器。构建和依赖锁流程见 [运行说明](RUNBOOK.md)。

## 分类口径

以下错误码全部按十六进制书写。表中名称为 Rust Error variant，机器 JSON 对应 `download_permission_denied`、`mutation_permission_uncertain`、`server`、`decode`。

`3004` 在下载响应中必须保持普通 `Server`，在本矩阵的变更动作响应中预期为 `MutationPermissionUncertain`。后者只提示可能涉及权限，**不证明无部分提交，不允许直接重试或标为完成**。`9005`、`a005` 在全部本矩阵动作中均排除权限分类，保持 `Server`。宽整数 `0x1_0000_3002` 必须 `Decode`，不得截断成 `3002`。

每项都要求服务端实际收到对应 action 及合成身份字段。下载失败须保留既有目标正文；文件上传须实际送达合成正文且源快照保持不变。每项独立连接和有界超时；任务 panic 或失败记录后继续后续矩阵。

## 直接动作响应

### 下载动作

公开 API：`Client::download_checked`。

| 实际 case.name | 注入响应码 | 预期 classification |
|---|---|---|
| `download_reply_3002` | `0x3002` | `DownloadPermissionDenied` |
| `download_reply_3008` | `0x3008` | `DownloadPermissionDenied` |
| `download_reply_3004` | `0x3004` | `Server` |
| `download_reply_8004` | `0x8004` | `Server` |
| `download_reply_9005` | `0x9005` | `Server` |
| `download_reply_a005` | `0xa005` | `Server` |
| `download_reply_100003002` | `0x100003002` | `Decode` |

### 文件上传

公开 API：`Client::upload_snapshot_at_identity`。

| 实际 case.name | 注入响应码 | 预期 classification |
|---|---|---|
| `upload_reply_3002` | `0x3002` | `MutationPermissionUncertain` |
| `upload_reply_3004` | `0x3004` | `MutationPermissionUncertain` |
| `upload_reply_3008` | `0x3008` | `Server` |
| `upload_reply_8004` | `0x8004` | `Server` |
| `upload_reply_9003` | `0x9003` | `MutationPermissionUncertain` |
| `upload_reply_9004` | `0x9004` | `MutationPermissionUncertain` |
| `upload_reply_9005` | `0x9005` | `Server` |
| `upload_reply_a003` | `0xa003` | `Server` |
| `upload_reply_a004` | `0xa004` | `Server` |
| `upload_reply_a005` | `0xa005` | `Server` |
| `upload_reply_100003002` | `0x100003002` | `Decode` |

### 重命名

公开 API：`Client::rename_native`。

| 实际 case.name | 注入响应码 | 预期 classification |
|---|---|---|
| `rename_reply_3002` | `0x3002` | `MutationPermissionUncertain` |
| `rename_reply_3004` | `0x3004` | `MutationPermissionUncertain` |
| `rename_reply_3008` | `0x3008` | `Server` |
| `rename_reply_8004` | `0x8004` | `Server` |
| `rename_reply_9003` | `0x9003` | `Server` |
| `rename_reply_9004` | `0x9004` | `Server` |
| `rename_reply_9005` | `0x9005` | `Server` |
| `rename_reply_a003` | `0xa003` | `Server` |
| `rename_reply_a004` | `0xa004` | `Server` |
| `rename_reply_a005` | `0xa005` | `Server` |
| `rename_reply_100003002` | `0x100003002` | `Decode` |

### 身份删除

公开 API：`Client::remove_at_identity`。

| 实际 case.name | 注入响应码 | 预期 classification |
|---|---|---|
| `remove_reply_3002` | `0x3002` | `MutationPermissionUncertain` |
| `remove_reply_3004` | `0x3004` | `MutationPermissionUncertain` |
| `remove_reply_3008` | `0x3008` | `Server` |
| `remove_reply_8004` | `0x8004` | `Server` |
| `remove_reply_9003` | `0x9003` | `Server` |
| `remove_reply_9004` | `0x9004` | `Server` |
| `remove_reply_9005` | `0x9005` | `Server` |
| `remove_reply_a003` | `0xa003` | `MutationPermissionUncertain` |
| `remove_reply_a004` | `0xa004` | `MutationPermissionUncertain` |
| `remove_reply_a005` | `0xa005` | `Server` |
| `remove_reply_100003002` | `0x100003002` | `Decode` |

### 创建目录

公开 API：`Client::mkdir_native`。其原生 wire action 为 `upload`。

| 实际 case.name | 注入响应码 | 预期 classification |
|---|---|---|
| `mkdir_reply_3002` | `0x3002` | `MutationPermissionUncertain` |
| `mkdir_reply_3004` | `0x3004` | `MutationPermissionUncertain` |
| `mkdir_reply_3008` | `0x3008` | `Server` |
| `mkdir_reply_8004` | `0x8004` | `Server` |
| `mkdir_reply_9003` | `0x9003` | `MutationPermissionUncertain` |
| `mkdir_reply_9004` | `0x9004` | `MutationPermissionUncertain` |
| `mkdir_reply_9005` | `0x9005` | `Server` |
| `mkdir_reply_a003` | `0xa003` | `Server` |
| `mkdir_reply_a004` | `0xa004` | `Server` |
| `mkdir_reply_a005` | `0xa005` | `Server` |
| `mkdir_reply_100003002` | `0x100003002` | `Decode` |

## 登录与连接阶段隔离

公开 API 均为 `Client::connect`，尚未调用下载 API；即使码为 `3002`，也不能学习文件 Read 拒绝。

| 实际 case.name | 实际响应边界 | 预期 classification |
|---|---|---|
| `download_setup_Auth_3002_is_generic` | auth，SCMD `0x12` | `Server { code: 0x3002 }` |
| `download_setup_Connect_3002_is_generic` | channel connect，SCMD `0x0e` | `Server { code: 0x3002 }` |

## Profile-only long-poll

公开 API 均为 `Client::wait_changes`；请求订阅 view 7、cursor 77、digest `prior-digest`。服务端检查实际请求字段，然后发送以下合成 response。

| 实际 case.name | 响应条件 | 预期 classification / 结果 |
|---|---|---|
| `profile_only_true_empty_views` | changes=[]，flag=1，新 digest | notification，true，`next-digest`，views=[] |
| `profile_only_true_other_view` | changes=[8]，flag=1，新 digest | notification，true，`next-digest`，views=[8] |
| `profile_false_explicit_digest` | changes=[]，flag=0，显式 digest | notification，false，`same-digest` |
| `profile_false_omitted_digest` | changes=[]，flag=0，省略 digest | notification，false，保留 `prior-digest` |
| `profile_omitted_flag_and_digest` | changes=[7]，省略 flag/digest | notification，false，保留 `prior-digest`，views=[7] |
| `profile_flag_two_rejected` | flag=2 | `Decode` |
| `profile_flag_string_rejected` | flag="1" | `Decode` |
| `profile_flag_null_rejected` | flag=null | `Decode` |
| `profile_changed_without_digest_rejected` | flag=1，省略 digest | `Decode` |
| `profile_numeric_digest_rejected` | flag=1，digest=9 | `Decode` |
| `profile_changes_missing_rejected` | 缺 changes | `Decode` |
| `profile_changes_scalar_rejected` | changes=7 | `Decode` |
| `profile_changes_noninteger_rejected` | changes=["7"] | `Decode` |

该组覆盖公开 Client 的真实回环接收与解析，**不等于真实 worker ELF 的通知链**，也不证明真实 NAS 会以这些字段发送 ACL 变化。下表的产品 UNIX IPC 回归补齐 worker 投影函数、IPC 和 Bridge 部分；两个层次应分别报告。

## 对应产品回归入口

路径相对冻结仓库根。名称供逐项匹配日志，表中不记录执行状态，也不把参数循环的分支数当成 Rust 测试数。

| 层次 / 文件 | 准确测试名 | 设计验证范围与边界 |
|---|---|---|
| sproto `src/actions/transfer.rs` | `actions::transfer::tests::only_verified_download_reply_codes_are_permission_evidence` | 下载响应映射及本地 I/O 排除；外部回环补实际 Client 收发 |
| sproto `src/error.rs` | `error::tests::oversized_error_code_cannot_alias_a_permission_refusal` | 宽整数不能截断为受识别权限码 |
| sproto `src/error.rs` | `error::tests::mutation_permission_hints_are_action_scoped_and_never_definite_refusals` | 变更响应按 action 分类；未知提交仍需核对 |
| sproto `src/actions/native.rs` | `actions::native::tests::profile_only_changes_require_a_digest_and_preserve_the_notification_bit` | profile 标记和 digest 解析契约 |
| drive-runtime `src/download_publication_tests.rs` | `bridge::download_publication_tests::native_read_refusal_learns_only_the_requested_identity_and_recheck_releases_retry` | worker mapper → 真 UNIX IPC → Bridge；仅请求 node 的 Read 拒绝、兄弟/其他 action 不污染、在途失效及重查恢复；输入 typed error 为人工构造 |
| drive-runtime `src/download_publication_tests.rs` | `bridge::download_publication_tests::local_io_and_transport_errors_are_not_native_permission_evidence` | 本地 EACCES、超时、session、普通 Server 不升格为远端 Read 拒绝 |
| drive-runtime `src/event_transfer_tests.rs` | `bridge::event_transfer_tests::profile_only_notification_invalidates_observed_acl_without_file_events` | worker projection → 真 UNIX IPC → Bridge；profile 变化失效证据但保留用户只读；不是运行 worker ELF 的原生会话 |
| drive-runtime `src/rpc.rs` | `rpc::tests::protocol_version_mismatch_disconnects_all_waiters` | 显式旧 20 和 VERSION+1 拒绝、连接关闭；当前各版本只有一个 pending request，不据测试名宣称多并发等待者均已验证 |
| drive-runtime `src/upload_tests.rs` | `bridge::upload_tests::definite_permission_refusal_retains_body_allows_sibling_and_rechecks` | 既有确定拒绝通道的正文/兄弟队列/重查；不得拿它替代新 mutation uncertain 路径 |
| drive-core `src/write_permissions.rs` | `write_permissions::tests::mutation_permission_hint_preserves_uncertainty_bytes_and_unknown_acl` | upload uncertain 保留正文、未知 ACL 和不可重放意图 |
| drive-core `src/write_permissions.rs` | `write_permissions::tests::directory_permission_hint_keeps_journal_and_reconciliation_barrier` | 目录意图保留及核对屏障 |
| drive-core `src/write_permissions.rs` | `write_permissions::tests::profile_change_expires_all_node_sources_but_preserves_scope_policy_and_intents` | profile 失效所有 node 证据来源；scope 策略和持久意图保留 |
| drive-core `src/write_permissions.rs` | `write_permissions::tests::parent_self_denials_do_not_stall_child_remove_or_rename_journals` | 父目录自身的 Delete/RenameSource 等拒绝不误挡独立子项意图；仍检查祖先 Traverse、子项 Delete/RenameSource、目的父目录 RenameTarget；只验证本地 journal 的可调度性 |
| drive-fs `tests/permissions_kernel.rs` | `parent_self_permissions_do_not_deny_independent_child_removal_or_rename` | 显式 ignored 内核用例；真实 FUSE 挂载启用 DefaultPermissions，合成 ACL 检查子项删除、覆盖 rename 及空目录删除不被父目录自身权限误挡，同时保留真正拒绝的 EACCES |
| drive-runtime `tests/trust_worker.rs` | `invalid_trust_rejection_is_delivered_before_worker_exit` | 真实 worker ELF 的无效证书拒绝先于退出，不执行原生联网 |

### 父目录自身权限与子项操作的补充边界

core 用例先拒绝 A 自身的 Delete 与 CreateDir，允许其子项 f 的 Delete，要求既有 remove journal 可调度；随后拒绝 A 的 Traverse 或 f 的 Delete 时要求不可调度，恢复后仍取回同一 operation。rename 分支拒绝 A 自身的 RenameSource 与 B 自身的 Delete，要求 A/f → B/moved 的 journal 可调度；B 的 RenameTarget 或 f 的 RenameSource 被拒绝时则必须停止调度。该用例不执行远端删除/重命名，也不证明远端提交完成。

FUSE 用例的第一组在 A 的 CreateFile、CreateDir、Delete、RenameSource、RenameTarget、Setattr 均为 Deny 时，仍允许显式 Delete=Allow 的 A/file 被删除；删除 A 本身、在 A 内创建目录或文件仍须返回精确 EACCES。第二组在 root 的 Delete/CreateDir、A 的 RenameSource、B 的 Delete 为 Deny 时，显式允许源 file 的 RenameSource 和覆盖目标 target 的 Delete，要求 A/file → B/target 成功且正文可读，再允许删除已空的 A。它针对 DefaultPermissions 的 mode 预检查与产品 hook 的组合，不把父目录自身拒绝推广成所有子项操作拒绝。

该 FUSE 用例需要普通 Linux 用户、`/dev/fuse` 与 `test-seams`，并显式选择 `--ignored`；默认测试输出中的 ignored 不构成执行证据。两个用例都使用注入的本地 ACL，均不计入上述 66 项回环矩阵；是否实际执行及通过由根报告记录。

## 与本矩阵分开记录的独立补测

这些名称来自协调者维护的脚本，不计入上述 66 项。

| 独立 harness | 准确 case/test 名称 | 设计范围 |
|---|---|---|
| `verify-worker-versions.py` | `worker_elf_handshake_v20`、`worker_elf_handshake_v21`、`worker_elf_handshake_v22` | 实际 worker ELF + 本地 UNIX socket；20/22 EOF且退出1，21无效CA返回固定拒绝且退出0；无网络通知会话 |
| `core-history-probe.rs` | `first_invalidation_creates_and_advances_authoritative_revision` | 首次无 policy 行时，失效也建立并递增 revision |
| `core-history-probe.rs` | `upload_confirmed_history_clears_hint_but_retains_original_response_detail` | upload 核对成功后清提示、保留响应 detail 和正文 |
| `core-history-probe.rs` | `mkdir_conflict_history_replaces_hint_and_retains_original_response_detail` | mkdir 冲突替换 pending 提示，保留响应 detail |
| `core-history-probe.rs` | `mkdir_confirmed_history_clears_hint_but_retains_original_response_detail` | mkdir 成功清提示、保留 detail |
| `core-history-probe.rs` | `replace_cleanup_uncertainty_never_replays_and_final_history_keeps_reply_detail` | 本地 source ID 保留，cleanup shadow 单独核对；最终保留最后一次清理响应 detail |
| `verify-history-labels.py` | `PermissionHistoryLabels.test_mkdir_label_is_chinese`、`PermissionHistoryLabels.test_replace_label_is_chinese`、`PermissionHistoryLabels.test_confirmed_row_removes_hint_from_tooltip_and_retains_response_detail` | 实际 HistoryModel 的合成行；不是 IPC 或持久历史生命周期验证 |

## 证据边界

所有 raw response 都是合成输入；未连接 NAS、未使用真实凭据、未扫描生产文件。这里的“实际”指本机 socket、公开 API 和对应产品函数实际参与，不代表真实服务端权限语义已经验证。外部回环不调用私有 worker mapper；worker ELF 补测只触及握手/版本/证书拒绝。产品核心确认 API 在独立历史补测中由 harness 显式调用，不能据此声称已经从真实远端证明提交。

回环类必须同时满足预期分类和服务端请求校验；不能将早期 InvalidConfig、未发送请求、连接失败或超时算作权限分类通过。执行报告应分别列本矩阵、产品测试及额外补测的计数、原日志和未实测项。真实 NAS 的通知字段、ACL 恢复、原生提交核对及安装包升级/重装保留，不由本文设计映射给出通过结论。

