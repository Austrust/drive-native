# T2（L2 来源复核）独立审阅

被审对象：`work/fix-7/docs/validation/issue7-native-001/l2-source-002/REPORT.md`（工作树，**未提交**）与
`work/issue7-native-evidence/l2-source-002/` 证据包。审阅者未修改任何产品代码、旧冻结证据或 T2 文件。

## 1. 证据完整性（独立复算）

| 检查 | 结果 |
|---|---|
| 证据包 `MANIFEST.json` / `SHA256SUMS.txt` 自洽 | 通过：逐文件重算 SHA256 一致，MANIFEST 列出的函数原件全部存在，无缺项 |
| 输入与上一轮冻结包同 SHA | 通过：`inputs/{download,libraries,selected-members}-manifest.json` 与 `server-static/`、`l2-static/inputs/` 冻结副本逐字节一致 |
| 四个 ELF 的 SHA 与报告表一致 | 通过：`syncd` `e2288b19…`、`libsynosyncserver.so` `1677fcc0…`、`libsynosyncservercore.so` `cfcad818…`、`libsynodrivecommon.so.7.0` `9f213f24…`；本轮独立读取同目录原件复算一致 |
| 旧冻结证据未被改动 | 通过：`server-static/`、`static/`、`l2-static/`、`server-capability/`、`issue7-native-001/REPORT.md` 的 git 工作树状态干净（`git status --porcelain` 为空；commit 内与工作树一致） |
| 是否只追加 | 工作树唯一的未跟踪项就是 T2 的 `l2-source-002/` 文档目录；仓库无其它修改 |
| 未执行/未联网声明 | 与文件证据一致：本包只含符号清单、反汇编文本与 JSON 索引，无执行/下载产物 |

完整机检结果在 `environment/t2-evidence-check.json`。

## 2. 关键发现：T2 报告不在待验证提交内

`fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4` 的树里**没有**
`docs/validation/issue7-native-001/l2-source-002/`（`git ls-tree fefb8ee -- <该目录>` 为空）；
T2 报告与证据包都只存在于工作树/仓库外。REPORT 中引用的行号与路径因此不能在不可变快照中复算。
影响：若进入打包/冻结，包含“空间只读不能接线”结论的文档不会出现在最终 commit；T2 证据包也在
git 之外（无版本控制），只能靠自身 MANIFEST 保证完整性。
建议（不自行修改 T2 文件）：**在最终冻结提交前提交该报告目录**；若决定不提交，需在最终交付里明确
T2 结论的存放位置与 SHA 记录方式。这一点不影响产品行为，但影响“最终提交自包含证据”的门槛。

## 3. 论证审阅：是否存在被漏掉的可证明路径

逐条核对报告的三类候选与“缺失证明”清单，并对照 `l2-static`/`server-static`/`l2-source-002` 原件：
本轮独立重扫了四个公开 ELF 的全部可执行段（`environment/server-immediates-013.json`），结论与 T2 的
过滤/权限码边界一致（附带发现：`0x9003` 写立即数产生点唯一，`0x9004` 共 8 处立即数引用/6 处写立即数）。

- **共享高级权限（SDK SimpleSharePrivilege / isShareUser）**：报告证明普通用户节点能力里的 share
  privilege 是本地推导 2/4，只有 `isShareUser` 才读 `GetAdvSharePrivilege`；且该值被物化为根路径 ACL
  条目，而非“整空间后置否决”。要升级为 L2 还缺 `ACLRule::Entry` 类型/继承位与 `CalculatePermission`
  采纳优先级的权威定义、子 ACE 可否覆盖根条目的运行证据、以及 0x9004/0xa004 在同码过滤下的归属。
  这三项在现有静态包内确实无法补齐，我未发现遗漏的已归档证明。
- **profile/session 只读路径过滤**：`0x9004`+“filtered by path (read only)”存在，且 upload 的 TestFilter
  在 Commit 前；但 profile 写侧（谁按什么权限生成 read-only 路径）在 syncd/两个 syncserver 库内没有
  已证来源（`ProfileHelper::ReloadProfile` 只做 DB/profile 装配），作用域/覆盖 remove/rename/失效
  契约也未证。把它当 L2 会越过“来源权威性”这一步，报告拒绝接线是合理的。
- **整共享只读标志/其他 SDK 接口**：`Share::IsReadOnly` / `IsForbidden` /
  `IsShareReadOnlyRegisteredBy` / `GetPathAclPrivilege` 在已扫描的四个 ELF 内只有包装层或没有调用者；
  `isFileWRForbidden` 只是 `access()` 包装。报告明确“未解包组件不能排除，但同样不能据此接线”，这是
  正确的不对称处理。
- **其它方向**：报告也检查了 `allowed` 方向（无证据可推全空间可写）与 root-only 投影（`/` 且
  privilege==1 才 ReadOnly；`path=="root"` 直接 Full）。我未找到“root 能力或 view privilege 可安全
  放大为整空间”的已归档证明。
- **结论**：在“不新增运行时观测”的前提下，**没有发现被遗漏的可证明 L2 安全来源**；“保持
  `server_capability=unknown` + 保持 profile 失效路径 + 等 T3 真实 NAS 实验”的结论成立。
  T3 实验设计（M1–M4 判别矩阵与记录/安全要求）可执行，且正确要求 File Station/POSIX 与 Drive 原生
  两侧同时观测。

## 4. 文字/边界观察（非缺陷）

1. 报告把“共享权限物化为根 ACL 条目”与“root-only 后置投影”分开写，避免把根结论当整空间结论，
   与 `l2-static` 旧证据一致；旧 `l2-static/REPORT.md` 没有被改写（其明确“未找到可接线的空间来源”）。
2. 报告中新引入的“Lead 只读 File Station 探测（2026-11-05）”标注了来源与“不属于本包静态证据”，
   未把它当作静态证明；冻结时应保留该标注。
3. 证据包在 git 之外，且报告引用的大文件 ELF 不在包内（只有清单与反汇编）；这与 README 的边界声明
   一致，但需要在最终交付中保留 `work/issue7-native-evidence/public-server/` 的可用性。

## 5. 结论

- T2 证据完整、未改旧冻结证据，**“无安全 L2 来源、保持 unknown”结论成立**；未发现需要产品修复的缺陷。
- 唯一流程性问题是 T2 报告/证据未进入 `fefb8ee`，需在冻结前处理（见 §2）。
- 未实测项沿用报告：真实共享配置/子 ACL 组合、profile 写侧、remove/rename 的 read-only 过滤、
  其他版本与未解包组件。
