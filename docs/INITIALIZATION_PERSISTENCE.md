# 持久初始化与生产调度器

本开发分支使用 **SQLite schema20、内部 IPC17**。生产 `Service` 已按任务身份配置 FSD 初始化调度器，分步原生读取、持久候选恢复与写回门禁已进入源码。**本轮新增回归尚未运行，不能据此声明测试通过、真实性能改善或新包交付。** 已发布基础版本的历史验证见[包验证007](validation/native-package-007/REPORT.md)，不替代本轮验证。

逐目录提前向 FUSE 显示仍未实现；权威目录树继续在完整候选通过独立 fence 后一次提交。真实 NAS 恢复、性能、升级及最终数据安全验收仍开放。

## 存储与事务

`initialization_run` 绑定任务、所选根范围、服务器恢复身份、起点 S0 和预期的权威游标；`initialization_entries`、`initialization_directories` 保存候选节点与直接子目录覆盖，`initialization_events` 保存按序接收的事件批次。候选表不直接向 FUSE 提供可见元数据，也不把部分列表写入权威节点表。

每项候选变更在 SQLite 事务中核验身份、代次及预算，成功提交后才推进调度。节点数、序列化字节数、事件批次数和事件字节数均有上限；超预算返回错误，失败候选保留写回门禁及原权威内容，不悄悄丢弃事件或把未完成目录标成完成。

| API族 | 约束 |
| --- | --- |
| `begin_initialization` / `begin_initialization_verified_restore` / `resume_initialization` / `restart_initialization` | 不隐式覆盖活动候选；区分同一会话身份下续接、已验证恢复身份变化和显式重新建立候选。任务、所选范围及权威基准不可任意替换。 |
| `begin_initialization_directory` / `store_initialization_directory` / `store_initialization_denied` | 只接收完整、已核验的直接子项列表。相同回执与正文重复为幂等；复用回执但正文不同则拒绝。所选根不可被标为无权限子树。 |
| `record_initialization_events` / `pending_initialization_events` / `apply_initialization_events` | 接收与应用游标及序列分别保存。接收不等于应用；事件引起的结构变化使相关目录覆盖或根核对失效。 |
| `begin_initialization_root_refresh` / `refresh_initialization_root` | 当前根回应须对应请求前创建的票据，不能由迟到响应满足新请求。 |
| `begin_initialization_fence` / `confirm_initialization_fence` / `complete_initialization` | fence 对应当前候选边界；完整覆盖、事件全部应用及根核对满足后，才可原子安装权威快照。 |

## 原生分步读取与可靠交付

worker 接受 `Seed`、`Root`、`Directory { node, path }` 和 `Pull { cursor }` 四种只读请求。回复绑定该认证会话的 scope 和 restore，通过可靠请求／响应及文件描述符传输 JSON 正文；序列化上限为512 MiB，接收方校验文件类型、大小、摘要和会话身份。请求不接受传入正文 FD。可合并或遗漏的 `IndexProgress` 只用于显示，不传递候选数据。

worker 不维护一个“发送后即提交”的隐式游标。FSD 将收据事务落盘，再执行依赖该结果的下一步读取或游标推进；同批最多四个互不依赖的目录读取可以并发，其票据先持久保存。因此不需要再增加一个远端 ACK 协议。回复丢失或进程重启后，可从持久的 received/applied 位置重新发起显式读请求；连接身份、run、epoch、票据及回执校验防止旧回应误提交到新任务。

新候选的 `Seed` 先取得已验证的当前水位，再核对根。对非根路径，`Root` 直接使用现有 `current_selected_node` 合同检查 node ID、file ID、名称、目录类型与存活状态，不额外取新水位。仅所选路径为 `/` 或明确的父列表12290拒绝时保留历史种子回退；该回退仍可能从零读取历史，不把它描述为常数成本。

目录读取校验节点版本token、所选根、restore、路径与父身份字段，然后调用既有 `list_dir`。深层路径从候选父链生成；原生列表不是已验证的原子身份寻址接口，因此列表之后仍必须处理事件并通过独立 fence。符号链接被过滤；明确12290可将子目录记为 denied，但根拒绝不会安装为空树。

## 恢复、票据与游标重置

票据绑定 `run_id`、进程代次 `epoch` 和请求 `generation`。目录请求使用 run 内单调分配的代次，避免同一身份删除后重建使旧回应重新有效。根核对与 fence 使用各自一次性票据；后续相关变化使旧回应失效。

重新打开 Store 会推进未完成候选的 epoch，撤销旧根／fence 票据及同代游标重置许可；未完成的 loading／failed 目录回到待查状态，已完整接收的目录与事件回执保留。失败候选仍阻止写回。当前协议没有已验证的跨会话分页快照承诺，持久覆盖单位是完整目录列表，不能从中间页 offset 直接续读。

同一 restore 下恢复已有候选时，owner 使用原 binding 恢复，再创建根票据并发出 `Root`，**不重新发 Seed，不替换旧 S0、received 或 applied**。先应用已接收但未处理的事件，再从持久 received 继续读取。根变化需要重新核对，不能因候选落盘就跳过会话验证或 fence。

同一 restore 下最终提交通常必须追平原权威游标。如果新候选起点低于旧权威游标，owner 对该旧游标发起真实 Pull；只有当前认证会话明确返回 `native_cursor_reset`，才调用 `authorize_initialization_cursor_reset`，记录绑定 run epoch 和原游标的许可。较低 S0 本身不是重置证据。重新打开或恢复候选会撤销许可；恢复身份变化则走单独的已验证路径，不放宽普通 `bind_native_scope`。

## 有界调度与会话切换

每轮最多追赶16页事件后安排目录工作，避免全局游标持续前进使目录长期饥饿；终点仍要求一次独立、无推进的 fence。目录每组最多四路并发读取，FSD owner 逐一核验票据并提交回执；事件应用与这些有界请求组交错执行。已失效的同组回应不会覆盖较新的候选状态，局部目录碰撞只有限重试该目录，不自动把普通错误改成整树重试。

pending 查询使用仅覆盖 `pending/failed` 行的部分索引，并沿候选父链选取尚未完成的最上层祖先作为可派发集合（frontier）。尚未重新核对的祖先优先于后代；有 loading 祖先时不重复派发其后代。临时没有可派发目录不等于候选已完整，最终仍由覆盖检查与 fence 决定。此实现避免单向 keyset 漏掉重新置为 pending 的较小身份，但大树执行计划与整体成本仍待测量。

生产入口先分流活动候选与恢复身份变化，再决定是否使用普通 Pull，避免把候选事件误送入普通 inbox 门禁。新连接在 `session_pending` 屏障之后发布；旧会话的晚到回应须通过当前连接核对才能落盘。会话切换只在本地检查、短事务和连接替换期间持恢复写锁，不持锁等待网络，也不持 `Service::Control` 等旧请求结束，以便暂停可取消旧 RPC。

普通事件接收／应用、快照安装及 runtime 上传、改名、删除等路径检查候选或会话门禁。原权威元数据、完整缓存和未上传内容保留；不能用清缓存解除初始化状态。候选 ready 原子提交或普通会话追赶完成后才解除写回屏障并唤醒上传。

## 保留的性能与验收边界

- 目录事件、根事件、未知父身份等情况仍保守地使整树目录覆盖失效。活跃目录树仍可能反复重查；本批没有证明所有长初始化问题已解决。
- `complete_initialization` 再次验证完整图、覆盖、预算和游标，再走整份快照安装事务。权威节点、游标及 ready 状态一起提交；整树内存、重复验证和最终 Store 锁占用成本尚未消除，不能把短锁会话切换推广为所有提交都很短。
- 新目录仍需等待完整候选提交后才在 FUSE 可见；生产端到端跨重启恢复、旧缓存不中断、权限变化、真实 NAS 事件及大树性能须独立验证。
- 新回归与安装包验证待统一运行；此文只说明当前源码，不继承包验证007或旧四路枚举的测试与性能结论。

## 版本与验收入口

schema20／IPC17 是本开发分支版本，已发布包及升级历史应分别理解；升级操作与数据保留原则见[升级与恢复](UPGRADE_RECOVERY.md)。可观察的阶段和类型化重试原因见[初始化进度](INITIALIZATION_PROGRESS.md)。测试与交付状态只维护在[唯一验收入口](../DELIVERY_GATES.md)。

源码入口：[候选模型与恢复](../crates/drive-core/src/initialization_store.rs)、[事务与安装操作](../crates/drive-core/src/initialization_store_ops.rs)、[原生分步读取](../crates/drive-core/src/native_initialization.rs)、[FSD生产调度器](../crates/drive-runtime/src/initialization_owner.rs)、[worker可靠应答](../crates/drive-runtime/src/worker.rs)、[Service接入](../crates/drive-runtime/src/service.rs)。
