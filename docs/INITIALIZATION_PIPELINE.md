# 持久初始化：有界调度与成本诊断

本文说明实现机制和诊断字段。性能数值、真实 NAS 验收与发布状态由对应验证报告和 [DELIVERY_GATES.md](../DELIVERY_GATES.md) 维护；本文不把源码审查或模拟测试提升为真实初始化已通过。

## 目录任务如何推进

初始化候选与权威索引仍分开保存。目录列表、候选事件和阶段进度不会单独推进产品的已提交游标，也不会解除持久写回门禁。完整覆盖、身份核验、独立事件 fence 与最终原子安装仍是发布条件。

调度入口是 [`Store::claim_initialization_directories`](../crates/drive-core/src/initialization_scheduler.rs)：

1. 在同一事务内核验 run、epoch、scope、restore 和原权威游标。
2. 从 `initialization_pending` 部分索引按 ID 读取一页，最多检查 64 个 pending／failed 候选；额外读取一个 key 只用于判断是否有下一页。
3. 沿每个候选的有限祖先链提升到最高 pending／failed 祖先。loading 祖先阻止后代并发领取；同页提升到相同祖先的候选合并。遇断链、循环、无效路径或不可用祖先则整页回滚。
4. 复用已验证链生成路径，在同一事务内分配全 run 单调 generation 并标记 loading。单次 API 最多返回 16 个任务；生产 owner 只领取当前四个并发槽位允许的数量。

这里不读取全部候选正文、native identity map 或完整 coverage 向量。每项仍有祖先查询成本，64 项与既有深度边界限制的是工作量，不是本地存储 I/O 的严格毫秒上界。

返回的 `next` 只用于 keyset 调度续页，绑定 run／epoch，不代表覆盖完成。候选事件或目录回执改变树之后，调用方从头选择；否则新增到较小 key 的任务可能被跳过。空页也可能来自 loading 祖先，不能据此安装索引。最终 coverage／fence 检查独立执行。

## 完成即提交，再补位

[`Bridge::initialization_directory_window`](../crates/drive-runtime/src/initialization_owner.rs) 保持最多四个网络读取并发。一个目录读取完成后，owner 先核验并持久化其回执，再为释放的槽位选择下一项，不等待同组最慢目录才提交其他结果。

还有空槽时，owner 可以沿 keyset 继续越过 loading／同祖先合并的前缀，不因第一张64项页面只发出一个祖先任务就等待它完成。每页之间先尝试收取已经完成的回执、检查期限并释放 Store；处理回执后丢弃旧扫描位置，从头核验当前树。

每个事件窗口最多接纳 16 个目录任务。单任务既有的 typed directory-race 重试最多三次，因此网络尝试数可能高于 16。事件收集／应用仍安排在目录窗口之间，不与该窗口的候选回执提交并行。

重试前必须在同一 recovery／Store 锁边界内检查原目录票据仍为 current。兄弟目录回执可能移动、替换或撤销正在读取的对象；失效旧票据不能为旧 `node/path` 领取新 generation。该结果保留旧票据，由协调器丢弃，当前树上的正确路径交给后续调度。

发现一个错误后停止补位，但继续收取已发请求。仍属于当前 RPC、run／epoch 和目录 generation 的成功回执可以落盘；失效回执丢弃。排空后返回错误，不安装候选、不确认产品 checkpoint。普通重开／恢复使未完成 loading 项重新 pending，已经完成且仍有效的目录覆盖继续保留。

停止或连接替换依靠当前 RPC 核验及既有取消机制。网络读取不持有 Store 锁；本地回执事务仍需要 Store。该设计不承诺任意大小目录的提交没有锁等待，也不改变旧完整缓存的本地访问规则。

## 等待事件依赖时的有限优先级

当持久事件页因普通文件的父身份尚未发现而处于 `AwaitingCoverage`，[`initialization_priority`](../crates/drive-core/src/initialization_priority.rs) 可为本窗口建议一个已有 pending 目录。它根据已验证 hint 的有效目标父路径，沿候选 `(run,parent,name)` 索引走到最近的未完成目录；该路径只是排序线索，不是文件身份或访问授权。

领取事务重新核验当前 head、run／epoch、候选边、native numeric identity、祖先覆盖、sticky deny 和根有效性。不能从路径创建缺失父节点，不能使 received 目录重新 pending，不能推进事件或标记覆盖完成。若完整已接收路径中仍找不到子项，继续原普通队列和事件处理，不猜测它的新位置。

每16任务窗口最多一项优先任务，其余槽位由普通队列领取。准备阶段只处理正文和 hint 合计不超过256KiB的页，最多缓存8个缺父依赖，每个路径最多128层；没有 hint、超预算或权限／证明链不完整时不提供优先任务。缓存绑定 head sequence、batch／hint digest 和 run／epoch，仅缓存排序线索；同一 head 后续窗口不再解析整页。原事件应用仍独立执行完整性验证，损坏响应不能靠缓存跳过。

这是本客户端的有界调度策略，不是已逆向确认的 Windows 调度算法。它旨在缩短事件等候已知分支的时间，不改变未知父在全覆盖后仍缺失时的保守处理方式。

## 如何读取诊断

### 未知目录与同页嵌套依赖的重列义务

[`initialization_local_recheck`](../crates/drive-core/src/initialization_local_recheck.rs) 对部分未知目录事件安排局部重列。事件页中的目录身份只作为待核对的依赖，不会直接写成候选节点，也不会授予访问权限：

1. 仅处理规范 scope 内的 `Modify` 目录。该 ID 在候选中必须缺失，不能有以它为父的孤儿项；该 ID 在整页必须唯一，目录事件的 `(parent,name)` 关系也必须唯一。
2. 若父目录已存在，核验候选父链、原生 numeric identity 与节点摘要的绑定、路径、覆盖状态和 sticky deny。事件路径必须等于当前父路径加名称；同名不同 ID 的既有项不能当作缺失边。
3. 若父目录也未知，只有同页存在可闭合的目录依赖时才继续：每一边的父 ID、metadata parent 与路径／名称均需一致，最终连接到上述真实祖先。与祖先尚未连接的环或缺边不能自行成为合法计划；事件顺序不会决定能否闭合。
4. 同页缺父文件必须精确引用已闭合的目录依赖，并通过文件路径、名称及 metadata parent 核验。一般未知父的活文件不能仅凭历史路径消耗事件，仍按原等待或保守恢复处理。

计划只把真实祖先的**直接列表**标记为 pending。真正的列表回执负责创建第一层身份和新的 pending 子目录，后续列表再发现更深层级。未知 ID 没有可沿用的已有候选子树，因此不需要从提示中构造中间节点。既有目录事件仍安排其必要子树及源／目标父列表重查；混合页中任何未覆盖的风险仍可使整页退回保守全树重查。

该优化最多处理 4096 个事件、64 个目录事件，目录深度上限为 128；超预算继续既有保守路径。重列义务与候选 applied cursor 在同一事务提交；`Applied` 表示义务已持久化，**不表示列表完成或产品游标已提交**。父列表缺项、同名替换、权限拒绝、迟到票据和进程恢复继续由现有候选规则处理。独立 fence、完整覆盖和最终安装门禁不变。

**未知 Remove 仍保留全树回退。** 当前 tombstone 解码没有可验证的历史旧父身份；即使目标 ID 已不在候选中，也不能据此证明远处旧父自身的版本或修改时间已经更新。历史路径仅能提名核对位置，不能冒充旧父身份，更不能据它删除同名替代对象。`Move`／`Modify` 投影出的 Delete 也没有获得新的局部豁免。

这是本客户端的局部协调策略，不是已确认的 Windows 内部算法。上述源码机制及测试入口不代表真实 NAS 上所有事件页已经收敛，也不表示已消除所有整树重查。

### 事件应用计划诊断

核心兼容 API 仍返回 `InitializationApplyResult`；`apply_initialization_events_diagnosed` 额外返回固定结构摘要。runtime 只在 API 成功返回后累计：

- `event_local_parent_plans`、`event_local_subtree_plans`、`event_unknown_directory_dependency_plans` 是成功事务的计划数量，不是实际改变的行数；重叠计划或已 pending 项可能使 SQL 更新量更小。
- `event_full_recheck_*` 使用固定原因。多项同时出现时优先级为 `root`、`directory_race`、`type_mismatch`、`unsupported_directory`、`unknown_delete`、`missing_parent`。全树回退的局部计划计数为零。
- `event_awaiting_missing_file_parent` 表示此次观察仍等待文件父覆盖，同一未完成页可能重复计数；它不代表提交了重列计划。
- 已应用页的重复调用返回零摘要；事务失败不返回成功摘要。诊断不另扫目录树，也不参与正确性判定。

初始化快照的 `work` 来自 [`initialization_metrics`](../crates/drive-runtime/src/initialization_metrics.rs)，是当前后台进程生命周期内的观察值；普通网络重连不会清零。它不参与覆盖判定、权限判断或游标提交。

| 字段 | 含义与使用边界 |
|---|---|
| `stages` | 每阶段完成 span 的 `count/total_us/max_us`。包括 RPC、payload decode、Store／recovery 锁等待、调度、目录提交、事件收集／应用、root 提交、最终检查／提交和队列取样 |
| `in_flight` | 尚未结束的阶段及已持续时间，用于区分卡在网络还是本地处理 |
| `max_in_flight` | 同时活动的诊断 span 峰值；嵌套 span 也计入，不能当作网络并发数 |
| `queue`、`queue_sample_age_ms` | 候选节点数，pending/loading/received/denied/failed 目录数及 received/applied sequence；显示取样年龄，不伪装成实时值 |
| `claim_examined` | 检查过的 pending key 数，不是唯一目录总数 |
| `claim_ancestor_promotions`、`claim_coalesced` | 提升到祖先领取、同页合并的次数 |
| `claim_blocked_loading`、`claim_scan_yields` | 被在途祖先阻塞及扫描续页次数，可识别 loading 前缀影响 |
| `claim_new`、`claim_previously_claimed` | generation 原先为零／非零的领取数量。再次领取可能是合法事件重查或恢复，不能直接称为重复下载 |
| `priority_prepared/reused/claimed/skipped` | 新的有界依赖计划、复用计划、实际发放优先票据及未提供加速的次数；优先票据不重复计入普通 `claim_new` |
| `directory_committed/denied/retry`、`stale_receipt` | 持久化目录回执、拒绝访问、重新发放重试票据及丢弃失效回执的次数 |
| `pull_pages/pull_events/applied_pages/awaiting_coverage` | 投影后收页、事件数、应用页数及等待覆盖的观察次数；等待同一页会重复观察 |
| `resumed/restarted/coverage_race` | 候选恢复、重新建立候选及受限保守逃生次数 |
| `fence_attempts/checkpoints` | 尝试独立 fence 与完成最终安装的次数；二者不能互换 |

队列状态聚合需要扫描目录状态。owner 在循环边界、距上次取样至少十秒时采集，不在每次 claim 或桌面状态查询时扫描；长时间请求期间该值可能变旧。

`priority_prepare` 与 `priority_claim` 分别计时准备／复用检查和领取阶段。`priority_skipped` 可以来自预算、缺 hint、依赖已出现、在途任务或不可用路径；它不是协议丢事件计数。

不同阶段可能嵌套，也可能由并发任务重叠。`total_us` 不能直接相加解释为整个初始化墙钟时间；`rpc_directory` 也不是纯 NAS 执行时间，它包含本地 IPC、worker 处理和远端等待。

## 原生请求、分页与事件的区分

`work.native` 使用 [`sproto::diagnostics`](../crates/sproto/src/diagnostics.rs) 的封闭字段：连接池等待、新建／复用连接、连接建立、请求发送／接收、帧数、continuation／keepalive、列表页数、条目数和节点解析时间。`decode_active_poll_us` 是同步 Future polling 的累计活动时间，包含传输 polling，不是独立测得的纯解析 CPU 时间。

请求样本中的 `received_bytes` 和 `positive_read_count` 统计解密流进入 `BufReader` 前的正字节读取。它们包含预读；从已有缓冲区消费字节、零字节读取及 EOF 不增加计数。`first_byte_us/last_byte_us` 是相对同一请求开始时刻（包括发送阶段）的首／末次正读偏移，没有正读时为 `null`。它们不包含内容或身份，不表示已解析对象大小，也不能单独证明整个响应期间持续有流量。`elapsed_us - last_byte_us` 可表示该请求结束前多久没有观察到新正读。累计请求统计只合并读取次数和字节数，不相加各请求的首末时间。缺少新读取计数的旧诊断消息不能伪造为零读取；本批 worker 与 FSD 必须使用同一冻结构建，IPC 版本为 20。

普通 `list` 每页最多请求 1000 项；每页的 30 秒 `native_response_timeout` 包含发送、完整 PObject 解码及该响应全部 continuation 的总 exchange 时间，收到字节不会延长它。连接池等待、其他分页和完整响应后的节点字段转换不属于该截止。`frames` 只在完整 PObject 返回后增加，且包含 keepalive：已收到字节但 `frames=0` 可以是半个对象；`frames=1/keepalive_frames=1` 不能当作一页目录已经成功。长通知的 receive-idle 截止是另一条路径，不能据普通请求超时推断 30 秒完全没有网络流量。本批只增加观察指标，不改变这两种超时策略。

原生 Pull 诊断区分：

- `raw_event_count/raw_empty_responses`：协议响应在根范围往返过滤之前的事件观察。
- `cursor_advanced_responses/cursor_unchanged_responses`：响应游标是否推进；游标未推进不等于原始事件为空。
- `root_roundtrip_filtered_count`：根范围核验过滤数量。
- `projected_pages/projected_event_count/projected_empty_pages`：成功验证后的初始化事件包。
- `no_advance_results`：最终没有前进结果的次数，不应改称“空网络响应”。

`recent_commands` 最多保存 64 条命令摘要；计数、阶段、耗时和错误类别不包含远端名称、路径、事件 ID、游标值、凭据或正文。`native_merge_rejected` 表示没有接受不合法或无法安全合并的诊断值，不表示协议事件已被静默跳过。

## 针对性验证入口

- [核心队列回归](../crates/drive-core/src/initialization_scheduler_tests.rs)：祖先去重后的补足、loading 前缀续页、原子回滚、generation 溢出、不可用祖先、跨 epoch 游标、空页与 fence，以及固定 pending 页的 SQLite VM 工作量。
- [依赖优先级回归](../crates/drive-core/src/initialization_priority_tests.rs)：计划复用、原子领取、权限与身份变化、损坏证据、页／依赖预算、旧 epoch、无提示页和危险混合页退回普通调度。
- [未知目录局部重列回归](../crates/drive-core/src/initialization_unknown_directory_tests.rs)：事件顺序、真实列表创建身份、权限／替换、失败回滚、重启及固定诊断摘要。
- [同页嵌套依赖回归](../crates/drive-core/src/initialization_nested_directory_tests.rs)：依赖顺序排列、环／缺边／同名冲突、已存在于其他位置的 ID、父移动双序、真实列表缺项／拒绝／替换，以及未知 Remove 仍保守。
- [owner 回归](../crates/drive-runtime/src/initialization_owner_tests.rs)：独立回包与慢请求、80个后代的阻塞前缀、超时后的成功回执保留／重开复用、兄弟目录移动后的旧路径重试拒绝，使用本地 Unix RPC 和 SCM_RIGHTS。
- [公开合成 replay](../crates/drive-core/examples/initialization_claim_replay.rs)：`legacy` 与 `batched` 在新建隔离 Store 上处理相同的 64 个待枚举目录，仅累计 claim 调用时间；准备数据和写回目录回执不计入该指标。

上述入口的存在不是执行记录。性能比较应先完成编译，在无计划竞争负载的顺序窗口运行，保留源版本、退出状态和失败历史。真实 NAS 的全量完成、断线恢复和复杂权限收敛仍须独立验收。
