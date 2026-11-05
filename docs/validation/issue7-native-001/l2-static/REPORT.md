# view privilege 的来源及 L2 边界

**结论：本轮不能把 `query_user_info` 的 `privilege="read-only"` 接成 L2 `SpaceForbidden`，即使服务器 build 限为 27892。** 它有不同于本次根节点能力查询的 SDK 共享配置来源，但这不等于已证明服务器对所有子节点执行不可覆盖的写入禁止。现有节点能力代码还把共享只读投影限定在一个由 ACL 计算输出的路径条件上。产品应保持此来源 `Unknown`，不把根 `can_write=false` 或组合后的 `ShareInfo.writable=false` 升级为整空间拒绝。

这是 Drive Server 4.0.3-27892 公开软件的有界静态分析，复用本地已下载文件；**NAS、真实原生请求、共享配置与子 ACL 组合行为、其他部署版本均未实测**。没有运行 Synology 程序、WSL、产品构建、测试或基准，没有新增下载、读取凭据、连接 NAS 或修改产品代码。此前两个冻结包保持不变。

## 来源和范围

官方[版本目录](https://archive.synology.com/download/Package/SynologyDrive/4.0.3-27892)及 [x86_64 SPK](https://global.synologydownload.com/download/Package/spk/SynologyDrive/4.0.3-27892/SynologyDrive-x86_64-4.0.3-27892.spk)。来源与原始获取记录见 [download-manifest.json](inputs/download-manifest.json)、[软件成员摘要](inputs/selected-members-manifest.json)、[库摘要](inputs/libraries-manifest.json)。本补充没有重新请求这些 URL。

| 软件文件 | SHA-256 |
|---|---|
| 官方 SPK | `58417bbeb433d303b18715649b7bac07efbcef686468378456741eff8ef060fc` |
| `sbin/syncd` | `e2288b191e8db97673c3468595e3a2d441579d2da365927c1108c000549fdf9c` |
| `lib/libsynosyncserver.so` | `1677fcc08004cb9128ebd4efccd0d7053586de87042dd54880bc3e2e124b617c` |
| `lib/libsynosyncservercore.so` | `cfcad81883c49285b34a986eec78dc4702bf8312d4b1d847a07b095fbb93ab3b` |

主范围为四个已定位函数：`AppenUserView`、`AppendOtherViewAttributes`、`GetHomeView`、`ListTeamFolderHandler::Handle`。必要调用链限于 SDK 共享权限查询及名单匹配、既有 NodeCapability/GetCapability 数据流、字段序列化。完整命名函数原件按动态符号区间保存；无符号名单匹配函数保存明确的地址窗口，未伪造符号长度。工具、复现步骤与引用原件见 [COMMANDS.md](COMMANDS.md)，文件摘要见 [SHA256SUMS.txt](SHA256SUMS.txt)。

## 四个入口实际产生什么

| 入口与原件 | 关键地址及数据流 | 能支持的判断 |
|---|---|---|
| [AppenUserView，syncd 0x41b260](selected/function-41b260.asm) | `0x41b6a5–0x41b6d1` 以 `/` 调 QueryNode；`0x41b761` 把该根 Node 交给 AppendOtherViewAttributes；`0x41b7b2`、`0x41b819` 给同一结果对象写 view_id、node_id | team privilege 和根节点能力有关，不能把整条 view 对象误认为天然空间级权限。 |
| [AppendOtherViewAttributes，syncd 0x414170](selected/function-414170.asm) | `0x41458f` NodeCapability::Get；`0x414594` 取结果 `+1`，`0x4145a9` 取 `+2`；`0x41499f` 写 privilege | `+1/+2` 是 can_read/can_write，见 [字段布局 0x256ea0](fields/function-256ea0.asm)。其字符串有根能力和 SDK 两种来源，详见下节。 |
| [GetHomeView，syncd 0x41a400](selected/function-41a400.asm) | `0x41a51c` 查询 `/`；`0x41a763–0x41a76a` 直接写字面量 read-write；`0x41a7ff–0x41a804` 写 share_priv_disabled=true | 该版本个人 home 的 read-write 不是动态读取出的全空间可写证明；不能据此生成 L2 Allowed。 |
| [ListTeamFolderHandler::Handle，syncd 0x3efdd0](selected/function-3efdd0.asm) | `0x3f04f1` QueryNode(`/`)；`0x3f053d` 构造根节点参数；`0x3f0565` NodeCapability::Get；`0x3f07ec` ToPObject；`0x3f0841` 写 capabilities | list_team_folder 的 can_write 是根节点结果，不能作为全子树禁写。 |

`AppendOtherViewAttributes` 的主要分支为：

```text
root.can_read == false                         -> "denied"
root.can_read == true && root.can_write == true -> "read-write"
root.can_read == true && root.can_write == false:
    SDK::GetAdvSharePrivilege(share, username) == 4 -> "denied"
    SDK::GetAdvSharePrivilege(share, username) == 1 -> "read-only"
    other SDK values                             -> "read-write"
```

对应地址为 `0x41467c`、`0x4146a2`、`0x414f1a`、`0x414f26`、`0x414f2f`；三个末端字符串在 `0x414f71`/`0x41548b`、`0x414f45`、`0x41546b`。因此不能把所有 privilege 都说成“仅根 ACL”，也不能把所有非 read-write 都说成“空间禁止”。`disable_download` 是单独的布尔参数；`share_priv_disabled` 在 `0x414ba7` 经 SDK share 对象虚调用取得，本范围没有证明其等于空间禁写开关，不能凭名称使用。

## SDK 只读来源与 OnlyAcl 分支

[SDK::GetAdvSharePrivilege，libsynosyncserver.so 0x10f940](sdk/function-10f940.asm) 输入 share 名与用户名，经 `SYNOShareGet` 取得共享配置，不接收根 Node 或路径。其返回路径是：

- `0x10f9fc` 调 `SYNOShareGet`；失败进入 `0x10fa4b`，返回 4。`SLIBShareIsForbiddenGet`、`SLIBShareIsOnlyAcl` 或用户组查询失败也可返回 4。因此 4、wire denied 不能识别一个确定的全空间 ACL 拒绝。
- `0x10fc73` 调 `SLIBShareIsOnlyAcl`。输出非零时，`0x10fc85` **直接跳至 `0x10fd10`，跳过用户名/组名单匹配**。此时先置返回值 2，再调 `SLIBShareIsReadOnlyGet`；输出非零才经 `0x10fd30` 改为 1。
- OnlyAcl 输出为零时，`0x10fc93` 取得用户组；依次把共享对象 `+0x18`、`+0x20`、`+0x28` 的名单传给 [直接匹配 helper 0x10ae80](sdk/window-10ae80-10b120.asm)。其匹配只做字符串切分、用户名比较及 `@group` 查找。三个名单在这里分别导向 4、2（可受只读标志降为 1）、1；这些角色由返回控制流推断，不伪称已取得 DSM 结构体字段名。
- `0x10ff23–0x10ff3b` 的第三名单匹配路径直接导向返回 1；`0x10fd25–0x10fd35` 则根据共享只读标志输出 1。wire read-only 没有携带是哪一条路径产生的标记。

**能证明的只是来源不同：** SDK 判定使用共享配置/用户组，而不是把刚得到的 root.can_write 直接换名。这里没有证明 `SLIBShareIsOnlyAcl` 或用户名单的完整 DSM 契约，也没有证明返回 1 在所有子节点操作中都压过独立子 ACL。把“不传 Node”当作“必然全子树禁止”会跳过这个必要证明。

## 必要节点数据流为何阻止升级为 L2

原件均来自同 SHA 的 core 库，复用已冻结材料：

1. [NodeCapability::Get，0x2fae20](core/function-2fae20.asm) 调 [Impl::Get，0x2fac70](core/function-2fac70.asm)；后者按 Param 分派 GetByAdvanceSharing 或 [GetByUser，0x2f8ad0](core/function-2f8ad0.asm)。本节只分析所需的 GetByUser 路径，不宣称覆盖另一分派及所有服务器操作。
2. GetByUser 在 `0x2f8c58` 初始化一个权限值，随后经含 `SDK::SimpleSharePrivilege` 的缓存元组取得该值，`0x2f8ebd–0x2f8ec3` 保存到 `[rbp-0x60c]`。本文没有把此路径的每个来源都等同于前述 SDK 函数返回值。
3. `0x2fa1c3` 令 r15 指向 `[rbp-0x420]`；`0x2fa1e5` 把它作为字符串输出指针交给 [DriveAcl::GetCapability，0x25b7a0](core/function-25b7a0.asm)。`0x2fa201–0x2fa204` 先保存该调用计算出的能力。
4. GetCapability 在 `0x25b7af` 保存该输出指针；`0x25b8f2` 调 `IsAccessible`，让它填充字符串 vector；`0x25b919–0x25b982`（不可访问路径）或 `0x25bb17–0x25bba0`（可访问路径）把 vector 的首项写入该输出字符串。**它是 ACL 计算的输出，不能未经证明把它当作调用者原始 path。**
5. 回到 GetByUser，`0x2fa67b–0x2fa686` 将这个输出字符串与 `/` 比较；**不相等时，`0x2fa688` 跳过下述只读投影**。相等时才在 `0x2fa68a` 比较 `[rbp-0x60c] == 1`，并经 `0x2faae7–0x2faaf6` 把能力替换成 `Capability::ReadOnly`。后续下载/锁定/水印检查另行继续。

这不是“已证明某个真实子目录可以越过共享只读”的结论：本范围没有构造那个运行场景，也没有证明所有前置条件。它足以说明，在已查看的能力计算里只读投影存在 ACL 输出路径条件，不能从 view privilege 或 SDK 名称推出无条件全子树限制。OnlyAcl 分支又未原样暴露在 wire 上，所以只接受 read-only、再加 build 白名单仍未补上缺失证明。

## 可执行决定与剩余边界

- **本轮生产决定：不新增此 L2 映射，保留 Unknown。** 保持 L1 用户只读与已有逐动作门禁；不要使用 ShareInfo.writable 的合成布尔，也不要把成功的 read-write 解释为全空间 Allowed。没有产品实现随本包交付。
- 现有构建号 27892 能限制版本范围；同条 view_id 能限制对象归属；session revision 能防本地已知变化后的迟到回复。这三者都不能证明共享只读具有不可被子 ACL 覆盖的语义，不能替代本次缺失证据。
- 后续若继续 L2，必须得到独立且明确覆盖整个 view/同步空间的服务器限制及其更新契约，或验证当前共享只读的全子树强制语义；在取得授权前不连接 NAS。不能用修改本地 fake 测试为这种服务器语义背书。
- 本结果不解决 mutation 的精确拒绝学习：原生操作级权限提示继续保持 uncertain/reconcile，不据此重发或写精确目录 ACL。能力查询的路径身份限制见既有 capability 补充，提交前/后错误边界见既有服务器静态证据；两者均可从[证据导航](../README.md)进入。

本有界调查至此收口；未扩大到新版本、完整 DSM SDK 或全协议逆向。上述不可用结论补充既有报告中“没有可安全接入的独立 L2 来源”的范围，不修改其冻结原件。
