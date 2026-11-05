# #7 原生权限错误：静态证据与使用边界

本证据支持一个小集合：**仅在直接 `download` 响应中，将数字码 `0x3002` 和 `0x3008` 归类为本次 Read 被拒绝**。不能将前置目录读取失败转换为最终写操作拒绝，也不能用相同数字码猜出父目录、目标对象或整个服务空间的写权限。

本次未运行客户端、服务端、产品测试、基准或构建；未连接 NAS；未读取凭据、用户数据库或生产内容。写入仅限本证据目录。所有协议运行时效果均 **未实测**。

## 来源和复核方法

- 已下载的官方 Linux `cloud-drive-daemon`，SHA-256 `ea01d634762060424f4a5fd3fccb3f5c0ba7c10a81e90e144706fb2b8406314a`。本次重新读取本地二进制并校验摘要，没有执行它。
- 符号来源为已有 `work/platform-deep/daemon-symbols.json`，摘要、输入路径、函数地址与长度见 [manifest.json](manifest.json)。
- [提取脚本](extract-client-static.py)读取 ELF 节表与选定符号，用 Capstone 输出静态反汇编和可打印字符串引用。输出不包含软件二进制。
- [内部错误字符串表](internal-errors.json)来自 `_ZZL14GetErrorStringiE3map @ 0x5d27c0`。索引并非猜测：[Channel::Close](function-2baaf0.asm) 在 `0x2baca4` 对内部错误码取负，在 `0x2bacb1` 按该索引读取指针表。

在项目根运行的复核命令：

```powershell
C:/Users/Public/Python3123/python.exe work/issue7-native-evidence/extract-client-static.py --binary 'C:/Users/a_tas/Documents/deepseek-harness/default-workspace/synodrive/linux_x/opt/Synology/SynologyDrive/package/cloudstation/bin/cloud-drive-daemon' --symbols work/platform-deep/daemon-symbols.json --pylibs 'C:/Users/a_tas/Documents/deepseek-harness/default-workspace/synodrive/pylibs'
```

该命令只执行本证据提取脚本；脚本拒绝不匹配的二进制摘要，并只向自身目录写出选定证据。函数名及静态字符串用于确认数字码语义，产品分类不应匹配服务器 `reason` 字符串。

## 已确认的码与最小使用范围

| Wire 数字码 | 静态映射或分支 | 可支持的结论 | 不支持的推断 |
|---|---|---|---|
| `0x3002` / 12290 | `ProtoResp2ErrorCode` → -19 → `no permission` | 直接 download 回复可拒绝本次 Read | 任意阶段的该码均等于写拒绝；具体父目录或目标 ACL |
| `0x3008` / 12296 | 同函数 → -55 → `view is disabled download` | 直接 download 回复可拒绝本次 Read | 禁止写入或禁止整挂载全部动作 |
| `0x3004` / 12292 | 同函数 → -26 → `permission is read-only` | 只读语义成立 | 读取被拒绝；独立于目录 ACL 的 L2 服务空间禁写 |
| `0x8004` / 32772 | `ConvertResponseErrorToHandlerError` → -52 → `Downloading a no hash version (not ready yet)` | 内容尚未准备好 | 权限拒绝 |
| `0x9003` / 36867、`0x9004` / 36868 | Upload 错误处理器标为 `ACL` | 上传响应具有 ACL 拒绝语义 | 服务端提交前终态已被证明；具体父/目标动作可直接学习 |
| `0x9005` / 36869 | Upload 错误处理器标为 `Selective Sync` | 选择性同步过滤 | ACL 拒绝 |
| `0xa003` / 40963、`0xa004` / 40964 | Remove 错误处理器分别标为 `no access`、`no permission`，过滤类型为 `ACL` | 删除响应具有 ACL 拒绝语义 | 服务端无部分提交已被证明；所有后代均可重试 |
| `0xa005` / 40965 | Remove 错误处理器标为 `filtered`，过滤类型为 `SELECTIVE_SYNC` | 选择性同步过滤 | ACL 拒绝 |

`0x9005` 在通用 Helper 中也被映成内部 -32（`no access permission`），但上传动作专用分支明确区分选择性同步。因此不能只凭通用错误字符串建立跨动作权限表。

关键原件与地址：

- [ProtoResp2ErrorCode](function-22c090.asm)，`0x22c090`：`0x3002` 分支返回 -19，`0x3004` 返回 -26，`0x3008` 返回 -55。
- [HandlerHelper::ConvertResponseErrorToHandlerError](function-11e600.asm)，`0x11e600`：`0x9004` → -27，`0x9003/0x9005` → -32，`0x8004` → -52，其余转发给 ProtoResp2ErrorCode。
- [DownloadRemoteHandler::HandleDownloadResponseError](function-b8af0.asm)，`0xb8af0`：`0x3002/0x3008` 均落入 `0xb8b51` 的 Helper 转换；没有将只读码重新定义成读拒绝。
- [UploadLocalHandler::HandleUploadResponseError](function-f9040.asm)，`0xf9040`：`0xfa330–0xfa357` 将 `0x9005` 标为 Selective Sync，其余进入该分支的 `0x9003/0x9004` 标为 ACL。
- [RemoveRemoteHandler::HandleRemoveResponseError](function-e0840.asm)，`0xe0840`：`0xe08ac–0xe0934` 区分 a003/a004/a005；`0xe09f3–0xe0a0f` 区分 ACL 与 SELECTIVE_SYNC。
- [RenameRemoteHandler::HandleRenameResponseError](function-eee00.asm)，`0xeee00`：读取 `code` 与 `reason`；错误路径可能调用 `FallbackToDelAndAddEvents`。此处未发现能证明具体拒绝对象的 `result` / `element` 读取。不能从官方回退动作推导本产品可安全重复提交。

## 产品接线的阶段约束

只读检查当前源码：`drive-core/src/native.rs::download()` 验证 token 后调用 `sproto::download_checked()`；该路径虽然没有目录列表预检，仍可能在取得连接时发生认证或连接初始化错误。因此 **不能分类 `session.download()` 返回的任意 `Server` 错误**。当前接线在 `sproto/src/actions/transfer.rs` 已发送 `action=download` 后，仅将 `recv_download()` 返回的 `Server { code: 0x3002 | 0x3008, .. }` 包装为 `DownloadPermissionDenied`；worker 只接受该专用 variant，才建立请求身份的 Read 拒绝。pool/auth/preflight 的普通 `Server`、本地 I/O、超时、会话错误及其他数值均不属于该集合。

当前错误解码只保留 `error.code` 与 `error.reason`。本次没有发现可验证的 wire `error.result` / `error.element` 对象身份定义；不能把“未发现”写成“协议不存在这些字段”。数字码解码必须拒绝超出 u32 的原始值，避免截断后误命中权限码。

写路径需要另外的证据：

1. 前置 `list_dir`、身份检查、查询失败仅说明相应读请求失败，不能证明准备执行的 WriteContent / CreateChild / RemoveChild / Rename 被服务器拒绝。
2. `mkdir` 在现有 sproto 中使用 `action=upload` 加 `is_dir=true`；上传错误标签本身还不足以证明目录创建没有提交。
3. 写成功后的列表、下载核验、事件观察失败属于提交后确认失败。必须保留 uncertain / reconcile，不能转换为可重发的 permission-blocked pending。
4. 只有明确证明是本次 mutation 的终态拒绝且无提交，才可解除“不重复提交”屏障。客户端错误标签、错误函数名和 reason 文本均不能替代服务端提交边界证据。

## 尚缺的证据

- **服务端 mutation 拒绝的提交边界未证明。** 既有 `outputs/drive-native/evidence/server-namespace-disassembly.txt` 包含部分 Rename/Remove 检查和提交路径，不能覆盖完整共用 ACL 门禁及 upload。已有来源声明的 syncd 摘要为 `e2288b191e8db97673c3468595e3a2d441579d2da365927c1108c000549fdf9c`；本次未找到该软件副本，未重新哈希，也未再次连接服务器。
- 本次允许范围内查找了 Windows 项目/已知软件工作区的 syncd 文件名，以及 WSL Ubuntu-24.04 的 `/tmp` 和 `/home/drive-native-test/.cache` 中名称包含 syncd 的文件；没有输出匹配文件。该结果不证明副本不在其他位置。
- **独立 L2 禁写来源未找到。** 现有 `query_user_info.view_list.privilege` 和 share capabilities 会混入所选根的可写性，不能据此将整个服务空间判定为禁止写入。保持 unknown。
- [IsProfileChanged](function-173570.asm) 和 [ReloadProfile](function-172620.asm) 是官方权限档案更新路径的选定静态原件；本证据没有将其档案格式解释为已实现的逐目录权限能力协议。
- 未对新的 native Read 分类运行产品测试或真实服务器验证。历史子目录列表返回 12290 的已有实测只能证明当次目录枚举被拒绝，不是本次 download 分类的实测，更不是写拒绝的实测。

建议此次可交付范围为：保留并验证两个直接 download 拒绝码，保持其他读错误原有行为；mutation 可在直接响应边界保留动作限定的数字码为权限相关诊断，但继续保持 uncertain / cleanup_uncertain 和 reconcile，不写目录 ACL、不改成 pending；L2 继续 unknown。诊断字符串应为受控文本，不能透传服务器 reason。本证据不能作为 #7 全部验收通过的声明。
