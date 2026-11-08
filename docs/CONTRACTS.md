# Issue #5 / #2 / #7 冻结契约

G0，协调者冻结于 2026-11-04。基线 `codex/fs-recovery-safe-9 @ d555944`。
规格来源：项目 `work/plan-5-2-20261104/{PLAN,PLAN-7,PLAN-ULTRA}.md`。
本文是实现接口与边界，不是已验证结论；G4 前所有行为均为**未实测**。

## 阶段与所有权

`codex/fix-5-2` 集成 A → B → C → D；随后 `codex/fix-7` 单线实现 ACL。
最终汇总分支 `codex/fix-all-5-2-7` 保留完整提交历史。schema **24 = history，25 = permissions，26 = 持久上传只读核对预算**（`operations.reconcile_attempts`/`reconcile_at`；当前 `STATE_SCHEMA_VERSION=26`）。
每个 stream 独立 worktree、独立 CARGO_TARGET_DIR；G1 只可 cargo check，禁止测试、基准、NAS。
G2 全量编译；G3 完整实现后才由独立验证代理进入 G4；G5 独占基准窗口。
不 push、不发布、不读取凭据、不访问生产文件，不修改已发布 .9 资产。

| Stream | 唯一可写范围 |
|---|---|
| A | core namespace_index.rs；core lib.rs 只追加导出；filesystem_posix.rs 可抽公共投影但不改 posix_* 签名 |
| B | drive-fs/**、vendor/fuser/**；runtime bridge.rs/service.rs 仅索引接线 |
| C | core state.rs、sync_history.rs、upload_jobs.rs、download_jobs.rs、status_changes.rs、namespace.rs、removals.rs；runtime desktop.rs、tests/desktop.rs |
| D | desktop/drive_native_gui/**、desktop/tests/** |
| 协调者 | 本契约、集成冲突、跨所有权接线、报告与导航 |
| E（G3） | PLAN-7 指定文件；不得与 G1 并行 |

跨范围需求先报告，由协调者 G2 接线。C 可以在 state.rs 用 `#[path = "sync_history.rs"] pub mod sync_history;` 导出历史模块，避免与 A 的 lib.rs 所有权冲突；稳定公开路径为 `drive_core::sync_history`（state 的 glob re-export）。
现有 fix-5-2 的 bus.py 未提交改动保留，D 从冻结 commit 起步，G2 比对后整合。

## C1 NamespaceIndex

```rust
// drive_core::namespace_index; Result = drive_core::Result
pub fn build(store: &Store) -> Result<NamespaceIndex>;
impl NamespaceIndex {
    pub fn build(store: &Store) -> Result<Self>;
    pub fn degraded() -> Self; // G2: refreshable empty holder for initial preview
    pub fn entry_by_ino(&self, ino: u64) -> Option<Arc<EntrySlot>>;
    pub fn lookup(&self, parent_ino: u64, name: &str) -> Option<Arc<EntrySlot>>;
    pub fn dir_block(&self, ino: u64) -> Option<Arc<DirBlock>>;
    pub fn ensure_fresh(&self, store: &Store) -> RefreshOutcome;
    pub fn invalidate(&self, hint: InvalidateHint);
    pub fn is_degraded(&self) -> bool;
    pub fn revision(&self) -> u64;
    pub fn bytes(&self) -> usize;
    pub fn is_partial(&self) -> bool;
}
pub enum RefreshOutcome { Fresh, Refreshed, Degraded }
pub enum InvalidateHint { All, Node { node_id: String, parent_dir_ino: Option<u64> } }
pub enum FileKind { File, Directory }
pub struct EntrySlot {
    pub ino: u64, pub node: Arc<str>, pub parent: u64, pub name: Arc<str>,
    pub directory: bool, pub size: u64, pub mtime: u64,
    pub atime: (u64, u32), pub mode: u16, pub blocks: u64,
    pub dirty: bool, pub preview: bool,
}
pub struct DirEntryLite { pub ino: u64, pub name: Arc<str>, pub kind: FileKind }
pub struct DirBlock {
    pub ino: u64, pub entries: Vec<DirEntryLite>,
    pub version: u64, pub built_at_gen: u64,
}
```

所有公开值应支持 Debug/Clone，FileKind/RefreshOutcome 支持 Eq/Copy。
G0 对原示意签名的裁决：可刷新共享对象不能安全返回无守卫的 `&EntrySlot`，因此冻结为 `Arc<EntrySlot>`；不使用 unsafe 延长生命周期或泄漏旧代。

Store 是权威，索引只读，不写业务状态、不下载正文。装载采用批量投影，完全复用 PosixRecord 的 blob/draft/mtime/atime/mode/不可见/本地覆盖语义。缺失 inode、预览、不确定和脏正文可直接返回 miss，由 .9 SQL 路径回答；不得冒充不存在。
默认上限 2,000,000 条 / 512 MiB；超过上限或装载超过 1s，安全降级 SQL，诊断 partial/degraded。允许有界目录懒加载，但不允许半代刷新后把未刷新的条目标为新代。

读取者在回复前核对 SharedStore::projection_state() 的 revision 与 verified，以及既有 publication/目录 stamp 门禁（G2 接线细节见下节）。Store::local_revision() 是索引内部代数，不能假设与 SharedStore revision 数值相等。热路径不取 Store 锁；状态变化时回退 SQL/受序列化保护地刷新。只有在完整且 verified 的快照上才重新启用内存命中。通知丢失不能影响正确性。dirty/draft 的文件变化可能不提交 SQLite，因此不能靠 revision 缓存其属性；这些属性走 SQL。打开目录的 cookie、ACL/preview 重验、取消和 releasedir 生命周期保持 .9 语义。

### G1 静态核对后的 G2 接线补充

`DirBlock.entries` 前两项固定为 `.`、`..`，其余按 name/ino 排序；lookup 只在 children 中二分，保持既有 cookie 规则。
已核实 `Store::local_revision()` 是 SQLite total_changes，而旧 SharedStore 提交通知有意忽略 local_atimes/inode 分配。G2 由协调者在 shared_store.rs 添加独立 `projection_state() -> CommitState` 和 `try_projection_read() -> Option<ProjectionReadGuard<'_>>`，后者提供 `state() -> CommitState`。所有 Store guard 持有短时投影写保护；索引读租约持到回复结束，不取 Store 锁，失败则原路径回退。projection revision 覆盖任何 total_changes、Store 替换和未确认边界，保留旧 change_state()/通知语义。禁止在持有投影读租约时获取 Store。`StoreGuard::finish_projection(self) -> CommitState` 完成与 Drop 相同的版本发布，在写租约释放前捕获新 token；B 用此 token 绑定刚刷新的索引，避免释放 Store 后另一次提交造成错误绑定。B 可先按此 API 编写，G2 接通后全量编译。

## C2 SyncHistory schema 24 与 IPC

记录 JSON 固定字段：
```json
{"id":42,"seq":42,"mount":"root-id","node":"node-id","name":"example.txt","path_at_event":"folder/example.txt","action":"upload","result":"ok","attempts":1,"error_kind":null,"error_summary":null,"started_at":null,"completed_at":1793750400,"detail":null}
```
id/seq 为 SQLite i64 的非负整数；seq 持久单调，删除后不复用。时间为 UTC Unix 秒；name/path_at_event 为事件时值。mount 为 root id；每个服务只查询自己的 Store。可空字段为 null 或省略，GUI 兼容二者。action=`upload|download|remote_change|rename|move|delete|mkdir|replace`（`rename|move|delete` 由 namespace_ops 的 `rename|remove` 分类：`remove` 记 `delete`，父目录变化记 `move`，否则 `rename`；`mkdir`/`replace` 是目录创建与复合覆盖命名空间操作，使用同一 `operation_key`，供后续确认/冲突解析更新；`error` 不是独立 action，失败由 result=failed 与 error_kind 表达）；result=`ok|failed|cancelled|pending`。detail 为小型 JSON 对象或 null，绝不放正文/凭据。错误只持久化分类与受控摘要，不能复制原始网络错误。

除无参的 GetSyncHistoryRetention 外，各方法接受**一个 JSON 字符串参数**并返回一个 JSON 字符串，沿用 org.drivenative.Client1：

| 方法 | 参数 | 成功返回 |
|---|---|---|
| GetSyncHistory | `{mount?,result?,name?,cursor?,limit?}` | `{records:[record...],next_cursor:null或string}` |
| GetRecentActivity | `{limit:12}` | `{records:[record...]}` |
| ResolveHistoryTarget | `{id:42}` | `{path:null或string,exists:bool,mount_online:bool,reason:null或string,parent_path:null或string,offline_available:bool,cached:bool,mount_slot:null或string,suggested_action:null或string,node:null或string}` |
| ClearSyncHistory | `{mode:"older_than_days",days:30}` 或 `{mode:"all"}` | `{removed:integer}` |
| GetSyncHistoryRetention | 无 | `{schema:1,task_id:string,days:integer,records:integer}` |
| SetSyncHistoryRetention | `{schema:1,task_id:string,days:30,records:10000}` | `{saved:true,days:integer,records:integer}` |

GetSyncHistory 默认 limit=100，上限500，按 seq DESC/id DESC，cursor 是不透明字符串（可采用十进制 seq 上界），分页严格排他且过滤后返回游标。name 是字面量不区分大小写子串，不是 SQL 通配符。invalid query/cursor 拒绝；不得悄悄扩大清理范围。GetRecentActivity 默认12、上限100。缺 days 默认30，负数拒绝。GUI 超时15s，所有数据库 busy 必须快速返回 `{error:"busy"}`，不阻塞其他服务。全局历史由 GUI 按服务归属聚合，id 必须和所属服务一起保存，不能跨库解析。

ResolveHistoryTarget 基于 node 身份解析当前路径并限制在挂载根内；不直接打开历史 path_at_event。reason 使用 `deleted|offline|online_only|unavailable|not_found|permission_denied`；可用父目录也必须重新校验。offline_available=false 且在线时 GUI 标示下载并打开，不把发现元数据算下载成功；离线无缓存不打开。路径解析本身不 hydrate、不探测 NAS。未挂载时仍必须基于 Store 元数据返回真实 `exists` 与 `offline_available`（`path`/`parent_path` 为 null，`mount_online=false`；node 身份仍存在时为 `reason=unavailable`，缺失/已删除/无权限分别为 `not_found`/`deleted`/`permission_denied`）；GUI 据此显示“启动挂载”动作，不得直接打开未挂载路径，也不得把发现元数据当作内容可用。`cached` 与 `offline_available` 同值（目录、可读 dirty draft 或已发布缓存正文），供 GUI 优先判断是否可打开；`mount_slot` 是本服务实例槽（`DRIVE_NATIVE_INSTANCE`，legacy 单挂载为 null，GUI 回退自身来源槽）；`suggested_action` 仅在未挂载且 node 身份仍可解析（`exists=true`）时为 `start_mount`，其余为 null；旧 GUI 忽略新增键。

默认保留30天/10,000条，设置存在 metadata（`sync_history_days`/`sync_history_records`），GetSyncHistoryRetention 读取、SetSyncHistoryRetention 修改；清理只动历史。GetSyncHistoryRetention 缺键、损坏或越界行按默认值返回，不因坏设置失败。SetSyncHistoryRetention 校验 schema=1、task_id 归属，范围 1..=3650 天、1..=1_000_000 条，拒绝未知字段；成功后立即按新上限裁剪元数据历史，返回 `{saved:true,days,records}`。保留期限与清理都只影响元数据历史，不删除、不修改文件、草稿、缓存内容或排队操作；Store busy 时两个方法都快速返回 `{error:"busy"}`。操作结果与历史在同一事务提交；重试以既有 durable job/operation 身份合并 attempts，不能误合并不同版本。迁移23→24沿用备份、事务、未来版本拒绝保护。C 必须列出无法在所有权范围覆盖的发布/失败写入点，交 G2 接线；不得通过 status 查询制造成功历史。

## C3 Permission decision 与 overview（G3）

```rust
// drive_core::write_permissions
pub enum Action { Traverse, List, Read, CreateFile, CreateDir, Delete,
                  RenameSource, RenameTarget, WriteContent, Setattr }
pub enum Decision { Allow, Deny, Unknown }
pub enum LocalPolicy { ExplicitReadOnly, FollowRemote }
pub fn decide(store: &Store, root: &str, node: &str, action: Action) -> Result<Decision>;
pub fn local_policy(store: &Store, root: &str) -> Result<LocalPolicy>;
pub fn record_observed(store: &Store, root: &str, node: &str,
                       action: Action, allowed: bool) -> Result<()>;
```

三层判定：用户 explicit_read_only → EROFS；有明确空间/同步禁止证据的 space_forbidden → EROFS；目标动作 denied → EACCES；unknown 允许尝试并交服务端裁决，**不得写成 allowed**。本地排队成功不是服务端允许证据。网络/uncertain_commit 不得记录 denied 或重发未知提交。现有访问拒绝、preview、草稿规则继续叠加。
读取动作不受本地只读策略限制。动作对象依 PLAN-7 §4.3；rename 必须检查源、两父目录及覆盖目标；祖先 traverse 与本目录 list 分离。已知拒绝不可扩散至无关兄弟目录或队列。

GetOverview 追加字段：`writable_source`（user/legacy），`local_policy`（explicit_read_only/follow_remote），`server_capability`（unknown/allowed/space_forbidden），`write_policy_revision`（非负整数）。保留旧 writable 字段兼容，表示挂载层允许尝试写入，不代表所有节点允许。逐节点来源为 native/file_station/observed/unknown，附 verified_at/revision。
旧 profile false→explicit_read_only/legacy；true→follow_remote/legacy。新向导显式保存策略，不能复制共享根 ACL 为所有子目录权限。重连、恢复、重新核对使 observed 状态回 unknown 并 bump revision，不修改用户只读策略。File Station 可选且默认关闭；无已验证逐节点 native 字段时明确标注未实测/不可得，不猜字段。

GetOverview 追加状态字段（schema 仍 1，旧 GUI 忽略即可）：`sync_state`（needs_attention/paused/transferring/checking/pending/idle，按连接错误、挂载与暂停、活动传输、初始化阶段、待处理及 uncertain 队列依次推导）、`active_transfers`（≥0 的活动传输数）、`issue_count`（与 GetIssues 同源的待处理冲突项计数；Store 查询失败时为 null，不得当 0）、`cache_availability`（`{state:empty/online_only/partial/complete/pinned,total,offline,pinned}`，实际五态；`pinned` 要求全部非目录节点已固定且可离线读；只统计非目录节点；offline 表示本地有可读内容且未被 access_blocked；目录不计入；字段缺失或旧后台时由 GUI 显示未知，不是后端枚举值）与 `version`（`{build:string,state_schema:27,ipc:21}`）。这些字段只读本地元数据，不打开文件、不探测 NAS，也不改变既有键含义。

GetShutdown 的 `shutdown` 快照新增 `blocking_reason`（null 或固定字符串）：waiting_for_workers/unmounting_mount/draining_open_files/persisting_local_changes/mount_still_present/unmount_failed/checkpoint_failed（`worker_join_failed` 为预留值，当前实现不产生，join 失败归入 unmount_failed；GUI 遇到未知值按通用失败处理）；完成或回到 idle 时自动清空，失败时保留。字符串不含路径或原始错误；`phase`/`elapsed_ms`/`stage_elapsed_ms`/`overdue` 语义不变。升级流程没有后端 stage 事件，也不新增 D-Bus 方法；GetOverview 不提供 `upgrade_stage`。GUI 仅依据既有 BackendUpgrade `changed`/`finished` 显示更新入口与最终结果，并用 `version`（state_schema=27、ipc=21）做配对：不匹配时拒绝确认接管，字段缺失（旧后台）按未知降级、不把版本当作确认依据。旧 GUI 忽略新增字段，BackendUpgrade 契约不变。

G3 追加两个 JSON-string IPC：`GetDirectoryPermissions({path})` 返回 `{task_id,path,node,policy,actions:{action:{state,source,verified_at,revision}}}`，state 为 allowed/denied/unknown；`SetWritePolicy({task_id,expected_revision,local_policy})` 用 revision 比较后保存，返回 `{saved:true,remount_required:bool}`。已挂载任务须重新挂载完整应用 RO/RW 选项；策略门禁立即限制新增写入。版本不符必须刷新，不覆盖另一窗口的选择。

已核实的逐操作拒绝经 worker `PermissionDenied{node,action}` 接收；必须匹配本次目标。原生下载仅在 `recv_download` 直接回复边界将已核实 `0x3002/0x3008` 分类为 Read 拒绝；认证、连接、前置读取、本地 I/O、只读码和内容未就绪码不转换。学习拒绝与终态下载重试门禁在同一 Store guard 下发布。
下载请求捕获权限 revision；期间发生通知或权限更新，迟到的读结果不再持久化旧 allow/deny。迟到拒绝仍向该次调用返回 EACCES，但下载意图回 pending，不留下终态权限重试门禁；后续只读尝试重新判定。此规则不重发任何写入。

下载自动退避保留 schema1 的持久 retry_at，并按 `(node,version,完整failure指纹)` 绑定进程内 Instant 门限。失败发布时以传给 fail_download 的同一墙钟值计算完整退避；前台等待和前台/后台发送资格均检查双门限，重连保留、进程重启仅从持久剩余时间恢复一次。双门限到期正常放行；清除失败记录则提前释放当前门限。旧版本 ticket 不得读取新版本 failure 或驱逐其门限。该规则不改变 core 的纯数据库 now 参数契约、schema26、IPC21 或 FUSE TTL。

写请求的直接权限相关响应默认经 `MutationPermissionUncertain{code}` 保留，按 upload/remove/rename 的已核实小集合验证；进入 uncertain/cleanup_uncertain，不改成 pending，不猜改名哪端拒绝。唯一例外（本轮新增，见[精确写拒绝自动学习增量](validation/issue7-native-001/write-learning/REPORT.md)）：**upload 直接响应 `0x9003` 且请求目标是已存在节点**时，sproto 在直接回复边界产生 typed `MutationPermissionDenied{code}`，worker 把该拒绝绑定到本次请求节点与 `write_content`，经既有 `refuse_operation` 记为 `denied(observed, revision=当前策略 revision)`：journal 由 `targets_denied` 门禁阻止重发（保留本地内容），history 记 failed/permission_denied，profile 变化/重连/重新核对按既有 revision 失效回 unknown。该例外只成立于此已证子集：`0x9004`/`0xa004` 与 profile 只读过滤同码且 wire 无字段可区分，`0xa003` 无已证生产者，rename 源/目标与复合 replace 阶段不可归属，创建（含 mkdir）的 capability 不是 CreateFile/CreateDir primitive——这些全部保持原 hint/uncertain，不写 ACL 行。历史 result=pending、error_kind=permission_commit_uncertain，summary 明确“提交状态待核对”；detail.permission_response 仅含请求动作和数字码，终态清错误提示而保留该次历史响应证据。重连/核对只可查询结果，不重发未知写入。静态证据证明特定 ACL middleware 的提交前拒绝，但同码多来源、精确动作归属和复合阶段的证明仍未齐，真实 NAS 响应未实测。

内部 worker IPC21 的 `Notification` 保留原生 `profile_changed`（与当前 view 文件变化 `changed` 分开）。该通知使当前挂载根的全部逐节点来源回 unknown、清除验证时间并递增 revision，即使没有文件事件；不改变用户策略、明确空间禁止或未确认写入意图。普通通知不清权限。属性仍受既定 1000ms 上限约束，须配套更新 fsd/worker，拒绝 IPC20 混用。

## C4 FUSE 集成点、TTL 与权限门禁

```rust
CacheFilesystem::with_namespace_index(Arc<NamespaceIndex>) -> Self
CacheFilesystem::with_notify_hub(NotifyHub) -> Self
pub const ENTRY_TTL: Duration = Duration::ZERO;
```

`DRIVE_NATIVE_FS_INDEX=0` 禁用索引，完整保留 .9 路径；默认开启。
`DRIVE_NATIVE_FS_TTL_MS` **只控制 attr TTL**，默认1000ms、0关闭。脏/预览/不确定/失效属性 TTL=0；所有正、负目录项及 READDIRPLUS entry TTL 恒0。ReplyEntry::entry_with_timeouts(entry_ttl,attr_ttl) 调底层 new_entry(attr_ttl,entry_ttl)，保留协议四字段 ABI 回归。
NotifyHub 失败/丢消息与 publication degraded 联动，不能因索引命中绕过现有保护。索引集成只在 lookup/getattr/opendir/readdir/readdirplus/releasedir 读路径；G3 在各实际读写入口叠加 C3 判定，不复制 SQL 授权逻辑到缓存。

**DefaultPermissions 决策：保留。** 采用 PLAN-7 §4.2 明确允许的兼容方案。unknown/allowed 的远端合成展示 mode 为目录0755/文件0644；文件 WriteContent 与 Setattr 同时已知拒绝才去掉写位。目录保持原 POSIX 写位：目录自身及创建动作的拒绝不能枚举每个子项的有效 Delete/Rename 权限，收窄父目录 mode 会在 FUSE 之前误拒合法子项操作。本地显式 mode/uid/gid/sticky/umask 仍由既有 POSIX/内核规则执行。access() 对目录及 unknown 仅为乐观值，服务端仍最终裁决；逐操作实际入口检查 C3，mode 不是 ACL 证据。不得用 chmod/chown 改 NAS 权限。

**有效动作的对象边界（服务端静态证据澄清）：** Delete 表示目标自身的有效删除权限（已包含服务端父链 ACL），不是 delete_child；RenameSource 也表示源节点自身。unlink/rmdir 检查父 Traverse + 目标 Delete，不从父自身 Delete 或 CreateDir 推断子项删除。rename 检查源父 Traverse + 源 RenameSource + 目标父 RenameTarget + 覆盖目标 Delete。该语义同时用于 FUSE 与持久队列；父遍历和目标拒绝仍阻断操作。参见 [官方服务端证据](validation/issue7-native-001/server-static/REPORT.md)。

G4 错误码校正：只有普通文件 WriteContent 与 Setattr 同时 denied 才投影去掉写位；L1/L2 整任务限制不借 mode 抢先产生 EACCES，仍由挂载只读选项或实际操作入口返回 EROFS。预览 inode 保留自己的身份与拒绝检查，正文 EAGAIN、写操作 EROFS，不提前改成权威 ENOENT。

本地 mtime 准备按 Setattr 授权，不能因复用草稿 helper 额外要求 WriteContent；普通写/截断仍要求 WriteContent。
既有原生 mtime 写回采用整文件上传，其远端 WriteContent 门禁保留。Setattr 允许而内容上传拒绝时，只承诺本地持久保存，保留 pending；元数据专用远端写回未实现，见 [文件时间契约](FILE_TIMESTAMPS.md)。

元数据 dispatcher 遇到 Store 繁忙时沿既有工作队列回退，不以 EAGAIN 代替当前权限判断；父 Traverse 在查找子项之前检查，缺失名称也不能绕过。正文读取的等待只在 worker 上发生。

诊断名称固定：`IndexHits`、`IndexMisses`、`IndexFallback`、`IndexDegraded`、`IndexBuildUs`、`IndexBytes`、`PermissionDenied`、`SpaceForbidden`，另暴露 index_partial。诊断不得包含名称/路径/凭据。

G3 在 SharedStore 发布已知拒绝后暂停 NSI 回复，走权威逐操作检查。无拒绝时保留内存路径。正文/目录续读使用独立的 `permission_clear_lease()` 持至有界回复；首个拒绝的发布等待此前回复完成，但普通元数据事务不阻塞已缓存正文读取。持此租约不得阻塞获取 Store（目录续读仅 try_lock）；已知拒绝的回复在 Store 内重验后发布。所有权限检查同时覆盖本地祖先和未确认 namespace_overlay 的远端祖先，不继承其写动作。

明确的原生 Read 拒绝不设置自动下载重试截止（`retry_at=null`）。已知 Read 或祖先 Traverse 拒绝同样约束后台补全、重试唤醒及 Bridge 的缓存/新请求入口，返回 EACCES；仅重试传输不覆盖权限证据。重连、profile 通知或用户重新核对按既有 revision 规则恢复 unknown，再允许尝试；权限变更期间的迟到拒绝仍只影响当次调用，不建立旧拒绝门禁。该补充不改变 schema26、IPC21、entry TTL=0 或 attr 默认1000ms。

## 验收与证据

G4 独立代理按 PLAN §4 / PLAN-7 §9 运行并保留原始输出，作者不自评通过。G5 三套树100k/319679/327750，.9/本地/候选同窗口首扫重扫各3轮，记录负载、每目录成本、请求数、真值与 hydration。目标未达记录差距和可测分解，不能伪造耗时归因。真实 ACL/NAS 是外部验收，标注未实测。最后报告与导航只引用实际证据。
