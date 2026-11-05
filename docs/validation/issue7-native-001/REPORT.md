# Issue #7 原生反馈增量

本轮起点 `3d35d7b`，保留之前 `bae1ef1` 的 G4/G5 原件。仅补 #7；schema25、entry TTL=0、attr 默认1000ms 不变。不连接 NAS、不读凭据、不 push、不发布。

最终产品冻结 `9c9aaba` 的[独立G4-012与补测](g4-final-012/REPORT.md)通过：all-features976/0/42、default966/0/37，debug/release两种索引路径与attr0内核均通过，GUI148、210k容量1、真实本机worker链10、组件数据保留8通过。各组不可相加。旧41365a6真实worker的两项读取拒绝后重试反例在同断言下转绿，旧失败保留。本轮本地开发版本为`0.1.0~dev20261105.1`；包生成和实际内容检查见同交付`REPORT.md`，不把组件替换称为dpkg生命周期。

[G4-006–010独立报告](g4-through-010/REPORT.md)逐组列出通过/失败/忽略、命令与失败原件。冻结包含221项清单文件，另含MANIFEST，共222个文件；MANIFEST SHA256 `37109309399b943f2b942ef26b49c5d284e903052182b5bda1f9f7f633e19bb9`。故障前已有hash的146项全部匹配，无缺失或不一致。没有历史hash的原件只声明本次复制完整性。010复制stderr未完整落盘且工具截断，报告没有伪造完整trace。

## M1：原生权限变更通知

原生解析器已有 `profile_changed`，旧 worker 丢失该字段，只传文件事件标记和 digest。现在 IPC21 明确传递该字段；当前挂载根的逐节点缓存证据随 profile 变化回 unknown、清除 verified_at 并递增 revision。无文件事件也执行；普通通知不失效。用户只读、明确空间限制、其他挂载根与未确认操作意图不变。既有权限读取失败的重试门禁随证据失效解除，属性缓存仍使用原 TTL 上限。

作者检查：`cargo check --workspace --all-targets --all-features --locked --offline -j2`，退出0，用时28.87s；原件 [check-001.log](check-001.log)。该里程碑只编译，独立验证见下文。

未验证点：真实 NAS profile/ACL 通知行为、原生拒绝码的逐操作映射、空间能力来源、File Station 辅助探测、元数据独立写回，以及 IPC21 成对安装。真实 NAS 验收仍需用户在专用共享执行。

## M2：原生响应分阶段分类

下载仅在 sproto `recv_download` 边界识别官方客户端已核实的 0x3002/0x3008，产生 DownloadPermissionDenied；worker 将它绑定当前文件 Read，经 IPC 持久记录 observed 拒绝并返回 EACCES。前置认证/连接的普通 Server 错误不转换；0x3004、0x8004、上传码、网络与本地 I/O 均不进入该路径。超出 u32 的原始错误码拒绝解码，避免截断后误分类。权限拒绝与下载失败重试门禁在同一 Store guard 发布，避免通知夹在两者之间留下过时终态失败。

mutation 的直接响应只保留 action-scoped 权限提示：upload/remove/rename 的 0x3002/0x3004，upload 的 0x9003/0x9004，remove 的 0xa003/0xa004。选择性同步码 0x9005/0xa005 排除。在本里程碑时尚无服务端无提交证据；后续 M3 调查也未把这些码证明为可唯一归属的节点动作拒绝。因此保持 uncertain/cleanup_uncertain，禁止自动改回 pending，禁止据此更新 directory_permissions 或 L2。历史显示权限相关响应与提交待核对，保存数字码和请求类型；不保存服务器 reason。后续确认清除错误提示，detail.permission_response 作为历史尝试证据保留。

新增 mkdir/replace 历史生命周期触发器与中文动作名，使这些权限诊断能在核对完成/冲突后更新；没有新增表或改变 schema25。普通失败、字段不足和任意未识别数值仍保留原流程。

检查原件：[check-002](check-002.log) 为中间读取分类编译通过；[check-003](check-003.log) 保留新增 error variants 导致 setup 穷尽匹配遗漏的失败原件，修复后 [check-004](check-004.log) 通过。只读复核另发现通知后旧请求迟到的恢复竞态，加入请求 revision 守卫，并在旧 flight 移除后唤醒只读重试；[check-005](check-005.log)、最终 [check-006](check-006.log) 全量检查通过（最终18.78s）。本里程碑由作者只编译，测试结果另列。

数字码范围、排除项、官方函数地址和本地二进制摘要见[客户端静态证据](static/REPORT.md)。该证据只证明客户端码语义；后续服务端证据单独保留，不把静态分析当作 NAS 端到端验收。

## M3：父目录自身权限与子项动作分离

从官方公开 SPK 静态提取软件（没有执行 Synology 代码）取得 `syncd` 与 core 库。特定 ACL middleware 拒绝在业务处理器前发生；0x9004/a004 同时存在 profile 只读路径过滤来源，不能直接记录为目录 ACL。`get_file_capability` 的 header.capabilities 是已有路径能力快照，没有原子身份回包；安全的逐节点持久接线条件未满足。完整来源、函数与哈希见[服务端证据包002](server-static/REPORT.md)和[能力/身份补充001](server-capability/REPORT.md)，软件二进制没有提交到仓库。

有效 Delete 已包含父链 ACL；目录自身 Delete/RenameSource 的拒绝不能递归为孩子的拒绝。`99aa587` 修正 FUSE 和 namespace 调度对象，去掉 rmdir 对父 CreateDir 的错误要求，目录 mode 保留原 POSIX 写位，防止 default_permissions 提前误拒独立可删子项。新增真实内核与持久队列回归，保留父 Traverse、源/目标动作拒绝检查。作者仅 [check-008](check-008.log) 全量编译，退出0（30.39s）；独立最终验证另列。

后续[空间只读来源复核](l2-static/REPORT.md)追踪四个 view 入口及必要调用链：精确 read-only 可以来自 SDK 共享只读标志或用户/组名单，但 wire 不区分来源；节点只读投影还受 ACL 计算输出路径是否为 `/` 的条件控制。未证明其为不可被子节点能力覆盖的全空间限制，不能仅以构建号白名单和同 view_id 升级为 L2。故本轮保持 Unknown，无新增产品改动。补充29个文件，28项SHA核验一致；MANIFEST SHA256 `19ec9aa70de2b8aa205e5034e9da05c10e669c03f785f53c6b48dcabbbf63584`。这不是对真实子 ACL 覆盖行为的实测结论。

## 独立反馈与修复

G4-006 在冻结 `d39a597` 运行迟到读取拒绝回归：0 通过 / 1 失败 / 0 忽略，144 filtered。原断言 left=Deny、right=Unknown；日志 SHA256 `7a4e78d4ffa75f8188e109b9a0238d5d4a49236651f59c1ffc98f9e20bbf4b03`。新 Store 尚无 write_policy 行时，首次失效只 UPDATE 而未递增 revision，旧拒绝因此被重新学习。

`ca86a85` 在失效事务中 INSERT OR IGNORE 策略行再递增；已有用户/服务策略保持不变，没有调整失败断言。[check-007](check-007.log) 全工作区检查退出0（33.77s）。G4-007 初次运行误复用006的旧 drive-core 缓存，原日志/旧ELF/中断原样保留，不能当作 ca86 产品失败。独立验证随后清理隔离 target 的全部产品包并重编，源码内容哈希未变；fresh runtime 144通过/0失败/1忽略，原迟到拒绝断言通过。后续归档强制重编产品包。

G4-007/fresh 全 workspace 964通过/1失败/41忽略，失败为旧 schema6 夹具遗漏删除新的 history_permission_namespace_extra 触发器，导致 ALTER TABLE DROP kind 构造旧表时出现 `no such column: NEW.kind`。`4e6aaf7` 在该夹具及两个 schema23 夹具补删这个当时尚不存在的触发器，全部原断言保留；[check-009](check-009.log) 退出0（32.91s）。原99aa的 G4-008 尚未执行即撤下，最终4e6aaf7重新归档验证；原失败和未启动记录均保留。

全部作者检查与原始日志摘要见 [CHECKS.json](CHECKS.json)；含NUL字节的原始日志保留原字节，不清洗失败或成功原件。

## M4：下载退避的时钟与版本边界

G4-009 在4e6aaf7的 all-features workspace 为964通过/2失败/42忽略。两个既有并发读取用例在 `first.elapsed() >= 900ms` 原断言失败，随后假服务端退出引起其他读取线程连带错误。原ELF单跑两项通过；这些复跑不覆盖整组原失败。

独立诊断只在另一份源码增加计时/请求类型输出，原断言保留：2项并发与完整 runtime 分别2/0/0、144/0/1。6个第二包都是 Download，attempts=1、retry_at=failed_at+2；未复现早到。同窗1434点相邻双钟采样在单调时间12.289246558s时观察到 realtime 相对mono后跳22.386938317s，在12.552712513s时前跳22.383726646s，约0.263466s后恢复。异常点读取括号约2µs；全部重试诊断在前约6s完成，**这不证明原009失败由该次跳变造成**。原失败直接根因仍未确定，原件和派生补丁分别保留。

静态检查另确认原产品退避只有墙钟最短截止，Instant只限制35s总等待，运行中墙钟前跳可以缩短等待。41365a6补上共享的单调最短门限，覆盖前台/后台、新flight与重连；失败发布和readiness同时校验ticket版本，旧请求不覆盖新版本门限。新增4项可控双钟用例和1项真实Store旧ticket回归。原900ms阈值保留，诊断耗时在获取Store锁之前截取，避免日志工作掩盖早到。

作者[check-010](check-010.log)中间编译退出0（17.54s）；补齐旧ticket隔离后[check-011](check-011.log)退出0（14.47s）。最终独立验证使用新的41365a6归档，不将009其他通过组移作新候选结果。

## G4-010：环境阻塞，尚未测试

41365a6归档SHA为 `e0ac9b59fb6111129dece4dc15f10501160a9e9c49462d72ddaadbf742693ec3`。复制独立target时首先出现I/O error，随后Read-only file system；clean、构建与测试均未开始，测试数为0，不计产品失败或通过。宿主C盘同时实测仅余约52MiB，而guest df仍显示数百GB可用；两者不是同一容量。没有执行fsck或强制remount。

核对source.json的Git提交与SHA后，仅删除本轮Windows侧5份可再生candidate.tar，共215,050,240字节；源码、日志、清单、截图与hash全部保留，归档可按原命令重建。随后标准只读WSL调用恢复可访问，根挂载报告rw；又仅移除G4-010从未使用的复制半成品target，保留该轮源码/日志和此前所有原件。删除guest缓存没有立即收回VHD的宿主占用，C盘仍只有约276MiB。

用户明确授权停止Ubuntu-24.04并运行内置 `wsl --manage Ubuntu-24.04 --compact`。首次压缩退出0，VHD由532,881,080,320降至509,104,619,520字节，宿主可用空间恢复到约22.4GiB。随后只清理本项目9个 `target/debug/incremental` 缓存目录，核对解析后的绝对路径、无符号链接/挂载、无cargo/rustc进程；保留全部测试ELF、源码、日志、清单及失败原件。去重guest分配量为78,196,764,672字节，这不是承诺的宿主回收量。再次sync、停止同一发行版并压缩，退出0，VHD降至448,729,710,592字节，宿主实测可用84,682,522,624字节（约78.9GiB）。

验证runner增加宿主C盘门禁：启动前至少40GiB，每组命令前至少10GiB，不足或读数失败就记录未运行并停止；guest df不替代宿主读数。G4-011继续使用冻结41365a6，新建独立target，只复制registry依赖并强制重编产品包；`CARGO_INCREMENTAL=0` 只控制编译缓存，原优化profile、测试并行及断言不变。冻结包的harness保留各原轮版本，新轮使用 `work/issue7-validation/run-g4.py`。G4-010仍为0项测试的环境中断，不追改其历史原件。

## M5：已知读取拒绝约束后台下载

G4-011在41365a6的all-features workspace为971通过/0失败/42忽略，default为961/0/37；新增双钟与旧ticket5项、原退避2项及完整runtime149/0/1均通过。补充真实worker链时独立审查发现：typed Read拒绝虽记录Deny，通用fail_download仍产生自动重试截止，后台refresh_cached及Bridge新请求没有再次执行Read判定，可能在到期后重新下载。011的后续组因此暂缓，已有通过与未运行分别保留；新反例和最终矩阵需单独记录。

`2dafd12` 将明确的Read拒绝保存为无自动截止的终态失败；调度就绪、后台需求、begin、下次唤醒以及Bridge缓存/新flight/退避等待均受当前Read及祖先Traverse约束。仅清除传输失败不清除权限证据；profile/重连/重新核对的原revision失效机制保持。新增core2项、Bridge1项回归，并加强原拒绝用例的终态断言，不删除或放宽原断言。

作者仅运行[check-012](check-012.log)，全量cargo check退出0（31.73s）；新增回归随后在最终G4-012通过。工作分支已将`codex/fix-7`从bae1ef1安全快进至集成历史后继续，原集成分支保留，不改写提交。schema25、IPC21及TTL均不变。

旧41365a6的真实本机TLS服务→worker ELF→IPC→Bridge补测已实测复现：3002和3008两项均首次EACCES并学习Read=Deny；立即再读为EAGAIN，到原retry_at后后台实际下载次数从1变2，attempts也为2。两项0通过/2失败/0忽略，server错误为空且进程/线程/socket清理完成；不是TLS或harness未启动。其余8项通知、迟到Read与非权限码补测通过。完整原件保留在新的独立证据包，不覆盖原失败。

独立静态审查又发现取消与权限检查相遇时的早退清理缺口。`9c9aaba` 在原先取消检查之外，权限检查失败后先释放Store，再重新核对flight全部需求；仅无reader/keep/pin/缓存需求时清理当前ticket，否则保留原拒绝和意图。新增确定性入口和实际backoff交错两项，不以单纯换序代替并发守卫。[check-013](check-013.log)全量编译退出0（15.73s），独立只读审查核对锁序和需求保留。G4-012使用该新归档（SHA256 `81e6cf72a6858d53adf038ed255a3666347870bf8b5c1e4c3e94bff79d41e7d1`）完成最终矩阵与补测。

用户追加要求本地新deb：changelog版本为`0.1.0~dev20261105.1`。显式离线文档模式内置权限/历史报告、契约和操作说明，其余材料指向同交付的源码归档路径；保留原默认发布链接模式。两项文档回归已由012独立预检通过。以产品字节与9c9aaba一致、包含最终报告的完整源码commit构建新包；实际构建日志、包内验证和摘要由同交付报告维护。不push、不发布、不安装到宿主或生产任务。
