# G4-013 独立证据包复制核验记录

本文件记录把 T4 独立验证证据包复制进本仓库的逐字节核验；与包内 `g4-final-013/REPORT.md`（T4 原件）分属不同层次：本文件是**作者侧的复制与入库核验**，不改变 T4 任何结论。

- 来源（仓库外，只读）：`work/issue7-validation/G4-evidence-bundle-013-final/`
- 目标（仓库内）：`docs/validation/issue7-native-001/g4-final-013/`
- 复制方式：`cp -a`（保留内容、相对路径、执行位与 mtime；包内相对结构不变）
- 操作时间：2026-11-05（Asia/Shanghai）
- 权威值（Lead 更正后，旧值 `76142dac…`/`330e023d…` 作废）：
  - `MANIFEST.json` SHA256 = `1db81a18f19dbabac8ade2b3168d748ceaa133019fe18a8edb6733bfbb9fc4a2`（300 文件清单）
  - `SHA256SUMS` SHA256 = `5dd00f356b18195f7790df6581d58ac8eb9c2df5d0fe305c088efb92c62a43e4`（299 项）
- 文件规模：301 个文件 = 300 个 MANIFEST 条目 + `MANIFEST.json` 自身；总计 `5,608,092` bytes（源/目标一致）；无 ELF、共享库、tar/zip 等二进制交付物（仅文本/日志/JSON/Python 脚本；日志中出现的 “ELF” 仅为记录文本）。
- 包内结构：`COVERAGE.md`、`REPORT.md`、`SHA256SUMS`、`STATUS.json`、`environment/`、`evidence/`、`harness/`、`review/`；`MANIFEST.files` 的相对路径以包根为基准，复制后未做任何路径或内容改写。

## 核验结果

| 检查 | 结果 |
|---|---|
| 源包 `sha256sum MANIFEST.json` | `1db81a18…c4a2`（与权威值一致） |
| 源包 `sha256sum SHA256SUMS` | `5dd00f35…6a43e4`（与权威值一致） |
| `sha256sum -c SHA256SUMS`（目标目录内） | `299/299 OK`，exit 0 |
| MANIFEST 全量重算（目标目录，300 项逐一 sha256） | 0 缺失、0 多余（除清单自身）、0 hash 不符 |
| 源/目标复合树摘要（相对路径+逐文件 SHA 排序后总体 SHA256） | `74c1f7ece7c0e9065161f36b1538343dd497f184889848f24cbeca2f3df0325a`，两侧相同 |
| MANIFEST 条目摘要（排序后 JSON 的 SHA256） | `6e1ddc859cba81e53e7a51f7c26755b762fbd52a36cc19b321bdbfbe8ef30fb7`，两侧相同 |
| `diff -r` 源 vs 目标 | 无差异 |
| 目标文件数 / 总字节 | `301` / `5,608,092`（与源一致） |

## 入库注意

- 仓库根 `.gitignore` 忽略 `*.log`，本包含大量原始日志；按证据包惯例以 `git add -f` 强制入库。
- `docs/validation/issue7-native-001/.gitattributes` 增加 `g4-final-013/** -text`，避免 Git 的 `* text=auto eol=lf` 改写包内 CRLF/LF 字节，保证与源包逐字节一致。
- 包内 T4 原件含行尾空格（`git diff --check` 会逐行报告）；因“逐字节复制不得改内容”，不做任何清理，本仓库自己的修改文件在排除该冻结包后 `git diff --cached --check` 为 0。
- 12/010 等既有冻结证据目录未修改。

## 边界

- 本记录只证明“复制与包完整性”。T4 的主矩阵/补测结论、以及 012 冻结期望与 013 校正的差异以包内 `REPORT.md`/`COVERAGE.md`/`STATUS.json` 为准。
- 包 `MANIFEST.candidate_commit = fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4`；本次入库提交会在其之上追加 T2 报告、T4 审阅建议的 assert-only 纵深防御与文档/证据更新，产品行为不变，最终打包与验收由 Lead/T5 统一决定。
