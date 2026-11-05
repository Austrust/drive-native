# T7 受限真实 NAS 验收报告复制核验记录

本文件记录把 T3 的受限真实 NAS 验收报告复制进本仓库的逐字节核验；与报告内 T7 结论分属不同层次：本文件是**作者侧的复制/脱敏/入库核验**，不改变 T3 的验收结论。

- 来源（仓库外）：`work/issue7-acceptance/T7-RESULT.md`
- 目标（仓库内）：`docs/validation/real-nas-t7-20261105/T7-RESULT.md`
- 复制方式：逐字节复制（PowerShell `Copy-Item`，未做行尾/编码改写）
- 操作时间：2026-11-05（Asia/Shanghai）

## 核验结果

| 检查 | 结果 |
|---|---|
| 源文件大小 / 行数 | `12,762` bytes / `118` lines |
| 源文件 SHA256 | `1393fa9dec551791da67b10cec055fcd614a95d0f069349b106fae0df569e3bf` |
| 目标文件 SHA256 | 同源值（逐字节一致） |
| 敏感信息扫描（T7-RESULT.md） | 0 命中：无 IP/主机名/TLS 名、无账号/邮箱、无 `/mnt/c/` 或真实路径、无密码/凭据/token/session 值；报告只保留匿名角色 id（ROOT_MF/HOME_ROOT/EXP/B_STOP/A/B）、权限位、计数、错误码、时序与布尔结果 |
| 报告内自述 | 报告 §12 声明已移除服务器地址、账号、共享名、真实路径、文件名与凭据；本地 `local/`（profile、名称映射、原始 fsd 日志、state DB）不在 workspace/证据包 |

## 外部证据包（不复制入仓，仅引用）

- 路径（仓库外）：`work/issue7-acceptance/evidence/t7-limited-20261105/`
- `MANIFEST.json` SHA256：`372e1c8296f095f03d1113ecacec205a32636242135010d11f6739a8cf79f53e`
  （与 Lead 提供的 `372e1c82…f53e` 一致；MANIFEST 列出 23 个文件）
- `SHA256SUMS` SHA256：`85bf5683eddfa24b503bef430e6c5cbbb2cff6bab614ad5ae1b5ca366e93ae13`；`sha256sum -c` **23/23 OK**（exit 0）
- 包内文件总数 27（23 个清单项 + `MANIFEST.json`/`SHA256SUMS`/`README.md`/`REPORT.md`/`EXCLUDED.txt` 中的 4 个未入清单文件，具体以包内 `EXCLUDED.txt` 为准）
- 仓库侧复核：包内文本未命中 IP/真实路径/密码类模式；仅有字段名 `credential_source: "file"` / `credential_file` 等**非凭据内容**，未复制任何凭据。该包及其 22 个证据文件按 T3 报告已完成脱敏。

## 边界

- 本记录只证明“复制与脱敏检查”。T7 的通过/阻塞状态、11 场景逐项结果、缺口 2/3 未观测以及产品异常观察以报告正文（尤其 §5、§7、§8、§9、§11）为准。
- 被测候选为 `drive-native 0.1.0~dev20261105.2`（deb SHA256 见报告 §1）；产品 Rust 代码在 commit C 中未改。
- 根 scope `initial_index` 阻塞的既有证据（2026-11-05 只读健康检查）继续保留在[真实 NAS initial_index 阻塞报告](../real-nas-initial-index-20261105/REPORT.md)，不因 T7 部分通过而删除或改写。
