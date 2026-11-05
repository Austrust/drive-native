# get_file_capability：能力字段与身份边界

**结论：本轮不把原生 `get_file_capability` 查询持久化为 `(node, action)` 权限。** 成功响应只有能力对象，没有与这些能力原子绑定的节点身份；`id:` 语法也先解析为路径，再由 handler 重新按路径查询。路径替换会使能力归属不确定，前后 stat 不能排除 ABA。独立 L2 整空间禁写来源仍未找到。

这是 Drive Server 4.0.3-27892 公开软件的静态证据。**NAS、实际原生请求响应、其他部署版本均未实测。** 未运行 Synology 程序、产品测试、构建或基准，未接触凭据和生产数据。此前 evidence-bundle-002 保持不变；本补充包限制了其中“原生 L3 查询候选”及“按精确请求节点记录”的可实施范围，不撤销既有提交前 ACL 路径证据。

## 来源

官方[版本目录](https://archive.synology.com/download/Package/SynologyDrive/4.0.3-27892)及 [x86_64 SPK](https://global.synologydownload.com/download/Package/spk/SynologyDrive/4.0.3-27892/SynologyDrive-x86_64-4.0.3-27892.spk)。本轮复用已下载并校验的文件，没有再次访问 NAS。来源和提取摘要见 [download-manifest.json](download-manifest.json)、[selected-members-manifest.json](selected-members-manifest.json)。

| 文件 | SHA-256 |
|---|---|
| 官方 SPK | `58417bbeb433d303b18715649b7bac07efbcef686468378456741eff8ef060fc` |
| `sbin/syncd` | `e2288b191e8db97673c3468595e3a2d441579d2da365927c1108c000549fdf9c` |
| `lib/libsynosyncservercore.so` | `cfcad81883c49285b34a986eec78dc4702bf8312d4b1d847a07b095fbb93ab3b` |

所有下列指令地址，除 ToPObject 属于 core 库外，均属于上述 syncd。完整函数原件来自固定动态符号区间，方法见 [复现命令](COMMANDS.md)。这不是对整个服务端所有控制流的全域证明。

## 字段名不能直接变成 Action

core 库 [Capability::ToPObject，0x256ea0](raw/fields/function-256ea0.asm) 证明字段布局：`+0 can_preview`、`+1 can_read`、`+2 can_write`、`+3 can_delete`、`+7 can_rename`、`+8 can_download`、`+9 can_sync`。它逐项调用 `PObject::operator=(bool)`。

| 操作 | 实际检查原件 | 可采用的边界 |
|---|---|---|
| 下载正文 / Read | [DownloadNodeACLTest::PermissionCheck，0x2253b0](raw/acl/function-2253b0.asm)：`0x2253db` 读取 `+0`、`0x2253e8` 读取 `+1`、`0x2253f1` 读取 `+9`；该函数未读取 `+8` | 不能把 `can_read && can_download` 当成现有下载动作的精确判据。preview/read/sync 是本 ACL 检查的必要位；其他过滤、节点状态和业务失败仍可能阻止下载，不能由全 true 推出最终 Allow。 |
| 上传已有文件 / WriteContent 候选 | [CreateACLTest::PermissionCheck，0x223710](raw/acl/function-223710.asm) 检查 `+1 can_read` 和 `+2 can_write` | `can_write` 单字段不是完整许可。目录自身的 can_write 不独立表示 CreateFile/CreateDir，更不证明 Setattr；现阶段均不写节点权限表。 |
| 删除目标 / Delete 候选 | [RemoveACLTest::PermissionCheck，0x222c50](raw/acl/function-222c50.asm) 检查 `+3 can_delete` | 这是被操作目标的有效删除能力；父目录自身 can_delete 不能作为子项 delete_child。 |
| 重命名源 / RenameSource 候选 | [虚表](raw/rename-vtables.txt) 的 RenameSource `+0x28` 指向 RemoveACLTest::PermissionCheck，GetPath 指向 [RenameSourceACLTest，0x225d50](raw/acl/function-225d50.asm) | 实际源检查使用 can_delete，不是按名字猜 can_rename。现有 Action::RenameSource 仍应保持源节点语义，不继承父自身拒绝。 |
| 重命名目标 | [虚表](raw/rename-vtables.txt) 的 RenameDest `+0x28` 指向 CreateACLTest::PermissionCheck，GetPath 指向 [RenameDestACLTest，0x2261e0](raw/acl/function-2261e0.asm) | 源与目标独立；源允许不能推出目的地允许，can_rename 也不能推出 RenameTarget。 |

`can_download` 和 `can_rename` 确实存在，但本次没有证明它们等同于产品所需的 Read 或 RenameSource。`can_sync=false` 是此节点相关能力，不能当作 L2 全空间禁写。上述候选映射全部受下一节身份边界限制；本轮没有任何新原生节点 ACL 映射接入。

## 请求与响应的身份边界

1. [GetFileCapabilityHandler::CheckParameters，0x355dd0](raw/identity/function-355dd0.asm) 检查非空 path 并调用 IsSafePath。[Handle，0x3561b0](raw/identity/function-3561b0.asm) 在 `0x356423` 以 `asUInt64` 读 view_id，在 `0x35648d` 以 `asString` 读 path，在 `0x3564d2` 取得该 view 的 ViewDB，在 `0x3564f5` **按 path QueryNode**。查询失败设 `0x7000`，不能当成权限 deny。
2. Handle 在 `0x356814` 构造 NodeCapability，经 `0x356834` 参数构造、`0x356850` Get、`0x356933` ToPObject，最后 `0x35693f–0x35698c` **只向 response header 写 capabilities**。该函数不在响应写 file_id、node_id、sync_id 或可与能力匹配的 version/token；也没有对 request 中这些身份字段做匹配检查。
3. [PathConverterMiddleware，0x287e20](raw/identity/function-287e20.asm) 在 `0x288025` 检测 view_id 已存在后，转至 `0x288180` 检查 path 并返回；**view_id+path 不经过 id: 转换**。所以给 view_id 同时传 `path="id:123"` 不能获得身份寻址保证。
4. view_id 缺省时，middleware 才在 `0x288294` 调用 PathConvertHelper::Convert。其 [Process，0x24ca00](raw/identity/function-24ca00.asm) 与 [ProcessID，0x24bc60](raw/identity/function-24bc60.asm) 分派 `id:`。随后 [QueryNodeByPermanentId，0x24ab80](raw/identity/function-24ab80.asm) 在 `0x24ac6f` 按 ID 找节点，构造 view_id/node_id/sync_id/file_id/path，并在 **`0x24b09d` 归还 ViewDB**。
5. [Convert，0x24cdc0](raw/identity/function-24cdc0.asm) 将这些解析结果写入 **request header**。它们不是响应中的原子身份绑定。之后 GetFileCapabilityHandler 仍另取 ViewDB、重新按 path 查询且忽略这些身份字段。因此 id: 不能消除解析和查询之间的替换风险。

推论：同一路径先后指向 A、B，再回到 A 时，前后 stat 都可以看到 A，而中间能力取自 B。客户端 session revision 能丢弃本地已知通知之后的迟到结果，但不能排除远端未被观察到的 ABA。因此不能把成功查询写入旧 A 的 native 权限行，也不能把查询失败转换成 A 的拒绝。当前方案保持该来源 unknown。

## 将来只读诊断入口的最小约束

以下是接入建议，**没有产品实现或运行验证**：

- 请求采用从已验证 Scope/目录选择得到的 `view_id: u64`，不用字符串、截断整数或猜测默认值。path 采用该 view 内以 `/` 开头的规范路径，不是本地挂载路径或 NAS 主机文件系统路径；保留 Unicode，拒绝 NUL、空组件、`.`、`..` 与反斜杠，根仅 `/`。服务端 [IsSafePath，0x374e50](raw/identity/function-374e50.asm) 检查危险组件；产品现有 validate_path 尚不排除反斜杠，不能原样声称完全等价。这里给的是保守客户端规范，不宣称服务端只接受这一种文本形式。
- response header 的 capabilities 必须是 Map。产品 PObject 没有 Bool variant，bool 编码为 Integer；建议仅接受明确的 `Integer(0|1)`，不把字符串、空值、非 0/1 整数或缺失字段默认成 false。缺字段、类型坏、认证/网络/超时/服务端错误和查询 `0x7000` 均保持查询 unknown，不能写 deny。
- 最小读取点可用 sproto Client 现有 pool + `SCMD_REQUEST` 模式；core NativeReadSession 校验 Scope 并返回路径快照。若跨 worker，新增独立只读查询和 `PathCapabilities` 响应，不复用 `PermissionDenied{node,action}`；遵循 IPC 版本一致性检查。
- 展示应明确是 path 快照，可携带采集时刻和本地 revision；不要写 directory_permissions，不动未确认写入，不改重试状态。查询不放入 FUSE lookup/getattr/readdir，不持有 Store guard 等网络。缺少服务器原子身份绑定前，不把它升级成精确节点门禁。

## 与当前门禁更正的关系

删除/改名目标能力与父目录自身能力分离，可直接用于纠正本地 Action 对象语义；无需先把上述查询接入。父 Traverse、目标 Delete、源 RenameSource、目标父 RenameTarget 仍分别检查。目录 mode 不因为自己的写动作拒绝而收窄，避免内核在收到目标检查前误拒合法子项操作。未知能力仍允许向服务器尝试，不表示允许证据。

真实远端 ACL、权限变化时序、原生查询实际响应及独立 L2 来源仍未实测或未闭合；本报告不替代 G4 独立验证。
