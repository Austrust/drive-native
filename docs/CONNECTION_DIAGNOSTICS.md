# 连接失败与 worker 清理前诊断

2026-10-31，接收空闲期限分支的后续实现，已随`.3`发布（IPC13），状态数据库schema18不变。当前整合源码新增[初始化进度](INITIALIZATION_PROGRESS.md)后使用IPC14；诊断文件契约不变。没有修改正在运行的基线。

监督器现在在关闭IPC、终止或回收worker之前，保存失败阶段、固定错误类别、恢复分类和子进程当时状态。阶段区分启动、认证、配置、绑定范围、初始索引、续接事件、拉取事件与通知等待。`running`表示采样时子进程仍存在，不能将之后清理产生的SIGKILL当作原始故障。`exited`记录已观测的退出码或信号；无法查询时明确记录失败，不猜测。

协议接收空闲超时、获取通知通道超时、请求发送超时、普通协议响应超时、IPC空闲、IPC排队失败和IPC断开分别保留。会话失效及服务器数字错误码亦可区分。重连策略沿用原有错误类别判断；下载超时仍进入传输重试，不因更细诊断而变成内容校验失败。普通重连不因此触发全目录恢复。

每个任务数据目录保存`connection-diagnostics.json`，采用0600临时文件、文件fsync、原子替换和目录fsync。仅保存最近一次连接结束及最近一次故障，空间有界；正常暂停/停止不会覆盖上一次故障。读取限16KiB，拒绝符号链接、非普通文件、未知schema和损坏内容。持久化失败仍保留内存诊断、标出`persistence_failed`，并输出只含固定字段的结构化事件；不把失败保存标成已持久化。该文件不是长期验收日志的替代品。

本地D-Bus `GetOverview`增加`connection_diagnostics`字段，包括`schema`、`last_transition`、`last_failure`、`load_failed`和`persistence_failed`。查询读取内存快照，不扫描磁盘或访问NAS。诊断IO不持有Store或控制状态锁。记录不包含账号、凭据、目录、游标、报文或原始错误字符串。

## 尝试级业务心跳与 `attempt_lifetime_ms`（issue #58）

`attempt_elapsed_ms` 一直是**整次 worker 在线尝试的寿命**，不是单个 RPC 的阻塞时间；单请求期限另有限制。为避免继续被误读为“某次调用卡了数小时”，新记录同时写入同值的 `attempt_lifetime_ms` 别名，`attempt_elapsed_ms` 保留兼容。旧诊断文件没有该字段时反序列化为0，不影响读取。

监督器为每次尝试维护业务心跳 `last_business_progress_at`（Unix ms）：只有 cursor 推进、实际应用事件、索引单元推进或会话建立/快照安装才刷新；通知 keep-alive 字节和“通知变化但未应用事件”都不算。当尝试已存活超过配置下限**且**业务心跳静默超过配置窗口**且**仍有未完成的追赶/初始化工作时，判定为 `attempt_stalled`：

- `last_failure.cause.kind = "attempt_stalled"`，`stalled_reason = "no_business_progress"`，并携带 `last_business_progress_at`；即使看门狗通过关闭连接触发重建，也不会把原因记成普通传输错误。
- 重建按 1/2/5 秒有界退避，滚动1小时内最多3次；第4次进入用户可见的 `needs_attention`（`error = "worker_stalled"`）。重建只重连传输，不重置持久化游标、目录或队列。
- 初始化的 D-Bus 投影同时暴露 `catching_up`、`catch_up_budget_ms`、`budget_remaining_ms`、`last_progress_age_ms`、`last_business_progress_at_unix_ms`、`backlog_estimate` 与最近一次 `no_progress` 事件；追赶预算耗尽后 `catching_up=false`，但缓存读取与增量追赶继续。

超时全部可配置（缺省值见括号）：`DRIVE_NATIVE_CATCHUP_BUDGET_MS`（600000）、`DRIVE_NATIVE_NO_PROGRESS_TIMEOUT_MS`（120000）、`DRIVE_NATIVE_NO_PROGRESS_STREAK`（3）、`DRIVE_NATIVE_ATTEMPT_STALL_MIN_ELAPSED_MS`（1800000）、`DRIVE_NATIVE_ATTEMPT_NO_PROGRESS_TIMEOUT_MS`（600000）。

本地合成验证：短超时事件循环测试覆盖预算耗尽→`catching_up=false`+backlog+结构化 `initialization.no_progress`、缓存可读、增量追赶继续；业务心跳测试覆盖 keep-alive/`changed_without_advance` 不刷新心跳；100k 单调推进（tracker 层模拟 + 真实 store 批次）不触发看门狗。**未做** 24 小时实网运行与真实 NAS 大 backlog 验证。

## 验证范围

- 完整Rust388项通过、0失败、9项默认忽略；8项FUSE另行显式通过，剩余为文档示例。
- GUI49项分别在独立Qt进程通过。第一次合并测试模块时因多个QApplication冲突而中止，保留失败记录；不是产品GUI回归失败，也不是一次完整成功运行。
- 实际本地子进程验证认证拒绝、初始索引失败、通知接收空闲及IPC断开。确认保存阶段、重连/需处理分类，以及清理前进程仍存活；完整缓存继续可读。
- 暂停长轮询记录为主动中断、不创造故障、不重置游标；运行中与自行退出23的子进程状态分开验证。
- 重启保留诊断，正常中断不覆盖故障；坏文件、超限文件和符号链接拒绝；模拟保存空间不足时旧完整文件保留、内存状态标明保存失败。不是P3真实断电或磁盘满验收。

独立发布二进制随后纳入`.3`开发包。三范围只读实网窗口已结束，详见[通知验证](NOTIFICATION_IDLE.md)。每范围181个本地样本均在线且目录可读，MFM与Personal_Sync已有缓存可读；培养没有现成缓存，未验其正文。窗口末主动暂停后挂载与已有缓存仍可访问，诊断为`requested_interruption`，保留`last_failure=null`，没有将正常取消记成新故障。

三个开发服务最后均正常退出，停止合计58.583秒；未发生强杀，但退出耗时仍明显，不能因此关闭既有停止/句柄排空缺陷。采样存在最大约10.81秒间隔，结果不证明间隔内绝无短暂异常，也不是性能验收。原六个基线和P0观察器保持。最终数据安全、性能和发布要求见[验收门槛](../DELIVERY_GATES.md)。
