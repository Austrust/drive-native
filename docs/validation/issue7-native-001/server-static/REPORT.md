# Drive Server 4.0.3-27892 权限边界静态调查

本次从官方公开 SPK 取得与历史证据 **同 SHA** 的 `syncd`，确认了提交前 ACL 拒绝路径，也发现 `0x9004/0xa004` 同时用于 profile 路径只读过滤。因此，不能把这两个码直接等同于目录 ACL 位。原生协议确实提供 `get_file_capability(view_id,path)`，可查询已有节点的有效能力；本次没有找到可与根节点 ACL 独立区分的 L2 整空间禁写来源。

这是静态证据，**NAS / 实际原生请求响应均未实测**。没有运行 Synology 程序、产品构建、测试或基准，没有读取账户凭据或生产文件。

## 来源及可复现性

官方[版本下载目录](https://archive.synology.com/download/Package/SynologyDrive/4.0.3-27892)列出 [x86_64 SPK](https://global.synologydownload.com/download/Package/spk/SynologyDrive/4.0.3-27892/SynologyDrive-x86_64-4.0.3-27892.spk)。HTTP 200、37,974,657 字节；完整来源、响应头和 SHA 见 [download-manifest.json](download-manifest.json)。

| 文件 | SHA-256 |
|---|---|
| 官方 SPK | `58417bbeb433d303b18715649b7bac07efbcef686468378456741eff8ef060fc` |
| `package.tgz` | `8465acfa404b662dd13a2759f7d5226e2095ed39c4624a90e620ebc9454df9e4` |
| `sbin/syncd` | `e2288b191e8db97673c3468595e3a2d441579d2da365927c1108c000549fdf9c` |
| `lib/libsynosyncservercore.so` | `cfcad81883c49285b34a986eec78dc4702bf8312d4b1d847a07b095fbb93ab3b` |

归档采用 Synology 加密格式。自写的选择性提取器参考公开的 [Synacktiv 格式研究](https://www.synacktiv.com/en/publications/extraction-of-synology-encrypted-archives-pwn2own-ireland-2024.html)和 [SynoXtract 源码](https://github.com/prt1999/SynoXtract)，校验归档头 Ed25519 签名以及 80 个条目的 BLAKE2b 摘要，只输出 `INFO` 和 `package.tgz`，再按固定成员名提取软件 ELF。未执行下载的脚本或 Synology 代码。见[格式参考来源摘要](reference-sources.json)、[提取原件](spk-extraction.json)、[固定成员提取脚本](extract-selected-members.py)及[全部命令](COMMANDS.md)。

## 已闭合的提交前 ACL 路径

以下地址均属于上述 `syncd`；虚调用目标由[虚表原件](vtables.txt)解析，不依赖函数名猜测。

1. [InitRouteMap](dispatch-boundary/function-20b460.asm) 在 `0x210c0d–0x210c5e` 将 `remove` 注册 `RemoveACLTest`，在 `0x210e0b–0x210e5a` 将 `upload` 注册 `CreateACLTest`。`rename` 注册源 Remove 测试及目标 Create 测试，地址 `0x210c98–0x210d74`。
2. [TestPhaseMiddleware::HandleAfterHeaderRead](dispatch-boundary/function-29eb40.asm) 跳入 [Test](dispatch-boundary/function-29e7b0.asm)，按 action 取测试集合。[TestPhaseSet::Test](control-details/function-556600.asm) 遇非零测试结果返回 false。
3. [ACLTest::Test](acl-boundary/function-222a20.asm) 依次调用 `PrepareInfo`、`GetUserCapability`、`PermissionCheck`；失败则调用相应 `HandleError` 并返回 `-1`。
4. [Router::RouteAfterHeaderRead](dispatch-boundary/function-555320.asm) 在 middleware 为 false 时直接返回，跳过 handler 的 `HandleAfterHeaderRead`；[SyncTask::RunFlowBody](flow/function-220610.asm) 的 `0x2206d7–0x2206e1` 接到 false 后转 `DiscardBody`，随后写响应，跳过 `ReadBody`、业务 `Handle` 与 Commit。

因此，**由这条 ACL middleware 路径产生的拒绝，是本请求业务提交前拒绝**。这不等于对任何来源、任何版本、任何复合操作收到相同数字码都证明了未提交。

| 生成点 | 条件与响应 | 能证明什么 |
|---|---|---|
| [CreateACLTest::PermissionCheck](acl-boundary/function-223710.asm)，[HandleError](acl-boundary/function-223140.asm) | capability `+1` 为 false → `-3` → `0x9003`；`+1` true、`+2` false → `-4` → `0x9004` | 当前上传路径的前置有效能力检查失败；不是独立的 CreateFile/CreateDir primitive |
| [RemoveACLTest::PermissionCheck](acl-boundary/function-222c50.asm)，[HandleError](acl-boundary/function-2232f0.asm) | capability `+3` false → `-5` → `0xa004`；HandleError 另将 `-3` 映射 `0xa003` | 当前请求目标有效删除能力检查；不能改记为父目录自身 Delete |
| [Response::SetError](acl-boundary/function-553b20.asm) | `_type=error`，`error.code`、`error.action`、`error.reason` | 上述调用第二数值参数为 0，写入 `error.action=0`；该字段不是提交状态 |

capability 字节的意义由 core 库 [Capability::ToPObject](core-capability/function-256ea0.asm) 确认：`+1=can_read`、`+2=can_write`、`+3=can_delete`。文本 reason 只用于核对，不建议生产代码按字符串推断拒绝。

## 同码的其他来源：必须保留操作与来源边界

[立即数扫描](permission-code-immediates.json)覆盖 `syncd` 的 **3,593 个去重已定义动态函数区间**，得到 **13 处**四码立即数引用。`0x9003/0xa003` 在该扫描范围仅见上述 ACL `HandleError` 生成；`0x9004/0xa004` 另见业务过滤及其他 action/background handler。扫描不涵盖无动态符号代码的完整控制流，也不证明不存在计算出的码。

- [UploadHandler::TestFilter](filter-boundary/function-4b7da0.asm) 把 FilterChecker 的 `0x9004` 保留为 `0x9004`，其他远端过滤结果变为 `0x9005`。
- [RemoveHandler::TestFilter](filter-boundary/function-41f2b0.asm) 把 FilterChecker 的 `0x9004` 变为 `0xa004`，其他远端过滤结果变为 `0xa005`。
- core 库 [FilterChecker::TestFileFilter](filter-source/function-2607f0.asm) 在 `0x261879–0x2618af` 把 profile 的只读路径过滤失败设为 `0x9004`。对应 reason 按定长块拼接为 `filtered by path (read only)`；反汇编工具的相邻字符串注释不是完整 reason。
- 这些 TestFilter 错误也用 `error.action=0`，因此不能据该字段区分 ACL 拒绝与 profile 过滤。`0x9005/0xa005` 不应学习成权限 deny。
- [UploadHandler::HandleUpload](control-details/function-4db720.asm) 在 `0x4dcdfe` 检查 TestFilter，成功才到 `PrepareUploadInfo`、`CommitUpload`（`0x4dd7fe`）。[RemoveHandler::HandleRemove](control-details/function-420420.asm) 在 `0x421651` 检查 TestFilter，失败跳清理退出；业务 `RemoveCommitter::Commit` 在 `0x42241b`。这证明所列过滤分支早于这些 Commit 调用；本次未对上传 staging、所有辅助副作用、内部背景任务和其他版本作全面非提交证明。

**实现建议：** 当前操作级 permission hint 与 uncertain/reconcile 路径有充分保守依据。若要升级为确定拒绝，应限制至已证明的 action、协议响应阶段和版本语义，且分别评估“本请求未提交”与“允许学习哪一个 `(node, action)`”。不能从未提交直接推出可给父目录/兄弟路径写入 deny。复合 replace 的 upload、目标清理、源清理分别处理；源/目标 rename 不以普通上传码猜测归属。

## 原生逐节点能力查询与父目录语义

`get_file_capability` 在 [InitRouteMap](dispatch-boundary/function-20b460.asm) 中注册为真实 action。其 [CheckParameters](dispatch-boundary/function-355dd0.asm) 要求合法 `path`；[Handle](dispatch-boundary/function-3561b0.asm) 读取 `view_id` 和 `path`，在 `0x3564f5` 查询已有节点，成功后构造 NodeCapability 参数、调用 Get，最后把 ToPObject 的结果写入响应 header 的 `capabilities`。节点查询失败返回 `0x7000`，不能当 deny。

返回 bool 字段由 [ToPObject](core-capability/function-256ea0.asm) 明确包含：`can_preview`、`can_read`、`can_write`、`can_delete`、`can_comment`、`can_share`、`can_encrypt`、`can_rename`、`can_download`、`can_sync`、`can_lock`、`can_auto_lock`、`can_organize`。这提供原生 L3 查询候选，**未实测请求编码、部署版本兼容和实际返回**。查询读取已有节点，不直接提供不存在新路径的 CreateFile/CreateDir、DeleteChild ACL primitive。

实际 ACL middleware 的 [DefaultACLTest::GetPath](control-details/function-222a90.asm) 读取当前请求 `header.path`，[PrepareInfo](acl-boundary/function-226670.asm) 给 Node 设置该路径，[GetUserCapability](acl-boundary/function-223df0.asm) 调用同一 NodeCapability。core 库 [LoadAclByPath](acl-path/function-2f5330.asm) 遍历父链，之后 [DriveAcl::GetCapability](acl-path/function-25b7a0.asm) 汇总有效操作能力。**父目录自身 `can_delete=false` 不等于对子项的 delete-child 禁止；父目录自身不能重命名也不等于子项不能重命名。**

可实施约束：已有文件 WriteContent / Read / Delete 只记录精确请求节点；目录能力不递归下发给孩子。删除目录被拒绝后，其下其他独立可删文件应保持可操作。新建父目录的 CreateFile/CreateDir 与 rename 两方规则需要独立的动作语义，不能把该查询的 `can_write`/`can_delete` 无条件重命名为 ACL primitive。

## 未闭合项

- 独立 L2 整空间/同步禁写来源：**未找到**；`get_file_capability` 与根的 `can_write=false` 均是节点语义，不能据此挂整盘 RO。
- 真实服务器权限变更、profile_changed 后重探测、版本兼容、拒绝后的本地保留和恢复：**未实测**，仍须独立验证；本报告不代替 G4。
- 通用 `0x3002/0x3004`、rename 源/目标细分、所有复合提交阶段：未完成全域非提交证明，继续 operation hint/reconcile。
- 此前 Download 拒绝只应在 `recv_download` 的直接响应边界识别；不能分类任意 `session.download` 返回的一般 Server 错误。该既有结论不因本次服务端调查放宽。
