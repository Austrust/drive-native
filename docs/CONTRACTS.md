# Issue #5 / #2 / #7 冻结契约

G0，协调者冻结于 2026-11-04。基线 `codex/fs-recovery-safe-9 @ d555944`。
规格来源：项目 `work/plan-5-2-20261104/{PLAN,PLAN-7,PLAN-ULTRA}.md`。
本文是实现接口与边界，不是已验证结论；G4 前所有行为均为**未实测**。

## 阶段与所有权

`codex/fix-5-2` 集成 A → B → C → D；随后 `codex/fix-7` 单线实现 ACL。
最终汇总分支 `codex/fix-all-5-2-7` 保留完整提交历史。schema **24 = history，25 = permissions**。
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
id/seq 为 SQLite i64 的非负整数；seq 持久单调，删除后不复用。时间为 UTC Unix 秒；name/path_at_event 为事件时值。mount 为 root id；每个服务只查询自己的 Store。可空字段为 null 或省略，GUI 兼容二者。action=`upload|download|remote_change|rename|move|delete|error`；result=`ok|failed|cancelled|pending`。detail 为小型 JSON 对象或 null，绝不放正文/凭据。错误只持久化分类与受控摘要，不能复制原始网络错误。

四个 D-Bus 方法均接受**一个 JSON 字符串参数**并返回一个 JSON 字符串，沿用 org.drivenative.Client1：

| 方法 | 参数 | 成功返回 |
|---|---|---|
| GetSyncHistory | `{mount?,result?,name?,cursor?,limit?}` | `{records:[record...],next_cursor:null或string}` |
| GetRecentActivity | `{limit:12}` | `{records:[record...]}` |
| ResolveHistoryTarget | `{id:42}` | `{path:null或string,exists:bool,mount_online:bool,reason:null或string,parent_path:null或string,offline_available:bool,node:null或string}` |
| ClearSyncHistory | `{mode:"older_than_days",days:30}` 或 `{mode:"all"}` | `{removed:integer}` |

GetSyncHistory 默认 limit=100，上限500，按 seq DESC/id DESC，cursor 是不透明字符串（可采用十进制 seq 上界），分页严格排他且过滤后返回游标。name 是字面量不区分大小写子串，不是 SQL 通配符。invalid query/cursor 拒绝；不得悄悄扩大清理范围。GetRecentActivity 默认12、上限100。缺 days 默认30，负数拒绝。GUI 超时15s，所有数据库 busy 必须快速返回 `{error:"busy"}`，不阻塞其他服务。全局历史由 GUI 按服务归属聚合，id 必须和所属服务一起保存，不能跨库解析。

ResolveHistoryTarget 基于 node 身份解析当前路径并限制在挂载根内；不直接打开历史 path_at_event。reason 使用 `deleted|offline|online_only|unavailable|not_found|permission_denied`；可用父目录也必须重新校验。offline_available=false 且在线时 GUI 标示下载并打开，不把发现元数据算下载成功；离线无缓存不打开。路径解析本身不 hydrate、不探测 NAS。

默认保留30天/10,000条，设置存在 metadata；清理只动历史。操作结果与历史在同一事务提交；重试以既有 durable job/operation 身份合并 attempts，不能误合并不同版本。迁移23→24沿用备份、事务、未来版本拒绝保护。C 必须列出无法在所有权范围覆盖的发布/失败写入点，交 G2 接线；不得通过 status 查询制造成功历史。

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

G3 追加两个 JSON-string IPC：`GetDirectoryPermissions({path})` 返回 `{task_id,path,node,policy,actions:{action:{state,source,verified_at,revision}}}`，state 为 allowed/denied/unknown；`SetWritePolicy({task_id,expected_revision,local_policy})` 用 revision 比较后保存，返回 `{saved:true,remount_required:bool}`。已挂载任务须重新挂载完整应用 RO/RW 选项；策略门禁立即限制新增写入。版本不符必须刷新，不覆盖另一窗口的选择。

已核实的逐操作拒绝经 worker `PermissionDenied{node,action}` 接收；必须匹配本次目标。原生下载仅在 `recv_download` 直接回复边界将已核实 `0x3002/0x3008` 分类为 Read 拒绝；认证、连接、前置读取、本地 I/O、只读码和内容未就绪码不转换。学习拒绝与终态下载重试门禁在同一 Store guard 下发布。
下载请求捕获权限 revision；期间发生通知或权限更新，迟到的读结果不再持久化旧 allow/deny。迟到拒绝仍向该次调用返回 EACCES，但下载意图回 pending，不留下终态权限重试门禁；后续只读尝试重新判定。此规则不重发任何写入。

写请求的直接权限相关响应经 `MutationPermissionUncertain{code}` 保留，按 upload/remove/rename 的已核实小集合验证；进入 uncertain/cleanup_uncertain，不改成 pending，不学习任何精确节点 ACL，不猜改名哪端拒绝。历史 result=pending、error_kind=permission_commit_uncertain，summary 明确“提交状态待核对”；detail.permission_response 仅含请求动作和数字码，终态清错误提示而保留该次历史响应证据。重连/核对只可查询结果，不重发未知写入。写入终态无提交证明仍缺失，原 typed 写拒绝接收路径的覆盖仍来自隔离夹具。

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

**DefaultPermissions 决策：保留。** 采用 PLAN-7 §4.2 明确允许的兼容方案。unknown/allowed 的远端合成展示 mode 为目录0755/文件0644；known write denied 可去掉相应写位，但不能因为父根不能 create 就错误掩盖其他允许动作；本地显式 mode/uid/gid/sticky/umask 仍由既有 POSIX/内核规则执行。access() 在 unknown 下仅为乐观值，服务端仍最终裁决；必须在报告说明。逐操作实际入口仍检查 C3，mode 不是 ACL 证据。不得用 chmod/chown 改 NAS 权限。

G4 错误码校正：只有逐节点所有相关写动作均 denied 才投影去掉写位；L1/L2 整任务限制不借 mode 抢先产生 EACCES，仍由挂载只读选项或实际操作入口返回 EROFS。预览 inode 保留自己的身份与拒绝检查，正文 EAGAIN、写操作 EROFS，不提前改成权威 ENOENT。

本地 mtime 准备按 Setattr 授权，不能因复用草稿 helper 额外要求 WriteContent；普通写/截断仍要求 WriteContent。
既有原生 mtime 写回采用整文件上传，其远端 WriteContent 门禁保留。Setattr 允许而内容上传拒绝时，只承诺本地持久保存，保留 pending；元数据专用远端写回未实现，见 [文件时间契约](FILE_TIMESTAMPS.md)。

元数据 dispatcher 遇到 Store 繁忙时沿既有工作队列回退，不以 EAGAIN 代替当前权限判断；父 Traverse 在查找子项之前检查，缺失名称也不能绕过。正文读取的等待只在 worker 上发生。

诊断名称固定：`IndexHits`、`IndexMisses`、`IndexFallback`、`IndexDegraded`、`IndexBuildUs`、`IndexBytes`、`PermissionDenied`、`SpaceForbidden`，另暴露 index_partial。诊断不得包含名称/路径/凭据。

G3 在 SharedStore 发布已知拒绝后暂停 NSI 回复，走权威逐操作检查。无拒绝时保留内存路径。正文/目录续读使用独立的 `permission_clear_lease()` 持至有界回复；首个拒绝的发布等待此前回复完成，但普通元数据事务不阻塞已缓存正文读取。持此租约不得阻塞获取 Store（目录续读仅 try_lock）；已知拒绝的回复在 Store 内重验后发布。所有权限检查同时覆盖本地祖先和未确认 namespace_overlay 的远端祖先，不继承其写动作。

## 验收与证据

G4 独立代理按 PLAN §4 / PLAN-7 §9 运行并保留原始输出，作者不自评通过。G5 三套树100k/319679/327750，.9/本地/候选同窗口首扫重扫各3轮，记录负载、每目录成本、请求数、真值与 hydration。目标未达记录差距和可测分解，不能伪造耗时归因。真实 ACL/NAS 是外部验收，标注未实测。最后报告与导航只引用实际证据。
