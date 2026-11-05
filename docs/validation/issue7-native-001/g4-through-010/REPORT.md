# #7 增量独立验证：环境阻塞交接

待验证候选 `41365a650eb7d7df696046e21b7671697231f851` 的 G4-010 在复制构建缓存时发生 I/O/只读文件系统错误，尚未启动清理、构建或测试，测试数为 0。本包不宣称该候选通过；已执行的最后候选为 G4-009 / `4e6aaf7166fd370076adf43a0dad3e6ecdca36d6`。

仅收录 G4-006–010 本轮小型原件；不重新导出 G4-005 及更早历史。无 NAS、真实凭据或生产树输入，无 push/发布。未运行新的 G5、大树性能或 10k 历史压力。

故障后核对原件与故障前已有 SHA 记录：[PRIOR_HASH_AUDIT](PRIOR_HASH_AUDIT.json) 共 146 项，匹配 146 项，不一致/缺失 0 项。没有历史 SHA 的文件仅具备本次复制和 manifest 完整性，未冒称故障前后已比对。

## 失败归属与未执行项

- G4-006：迟到 Read 拒绝预检 0/1/0，真实产品失败，Deny 与预期 Unknown 不符。
- G4-007 首次：复制 target 后没有重编 core，保留为缓存污染，不能当 ca86a85 产品失败；workspace 编译按协调要求中断。
- G4-007/fresh-001：强制重编后 runtime 144/0/1，原迟到 Read 通过；workspace 964/1/41 与三包 default 791/1/36 保留同一旧 v6 夹具 trigger 错误。
- G4-008：候选已归档但未开始测试，被后续夹具修复候选替代，测试 0 项。
- G4-009：workspace all-features 964/2/42，两个原 900ms 前台重试断言失败；default 956/0/37。release INDEX1 已启动且通过，release INDEX0、runtime release、attr0 按协调要求暂缓，未计通过。
- G4-010：环境准备失败，测试 0 项；未执行新时钟用例、原失败复验或最终完整矩阵。

## 逐组实际结果

重复执行组不能相加为唯一测试总数。每轮 inventory 保存精确 suite/name、各组状态与原日志 hash；全部已报告计数与逐名解析一致。

### g4-006

候选 `d39a59769e0c0abcde033ae0f74bb2806f218d8a`；[summary](evidence/g4-006/summary.json)、[inventory](evidence/g4-006/inventory-001.json)。

| 命令 | 通过 | 失败 | 忽略 | 结果 |
|---|---:|---:|---:|---|
| [native-read-stale-preflight](evidence/g4-006/logs/native-read-stale-preflight.log) | 0 | 1 | 0 | 失败/中断，见原件 |

### g4-007

候选 `ca86a85e2be9c83c7a59407fc0737114158ecf7f`；[summary](evidence/g4-007/summary.json)、[inventory](evidence/g4-007/inventory-001.json)。

| 命令 | 通过 | 失败 | 忽略 | 结果 |
|---|---:|---:|---:|---|
| [native-read-stale-preflight](evidence/g4-007/logs/native-read-stale-preflight.log) | 0 | 1 | 0 | 失败/中断，见原件 |
| [runtime-permission-preflight](evidence/g4-007/logs/runtime-permission-preflight.log) | 4 | 0 | 0 | 通过 |
| [runtime-profile-preflight](evidence/g4-007/logs/runtime-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [runtime-version-preflight](evidence/g4-007/logs/runtime-version-preflight.log) | 1 | 0 | 0 | 通过 |
| [core-permission-preflight](evidence/g4-007/logs/core-permission-preflight.log) | 22 | 0 | 0 | 通过 |
| [sproto-permission-preflight](evidence/g4-007/logs/sproto-permission-preflight.log) | 3 | 0 | 0 | 通过 |
| [sproto-profile-preflight](evidence/g4-007/logs/sproto-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [native-loopback](evidence/g4-007/logs/native-loopback.log) | 66 | 0 | 0 | 通过 |
| [worker-trust-preflight](evidence/g4-007/logs/worker-trust-preflight.log) | 1 | 0 | 0 | 通过 |
| [worker-elf-versions](evidence/g4-007/logs/worker-elf-versions.log) | 3 | 0 | 0 | 通过 |
| [core-history-probe-prepare](evidence/g4-007/logs/core-history-probe-prepare.log) | — | — | — | 通过 |
| [core-history-probe](evidence/g4-007/logs/core-history-probe.log) | 5 | 0 | 0 | 通过 |
| [kernel-setattr-preflight](evidence/g4-007/logs/kernel-setattr-preflight.log) | 1 | 0 | 0 | 通过 |
| [workspace-all-features](evidence/g4-007/logs/workspace-all-features.log) | — | — | — | 失败/中断，见原件 |

### g4-007/fresh-001

候选 `ca86a85e2be9c83c7a59407fc0737114158ecf7f`；[summary](evidence/g4-007/fresh-001/summary.json)、[inventory](evidence/g4-007/fresh-001/inventory-001.json)。

| 命令 | 通过 | 失败 | 忽略 | 结果 |
|---|---:|---:|---:|---|
| [clean-private-product-packages](evidence/g4-007/fresh-001/logs/clean-private-product-packages.log) | — | — | — | 通过 |
| [runtime-lib-all-features](evidence/g4-007/fresh-001/logs/runtime-lib-all-features.log) | 144 | 0 | 1 | 通过 |
| [native-loopback-build](evidence/g4-007/fresh-001/logs/native-loopback-build.log) | — | — | — | 通过 |
| [native-loopback](evidence/g4-007/fresh-001/logs/native-loopback.log) | 66 | 0 | 0 | 通过 |
| [core-history-probe](evidence/g4-007/fresh-001/logs/core-history-probe.log) | 5 | 0 | 0 | 通过 |
| [workspace-all-features](evidence/g4-007/fresh-001/logs/workspace-all-features.log) | 964 | 1 | 41 | 失败/中断，见原件 |
| [core-fs-runtime-default](evidence/g4-007/fresh-001/logs/core-fs-runtime-default.log) | 791 | 1 | 36 | 失败/中断，见原件 |
| [worker-elf-versions](evidence/g4-007/fresh-001/logs/worker-elf-versions.log) | 3 | 0 | 0 | 通过 |
| [gui-gui_unit_tests](evidence/g4-007/fresh-001/logs/gui-gui_unit_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-cache_gui_tests](evidence/g4-007/fresh-001/logs/gui-cache_gui_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-conflict_gui_tests](evidence/g4-007/fresh-001/logs/gui-conflict_gui_tests.log) | 24 | 0 | 0 | 通过 |
| [gui-hidden_dialog_tests](evidence/g4-007/fresh-001/logs/gui-hidden_dialog_tests.log) | 3 | 0 | 0 | 通过 |
| [gui-lazy_gui_tests](evidence/g4-007/fresh-001/logs/gui-lazy_gui_tests.log) | 4 | 0 | 0 | 通过 |
| [gui-multi_mount_tests](evidence/g4-007/fresh-001/logs/gui-multi_mount_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-trust_gui_tests](evidence/g4-007/fresh-001/logs/gui-trust_gui_tests.log) | 6 | 0 | 0 | 通过 |
| [gui-transfer_gui_tests](evidence/g4-007/fresh-001/logs/gui-transfer_gui_tests.log) | 6 | 0 | 0 | 通过 |
| [gui-theme_tests](evidence/g4-007/fresh-001/logs/gui-theme_tests.log) | 1 | 0 | 0 | 通过 |
| [gui-selector_gui_tests](evidence/g4-007/fresh-001/logs/gui-selector_gui_tests.log) | 7 | 0 | 0 | 通过 |
| [gui-backend_upgrade_test](evidence/g4-007/fresh-001/logs/gui-backend_upgrade_test.log) | 11 | 0 | 0 | 通过 |
| [gui-pause_handover_test](evidence/g4-007/fresh-001/logs/gui-pause_handover_test.log) | 5 | 0 | 0 | 通过 |
| [gui-initialization_gui_tests](evidence/g4-007/fresh-001/logs/gui-initialization_gui_tests.log) | 10 | 0 | 0 | 通过 |
| [gui-unified_manager_tests](evidence/g4-007/fresh-001/logs/gui-unified_manager_tests.log) | 27 | 0 | 0 | 通过 |
| [gui-gui_instance_test](evidence/g4-007/fresh-001/logs/gui-gui_instance_test.log) | 3 | 0 | 0 | 通过 |
| [gui-queue_gui_tests](evidence/g4-007/fresh-001/logs/gui-queue_gui_tests.log) | 4 | 0 | 0 | 通过 |
| [gui-history_gui_tests](evidence/g4-007/fresh-001/logs/gui-history_gui_tests.log) | 17 | 0 | 0 | 通过 |
| [gui-permission_gui_tests](evidence/g4-007/fresh-001/logs/gui-permission_gui_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-permission-history-labels](evidence/g4-007/fresh-001/logs/gui-permission-history-labels.log) | 3 | 0 | 0 | 通过 |

### g4-009

候选 `4e6aaf7166fd370076adf43a0dad3e6ecdca36d6`；[summary](evidence/g4-009/summary.json)、[inventory](evidence/g4-009/inventory-001.json)。

| 命令 | 通过 | 失败 | 忽略 | 结果 |
|---|---:|---:|---:|---|
| [clean-private-product-packages](evidence/g4-009/logs/clean-private-product-packages.log) | — | — | — | 通过 |
| [native-read-stale-preflight](evidence/g4-009/logs/native-read-stale-preflight.log) | 1 | 0 | 0 | 通过 |
| [runtime-permission-preflight](evidence/g4-009/logs/runtime-permission-preflight.log) | 4 | 0 | 0 | 通过 |
| [runtime-profile-preflight](evidence/g4-009/logs/runtime-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [runtime-version-preflight](evidence/g4-009/logs/runtime-version-preflight.log) | 1 | 0 | 0 | 通过 |
| [core-permission-preflight](evidence/g4-009/logs/core-permission-preflight.log) | 23 | 0 | 0 | 通过 |
| [core-v6-migration-preflight](evidence/g4-009/logs/core-v6-migration-preflight.log) | 1 | 0 | 0 | 通过 |
| [kernel-parent-actions-preflight](evidence/g4-009/logs/kernel-parent-actions-preflight.log) | 1 | 0 | 0 | 通过 |
| [sproto-permission-preflight](evidence/g4-009/logs/sproto-permission-preflight.log) | 3 | 0 | 0 | 通过 |
| [sproto-profile-preflight](evidence/g4-009/logs/sproto-profile-preflight.log) | 1 | 0 | 0 | 通过 |
| [native-loopback](evidence/g4-009/logs/native-loopback.log) | 66 | 0 | 0 | 通过 |
| [worker-trust-preflight](evidence/g4-009/logs/worker-trust-preflight.log) | 1 | 0 | 0 | 通过 |
| [worker-elf-versions](evidence/g4-009/logs/worker-elf-versions.log) | 3 | 0 | 0 | 通过 |
| [core-history-probe-prepare](evidence/g4-009/logs/core-history-probe-prepare.log) | — | — | — | 通过 |
| [core-history-probe](evidence/g4-009/logs/core-history-probe.log) | 5 | 0 | 0 | 通过 |
| [kernel-setattr-preflight](evidence/g4-009/logs/kernel-setattr-preflight.log) | 1 | 0 | 0 | 通过 |
| [workspace-all-features](evidence/g4-009/logs/workspace-all-features.log) | 964 | 2 | 42 | 失败/中断，见原件 |
| [workspace-default](evidence/g4-009/logs/workspace-default.log) | 956 | 0 | 37 | 通过 |
| [vendored-fuser-wire](evidence/g4-009/logs/vendored-fuser-wire.log) | 55 | 0 | 0 | 通过 |
| [kernel-fs-debug-index1](evidence/g4-009/logs/kernel-fs-debug-index1.log) | 38 | 0 | 0 | 通过 |
| [kernel-fs-debug-index0](evidence/g4-009/logs/kernel-fs-debug-index0.log) | 36 | 0 | 0 | 通过 |
| [kernel-runtime-debug](evidence/g4-009/logs/kernel-runtime-debug.log) | 1 | 0 | 0 | 通过 |
| [kernel-fs-release-index1](evidence/g4-009/logs/kernel-fs-release-index1.log) | 38 | 0 | 0 | 通过 |
| kernel-fs-release-index0 | — | — | — | 未运行：协调暂缓 |
| kernel-runtime-release | — | — | — | 未运行：协调暂缓 |
| kernel-fs-release-attr0 | — | — | — | 未运行：协调暂缓 |
| [runtime-private-dbus](evidence/g4-009/logs/runtime-private-dbus.log) | 1 | 0 | 0 | 通过 |
| [gui-gui_unit_tests](evidence/g4-009/logs/gui-gui_unit_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-cache_gui_tests](evidence/g4-009/logs/gui-cache_gui_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-conflict_gui_tests](evidence/g4-009/logs/gui-conflict_gui_tests.log) | 24 | 0 | 0 | 通过 |
| [gui-hidden_dialog_tests](evidence/g4-009/logs/gui-hidden_dialog_tests.log) | 3 | 0 | 0 | 通过 |
| [gui-lazy_gui_tests](evidence/g4-009/logs/gui-lazy_gui_tests.log) | 4 | 0 | 0 | 通过 |
| [gui-multi_mount_tests](evidence/g4-009/logs/gui-multi_mount_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-trust_gui_tests](evidence/g4-009/logs/gui-trust_gui_tests.log) | 6 | 0 | 0 | 通过 |
| [gui-transfer_gui_tests](evidence/g4-009/logs/gui-transfer_gui_tests.log) | 6 | 0 | 0 | 通过 |
| [gui-theme_tests](evidence/g4-009/logs/gui-theme_tests.log) | 1 | 0 | 0 | 通过 |
| [gui-selector_gui_tests](evidence/g4-009/logs/gui-selector_gui_tests.log) | 7 | 0 | 0 | 通过 |
| [gui-backend_upgrade_test](evidence/g4-009/logs/gui-backend_upgrade_test.log) | 11 | 0 | 0 | 通过 |
| [gui-pause_handover_test](evidence/g4-009/logs/gui-pause_handover_test.log) | 5 | 0 | 0 | 通过 |
| [gui-initialization_gui_tests](evidence/g4-009/logs/gui-initialization_gui_tests.log) | 10 | 0 | 0 | 通过 |
| [gui-unified_manager_tests](evidence/g4-009/logs/gui-unified_manager_tests.log) | 27 | 0 | 0 | 通过 |
| [gui-gui_instance_test](evidence/g4-009/logs/gui-gui_instance_test.log) | 3 | 0 | 0 | 通过 |
| [gui-queue_gui_tests](evidence/g4-009/logs/gui-queue_gui_tests.log) | 4 | 0 | 0 | 通过 |
| [gui-history_gui_tests](evidence/g4-009/logs/gui-history_gui_tests.log) | 17 | 0 | 0 | 通过 |
| [gui-permission_gui_tests](evidence/g4-009/logs/gui-permission_gui_tests.log) | 5 | 0 | 0 | 通过 |
| [gui-permission-history-labels](evidence/g4-009/logs/gui-permission-history-labels.log) | 3 | 0 | 0 | 通过 |
| [provider-nemo](evidence/g4-009/logs/provider-nemo.log) | 10 | 0 | 0 | 通过 |
| [provider-nautilus](evidence/g4-009/logs/provider-nautilus.log) | 10 | 0 | 0 | 通过 |
| [unified-ui-render](evidence/g4-009/logs/unified-ui-render.log) | — | — | — | 通过 |

## 重试与双钟诊断

G4-009 原失败日志未给出精确 elapsed 或第二包类型，根因尚不能确定。[原 ELF 单跑](evidence/g4-009/clock-probe-001/results.json) 两项通过，797 个双钟样本最大间隔差约 11.139µs，不能抵消原失败。

[派生诊断](evidence/g4-009/clock-diagnostic-001/summary.json) 两项并发 2/0/0、完整 runtime 默认并行 144/0/1。原 900ms 断言保留，但前置插桩读锁/输出可能影响断言，因此仅看提前捕获的原断言点 elapsed：6 条最短 1.186085949s，全部第二包为 Download，attempts=1、retry_at=failed_at+2。本次未捕获早到请求，诊断通过不计候选验收。[测试差异](evidence/g4-009/clock-diagnostic-001/test-only-instrumentation.patch) 与[局限审计](evidence/g4-009/clock-diagnostic-001/diagnostic-audit.json)保留。

[1434 个采样点](evidence/g4-009/clock-diagnostic-001/clock-samples.json)的[异常区间](evidence/g4-009/clock-diagnostic-001/clock-outliers.json)：第1201点距首样本 mono 12.289246558s，realtime 相对 mono 后跳约22.386938317s；第1227点 12.552712513s 前跳约22.383726646s。重试 helper 的第二包此前已完成，不能用此次分歧确定原失败根因，也不能确定宿主暂停、校时或其他来源。

## 环境与清理

[环境失败记录](evidence/g4-010/environment-failure/summary.json)记录复制首个 Errno5、随后 Errno30，以及 WSL 启动 E_UNEXPECTED。[宿主空间原件](evidence/g4-010/environment-failure/windows-filesystem-drives.json)显示 C: 当时仅余54,185,984 bytes；此前 guest df 的可用空间不代表宿主空间。复制完整 stderr 未落盘且工具输出被截断，本包明确保留的是故障观察记录，未伪造完整 trace。

根协调者先核对 SHA 后删除5个可再生 Git tar，共215,050,240 bytes，见[archive 清理原件](environment/archive-cache-cleanup.json)；各 source.json/hash/重建命令保留。标准 WSL df 后可访问且根挂载恢复 rw，未执行 fsck/remount/服务重启。其后只删除从未用于构建或测试的010半成品 target，见[清理记录](environment/partial-target-cleanup.json)及[执行源码](environment/cleanup-partial-target.py)。010 source/logs 与009及更早原件保留。宿主空间仍不足，未重启验证。

## 覆盖边界与复现

[COVERAGE](COVERAGE.md)映射原生 pstream 回环66项、实际 worker ELF 版本握手3项、实际投影→UNIX IPC→Rpc→Bridge 回归。它们不是单一真实 worker 网络全链测试，更不是 NAS 实测。独立历史5项与标签模型3项验证 uncertain/cleanup_uncertain 不盲重发及完成后提示清除而 detail 保留。

INDEX0 两项必须有 NSI 的 continuation 用例明确 N/A；doc-ignore 与本轮未跑 core 210k 功能 ignored 单列，见 inventory。本次未声称所有 ignored 均已运行。未实测边界仍包括真实 NAS ACL、整空间能力来源、服务端部分提交真值、成对安装/重装及真实系统桌面面板。

命令见各 summary，[harness](harness/) 保留冻结 runner、入口与辅助源码；[PRIVATE_ORIGINALS](PRIVATE_ORIGINALS.json)记录排除的大产物位置与已保存 hash。原始日志、诊断、失败、截图不覆盖；本包没有 ELF、target、源代码大归档、数据库或真实 manifest 名称。[MANIFEST](MANIFEST.json)覆盖全部小文件。
