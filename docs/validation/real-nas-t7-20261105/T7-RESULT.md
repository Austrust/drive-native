# T7-RESULT：真实 NAS 受限端到端结果（Issue #7 外部验收）

- 执行时间：2026-11-05 15:35–17:06（Asia/Shanghai）
- 产品候选：`drive-native 0.1.0~dev20261105.2`（deb SHA256 `044a8e78…34fd9b`，54,160,240 B；已安装到 WSL Ubuntu 24.04）
- 结论：**受限 T7 部分完成；Issue #7 真实 NAS 完整验收未通过（保持开放）**。子目录 scope 的挂载/写路径已在真实 NAS 上通过；**根 scope 的 legacy seed 初始化仍阻塞**，因此 root 拒建、同挂载 root/A/B、B 只读、多数 11 场景仍 blocked。没有任何场景被记成"通过"来掩盖阻塞。

## 1. 环境与安装

- 安装：`dpkg -i` 首次缺 `gvfs-backends`、`libxcb-xinerama0`，按 RUNBOOK 记录后用 `apt-get install -f` 解决并重装成功；未跳过依赖。
- 版本：`0.1.0~dev20261105.2`。二进制 SHA：fsd `ba15f6ba…`、worker `a38f97b1…`、setup `889e5b2d…`、GUI wrapper `037b1847…`。
- `compat_check`：pass=16 / warn=1 / fail=0。唯一 warn：`setup_cli_markers`（二进制内未搜到 "login"/"list" 字符串；`--private-stdin` quit handshake 与 fsd `--config` 参数路径实测通过）。
- File Station 登录形态（现场验证）：`GET /webapi/auth.cgi` + `SYNO.API.Auth v6` + `session=FileStation` + `format=sid`；查询 `GET /webapi/entry.cgi`；TLS 强制校验；TCP 连 IP、SNI/Host=证书名。
- 凭据：仅由脚本从 Lead 提供的临时 0600/DrvFs 文件读入内存；未复制、未打印、未进证据/argv/日志；执行后文件由 Lead 删除。

## 2. 授权范围与实际写操作（完整清单）

授权：`write_isolated`（仅 A=个人空间测试目录）；ROOT_MF 只允许预期失败尝试；B_STOP 只读；`full` 仅 offline（本轮未执行）；revoke/L2 管理员实验不做。

**实际写操作只有以下（全部在 A 内/删除 A）：**
1. 在已批准的个人空间小目录（M_DIAG parent scope）内 `mkdir` 创建 A：`P032-acceptance-<YYYYMMDD-HHMMSS>`（A 名）。
2. S02：A 内 mkdir 一个测试目录（e2e token，自动清理）。
3. S12：A 内 create/write/truncate/rename/delete 各测试 token（e2e 自动清理）。
4. 清理：A 内已被 e2e 清空后 `rmdir` A；远端删除以 File Station list(parent)+list(A) 复核（见 §5）。

未写 ROOT_MF（根 scope 挂载失败，未做任何尝试）、未写 B_STOP/EXP/其他共享、未删除/改名任何既有文件、未做 File Station 写 API。

## 3. File Station 只读真值（基线，5/5 角色）

| 角色 | append | write | read | exec | del | posix | is_acl_mode | 用途 |
|---|---|---|---|---|---|---|---|---|
| ROOT_EXP（EXP 共享根（匿名）） | false | false | true | true | true | 555 | true | 原主 ROOT；**非 Drive 可见** → blocked |
| A_PARENT（原 EXP 777 子目录） | true | true | true | true | true | 777 | true | 原 A 父目录；EXP 不可见 → 改为个人空间 |
| B_STOP（B_STOP 只读共享根） | false | false | true | true | true | 555 | true | B 只读候选；**非 Drive 可见** → blocked |
| ROOT_MF（ROOT_MF 团队协作共享根） | false | false | true | true | true | 555 | true | 根拒建场景（挂载失败） |
| HOME_ROOT（个人空间根） | true | true | true | true | true | 777 | true | 个人空间 |

fingerprint：`d6850f29…`（完整值在证据）。注意：`acl.del=true` 与 posix 555/755 不一致（File Station del 语义不等价于 delete-child），未据此断言任何删除权限。

## 4. Drive 空间可见性（产品 selector，非挂载）

可见 5 个空间：个人空间（personal，writable_hint=true）、`ROOT_MF`（团队协作共享，writable_hint=false）、以及 3 个其它团队共享（不在本轮批准范围，匿名）。**`EXP` 共享与 `B_STOP` 共享不在 Drive 空间列表** → 原 ROOT_EXP/A_PARENT/B_STOP 挂载不可行，按 Lead 规则记 blocked/改个人空间，不强行映射。selector `list("/")`：团队根成功 0.03s、root_scope 完整；个人根成功、root_scope 完整（认证与 native list 正常）。

## 5. 挂载与写路径结果

| 观测 | 结果 | 证据 |
|---|---|---|
| ROOT_MF 根（555）300s 超时 | **失败**：FUSE 未就绪；`initial_index` 反复 `native_transport`(6)/`native_response_timeout`(2)，attempt 4.5–56s；nodes=0 | `evidence/mount/failure-readonly-health.json` |
| HOME_ROOT 根 长超时 | **仅 preview 挂载**：555.6s 后 preview_only=true、phase=enumerating；随后 `native_server_rejected` code `8193`(0x2001)、recovery=needs_attention；不是完整初始化 | `evidence/mount/root-scope-HOME-preview.json`、`root-scope-long-attempts.json` |
| 个人空间小 scope（预览） | preview 挂载 1.1s（preview_only=true, phase=catching_up） | `evidence/mount/small-scope-preview*.json` |
| 个人空间小 scope（RW 完整初始化） | **成功**：90.7s，preview_only=false，phase catching_up→enumerating→importing；FUSE rw | `evidence/mount/parent-scope-full-init.json` |
| A 创建（approved mkdir） | **成功**：FUSE mkdir 成功；File Station 真值 A：append/write/read/exec/del=true，posix 777，ACL=true | `evidence/experiments/create-A-local.json`、`truth/a-created.json` |
| M_A 独立挂载（scope=A） | **成功**：1.2s，preview_only=false，rw=True | `evidence/mount/start-M_A.json` |
| S02 单独挂载 A | **pass**：A 内 mkdir 异步终态 committed，File Station 真值确认 | `evidence/e2e/matrix-M_A.json` |
| S12 A 写矩阵（补充场景，非 issue 11） | **pass**：create/write/truncate/rename/delete 6/6 committed，server truth 逐项确认；本地数据保留、无错误分类异常 | 同上 |
| GUI 四态（M_A） | 无可观测态（A 全允许、无 deny）；四态真实截图不可得 | `evidence/gui/gui-states.json` |

结论：**非根 scope 的 watermark/S0 初始化路径可用（子目录 scope 完整挂载+RW 写通过）；根 scope 的 legacy seed/recovery snapshot 初始化阻塞**。二者行为差异与 Lead 协议探针一致（长初始化序列在真实 NAS 中断连接；本工具看到 `native_response_timeout`/`native_transport`，长重试最终 `8193`）。

## 6. 清理确认

- A 已删除：parent listing 中 A 数量 **0**；`list(A)` 返回 `file_not_found`（408）；**getinfo(A) 曾返回 stale_ok**（File Station 删除后 getinfo 缓存滞后，仅记录、不作为存在依据）。
- FUSE `rmdir` 是异步 local-first：本地删除后若立刻停止父挂载，远端提交有延迟；本轮远端删除在父挂载停止后最终可见，并在最终证据中复核。
- 无残留：无 `drive-native-fsd`/worker/relay/私有 dbus-daemon 进程；所有 mount point 均非挂载且为空目录；`local/mount-state.json` 已清理。
- 未改动临时凭据文件；未安装别的包；未修改产品源码/冻结交付。

## 7. 逐场景状态（Issue #7 11 场景 + 补充）

| 场景 | 状态 | 原因 |
|---|---|---|
| S01 root 拒建 + A 可建 + B 只读同挂载 | **blocked** | 根 scope 不可挂载；EXP 非 Drive 可见、B_STOP 非 Drive 可见；无同挂载只读 B |
| S02 单独挂载 A | **pass** | M_A 完整初始化 + mkdir committed + File Station 真值 |
| S03 只建目录不建文件 | **blocked** | 无 C 夹具（用户不加夹具） |
| S04 可写不可删/改名 | **blocked** | 无 D 夹具 |
| S05 ACL 继承/allow+deny 组合 | **blocked** | 无 G 夹具；无管理员 ACL 操作 |
| S06 仅遍历不可列 | **blocked** | 无 E/E_CHILD 夹具 |
| S07 A→B/覆盖/跨目录 | **blocked** | A→B 目标 B 不可达/非 Drive 可见；A 内 rename 已由 S12 覆盖一部分，但不等于 S07 |
| S08 撤销恢复/离线重连 | **blocked** | revoke 需 DSM 管理员；offline relay 本轮未执行（full 仅授权 offline，但未进入该子阶段） |
| S09 显式只读/空间禁止 | **blocked** | 只读根 scope 不可挂载；space_forbidden 生产未接线 |
| S10 拒绝不阻塞无关操作 | **blocked** | 依赖 root 拒建挂载 |
| S11 GUI 四态 | **blocked** | 四态真实不可得（无只读/拒绝态）；服务禁止不可观测 |
| S12（补充）A 写矩阵 | **pass** | create/write/truncate/rename/delete 6/6 committed + 真值确认 |

统计：Issue 11 场景 = **pass 1（S02）/ blocked 10**；补充 S12 = pass；无 fail/observed/delegated 记成通过。

## 8. 缺口 2 / 缺口 3

**缺口 2（精确写拒绝学习）**：本轮 **未观测到真实拒绝响应**。ROOT_MF/B_STOP 根不可挂载，无法做真实 root 拒建/B 只读的 FUSE create/mkdir 尝试；A 是正向路径。因此 T1 学习子集（upload `0x9003`→write_content denied）的真实可达性、history 是否落 `permission_denied`、fail-closed 行为**均为未观测**，不能写"已验证"。`0x9004/0xa004` 歧义、rename 码不落 history 的现象在真实 NAS 上也未复现。唯一相关信号是根 scope 初始化阶段的 `native_server_rejected code 8193`，与写权限无关。
**缺口 3（独立 L2 来源）**：仅完成自然观测（EXP/B_STOP 非 Drive 可见；ROOT_MF 根 555 而一级子目录 777 的 File Station 真值）；共享级只读/任务只读需管理员改 DSM 权限，按无管理员凭据记为 **blocked**，未伪造结论。

## 9. 产品异常/缺陷观察（不改产品代码，供 T1/T4/T5）

1. **根 scope legacy seed 初始化阻塞**：`initial_index` 反复 `native_transport`/`native_response_timeout`，无节点导入；长重试后 `native_server_rejected` code `8193`（0x2001），recovery=needs_attention。selector native list 同 scope 0.03s 正常 → 问题集中在 recovery snapshot/初始化序列，非认证/TLS/端口。
2. **preview-only 挂载语义**：`overview.mounted=true` 但 `preview_only=true` 时 FUSE 已可挂载/列出预览内容，但初始化未完成、writable=false。工具已修正：验收挂载必须 `preview_only=false` 才判定 ready（否则会把 preview 误记成成功）。
3. **File Station getinfo 删除后缓存滞后**：删除 A 后 parent listing=0、list(A)=file_not_found，但 getinfo(A) 仍 stale_ok。清理判定改用 parent listing + list(A)，并记录该不一致。
4. **FUSE 命名空间删除异步**：本地 rmdir 后立即停止挂载时远端提交有延迟；最终已提交。建议 T4 关注 namespace 队列在停止时的 flush 语义。
5. 初次安装依赖缺失（gvfs-backends/libxcb-xinerama0）已按规程解决，非产品缺陷。

## 10. 证据与复现

- 脱敏证据包：`work/issue7-acceptance/evidence/t7-limited-20261105/`
  - `REPORT.md`（自动骨架）、`MANIFEST.json`（SHA256 `372e1c82…f53e`，23 文件；`SHA256SUMS` 在证据包根目录可 23/23 校验通过）、`SHA256SUMS`、`EXCLUDED.txt`
  - 含：mount 成功/失败证据、A 创建/清理证据、truth 基线/A/清理后真值、S02+S12 矩阵、GUI states、blocked 场景登记、preflight。
  - 打包扫描：22 个证据文件，0 敏感模式命中；无 host/账号/共享名/真实路径/文件名/凭据。
- 本地原始日志/状态（敏感，不进 workspace/证据）：`<WSL 本地非同步 run 目录>/{local,evidence}`；更早失败 run 保留在同一本地父目录。
- 关键命令：`dpkg -i`（含依赖修复）→ `compat_check` → `mount_ctl spaces --confirm-drive-login` → `truth_collect` → `mount_ctl start/stop`（M_DIAG/M_A/M_PARENT）→ `e2e_matrix --scenarios S02,S12` → `gui_states` → `evidence_pack`。

## 11. 未实测 / 下一步建议（下轮）

- 根 scope legacy seed：需产品侧修复/调查（`initial_index` recovery snapshot 的分页/重试/断点续拉；Lead 探针显示 cursor=2000 时服务端断 TLS）；修复后再做 S01/S07/S09/S10 与真实 root/B 拒绝尝试。
- 需要 Drive 可见的只读 B 或管理员 ACL 夹具；否则 S01/S04/S05/S06/S07/S08 保持 blocked。
- 缺口 2：需要真实可挂载的 root/B scope 才能做真实拒绝响应；或改用**独立原生协议探针**（本次未交付）在真实 NAS 直接发 create/upload/remove/rename 并记录响应码/阶段（需另行授权构建 test-only 工具）。
- 缺口 3：管理员改共享只读/profile 任务只读实验（当前无管理员凭据）。
- GUI 四态：需要只读挂载/拒绝观测/注入构建三类条件；本轮全不可观测。
- full/offline：本轮未执行；如需可在修复后的 A scope 上用回环 relay 做离线重连。

## 12. 脱敏声明

本报告与证据包已移除服务器地址、账号、共享名、真实路径、文件名与凭据；仅保留匿名角色 id、权限位、计数、错误码、时序与布尔结果。`local/` 目录（profile、名称映射、原始 fsd 日志、state DB）不在 workspace，也不在证据包内。
