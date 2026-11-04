# 规格到实际测试的映射

结果原件见 [G4-STATUS.md](REPORT.md)。以下列出测试的真实名称与证据范围；第二轮 `652eabe` 的命令及逐项状态已保存在 [inventory-001.json](evidence/g4-002/inventory-001.json)，最终第五轮 `bae1ef1` 主矩阵及补测全部通过，最终逐名清单见 [inventory](evidence/g4-005/inventory-001.json)。某一命令通过不代替本表的场景边界。下文 `unified`、`history_gui`、`permission_gui` 分别指同名 GUI 测试模块；Rust 短名在逐项清单保留完整 suite 名。

## PLAN §3.3 托盘

| 要求 | 实际测试 | 边界 |
|---|---|---|
| 左键展开/收起最近活动 | 独立补测 `test_left_click_toggles_activity_and_double_click_activates_manager` | 调用实际 activation handler；合成事件 |
| 历史详情、打开/定位 | 独立补测 `test_recent_history_details_and_buttons_route_to_selected_record`；history_gui 的 `test_cached_file_opens_resolved_current_path`、`test_online_file_hydrates_before_opening`、`test_deleted_record_can_locate_validated_parent` | 托盘实际近期历史组件；外部应用启动用 mock；另有本地真实 reader 用例 |
| 右键主窗口/历史/设置/退出/暂停 | unified `test_tray_menu_has_history_settings_and_two_exit_choices`、`test_pause_all_only_resumes_tasks_it_paused_and_preserves_manual_pause`、`test_late_bulk_pause_reply_cannot_reclaim_manual_pause_from_any_page` | 无真实桌面右键事件 |
| 悬停汇总 | 同上菜单测试；独立补测 `test_tooltip_counts_and_text_distinguish_idle_pending_paused_and_attention` | 核对挂载、上传/待同步、问题数 |
| 关闭到托盘 | unified `test_close_with_tray_hides_without_stopping_and_without_tray_stays_reachable` | 实际窗口 close handler，托盘能力注入 |
| 再次启动 | gui_instance `test_second_process_activates_existing_gui_without_new_manager`、`test_strict_activation_and_startup_queue` | 本地第二进程与私有 D-Bus |
| 图标和文字状态 | 独立补测 `test_tooltip_counts_and_text_distinguish_idle_pending_paused_and_attention`；unified `test_online_uncertain_queue_is_visible_in_manager_and_tray` | 验证非空图标和状态文字；图标颜色/形状差异未做像素断言 |
| 两类退出语义 | unified `test_stop_all_waits_for_every_mount_and_ui_exit_does_not_stop`、`test_stop_all_request_is_not_completion_and_failed_stop_keeps_ui`、`test_stop_all_owner_loss_is_not_process_or_unmount_confirmation` | 停止状态回复为合成值；既有 shutdown 内核组验证真实卸载 |
| 跟随面板位置 | 独立补测 `test_injected_top_bottom_left_right_panel_geometry_and_cursor_fallback` | 4 角图标及光标回退坐标注入；真实 GNOME/KDE 面板 **未实测** |
| 托盘不可用仍可达 | unified `test_close_with_tray_hides_without_stopping_and_without_tray_stays_reachable`、`test_tray_disappearance_restores_window` | 合成 capability 变化 |

独立托盘补测的第二轮结果为 4 通过 / 0 失败 / 0 忽略，原件 [tray-synthetic.log](evidence/g4-002/supplement/tray-synthetic.log)。

## PLAN §3.4 历史

| 要求 | 实际测试 | 边界 |
|---|---|---|
| schema 24、备份与拒绝降级 | core sync_history `schema_23_upgrade_preserves_dirty_data_and_future_schema_is_rejected`、`failed_migration_rolls_back_schema_and_keeps_coherent_backup`；独立 schema reader 六项 | 独立 23/24 reader 来自冻结旧源码编译；非历史发布 ELF |
| 同事务写入与重试合并 | `history_is_atomic_with_operation_and_is_not_built_by_queries`、`failed_download_retries_merge_without_claiming_cancellation_as_success`、`downloaded_receipt_merges_with_transfer_completion`、`remote_events_are_recorded_once_in_application_transaction`、`rename_move_and_remove_follow_verified_namespace_receipts`、`preparation_failure_merges_attempts_without_submitting_or_reclassifying_uncertain_work` | 本地状态/事务与注入 receipt，无 NAS |
| 持久化 | `history_survives_store_restart` | 实际 Store 关闭/重开 |
| 保留及安全清理 | `retention_and_clear_preserve_local_content_and_operations`；runtime `history_queries_clear_only_history_and_reject_unsafe_clear_requests`；history_gui `test_clear_all_has_no_days_and_no_file_mutation_calls` | runtime 严格 unknown-field 清理拒绝在第二轮失败，第三轮已通过 |
| 分页、筛选、增量身份 | `pages_filter_literal_names_and_keep_monotonic_sequence_after_clear`；history_gui `test_filters_start_new_cursor_and_stale_reply_is_ignored`、`test_service_ids_are_scoped_and_busy_task_does_not_block_other`、`test_mount_filter_only_calls_selected_service`、`test_hidden_page_rejects_inflight_response_and_makes_no_new_query` | mock IPC + core 真数据库 |
| 当前身份 resolve、打开/下载/定位 | runtime `history_target_handles_online_offline_cached_deleted_and_unmounted`；history_gui `test_cached_file_opens_resolved_current_path`、`test_offline_without_content_and_deleted_do_not_open`、`test_path_escape_or_replaced_service_never_opens`、`test_actual_reader_success_revalidates_before_open_and_preserves_content`、`test_actual_reader_failure_never_opens_or_claims_completion`、`test_task_replacement_during_real_read_discards_completion`、`test_node_replacement_after_real_read_never_opens_replacement` | 实际本地 reader，服务器/外部应用用 mock |
| 不存正文/凭据 | `download_history_never_copies_unstructured_error_bodies`、`version_keys_do_not_collide_on_delimiters`；history_gui `test_record_and_model_keep_control_text_plain_and_stable_order` | 固定敏感字符串和不结构化错误样本；未声称穷尽任意错误文本 |

## PLAN-7 §9.2 权限

| 场景 | 实际测试与注入真值 | 边界 |
|---|---|---|
| 根拒绝、A 允许、B 拒绝 | core `root_denial_does_not_become_a_child_write_policy`；kernel `root_a_b_and_action_classes_are_enforced_in_one_mount`：root/B CreateDir=Deny、A CreateDir=Allow | 真实 FUSE 系统调用，合成权威有效权限 |
| 独立挂 A | `independent_subroot_stops_at_its_own_ancestry_boundary`；kernel `target_parent_cover_target_and_independent_submount` | 第二轮独立挂载已越过原身份构造错误，末尾 ExplicitRO errno 断言仍失败，第三轮已通过 |
| mkdir 与 create 分离 | `action_classes_are_independent`；kernel `root_a_b_and_action_classes_are_enforced_in_one_mount`：A CreateDir=Allow、CreateFile=Deny | DefaultPermissions 保留 |
| 写允许、删除/改名拒绝 | 同一 kernel 用例：file WriteContent=Allow、Delete/RenameSource=Deny | 实际写入、截断、unlink、rename |
| 继承中断、用户/组允许+显式拒绝 | 上述逐节点 effective decision 真值；`precise_source_wins_over_a_late_observed_success`、`remote_overlay_ancestry_is_not_bypassed_by_local_move` | **未实测原生 ACL 继承/用户组解析**；注入的是最终有效权限，不能冒充服务端 ACL 计算 |
| 可 traverse、不可 list | `traverse_is_distinct_from_list`；kernel `live_revocation_restoration_and_list_vs_traverse`：A List=Deny 时可读已知子项，A Traverse=Deny 后拒绝子项 | 另一目录仍可访问 |
| A→B/覆盖重命名 | kernel `target_parent_cover_target_and_independent_submount`：B RenameTarget=Deny→Allow；覆盖目标 Delete=Deny→Allow | 实际拒绝后恢复并成功 rename |
| 撤销/恢复/离线重连 | kernel `live_revocation_restoration_and_list_vs_traverse`；`decisions_survive_restart_and_recheck_preserves_data`、`recheck_releases_only_the_typed_read_refusal_retry_gate`、`first_denial_publication_waits_for_existing_bounded_reply`、`cached_reply_lease_does_not_wait_for_ordinary_metadata_store` | 撤销既有句柄、持久化重开和 retry seam 有测；**真实网络离线重连未实测** |
| 服务空间限制/显式 RO | `explicit_and_space_limits_override_child_allow`、`recheck_resets_observed_but_never_explicit_read_only`；runtime `permission_overview_query_and_user_policy_revision_are_consistent` | SpaceForbidden 注入；真实空间能力和服务端拒绝码映射 **未实测** |
| 单项拒绝不阻塞无关队列 | runtime `definite_permission_refusal_retains_body_allows_sibling_and_rechecks`；core `denied_namespace_intent_does_not_block_independent_sibling`、`definite_refusal_preserves_bytes_and_uncertain_is_never_reset` | 本地 typed transport refusal；uncertain 不被重发 |
| 旧 profile false/true | runtime `legacy_profiles_keep_their_original_read_only_choice` 同一用例遍历 false→ExplicitReadOnly、true→FollowRemote；GUI selector 7 用例 | 配置反序列化/保存；未连接账号 |
| schema 25 迁移/备份/旧版拒绝 | `schema24_migration_preserves_history_dirty_bytes_and_backup`；独立 schema reader 23→24→25、23拒24/25、24拒25 | 最终候选新 reader 六项补测已通过，旧两 reader 固定 |
| unknown 不持久化 allow | `unknown_attempt_does_not_persist_allow`、`recheck_resets_observed_but_never_explicit_read_only`；typed runtime refusal | 未发生权威确认时表行数为0；真实服务端 mapping 未实测 |
| POSIX 一致性 | 保留 DefaultPermissions；既有 kernel `kernel_permissions_owner_restore_and_local_capacity_are_real`、`kernel_random_write_shared_handles_truncate_fsync_and_recovery`、`kernel_unlink_open_handles_same_name_recreate_and_offline_missing_guard`、`kernel_atomic_replace_keeps_source_inode_and_old_target_handle` | 不触发“移除 DefaultPermissions 后逐项重做 POSIX”的条件；不声称新增穷尽所有 uid/gid/umask/sticky/exec 组合 |

UI 四态另由 permission_gui 5 用例及 MainWindow 四态浅深色 8 图覆盖。QLabel 文案图与完整产品窗口图分开保存；IPC 为本地合成回复。真实 NAS、File Station 和真实系统面板验收均未实测。

## 追加容量、超时与mtime交叉验收

- 默认保留10,000条的实际IPC及GUI：独立harness `default_retention_10001_to_10000`、`actual_ipc_10000_pagination`、`actual_ipc_combined_mount_result_literal_name_filter`、`actual_history_page_100_pages_gui_responsive`、`actual_history_page_result_name_filter`。真实ClientBus/Desktop与HistoryPage、合成私有SQLite，250ms Qt心跳预算为harness自定；非硬配置上限、非真实NAS。
- 实际15s配置的timeout：`real_clientbus_15s_timeout_page_error_healthy_peer_and_heartbeat`，同协议独立延迟peer、真实ClientBus和HistoryPage，另一正常服务仍可读；实际14.519s返回。原件[history-scale-verified](evidence/g4-005/history-scale-verified/results.json)。
- 安装包升级/重装历史保留：**未实测**；schema升级和Store重开只证明各自范围。
- PLAN-7 §4.3的Setattr/WriteContent交叉：`setattr_only_persists_mtime_without_granting_content_writes_or_uploads`，实际utimes、bytes、写/截断拒绝、queued upload门禁及重开mtime持久化；远端全文件上传在WriteContent拒绝时保持pending，不声称远端mtime同步。原Setattr拒绝组合新增chmod实际拒绝。Traverse拒绝新增missing路径EACCES。
