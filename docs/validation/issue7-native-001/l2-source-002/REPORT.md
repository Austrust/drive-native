# 空间级只读来源复核 II：NodeCapability 的共享权限注入、评估器边界与 profile 只读过滤

**结论：仍不能把任何可获得的原生来源安全接成 L2 `space_forbidden`，生产初始化保持 `unknown`。** 本轮沿 `get_file_capability` 背后的 `NodeCapability` 数据流找到三条上一轮未闭合的事实：共享高级权限值对普通 Drive 用户并不进入节点能力计算（普通用户是本地推导的 2/4）；对 `isShareUser` 用户该值才是 `GetAdvSharePrivilege`，且无论来源，它都被**物化为根路径 ACL 条目**再参与继承评估，而不是整空间后置否决；服务端另有一条 session-profile 只读路径过滤（upload 前 `0x9004`，reason 含 "read only"）但它是**任务/session 配置**，写侧不在本轮静态集内。三者都无法证明“整个空间禁写且不可被子节点 ACL 覆盖”这一 L2 必要条件。

本轮为 Drive Server 4.0.3-27892 公开软件的有界静态分析，输入与上一轮冻结包同 SHA；**未执行 Synology 程序、未连接 NAS、未读取凭据、未修改产品代码**。新增证据见 [l2-source-002 证据包](../../../../../../work/issue7-native-evidence/l2-source-002/README.md)（MANIFEST/SHA256SUMS/COMMANDS 齐全，工具为本轮新增只读 Capstone 脚本）。真实共享配置与子 ACL 组合行为、其他部署版本均未实测。

## 来源与范围

| 文件 | SHA-256 |
|---|---|
| 官方 SPK（同上一轮） | `58417bbeb433d303b18715649b7bac07efbcef686468378456741eff8ef060fc` |
| `sbin/syncd` | `e2288b191e8db97673c3468595e3a2d441579d2da365927c1108c000549fdf9c` |
| `lib/libsynosyncserver.so` | `1677fcc08004cb9128ebd4efccd0d7053586de87042dd54880bc3e2e124b617c` |
| `lib/libsynosyncservercore.so` | `cfcad81883c49285b34a986eec78dc4702bf8312d4b1d847a07b095fbb93ab3b` |

主范围为本轮新增定位的函数链：`NodeCapabilityImpl::GetByUser`（core `0x2f8ad0`）、`LoadAclByPath`（core `0x2f5330`）、`AppendSharePrivielge`（core `0x2edb60`）、缓存填充 lambda（无符号窗口 `0x2ee4e0`，长 `0xb00`）、`DriveAcl` 评估器（`CalculatePermission 0x25a3c0`、`CheckPermission 0x25a9f0`、`GetCapability 0x25b7a0`）、`FilterChecker`（core `0x25f0a0`/`0x2607f0`）与 `get_profile`/`ProfileHelper`（syncd `0x35eab0`/`0x35e4f0`/`0x24e100`）。所有地址均为上述 SHA 的二进制中的虚拟地址；原件与复现命令见证据包的 `COMMANDS.md`。

## 新增静态事实

### 1. 节点能力里的 `SimpleSharePrivilege` 来自共享权限缓存，且只对 `isShareUser` 读取 SDK 共享权限

`GetByUser` 用缓存元组 `(bool, DriveAcl::UserInfo, SDK::SimpleSharePrivilege)`（缓存模板键类型为 `pair<string, unsigned long>`，见 `0x2f8d5e` 的 `_Rb_tree<...find>`；string 字段语义本轮未展开断言）。缺失时调用缓存填充 lambda：

- lambda（窗口 `0x2ee4e0`）先以字符串键调用 `sdk_cache::UserService::GetUser`（`0x2ee75b`）组装 `DriveAcl::UserInfo`；
- 仅当 `UserInfo::isShareUser()` 为真时，才调用 **`SDK::GetAdvSharePrivilege(share, user)`**（`0x2eea1f`）并把返回值作为 `SimpleSharePrivilege`；
- 否则（普通用户）`SimpleSharePrivilege` 由本地条件推导，取 **2 或 4**（`0x2eec98–0x2eecc7`），不调用 `GetAdvSharePrivilege`。

`UserInfo::isShareUser` 的判定是 `UserInfo+0x3c == 1`（core `0x49c710`，3 条指令）。因此：**`query_user_info` 的 view privilege 由 `GetAdvSharePrivilege` 参与推导（旧证据），但普通 Drive 用户节点能力的 share privilege 并不来自该函数**；两者不是同一条数据流。旧报告“wire read-only 有 SDK 共享配置来源”仍然成立，但不能据此认为节点写门禁已经使用了同源限制。

### 2. 共享权限被物化为根路径 ACL 条目，而不是整空间后置否决

`GetByUser` 在 `0x2f8fbd` 调 `LoadAclByPath(path, node, &privilege, &userInfo, driveAcl, int&, string&)`（`privilege` 即上节的 `SimpleSharePrivilege`，来源随用户类型不同）。`LoadAclByPath` 从请求路径开始逐级 `ParentPath` 到 `/`，逐节点把 `db::Node` 的 ACL/权限 `DriveAcl::AddPrivilege`（`0x2f575a`）；当循环到达根路径（`0x2f57ff–0x2f5810` 与 `"/"` 比较）时，调用 `AppendSharePrivielge(privilege, depth-1, userInfo, driveAcl)`（`0x2f84b6`）。

`AppendSharePrivielge`（`0x2edb60`）先调用 `DriveAcl::IsUnixMode`（`0x2edb8a`）。该函数把参数当作 `DriveAcl` 内规则向量的下标（`0x2575b7–0x2575e4`：条目数 = `(end-begin)/0x80`，越界走日志分支），返回“该条目 `+0x8` 字段为 0”。`LoadAclByPath` 传入的是循环计数 `-1`（`0x2f8491`/`0x2f84b3`），按循环结构对应最后加入的根路径条目；下标/字段语义按原件推断，未取得结构体权威定义。随后按 `SimpleSharePrivilege` 值分支，在 `"/"` 上追加 `ACLRule::addACE` 或 `AddDefaultAcl`：

| privilege 值 | 根条目为 unix-mode | 根条目已有 ACL |
|---|---|---|
| 1（read-only） | `AddACE("/", entry)`，`entry.perm = 全局掩码2`（`0x2edd08`，无 `\|0x1fff`） | `AddACE("/", entry)`，`entry.perm = (全局掩码B\|0x1fff) & ~全局掩码A`（`0x2edc8e`） |
| 2 / 8 | `AddACE("/", entry)`，`entry.perm = 全局掩码1\|0x1fff`（`0x2edbe0`） | **不追加**，直接返回（`0x2edc59`） |
| 4（denied） | `AddDefaultAcl("/")`（`0x2edbad`） | `AddACE("/", entry)`，`entry.perm = 全局掩码B\|0x1fff`（`0x2edd70`） |

所有 `AddACE` 的 `ACLRule::Entry` 按 `{4 字节字段=1, uid, 权限位, 偏移 0xc=6, 偏移 0x10=1 或 0}` 在栈上构造；三次全局掩码经 GOT 读取（`0x505c92`/`0x503b3a` 及 `0x503bcf`+`0x505be0` 附近）。本轮**没有**取得 DSM/`ACL_API` 对这些字段（类型/继承位）的权威定义，见缺失证明。

结论边界：这是“**把共享级结论写成根 ACL 规则**”，随后由 ACL 评估继承/覆盖规则决定子节点结果；不是“在能力计算之后对整空间强制改写成 ReadOnly”。旧证据里 `GetByUser` 的只读投影（`0x2fa674` 比较路径是否为 `/`，`0x2fa68a` 比较 privilege==1，相等则 `DriveAcl::Capability::ReadOnly`，`0x2faae7`）是**额外的 root-only 后置投影**，不是全空间后置否决。另有 `path=="root"` 时直接 `Capability::Full` 的分支（`0x2f9096–0x2f90c6`）。

### 3. ACL 评估器有“当前条目未命中则回退父条目”的结构，但覆盖语义未闭合

`CheckPermission(UserInfo, idx, perm, parentIdx)`（`0x25a9f0`）：`parentIdx==-1` 时取 `idx-1`；对第 `idx` 条规则先看 `rule+0x71` 是否为“已计算”，未计算时调用 `CalculatePermission(user, perm, parentIdx)`（`0x25abc1`），再以 `rule+0x74 & ~rule+0x78 & perm` 判定权限位。`CalculatePermission`（`0x25a3c0`）内部遍历 `ACL_API::ACLRule::Entry`，按 entry 的类型码（原件中比较 `1/2/4/8`，`0x25a630–0x25a645`）与继承相关字节（`+0xb0` 处标志，`0x25a655`）决定是否采纳，并读取父级合并共享权限（`GetMergedSharingPermissions`，`0x25a549`）。

因此可以证明“**评估是逐路径规则 + 父级回退的继承模型**”，但不能仅凭这些原件证明“根上的共享权限 ACE 对子节点一定生效且子节点显式 ACE 不能覆盖”。`ACLRule::Entry` 的类型/继承位含义、`CalculatePermission` 的采纳优先级、以及 `AppendSharePrivielge` 在根上写入的条目属于“显式”还是“继承”都缺少权威结构定义，这是候选 (i) 的**核心缺失证明**。

### 4. profile/session 级“只读路径过滤”真实存在，但它是任务配置而不是权限判定

- wire 上确有 `get_profile` action（路由注册 `syncd` `InitRouteMap` `0x213813–0x213818`，旧包 `dispatch-boundary/function-20b460.asm`）。`GetProfileHandler::ParseParams`（`0x35eab0`）解析 `view_id`、`format`（另有 `sync_task_debug` 调试参数），`format` 非法时 `Response::SetError(0x7000, "format is invalid.")`（`0x35ee5c`）；`Execute`（`0x35e4f0`）从 `ProfileManager` 取用户/视图/session 三组 profile；输出可为 PObject 或文件（`FormOutput 0x364ca0`，另有 `FormPObjectOutput`/`FormFileOutput`）。
- profile 过滤器实现在 `BlackList2`（`libsynosyncserver.so`）：`Profile::TestFilePath`（`0xc8f40`）直接转发 `BlackList2::TestFilePath`（`0xc8070`）→ C 接口 `filter_test_path`；`Load/Write` 走 `filter_read/filter_write`（`0xc7ed0`/`0xc7f90`）。`CloudStation::SyncProfile::Description::Common::ToJson`（`0x1ba2c0`）给出的过滤键是 `black_char`、`max_length`、`max_path`、`black_name`、`black_prefix`、`black_suffix`、`black_dir_prefix`（排除类过滤），没有“整空间只读”字段。
- `FilterChecker` 构造（`0x25f0a0`，参数 `uid, view_id, string, string, FileOperation, bool`）在 `FileOperation==1` 时额外取 `ProfileManager::GetSessionProfileSet(view_id)`；`FilterChecker::TestFileFilter`（旧包 `filter-source/function-2607f0.asm`）对 user/view `ProfileSet` 路径过滤给出 `0x9010/0x9013`（reason "filtered by path"）、size 过滤 `0x9011`（"filtered by size."），对 **session profile** 路径过滤给出 `0x9004` 且 reason 为 `"filtered by path (read only)"`（`0x261879–0x2618af`，长度 `0x1c`；字符串块已按定长重建）。`UploadHandler::TestFilter`（旧包 `filter-boundary/function-4b7da0.asm`）以 `FileOperation=1` 构造并通过 `FilterChecker::Test` 在 `PrepareUploadInfo/CommitUpload` 之前过滤（旧包已证该分支早于 Commit）。
- 但 profile 的**写侧来源不在本轮静态集内**：`syncd` 只在 `ServiceTask::DispatchCGIRequest → HandleReloadProfile`、初始化与长轮询重载时调用 `ProfileHelper::ReloadProfile`（`syncd 0x24e100`）；该函数只调用 `UserManager::GetAllProfiles / GetBackupTasks / GetProfileUserMap / GetProfileViewMap / LinkViewWithInternalProfile` 等 DB 接口与 `ProfileManager` 方法，**没有任何 SDK 共享权限/ACL 调用**。DB 字符串显示 `profile_table(id,name,context,attribute)`、`profile_apply_table(user_id,profile_id)`、`profile_apply_view_table(view_id)`，即 profile 是持久化的任务/session 配置。其创建者（CGI/web API 或其他组件）不在 `syncd`/`libsynosyncserver*` 的已解包范围，profile 中的“read-only 路径”到底是服务端权限判定、Drive 管理端配置还是客户端任务设置无法由本轮静态证据区分。

## 为什么不能接成 L2（缺失证明清单）

对三个候选分别列出缺哪一步证明：

1. **共享只读（SDK 高级权限）**：已证它只对 `isShareUser` 进入节点能力，并被写成根 ACL 条目；缺 (a) `ACLRule::Entry` 类型/继承位与 `CalculatePermission` 优先级的权威结构定义；(b) 子节点显式 ACE 对根条目覆盖/不覆盖的运行证据；(c) `0x9004`/`0xa004` 等提交前拒绝在“根只读 + 子可写”组合下的归属（是 ACL 拒绝还是 profile read-only 过滤）；(d) `GetAdvSharePrivilege` 的 1/2/4 在 `OnlyAcl`、用户名单、用户组名单、失败路径下的完整映射（旧包已部分给出，wire 不携带来源标记）。
2. **profile/任务级只读过滤**：已证它存在且对 upload 在提交前生效（`0x9004` + read-only reason）；缺 (a) profile 写侧与权威性（是否由服务器按当前权限生成）；(b) 作用域是否可表达“整个 view 只读”而不是路径子集；(c) 是否覆盖 remove/rename/其他写动作（构造器只在 `FileOperation==1` 装载 session profile，其他操作是否另有 read-only 过滤未证）；(d) profile 更新/失效与权限变更的关系（`ProfileManager::GetProfileDigest`、`long_poll` 的 `profile_changed` 已有，但“权限变化 → profile 变化”未证）。
3. **整共享只读标志/其他 SDK 接口**：`SDK::Share::IsReadOnly()`（`0x13bb30`，共享属性 `+0x3c` bit14）与 `IsForbidden()`（`0x13b6b0`，bit3）在已扫描的三个二进制中**没有找到调用者**；`IsShareReadOnlyRegisteredBy`（`0x116580`，调用 `SYNOShareAttrReadOnlyIsRegBy`）、`GetPathAclPrivilege`（`0x116ba0`）同样只有自身包装层调用。`isFileWRForbidden`（`0xf6960`）本质是 `access(path, mode)` 的 `EACCES/EPERM` 判定（POSIX），只被 `File::CanRead/CanWrite/CanReadAndWrite`、`hasFilePermission` 调用，调用者不在本轮 ELF 集内。**不能排除这些接口被未解包的库/后台组件使用**，但同样不能据此接线。

另外，L2 的另一个方向（`allowed`）同样不成立：没有证据表明 `GetAdvSharePrivilege != 1` 或根 `can_write=true` 能推出整个 view 可写。

## build 白名单 + view_id 仍不足

- view_id 只绑定对象归属，不携带权限语义；同一 view 内不同子路径可以有不同的有效能力（本轮与旧包都证明能力是逐路径的）。
- build=27892 只锁定“某版本有这段代码”，不证明该版本在任何共享配置/ACL 组合下都执行同一条否决。
- 旧包已证 wire 的 `privilege="read-only"` 可能来自 SDK 共享只读标志或用户/组名单，wire 不区分来源；本轮进一步证明节点能力侧的 share privilege 只对 `isShareUser` 读取该 SDK 值，普通用户路径不同源。
- profile/session 过滤的 `0x9004` 与 ACL 的 `0x9004` 同码异源（reason 文本不稳定，不能作为生产判据）。

## 产品侧决定（建议）

- **不新增 L2 映射，保持 `server_capability=unknown`**；继续用 L1（用户/本地策略）与已有逐动作门禁；不使用根 `can_write=false`、`ShareInfo.writable`、`view_list[].privilege` 推导 `space_forbidden`。
- 现有 `profile_changed` → 逐节点证据回 `unknown` 的失效路径继续保留（它证明的是“远端可能变化”，不足以生成 L2）。
- 若将来要接 L2，门槛建议写成可验收条件：来源必须 (i) 绑定空间身份（view/share）而非节点，(ii) 有服务端对所有写路径统一否决的证据或“逐路径能力枚举 + 全空间覆盖”的可判定契约，(iii) 有明确的更新/失效信号与失败形态（错误/超时不得当成空间只读）。在拿到 T3 实测前不实现。

## 给 T3 的最小 NAS 实验设计（可执行）

补充一项外部实测事实（**实测来源：Lead 只读 File Station 探测，2026-11-05**；不属于本包静态证据）：真实 NAS 已存在名为 `/MFM（停用云盘功能）` 的共享（根 POSIX 只读 555）以及 `/MFM协作_云盘`（测试账号在根为 555、子目录为 777），说明“共享只读角色 / Drive 服务停用 / 根 POSIX 只读+子目录可写”在真实环境可能并存。因此本实验必须**同时**在 File Station/POSIX 侧与 Drive/native 协议侧观测同一路径，不能只凭任一侧推断空间权限。

目的：在真实 NAS 专用测试共享上区分四种解释——**M1 共享级只读（DSM 共享高级权限/只读角色）**、**M2 Drive 服务/空间限制（共享“停用云盘功能”、团队文件夹同步禁用等）**、**M3 逐节点 ACL/POSIX（根 555、子 777/ACL 条目）**、**M4 profile/session 只读过滤（服务端 upload 前 0x9004 read-only reason）**。

### 夹具（管理员准备；测试账号不持有管理员）

| 名称 | 用途 | 关键设置 |
|---|---|---|
| 账号 `A_ro` | 只读侧观测 | 非管理员；专用测试账号 |
| 账号 `A_rw` | 对照组 | 非管理员；对测试共享可写 |
| 共享 `S_NORM` | 正常对照 | `A_ro`/`A_rw` 可写；用于证明账号、探针、网络正常 |
| 共享 `S_RO` | M1 共享只读 | 对 `A_ro` 设只读共享权限（高级权限：只读/自定义只读角色），无子 ACL 提权；根下放 `seed.txt` |
| 共享 `S_CA` | M3 子 ACL 覆盖 | 对 `A_ro` 根只读；子目录 `F_W` 的 ACL/权限给 `A_ro` 完全控制（若 DSM 不允许该组合，记录“无法构造”并用 POSIX 777 子目录替代，并注明这是 POSIX 而非 ACL） |
| 共享 `S_DIS` | M2 服务停用 | 管理员启用 Drive 后对共享执行“停用云盘功能”；记录 Drive 侧是否可见 |
| 共享 `S_PROF` | M4 profile 只读 | 若可构造只读路径 profile（官方客户端任务/管理端设置），构造覆盖根或前缀的 read-only 过滤；否则记为未覆盖项 |

每个共享根预置 `seed.txt`、`F_W/seed2.txt`（由管理员上传）；记录每个共享的共享权限、根与 `F_W` 的 POSIX mode/ACL（File Station API 或管理员导出）、Drive view_id、服务器 build（期望 27892）与 DSM 版本。所有写操作只新建小文本文件并在测试共享内清理；**不触碰生产共享**。

### 操作序列（对每个共享、以 `A_ro` 身份按同序执行；`A_rw` 只做 2–4 的对照）

1. **File Station 层（同账号）**：`SYNO.FileStation.List`（带 `additional=["perm","real_path","size","owner","time"]`）记录根与 `F_W` 的 permission；对根与 `F_W` 各上传一个小文件，记录错误码/错误文本与是否落盘。
2. **Drive 视图层**：Drive web API `query_user_info`；记录该共享是否出现在 view_list、`view_id`、`privilege` 字符串（read-write/read-only/denied）与顺序。
3. **原生协议层**（产品 sproto 探针或官方客户端；WSL 内）：`list` 根与 `F_W`（读通路必须成功，作为对照组）；`upload` 新文件到根、到 `F_W`；`remove` 预置文件；`rename` 预置文件。对每个拒绝记录 **action、error.code、error.action、reason、响应阶段**，随后 `list` 验证是否已 commit。
4. **（可选，若时间允许）能力/profile 探针**：原生 `get_file_capability(view_id, "/")` 与 `(view_id, "/F_W")`，记录全部 capabilities 字段；`get_profile(view_id, "pobject")` 记录 filter 段与 hash；同时记录 `long_poll` 的 `profile_digest`/`profile_changed`。
5. **权限变更后复测**：管理员把 `F_W` 的 ACL 从完全控制改为只读（或反之），`A_ro` 重跑步骤 3 的 upload/rename，观察 `profile_changed`、能力缓存失效与错误码变化。
6. **对照组**：`A_rw` 对 `S_NORM` 执行同套 upload/remove，必须成功，用于排除探针/协议/夹具问题。

### 判据（如何区分）

| 观测组合 | 解释 |
|---|---|
| File Station 上传在根失败且 `F_W` 也失败；Drive 拒绝 reason 带 read only 或能力 `can_write=false` | 共享/文件系统级（M1/M3）；若 `F_W` 的 File Station 可写而 Drive 仍拒，转下一行 |
| `S_CA` 根 Drive 写拒绝，但 `F_W` Drive upload **成功**（随后 list 可见） | **子 ACL/POSIX 覆盖 → 共享只读不是不可覆盖的全空间限制**（M3；这是 B 的关键否证实验） |
| `S_CA/F_W` Drive upload 失败，错误 `0x9004` 且 reason "filtered by path (read only)"，`get_file_capability(F_W).can_write=true` 或 File Station 允许写 | 拒绝来自 **profile/session 只读过滤（M4）**，独立于 ACL；此时记录该路径是否在 profile read-only 范围内 |
| `S_CA/F_W` Drive upload 失败，错误 `0x9003/0x9004` 且 `get_file_capability(F_W).can_write=false`，File Station 也拒绝 | 节点 ACL/能力拒绝（M3），非 profile |
| `query_user_info` 中该共享不可见或整体 denied，但其他共享正常 | M2 服务/空间停用（"停用云盘功能"）；记录 Drive 客户端显示与 native `list` 结果 |
| `A_ro` 在 `S_RO` 根写被拒、在 `S_CA/F_W` 写成功且 `query_user_info` 显示 read-only | view privilege=read-only 非不可覆盖全空间限制（直接否证旧候选 i；供 B 结论与后续 L2 设计） |
| 权限变更后 `profile_changed=1` 且 digest 变化，但错误码/能力未变 | 更新信号存在但语义不足；记录为失效契约未闭合 |

### 记录与安全要求

- 每条观测记录：时间、服务器 build、DSM 版本、账号、共享、view_id、请求字段（path 原样）、响应 code/action/reason、前后 `list` 结果、File Station permission、管理员设置变更点；不记录凭据、不导出生产路径。
- 需要管理员：共享创建、共享权限/ACL 设置、POSIX mode、Drive/团队文件夹启用与“停用云盘功能”、预置 seed 文件、（可选）只读 profile 配置。
- 非管理员 `A_ro`/`A_rw` 可做：File Station 查询/上传、`query_user_info`、native list/upload/remove/rename、官方客户端观察、（若探针可用）能力/profile 查询。
- 安全：只用一个专用测试共享集合；写操作限小文件并在测试内清理；失败不重试放大；不把 NAS 地址/账号写入证据包可公开部分。

## 未闭合项

- `ACLRule::Entry` 的类型/继承位权威定义与 `CalculatePermission` 的采纳优先级（决定根 ACE 是否可被子 ACE 覆盖）——**L2 候选 (i) 的最后一步**。
- profile 写侧（谁、按什么权限生成 read-only 路径）不在本轮解包集内；`get_profile` 的实际响应、`profile_changed` 与权限变更的因果未实测。
- `remove/rename` 等非 upload 动作是否也应用 session profile 只读过滤未证。
- 真实共享高级权限（只读角色、OnlyAcl、ACL 组合）与错误码归属未实测，本报告不替代 T3。
- 其他 Drive Server 版本、SPK 内未解包组件（web API/后台任务）未覆盖。

## 附件

- 证据包：[`work/issue7-native-evidence/l2-source-002/`](../../../../../../work/issue7-native-evidence/l2-source-002/README.md)（MANIFEST、SHA256SUMS、COMMANDS、符号清单、函数原件、xref）。
- 上一轮结论与原件：[l2-static/REPORT.md](../l2-static/REPORT.md)。
- 提交前 ACL 路径与 profile 过滤边界：[server-static/REPORT.md](../server-static/REPORT.md)。
- 原生能力查询身份边界：[server-capability/REPORT.md](../server-capability/REPORT.md)。
