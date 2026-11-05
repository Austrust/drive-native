# Issue #7 原生反馈增量

本轮起点 `3d35d7b`，保留之前 `bae1ef1` 的 G4/G5 原件。仅补 #7；schema25、entry TTL=0、attr 默认1000ms 不变。不连接 NAS、不读凭据、不 push、不发布。

## M1：原生权限变更通知

原生解析器已有 `profile_changed`，旧 worker 丢失该字段，只传文件事件标记和 digest。现在 IPC21 明确传递该字段；当前挂载根的逐节点缓存证据随 profile 变化回 unknown、清除 verified_at 并递增 revision。无文件事件也执行；普通通知不失效。用户只读、明确空间限制、其他挂载根与未确认操作意图不变。既有权限读取失败的重试门禁随证据失效解除，属性缓存仍使用原 TTL 上限。

作者检查：`cargo check --workspace --all-targets --all-features --locked --offline -j2`，退出0，用时28.87s；原件 [check-001.log](check-001.log)。新增 core 与真实 UNIX IPC 回归用例尚未运行，等待独立验证。

未验证点：真实 NAS profile/ACL 通知行为、原生拒绝码的逐操作映射、空间能力来源、File Station 辅助探测、元数据独立写回，以及 IPC21 成对安装。真实 NAS 验收仍需用户在专用共享执行。

## M2：原生响应分阶段分类

下载仅在 sproto `recv_download` 边界识别官方客户端已核实的 0x3002/0x3008，产生 DownloadPermissionDenied；worker 将它绑定当前文件 Read，经 IPC 持久记录 observed 拒绝并返回 EACCES。前置认证/连接的普通 Server 错误不转换；0x3004、0x8004、上传码、网络与本地 I/O 均不进入该路径。超出 u32 的原始错误码拒绝解码，避免截断后误分类。权限拒绝与下载失败重试门禁在同一 Store guard 发布，避免通知夹在两者之间留下过时终态失败。

mutation 的直接响应只保留 action-scoped 权限提示：upload/remove/rename 的 0x3002/0x3004，upload 的 0x9003/0x9004，remove 的 0xa003/0xa004。选择性同步码 0x9005/0xa005 排除。服务端无提交边界未证明，故保持 uncertain/cleanup_uncertain，禁止自动改回 pending，禁止据此更新 directory_permissions 或 L2。历史显示权限相关响应与提交待核对，保存数字码和请求类型；不保存服务器 reason。后续确认清除错误提示，detail.permission_response 作为历史尝试证据保留。

新增 mkdir/replace 历史生命周期触发器与中文动作名，使这些权限诊断能在核对完成/冲突后更新；没有新增表或改变 schema25。普通失败、字段不足和任意未识别数值仍保留原流程。

检查原件：[check-002](check-002.log) 为中间读取分类编译通过；[check-003](check-003.log) 保留新增 error variants 导致 setup 穷尽匹配遗漏的失败原件，修复后 [check-004](check-004.log) 通过。只读复核另发现通知后旧请求迟到的恢复竞态，加入请求 revision 守卫，并在旧 flight 移除后唤醒只读重试；[check-005](check-005.log)、最终 [check-006](check-006.log) 全量检查通过（最终18.78s）。独立测试尚未启动；不能将本段当作验收通过。

数字码范围、排除项、官方函数地址和本地二进制摘要见[静态证据](static/REPORT.md)。该证据只证明客户端码语义，不能证明真实服务器无部分提交，也不是 NAS 端到端验收。
