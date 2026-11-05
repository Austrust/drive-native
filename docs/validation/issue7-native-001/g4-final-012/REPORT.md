# #7 最终增量验证证据

候选 `9c9aabae447d92275f93dae65320788240a03113`；主矩阵与补测验收状态：`通过`。原件独立保留，重复执行不相加为唯一测试总数。

[旧 G4-006–010 包](prior/G4-evidence-bundle-010-blocked/REPORT.md)按原 MANIFEST 逐文件验证后原样嵌入；其 blocked 状态是历史记录，不重写。

旧 `41365a6` 的真实本机 TLS→worker ELF→IPC→Bridge/RPC 十项补测为 **8 通过 / 2 失败**；失败是 3002/3008 拒绝后重试门禁的真实反例，后续修复通过不能覆盖它。六项 Bridge 链与四项直接 Worker RPC 的范围分别列出。

## 主矩阵与补测

### g4-011

候选 `41365a650eb7d7df696046e21b7671697231f851`；complete=`True`，all_commands_passed=`False`。
[summary](evidence/g4-011/summary.json) · [逐名 inventory](evidence/g4-011/inventory-001.json)

| 命令 | 通过 | 失败 | 忽略 | 状态 |
|---|---:|---:|---:|---|
| [clean-private-product-packages](evidence/g4-011/logs/clean-private-product-packages.log) | — | — | — | 通过 |
| [retry-clock-preflight](evidence/g4-011/logs/retry-clock-preflight.log) | 4 | 0 | 0 | 通过 |
| [obsolete-ticket-preflight](evidence/g4-011/logs/obsolete-ticket-preflight.log) | 1 | 0 | 0 | 通过 |
| [original-retry-pair-preflight](evidence/g4-011/logs/original-retry-pair-preflight.log) | 2 | 0 | 0 | 通过 |
| [runtime-lib-original-parallelism](evidence/g4-011/logs/runtime-lib-original-parallelism.log) | 149 | 0 | 1 | 通过 |
| [native-read-stale-preflight](evidence/g4-011/logs/native-read-stale-preflight.log) | 1 | 0 | 0 | 通过 |
| [runtime-permission-preflight](evidence/g4-011/logs/runtime-permission-preflight.log) | 4 | 0 | 0 | 通过 |
| [runtime-profile-preflight](evidence/g4-011/logs/runtime-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [runtime-version-preflight](evidence/g4-011/logs/runtime-version-preflight.log) | 1 | 0 | 0 | 通过 |
| [core-permission-preflight](evidence/g4-011/logs/core-permission-preflight.log) | 23 | 0 | 0 | 通过 |
| [core-v6-migration-preflight](evidence/g4-011/logs/core-v6-migration-preflight.log) | 1 | 0 | 0 | 通过 |
| [kernel-parent-actions-preflight](evidence/g4-011/logs/kernel-parent-actions-preflight.log) | 1 | 0 | 0 | 通过 |
| [sproto-permission-preflight](evidence/g4-011/logs/sproto-permission-preflight.log) | 3 | 0 | 0 | 通过 |
| [sproto-profile-preflight](evidence/g4-011/logs/sproto-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [native-loopback](evidence/g4-011/logs/native-loopback.log) | 66 | 0 | 0 | 通过 |
| [worker-trust-preflight](evidence/g4-011/logs/worker-trust-preflight.log) | 1 | 0 | 0 | 通过 |
| [worker-elf-versions](evidence/g4-011/logs/worker-elf-versions.log) | 3 | 0 | 0 | 通过 |
| [core-history-probe-prepare](evidence/g4-011/logs/core-history-probe-prepare.log) | — | — | — | 通过 |
| [core-history-probe](evidence/g4-011/logs/core-history-probe.log) | 5 | 0 | 0 | 通过 |
| [kernel-setattr-preflight](evidence/g4-011/logs/kernel-setattr-preflight.log) | 1 | 0 | 0 | 通过 |
| [workspace-all-features](evidence/g4-011/logs/workspace-all-features.log) | 971 | 0 | 42 | 通过 |
| [workspace-default](evidence/g4-011/logs/workspace-default.log) | 961 | 0 | 37 | 通过 |
| vendored-fuser-wire | — | — | — | 未运行 |
| kernel-fs-debug-index1 | — | — | — | 未运行 |
| kernel-fs-debug-index0 | — | — | — | 未运行 |
| kernel-runtime-debug | — | — | — | 未运行 |
| kernel-fs-release-index1 | — | — | — | 未运行 |
| kernel-fs-release-index0 | — | — | — | 未运行 |
| kernel-runtime-release | — | — | — | 未运行 |
| kernel-fs-release-attr0 | — | — | — | 未运行 |
| runtime-private-dbus | — | — | — | 未运行 |
| gui-gui_unit_tests | — | — | — | 未运行 |
| gui-cache_gui_tests | — | — | — | 未运行 |
| gui-conflict_gui_tests | — | — | — | 未运行 |
| gui-hidden_dialog_tests | — | — | — | 未运行 |
| gui-lazy_gui_tests | — | — | — | 未运行 |
| gui-multi_mount_tests | — | — | — | 未运行 |
| gui-trust_gui_tests | — | — | — | 未运行 |
| gui-transfer_gui_tests | — | — | — | 未运行 |
| gui-theme_tests | — | — | — | 未运行 |
| gui-selector_gui_tests | — | — | — | 未运行 |
| gui-backend_upgrade_test | — | — | — | 未运行 |
| gui-pause_handover_test | — | — | — | 未运行 |
| gui-initialization_gui_tests | — | — | — | 未运行 |
| gui-unified_manager_tests | — | — | — | 未运行 |
| gui-gui_instance_test | — | — | — | 未运行 |
| gui-queue_gui_tests | — | — | — | 未运行 |
| gui-history_gui_tests | — | — | — | 未运行 |
| gui-permission_gui_tests | — | — | — | 未运行 |
| gui-permission-history-labels | — | — | — | 未运行 |
| provider-nemo | — | — | — | 未运行 |
| provider-nautilus | — | — | — | 未运行 |
| unified-ui-render | — | — | — | 未运行 |

### g4-012

候选 `9c9aabae447d92275f93dae65320788240a03113`；complete=`True`，all_commands_passed=`True`。
[summary](evidence/g4-012/summary.json) · [逐名 inventory](evidence/g4-012/inventory-001.json)

| 命令 | 通过 | 失败 | 忽略 | 状态 |
|---|---:|---:|---:|---|
| [offline-documentation-staging](evidence/g4-012/logs/offline-documentation-staging.log) | 2 | 0 | 0 | 通过 |
| [clean-private-product-packages](evidence/g4-012/logs/clean-private-product-packages.log) | — | — | — | 通过 |
| [core-definite_read_refusal_survives_restart_without_automatic_retry](evidence/g4-012/logs/core-definite_read_refusal_survives_restart_without_automatic_retry.log) | 1 | 0 | 0 | 通过 |
| [core-revoked_traversal_suppresses_existing_transport_retry_wakes](evidence/g4-012/logs/core-revoked_traversal_suppresses_existing_transport_retry_wakes.log) | 1 | 0 | 0 | 通过 |
| [runtime-bridge_respects_read_denial_before_cached_body_or_new_flight](evidence/g4-012/logs/runtime-bridge_respects_read_denial_before_cached_body_or_new_flight.log) | 1 | 0 | 0 | 通过 |
| [runtime-cancelled_flight_clears_retry_even_when_read_or_traverse_is_denied](evidence/g4-012/logs/runtime-cancelled_flight_clears_retry_even_when_read_or_traverse_is_denied.log) | 1 | 0 | 0 | 通过 |
| [runtime-concurrent_read_revocation_and_last_reader_cancel_leave_no_retry_intent](evidence/g4-012/logs/runtime-concurrent_read_revocation_and_last_reader_cancel_leave_no_retry_intent.log) | 1 | 0 | 0 | 通过 |
| [retry-clock-preflight](evidence/g4-012/logs/retry-clock-preflight.log) | 4 | 0 | 0 | 通过 |
| [obsolete-ticket-preflight](evidence/g4-012/logs/obsolete-ticket-preflight.log) | 1 | 0 | 0 | 通过 |
| [original-retry-pair-preflight](evidence/g4-012/logs/original-retry-pair-preflight.log) | 2 | 0 | 0 | 通过 |
| [runtime-lib-original-parallelism](evidence/g4-012/logs/runtime-lib-original-parallelism.log) | 152 | 0 | 1 | 通过 |
| [native-read-stale-preflight](evidence/g4-012/logs/native-read-stale-preflight.log) | 1 | 0 | 0 | 通过 |
| [runtime-permission-preflight](evidence/g4-012/logs/runtime-permission-preflight.log) | 4 | 0 | 0 | 通过 |
| [runtime-profile-preflight](evidence/g4-012/logs/runtime-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [runtime-version-preflight](evidence/g4-012/logs/runtime-version-preflight.log) | 1 | 0 | 0 | 通过 |
| [core-permission-preflight](evidence/g4-012/logs/core-permission-preflight.log) | 23 | 0 | 0 | 通过 |
| [core-v6-migration-preflight](evidence/g4-012/logs/core-v6-migration-preflight.log) | 1 | 0 | 0 | 通过 |
| [kernel-parent-actions-preflight](evidence/g4-012/logs/kernel-parent-actions-preflight.log) | 1 | 0 | 0 | 通过 |
| [sproto-permission-preflight](evidence/g4-012/logs/sproto-permission-preflight.log) | 3 | 0 | 0 | 通过 |
| [sproto-profile-preflight](evidence/g4-012/logs/sproto-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [native-loopback](evidence/g4-012/logs/native-loopback.log) | 66 | 0 | 0 | 通过 |
| [worker-trust-preflight](evidence/g4-012/logs/worker-trust-preflight.log) | 1 | 0 | 0 | 通过 |
| [worker-elf-versions](evidence/g4-012/logs/worker-elf-versions.log) | 3 | 0 | 0 | 通过 |
| [core-history-probe-prepare](evidence/g4-012/logs/core-history-probe-prepare.log) | — | — | — | 通过 |
| [core-history-probe](evidence/g4-012/logs/core-history-probe.log) | 5 | 0 | 0 | 通过 |
| [kernel-setattr-preflight](evidence/g4-012/logs/kernel-setattr-preflight.log) | 1 | 0 | 0 | 通过 |
| [workspace-all-features](evidence/g4-012/logs/workspace-all-features.log) | 976 | 0 | 42 | 通过 |
| [workspace-default](evidence/g4-012/logs/workspace-default.log) | 966 | 0 | 37 | 通过 |
| [vendored-fuser-wire](evidence/g4-012/logs/vendored-fuser-wire.log) | 55 | 0 | 0 | 通过 |
| [kernel-fs-debug-index1](evidence/g4-012/logs/kernel-fs-debug-index1.log) | 38 | 0 | 0 | 通过 |
| [kernel-fs-debug-index0](evidence/g4-012/logs/kernel-fs-debug-index0.log) | 36 | 0 | 0 | 通过 |
| [kernel-runtime-debug](evidence/g4-012/logs/kernel-runtime-debug.log) | 1 | 0 | 0 | 通过 |
| [kernel-fs-release-index1](evidence/g4-012/logs/kernel-fs-release-index1.log) | 38 | 0 | 0 | 通过 |
| [kernel-fs-release-index0](evidence/g4-012/logs/kernel-fs-release-index0.log) | 36 | 0 | 0 | 通过 |
| [kernel-runtime-release](evidence/g4-012/logs/kernel-runtime-release.log) | 1 | 0 | 0 | 通过 |
| [kernel-fs-release-attr0](evidence/g4-012/logs/kernel-fs-release-attr0.log) | 38 | 0 | 0 | 通过 |
| [runtime-private-dbus](evidence/g4-012/logs/runtime-private-dbus.log) | 1 | 0 | 0 | 通过 |
| [gui-gui_unit_tests](evidence/g4-012/logs/gui-gui_unit_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-cache_gui_tests](evidence/g4-012/logs/gui-cache_gui_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-conflict_gui_tests](evidence/g4-012/logs/gui-conflict_gui_tests.log) | 24 | 0 | 0 | 通过 |
| [gui-hidden_dialog_tests](evidence/g4-012/logs/gui-hidden_dialog_tests.log) | 3 | 0 | 0 | 通过 |
| [gui-lazy_gui_tests](evidence/g4-012/logs/gui-lazy_gui_tests.log) | 4 | 0 | 0 | 通过 |
| [gui-multi_mount_tests](evidence/g4-012/logs/gui-multi_mount_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-trust_gui_tests](evidence/g4-012/logs/gui-trust_gui_tests.log) | 6 | 0 | 0 | 通过 |
| [gui-transfer_gui_tests](evidence/g4-012/logs/gui-transfer_gui_tests.log) | 6 | 0 | 0 | 通过 |
| [gui-theme_tests](evidence/g4-012/logs/gui-theme_tests.log) | 1 | 0 | 0 | 通过 |
| [gui-selector_gui_tests](evidence/g4-012/logs/gui-selector_gui_tests.log) | 7 | 0 | 0 | 通过 |
| [gui-backend_upgrade_test](evidence/g4-012/logs/gui-backend_upgrade_test.log) | 11 | 0 | 0 | 通过 |
| [gui-pause_handover_test](evidence/g4-012/logs/gui-pause_handover_test.log) | 5 | 0 | 0 | 通过 |
| [gui-initialization_gui_tests](evidence/g4-012/logs/gui-initialization_gui_tests.log) | 10 | 0 | 0 | 通过 |
| [gui-unified_manager_tests](evidence/g4-012/logs/gui-unified_manager_tests.log) | 27 | 0 | 0 | 通过 |
| [gui-gui_instance_test](evidence/g4-012/logs/gui-gui_instance_test.log) | 3 | 0 | 0 | 通过 |
| [gui-queue_gui_tests](evidence/g4-012/logs/gui-queue_gui_tests.log) | 4 | 0 | 0 | 通过 |
| [gui-history_gui_tests](evidence/g4-012/logs/gui-history_gui_tests.log) | 17 | 0 | 0 | 通过 |
| [gui-permission_gui_tests](evidence/g4-012/logs/gui-permission_gui_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-permission-history-labels](evidence/g4-012/logs/gui-permission-history-labels.log) | 3 | 0 | 0 | 通过 |
| [provider-nemo](evidence/g4-012/logs/provider-nemo.log) | 10 | 0 | 0 | 通过 |
| [provider-nautilus](evidence/g4-012/logs/provider-nautilus.log) | 10 | 0 | 0 | 通过 |
| [unified-ui-render](evidence/g4-012/logs/unified-ui-render.log) | — | — | — | 通过 |

### g4-011/native-worker-chain-denials-001

[results](evidence/g4-011/native-worker-chain-denials-001/results.json)：`{'passed': 0, 'failed': 2, 'ignored': 0, 'total': 2}`。

| case | 路径范围 | 结果 |
|---|---|---|
| [bridge_download_denied_3002](evidence/g4-011/native-worker-chain-denials-001/bridge_download_denied_3002/case.json) | 实际 worker→IPC→Bridge | failed / passed=False |
| [bridge_download_denied_3008](evidence/g4-011/native-worker-chain-denials-001/bridge_download_denied_3008/case.json) | 实际 worker→IPC→Bridge | failed / passed=False |

### g4-011/native-worker-chain-other-001

[results](evidence/g4-011/native-worker-chain-other-001/results.json)：`{'passed': 8, 'failed': 0, 'ignored': 0, 'total': 8}`。

| case | 路径范围 | 结果 |
|---|---|---|
| [bridge_profile_false_empty](evidence/g4-011/native-worker-chain-other-001/bridge_profile_false_empty/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_profile_true_empty](evidence/g4-011/native-worker-chain-other-001/bridge_profile_true_empty/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_profile_true_other_view](evidence/g4-011/native-worker-chain-other-001/bridge_profile_true_other_view/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_late_read_after_profile](evidence/g4-011/native-worker-chain-other-001/bridge_late_read_after_profile/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [worker_rpc_generic_3004](evidence/g4-011/native-worker-chain-other-001/worker_rpc_generic_3004/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |
| [worker_rpc_generic_8004](evidence/g4-011/native-worker-chain-other-001/worker_rpc_generic_8004/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |
| [worker_rpc_generic_9005](evidence/g4-011/native-worker-chain-other-001/worker_rpc_generic_9005/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |
| [worker_rpc_generic_a005](evidence/g4-011/native-worker-chain-other-001/worker_rpc_generic_a005/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |

### g4-012/native-worker-chain-001

[results](evidence/g4-012/native-worker-chain-001/results.json)：`{'passed': 10, 'failed': 0, 'ignored': 0, 'total': 10}`。

| case | 路径范围 | 结果 |
|---|---|---|
| [bridge_download_denied_3002](evidence/g4-012/native-worker-chain-001/bridge_download_denied_3002/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_download_denied_3008](evidence/g4-012/native-worker-chain-001/bridge_download_denied_3008/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_profile_false_empty](evidence/g4-012/native-worker-chain-001/bridge_profile_false_empty/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_profile_true_empty](evidence/g4-012/native-worker-chain-001/bridge_profile_true_empty/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_profile_true_other_view](evidence/g4-012/native-worker-chain-001/bridge_profile_true_other_view/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [bridge_late_read_after_profile](evidence/g4-012/native-worker-chain-001/bridge_late_read_after_profile/case.json) | 实际 worker→IPC→Bridge | passed / passed=True |
| [worker_rpc_generic_3004](evidence/g4-012/native-worker-chain-001/worker_rpc_generic_3004/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |
| [worker_rpc_generic_8004](evidence/g4-012/native-worker-chain-001/worker_rpc_generic_8004/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |
| [worker_rpc_generic_9005](evidence/g4-012/native-worker-chain-001/worker_rpc_generic_9005/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |
| [worker_rpc_generic_a005](evidence/g4-012/native-worker-chain-001/worker_rpc_generic_a005/case.json) | 直接 Worker RPC，未应用 Bridge | passed / passed=True |

[容量及生命周期补测 summary](evidence/g4-012/supplemental-001/summary.json) · [逐名容量 inventory](evidence/g4-012/supplemental-001/derived-inventory.json) · [组件更换结果](evidence/g4-012/component-policy-handover-001/result.json)

210k ignored 用例属于功能容量验收，不是性能基准；只按本轮实际日志计数。组件更换为私有目录中实际 fsd/worker 配对，不是 Debian 安装、升级或重装。

## 完整性、恢复与范围

[历史 SHA 交叉核对](PRIOR_HASH_AUDIT.json)覆盖已有 summary→log/JSON、inventory→summary/log、TLS case/build 流与旧包 MANIFEST。没有先验 SHA 的文件只声明本次复制完整性。

ELF witness 比较的是主矩阵结束时仍存在的 ELF 在补测前后的身份，不是每次执行瞬间采样，也不是 ELF 字节备份。源代码/ELF/数据库不导出；已有身份摘要随原件保存。

[普通用户结束审计](evidence/g4-012/closeout.json)因两个 systemd 进程的 exe 链接权限限制记录为不完整；[独立 root 只读补核](evidence/g4-012/closeout-root-assisted.json)确认无验证 ELF/cargo/rustc 进程、无残留挂载、原件未变。两份原件均保留，未把权限限制记为产品测试失败。

harness 内旧 HARNESS_SHA256.json 是原静态快照，可能早于 wrapper 修订；不把它当作后续执行版本的权威。实际已有来源关联按 build.json.driver_files、生成 crate 的 provenance/lock-verification、TLS results 校验；其余文件只证明导出时字节身份。

[compact-001 前](environment/compact-001/before.json) / [后](environment/compact-001/after.json)；[compact-002 前](environment/compact-002/before.json) / [后](environment/compact-002/after.json)。缓存盘点仅是对应时点的 guest 文件元数据，不能直接等同 Windows C 可回收量；清理/压缩的实际执行只以环境原件为准。

[覆盖映射](COVERAGE.md) · [状态与缺项](STATUS.json) · [私有原件边界](PRIVATE_ORIGINALS.json) · [逐文件 SHA](MANIFEST.json)

全部新链路均为合成本机服务与测试身份。真实 NAS ACL、空间能力来源、服务端部分提交真值、真实桌面面板及 Debian 包生命周期未实测。本包不新增 G5 性能结论；既有 G5 原件和未达标结论由对应报告维护。
