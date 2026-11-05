# G4-013 覆盖映射（T4 独立验证）

本文件按 012 冻结包的分组定义重新声明本轮**实际入口与边界**；设计与历史映射见
[G4-012 COVERAGE](../G4-evidence-bundle-012-final/COVERAGE.md) 与
[G4-010 设计映射](../G4-evidence-bundle-012-final/prior/G4-evidence-bundle-010-blocked/COVERAGE.md)，本包不继承其通过计数。

候选：`fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4`（`codex/fix-7`，worktree `work/fix-7`）。
源码快照：`git archive fefb8ee` → 私有 WSL 树；逐 blob/path/执行位与提交一致（2483 项，SHA 见 REPORT §快照）。
所有命令为 WSL 本机合成夹具，**不连接 NAS、不读取凭据、不 push/发布**。

| 组 | 实际入口（本包命令名） | 计数口径 | 边界 |
|---|---|---|---|
| workspace all-features | `workspace-all-features` | Rust harness 全部 `test result` 累加；按精确 suite/name 见 inventory | 首次在一个全新空 target 中离线编译（只种子 registry 依赖，见 REPORT §1）；重复组不累加为唯一测试数 |
| workspace default | `workspace-default` | 同上 | 同上 |
| 定向预检（权限/重试/版本/迁移） | `runtime-permission-preflight`、`runtime-profile-preflight`、`runtime-version-preflight`、`core-permission-preflight`、`core-v6-migration-preflight`、`sproto-permission-preflight`、`sproto-profile-preflight`、`retry-clock-preflight`、`obsolete-ticket-preflight`、`original-retry-pair-preflight`、`runtime-lib-original-parallelism`、`native-read-stale-preflight`、`worker-trust-preflight`、`core-definite_read_refusal_*`、`core-revoked_traversal_*`、`runtime-bridge_respects_read_denial_*`、`runtime-cancelled_flight_*`、`runtime-concurrent_read_revocation_*` | 各命令独立 Rust 计数 | 与本机 worker/Store 边界；不是真实 NAS 语义 |
| FUSE 内核（debug/release，INDEX1/INDEX0） | `kernel-fs-debug-index1`、`kernel-fs-debug-index0`、`kernel-runtime-debug`、`kernel-fs-release-index1`、`kernel-fs-release-index0`、`kernel-runtime-release` | 显式 `--ignored` 执行；`INDEX0` 用 `--skip` 跳过两个显式断言 NSI 启用的用例，记为不适用而非通过 | 需要真实 `/dev/fuse`、普通用户；不含 G5 墙钟性能 |
| attr TTL=0 | `kernel-fs-release-attr0`（release + `DRIVE_NATIVE_FS_TTL_MS=0`） | 显式 `--ignored` | 只覆盖内核用例组的 TTL 边界 |
| GUI 全套 | `gui-*` 18 个模块、`gui-permission-history-labels`、`provider-nemo`、`provider-nautilus`、`unified-ui-render` | Python `Ran N tests` / fail / skip | offscreen + 私有 D-Bus；不是真实桌面会话 |
| native loopback | `native-loopback`（冻结 012 期望，66 例）与 `supplement-001/native-loopback-updated`（校正期望，70 例） | 66 = 012 矩阵；70 = 66 + 4 个对抗新增 | 本机 127.0.0.1 合成 pstream + 公开 `sproto::Client`；不执行 worker ELF、不证明真实服务端 |
| native-worker-chain 真实 worker TLS 链 | `supplement-001/tls-harness-build` + `supplement-001/tls-actual-worker-all-ten` | 10 例：6 例 worker→IPC→Bridge 链 + 4 例直接 Worker RPC（不相加称完整 Bridge 链） | 合成 TLS 服务 + 真实 `drive-native-worker` ELF；不证明真实 NAS 错误码来源 |
| 210k 容量（ignored） | `supplement-001/capacity-ignored-list` → `supplement-001/capacity-210k` | 1 个显式 ignored 功能用例 | 功能容量验收，不是性能基准；不替代 G5 |
| doc/package 回归 | `offline-documentation-staging`（离线文档暂存测试）、`supplement-001/package-documentation-staging`（deb 构建用 `scripts/stage-docs.py` 预检）、`supplement-001/package-version`（`dpkg-parsechangelog`）、`supplement-001/package-metadata-equivalence`（打包输入相对产品冻结 9c28992 无差异 + 旧冻结 deb 交付未被改动） | 见 inventory | **新候选 deb 未构建**：全量 `dpkg-buildpackage`、解包 payload、安装/升级属打包任务（task-5），本包只给前置回归 |
| 组件策略替换 | `supplement-001/old-component-build`、`new-component-build`、`private-component-policy-handover` | 私有目录真实 fsd/worker 配对替换；旧侧为 41365a6 的保留构建（fixture），新侧由本快照在本 target 构建 | 不是 dpkg 安装/升级/重装；旧侧是旧候选夹具 |
| T1 write-learning 定向测试 | `supplement-001/t1-sproto-error-tests` 等 8 条命令 | 单独列出，见 REPORT §T1 与 inventory | 只证明隔离夹具/纯函数；真实 `0x9003` 响应与 FUSE 端到端未实测 |
| 完整性/环境 | 快照前后 hash（`snapshot-verification-before/after`）、`source-hashes-before` + 结束 `changed_source_files`、ELF witness、`residual_owned_mounts`、宿主空间门禁 | 见 `environment/` 与 `evidence/g4-013/` | 宿主 C 门禁 40 GiB 起测 / 每命令 10 GiB；guest `df` 不替代 |

## 与 012 的口径差异（必须显式记录）

1. **冻结 012 native-loopback harness 的期望不适用于 fefb8ee 的预期行为变化。** 冻结 harness 断言
   `upload 0x9003` 与 `mkdir 0x9003` 为 `mutation_permission_uncertain`；T1 后 sproto 在直接 upload
   回复边界把 0x9003 分类为 `MutationPermissionDenied`。冻结期望运行保留为原件（64/2），校正期望运行
   （只改这两处期望 + 新增 4 个对抗用例）为 70/0。详见 REPORT §native loopback 与
   `environment/native-loopback-expectation-delta.json`。
2. 主矩阵因一个**验证者自身**的 harness 路径错误在尝试 1 中止（helper 脚本不在 runner 同目录），
   该尝试原样保留在 `environment/attempt1-harness-path-failure/`，**不记为产品失败**；尝试 2 使用
   修正后的 runner 路径重跑，并只从验证者自己的尝试 1 target 按 Cargo.lock 复制 registry 依赖种。
3. 包回归只做前置静态/文档检查，**不把未构建的新 deb 说成通过**。

## 未实测边界（不进入任何通过计数）

- 真实 NAS：真实 ACL/共享高级权限、真实 `0x9003` 响应可达性、服务端提交真值、profile 写侧来源、真实桌面面板。
- FUSE 写路径的学到 deny → EACCES 端到端；真实 worker upload IPC 链（`0x9003` 从合成 TLS 服务穿过真实 worker 到 Bridge）。
- 新候选 deb 构建/解包/安装/升级/重装；G5 性能；其他 Drive Server 版本；SPK 未解包组件（web API/后台）。
