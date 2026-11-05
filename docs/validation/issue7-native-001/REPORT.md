# Issue #7 原生反馈增量

本轮起点 `3d35d7b`，保留之前 `bae1ef1` 的 G4/G5 原件。仅补 #7；schema25、entry TTL=0、attr 默认1000ms 不变。不连接 NAS、不读凭据、不 push、不发布。

## M1：原生权限变更通知

原生解析器已有 `profile_changed`，旧 worker 丢失该字段，只传文件事件标记和 digest。现在 IPC21 明确传递该字段；当前挂载根的逐节点缓存证据随 profile 变化回 unknown、清除 verified_at 并递增 revision。无文件事件也执行；普通通知不失效。用户只读、明确空间限制、其他挂载根与未确认操作意图不变。既有权限读取失败的重试门禁随证据失效解除，属性缓存仍使用原 TTL 上限。

作者检查：`cargo check --workspace --all-targets --all-features --locked --offline -j2`，退出0，用时28.87s；原件 [check-001.log](check-001.log)。新增 core 与真实 UNIX IPC 回归用例尚未运行，等待独立验证。

未验证点：真实 NAS profile/ACL 通知行为、原生拒绝码的逐操作映射、空间能力来源、File Station 辅助探测、元数据独立写回，以及 IPC21 成对安装。真实 NAS 验收仍需用户在专用共享执行。
