# G4-014 独立证据包复制核验记录

本文件记录把 T4 独立验证证据包复制进本仓库的逐字节核验；与包内 `g4-final-014/REPORT.md`（T4 原件）分属不同层次：本文件是**作者侧的复制与入库核验**，不改变 T4 任何结论。

- 来源（仓库外，只读）：`work/issue7-validation/G4-evidence-bundle-014-final/`
- 目标（仓库内）：`docs/validation/issue7-native-001/g4-final-014/`
- 复制方式：`cp -a`（保留内容、相对路径、执行位与 mtime；包内相对结构不变）
- 操作时间：2026-11-05（Asia/Shanghai）
- 权威值（Lead 定稿）：
  - `MANIFEST.json` SHA256 = `356993ca7e8ba36c7c3411bb732875e9eb1e4bcbf18fb508df76eee6c1c4b659`（289 条目 = 288 个数据文件 + `SHA256SUMS`）
  - `SHA256SUMS` SHA256 = `13d1ac35ddee3e5190024f78ba5e5b4ad018b7e7362b0857105afc2480393464`（288 项）
  - `STATUS.json` SHA256 = `15af07d171b014beb0290ef69d1dc080308483931b61e58369d49373c4e21919`
- 文件规模：`290` 个文件 = `289` 个 MANIFEST 条目 + `MANIFEST.json` 自身；总计 `6,563,698` bytes（源/目标一致）；无 ELF、共享库、tar/zip 等二进制交付物（仅文本/日志/JSON/Python 脚本与快照校验记录）。
- 包内结构：`COVERAGE.md`、`REPORT.md`、`SHA256SUMS`、`STATUS.json`、`environment/`、`evidence/`、`harness/`、`review/`；`MANIFEST.files` 的相对路径以包根为基准，复制后未做任何路径或内容改写。
- 仓库侧附加文件：`g4-final-014/README.md`（导航）在复制后单独添加，不属于 T4 包清单，不参与逐字节核验。

## 核验结果

| 检查 | 结果 |
|---|---|
| 源包 `sha256sum MANIFEST.json` | `356993ca…4b659`（与权威值一致） |
| 源包 `sha256sum SHA256SUMS` | `13d1ac35…93464`（与权威值一致） |
| 源包 `sha256sum STATUS.json` | `15af07d1…21919`（与权威值一致） |
| 目标 `sha256sum MANIFEST.json` / `SHA256SUMS` / `STATUS.json` | 与源包及权威值一致 |
| `sha256sum -c SHA256SUMS`（目标目录内） | `288/288 OK`，exit 0 |
| MANIFEST 全量重算（目标目录，289 项逐一 sha256） | 0 缺失、0 hash 不符（除清单自身） |
| 源/目标复合树摘要（相对路径+逐文件 SHA 排序后总体 SHA256） | `af98ebcd8f15adae4e09ba93dab2ac66dddd68c8364a1222f382147289b12502`，两侧相同 |
| `MANIFEST.files` 排序键值对文本的 SHA256 | `67ccc97d43cb4dfbe86f822c1ce8ba78cc5987d7bbd941a405770ebf7f40ec19`，两侧相同 |
| `diff -r` 源 vs 目标（排除仓库侧 `README.md`） | 无差异 |
| 目标包内文件数 / 总字节 | `290` / `6,563,698`（与源一致；另有 1 个仓库导航 README） |

## 入库注意

- 仓库根 `.gitignore` 忽略 `*.log`，本包含大量原始日志；按证据包惯例以 `git add -f` 强制入库。
- `docs/validation/issue7-native-001/.gitattributes` 增加 `g4-final-014/** -text`，避免 Git 的 `* text=auto eol=lf` 改写包内 CRLF/LF 字节，保证与源包逐字节一致。
- 包内 T4 原件含行尾空格（`git diff --check` 会逐行报告）；因“逐字节复制不得改内容”，不做任何清理，本仓库自己的修改文件在排除该冻结包后 `git diff --cached --check` 为 0。
- 源包另含两个空目录 `environment/attempt1-missing-nautilus-prefix/logs/` 与 `evidence/g4-014/native-loopback-crate/`（0 文件）；`cp -a` 后为空目录，Git 不跟踪空目录，从仓库 `git archive` 导出时不出现；不影响清单内 290 个文件的内容与逐字节核验。
- G4-013 与 012 等既有冻结证据目录未修改。

## 边界

- 本记录只证明“复制与包完整性”。T4 的主矩阵/补测结论、审阅与允许失败项以包内 `REPORT.md`/`COVERAGE.md`/`STATUS.json` 为准。
- 包 `MANIFEST.candidate_commit = b303ac6bf01f6ff66f48d1e74cb94740561a841c`；`acceptance_status = passed`（产品行为矩阵主矩阵 59/59、补测 28/28）。`STATUS.json` 的 `allowed_failed_checks` 有 2 项：`package-metadata-equivalence`（changelog 版本条目导致的打包元数据差异，非产品失败）与 `t9-measure-log-audit`（HOME attempts5 测量记账 caveat，非产品行为失败）；两项均需如实保留，不得当作通过项或隐藏。
- G4-014 是合成/本机矩阵的独立验证，不替代真实 NAS 的完整 ACL 验收；Issue #7 仍保持开放。本次入库提交（commit D）只追加 T10 文档、G4-014 证据与引用更新，产品代码零改动。
