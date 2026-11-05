# T10 真实 NAS 受限执行结果（2026-11-05）

> 结论先行：候选 `.3` 安装后，**ROOT_MF 根 scope 首次完整挂载成功**（`preview_only=false`、rw、list/stat 落盘、nodes=91,081），T7 的根 `initial_index` 阻塞未复现；**S01 根 deny 尝试完成非提交验证**（3/3，真实码 `0x9004`，fail-closed、本地保留），但同挂载 B 行仍不可用；**S02 + 补充 S12 通过**（S12 六步 6/6 committed）；A 已清理并经 File Station 父目录列表核验不存在；其余 10 个正式场景仍 `blocked`。Issue #7 的完整真实 ACL 验收**仍未关闭**。

## 1. 范围与授权

- 执行角色：验收执行者（T3/T6/T7/T10），在 Lead 的 T10 明确授权下执行；只使用本项目验收目录和本地私有运行目录。
- 授权写边界：仅 A（个人空间下的 `P032-acceptance-<YYYYMMDD-HHMMSS>` 测试目录）；ROOT_MF 根只做预期的 mkdir/create deny 尝试；B 只读；其它真实数据目录不写、不删、不改名。
- 本窗口**未做**：管理员权限实验（revoke/restore、共享级 L2）、offline/full relay、B 写/删/改名、同挂载 root/A/B 行、HOME 根正式判定。全部按 blocked 记录，没有以其它路径“替代”完成。
- 产品安装状态：候选 `0.1.0~dev20261105.3` 保持安装，未卸载；不修改产品代码。

## 2. 候选安装与基线

| 项目 | 结果 |
|---|---|
| 包版本 | `0.1.0~dev20261105.3`（由 `.2` 升级） |
| 候选 deb SHA256 | `c7b8a5ddd046ea7a1a40756bf19cf5e4f27bcd8e2174005c1b526856d639fb83` |
| `dpkg --verify drive-native` | exit 0，输出 0 行 |
| fsd SHA256 | `12f7a8687507a6111f9a9b9fd762e83575e37c9f9f422d0bfdf99bd997c438f6` |
| worker SHA256 | `e6ad3bfb11699b36d4789105a2427c39a668375182d2bf4b62516b3ebb24656f` |
| setup SHA256 | `afc48e47c306f0554b67d60b219407ad1fa4abd80c8d1e121f4c00f7fbe0468e` |
| GUI wrapper SHA256 | `037b1847de0335cc8411358918474d75c8583c113205d90ebdc9c4d087a09586` |
| dpkg 日志 | `19:09:09` upgrade `.2 → .3`；`19:09:11` unpacked/configured `.3` |

安装发生在本轮工具调用被用户中断之前；`installation-record.json` 是中断后补记的事实，未伪造时间线。证据：[install/installation-record.json](evidence/t10-20261105/install/installation-record.json)。

## 3. ROOT_MF 根 scope 挂载

| 尝试 | ready_seconds | preview_only | writable | 结果 |
|---|---:|---|---|---|
| #1 | 206.233 | false | true | 成功；用于首个 probe；工具调用随后被中断 |
| #2 | 121.049 | false | true | 成功；仅因 S01 harness 参数错误重跑 |
| #3 | 124.670 | false | true | 成功；仅因证据文件覆盖冲突重跑 |
| #4 | 171.954 | false | true | 最终成功；S01 deny 实验使用该次挂载 |

- 初始化 phases 均包含 `enumerating → catching_up → importing`；`scope_complete=true`、`scope_depth=0`、`local_policy=follow_remote`、`writable_source=user`。
- 最终 root state DB：`nodes=91,081`、`metadata=9`、`namespace_ops=0`、`sync_history=0`（[mount/root-db-counts.json](evidence/t10-20261105/mount/root-db-counts.json)）。
- FUSE probe（mount #1）：`list_ok=true`、`stat_ok=true`、`entries=6`、`dirs=6`、`files=0`（[mount/probe-M_ROOT_MF.json](evidence/t10-20261105/mount/probe-M_ROOT_MF.json)）。
- 对照 T7：T7 在 `.2` 上 ROOT_MF 根 300s 失败于 `initial_index`（`native_transport`/`native_response_timeout`，nodes=0）；T10 `.3` 未复现。根 scope 挂载成功是 T9 修复后的首个正式证据。
- 尝试序列证据：[summary/root-mount-attempts.json](evidence/t10-20261105/summary/root-mount-attempts.json)；最终 start：[mount/start-M_ROOT_MF.json](evidence/t10-20261105/mount/start-M_ROOT_MF.json)。

## 4. S01 根拒绝与缺口 2

### 4.1 实际动作与结果

在 mount #4 的 ROOT_MF 根执行 3 个预期失败尝试（不写真实数据）：

| 尝试 | 操作 | 同步结果 | 异步终态 | 真实响应码 | 服务端真值 |
|---|---|---|---|---|---|
| D.1 | mkdir | 本地先成功（`sync_ok=true`） | `refused_remote_uncertain` / `permission_commit_uncertain` | `0x9004` | `found=false`，未变化 |
| D.2 | create_file | 本地先成功 | `refused_remote_uncertain` / `permission_commit_uncertain` | `0x9004` | `found=false`，未变化 |
| D.3 | create_file | 本地先成功 | `refused_remote_uncertain` / `permission_commit_uncertain` | `0x9004` | `found=false`，未变化 |

共同字段：`code_certainty=ambiguous`、`code_class=acl_precommit_create_or_profile_filter`、`local_retained=true`、`unsafe_change=false`、`content_changed=null`。S01 truth 前后均为 ROOT_MF `append=false/write=false`、POSIX 555，fingerprint `77fc4c0d…` 一致；根**无实际变更**。证据：[experiments/denials.json](evidence/t10-20261105/experiments/denials.json)、[truth/t10-s01-pre.json](evidence/t10-20261105/truth/t10-s01-pre.json)、[truth/t10-s01-post2.json](evidence/t10-20261105/truth/t10-s01-post2.json)。

### 4.2 缺口 2 结论

- `non_commit_verified=true`、`fail_closed=true`：拒绝最终没有提交到服务端真值，本地数据保留。
- **没有唯一 ACL pre-commit 码**：`0x9004` 与本地 profile read-only filter 共享，因此仅凭码不能区分 ACL 拒绝与 profile 过滤。
- `learnable_candidates=[]`：本窗口未得到可用于 T1 自动学习的唯一映射。
- T1 期望的 upload `0x9003` 在本次 `mkdir/create_file` 子集**未观测**；这只能说明本子集未覆盖 upload，不能证明 upload 路径不可达。
- rename 的拒绝码**未进入 history**（本窗口也未专门执行 rename 拒绝）；保持“码未暴露”，不猜测。
- 完整汇总：[summary/gap2-gap3-summary.json](evidence/t10-20261105/summary/gap2-gap3-summary.json)。

## 5. S02、补充 S12 与 A 生命周期

### 5.1 A 创建

- 父目录挂载 M_PARENT：ready **104.604s**；FUSE `mkdir` A 返回 0。
- 但立即 `[ -d A ]` 为 false（FUSE namespace 更新异步）；后续 File Station truth 确认 A 已存在，故以服务端真值为准。`create-A-local.json` 如实记录 `local_created=true, local_exists_after=false`。
- truth `t10-a-created`：A 与 A_PARENT 均 `status=ok`、POSIX 777、append/write/read/exec/del/acl=true，fingerprint `366e3171…`。证据：[truth/t10-a-created.json](evidence/t10-20261105/truth/t10-a-created.json)。
- A 的 truth pre 尝试遭遇 File Station timeout，未生成文件；该 limitation 已保留，未补造。

### 5.2 独立挂载与矩阵

- 第一次 M_A 启动失败：`a mount is already recorded for this run; stop it before starting another`；父挂载已 stop 后，M_A 独立启动成功，ready **1.7s**。
- `e2e_matrix.py S02,S12` exit 0：**S02 pass**；补充 **S12 六步（create/write/truncate/rename/create/delete）全部 committed pass**；matrix summary `pass=2/fail=0/blocked=0`，7 steps。
- `gui_states.py` exit 0：四态均不可观测（`observable=none`），`user_read_only`/`service_forbidden`/`directory_denied`/`unknown` 均记 not_observable；不把 GUI 未观测当作通过。
- 证据：[e2e/matrix-M_A.json](evidence/t10-20261105/e2e/matrix-M_A.json)、[gui/gui-states.json](evidence/t10-20261105/gui/gui-states.json)、[mount/start-M_A.json](evidence/t10-20261105/mount/start-M_A.json)。

### 5.3 A 清理

- cleanup-pre truth 再次 timeout；M_PARENT(cleanup) 挂载 ready **139.255s**，但 fresh parent mount 本地看不到 A，未执行本地删除。
- 只读 File Station list 确认 A 为空（count=0）；因 A 不在本地父挂载命名空间，使用预先授权的 File Station Delete fallback 删除 A。
- fallback 的自身 `after` getinfo 仍返回 `exists=true`，因此 `ok=false`、exit 4；这是 **File Station 删除后 getinfo 缓存滞后**。随后 truth `t10-a-cleanup-post2` 显示 A `perm_missing`、A_PARENT ok，fingerprint `94449ab9…`；再以只读父目录列表连续 3 次核验，**A 均不在列表中**，getinfo 仍缓存为 true。
- 因此 A 清理结论：**目标已从父目录列表/truth 中消失**；不以 stale getinfo 作为失败或成功依据。证据：[experiments/cleanup-A-local.json](evidence/t10-20261105/experiments/cleanup-A-local.json)、[experiments/cleanup-verification.json](evidence/t10-20261105/experiments/cleanup-verification.json)、[experiments/approved-fs-action-delete.json](evidence/t10-20261105/experiments/approved-fs-action-delete.json)、[truth/t10-a-cleanup-post2.json](evidence/t10-20261105/truth/t10-a-cleanup-post2.json)。

## 6. blocked 清单

正式场景 S01–S11 中：**S02 pass**；**S01 blocked（root deny 部分证据）**；**S03–S11 共 9 个 blocked**。补充 S12 pass。完整登记：[summary/blocked-scenarios.json](evidence/t10-20261105/summary/blocked-scenarios.json)。

| 场景 | 状态 | 原因摘要 |
|---|---|---|
| S01 | blocked | root deny 已执行并验证非提交；同挂载 B 行不可用，无法完成 S01 全断言 |
| S02 | pass | A 独立挂载 + 写矩阵通过 |
| S03 | blocked | 缺少“可建目录、不可建文件”的夹具角色 |
| S04 | blocked | 缺少“可写文件、不可删/改名”的夹具角色 |
| S05 | blocked | 缺少 ACL 继承中断/显式 allow-deny 夹具，且不允许改生产 ACL |
| S06 | blocked | 缺少 traverse-only 父目录夹具 |
| S07 | blocked | B 只读且不可挂载，A→B/覆盖重命名/跨目录移动不可执行 |
| S08 | blocked | revoke/restore 需管理员；offline/full 未授权 |
| S09 | blocked | 服务级禁止/用户显式只读需管理员或第二受控账号 |
| S10 | blocked | 无安全可用的“队列中被拒绝的无关写”目标；未断言隔离 |
| S11 | blocked | GUI 四态不可观测（observable=none） |

HOME 根正式判定跳过（residual，不阻塞 team 根结论）；B 的 mkdir/create deny 与同挂载 root/A/B 依赖的拓扑未提供，未用其它目录替代。

## 7. 缺口 3

- 状态：`blocked_not_observed`。
- 原因：revoke/restore 与共享级只读/L2 实验需要管理员凭据或管理员侧配置；无管理员凭据，T10 未尝试，不做替代实验。
- 因此不能对“独立 L2 来源是否成立”给出结论；只能引用 G4 注入证据的既有边界。

## 8. 产品观察（不改代码）

1. **FUSE 新建目录本地即时不可见**：mkdir A 返回 0，但紧随其后的本地 `[ -d ]` 为 false；File Station 已能确认 A。属于本地命名空间/异步更新语义，需产品确认是否预期。
2. **fresh parent mount 不暴露已由服务端确认的 A**：A 创建成功后重新挂载父目录，本地仍看不到 A；导致无法在本地路径完成清理。
3. **File Station getinfo 删除后 stale**：Delete API 返回 `success=true`，但紧接的 getinfo 仍 `exists=true`；父目录列表/truth 已证明删除。与 T7 的 getinfo stale 现象一致；`approved_fs_action` 因此返回 `ok=false`/exit 4，证据保留该矛盾，未把它写成失败删除或成功。
4. **`0x9004` 歧义**：S01 的三个拒绝均只得到 `0x9004`，与 profile read-only filter 共享，无法作为唯一 ACL 拒绝码；这直接影响缺口 2/T1 学习与 fail-closed 判定粒度。

以上均未修改产品源码；报告 Lead/产品侧评估。

## 9. 清理与当前环境

- 所有产品挂载已 stop；无 fsd/worker/mount_ctl/私有 dbus 进程；无活动残留挂载点或 mount-state（mount1 harvest 留下的本地 stale 状态已移到 `run/local/aborted/`，非活动挂载；清理记录见证据中的 mount stop 与 operations 目录）。
- A 已删除（父目录列表 absent + truth perm_missing）；getinfo 仍缓存 true 属已知异常。
- 候选 `.3` 保持安装，未 uninstall；本窗口未再升级/降级。
- 未对 ROOT_MF 根或其它真实数据目录产生写入；S01 仅执行拒绝尝试并验证无提交。
- File Station 在本窗口出现数次 timeout（truth pre/A pre/cleanup-pre），均已如实记为 limitation；未把超时当“已通过”。

## 10. 证据与限制

- 脱敏证据包：`work/issue7-acceptance/evidence/t10-20261105/`（40 项 `SHA256SUMS` 全部校验通过；`MANIFEST.json` SHA256 `2010638472b7be478bdcb0b8add43005bf26a0799cce925e8480fa6bba1dd252`）。
- 运行目录的 `local/` 从未打包；证据只含 role id、计数、布尔、错误码、SHA、时长，无真实主机/账号/共享/路径/文件名。
- 关键证据入口：[installation-record.json](evidence/t10-20261105/install/installation-record.json)、[denials.json](evidence/t10-20261105/experiments/denials.json)、[matrix-M_A.json](evidence/t10-20261105/e2e/matrix-M_A.json)、[cleanup-A-local.json](evidence/t10-20261105/experiments/cleanup-A-local.json)、[blocked-scenarios.json](evidence/t10-20261105/summary/blocked-scenarios.json)、[gap2-gap3-summary.json](evidence/t10-20261105/summary/gap2-gap3-summary.json)。
- 未证明/未执行：S01 同挂载 B 行、S03–S11、gap3；HOME 正式判定；full/offline；revoke/L2；GUI 四态。Issue #7 完整真实验收仍开放。

## 11. 下一步（进入 Issue #7 完整验收所需）

1. 提供同挂载 root/A/B（或等价受控拓扑）与 B 的 Drive 可见性，完成 S01 全断言及 B 只读/deny 行。
2. 提供 C/D/E/G 夹具角色，或授权受控 ACL 配置窗口，完成 S03–S06。
3. 授权管理员侧的 revoke/restore 与共享级 L2 实验（S08/S09/gap3）；授权 offline/full relay。
4. 产品侧确认 FUSE 新目录可见性、fresh mount 可见性与 File Station getinfo stale 的预期行为；评估 `0x9004` 去歧义方案。
