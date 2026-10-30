# 参考线与证据级别

所有新增业务逻辑独立实现；本表不授权复制供应商代码。

| 参考线 | 已定位来源 | 可采用的约束与边界 |
|---|---|---|
| 群晖 Windows | 本项目静态记录 WINDOWS_PARITY_CONTRACT.md；WinCFWrapper.dll SHA-256 c25893b59b9c5c86ab7de856fe3078e7567a4ea4b24a2a6a93225566a134d22d；SetInSync RVA 0x23d10、PinToLocal 0x23dc0 | 固定、完整缓存、服务器同步分开；原生平台接口不可移植为Linux ABI。静态证据不证明完整运行时对等 |
| 群晖官方 Linux | 冻结的 official-drive-ondemand 实验与 ELF/DWARF 证据 | 已有元数据、按需读取、后台更新和离线实验保留。每次启用协议对照前必须重新核对具体构建/ELF哈希/调用链；当前不把内部接口视为稳定SDK |
| SeaDrive Linux | haiwen/seadrive-fuse，固定提交 `2d2cc7c7d81369bc5904eb030d144ef6d3d0cc5e`；2026-10-31 只读 GitHub 原始文件 | 下方记录已追踪调用链与静态锁范围。未运行上游性能测试；不复制源码、头文件或链接libsearpc，当前未引入其代码或依赖 |

后续每项借鉴必须补同类问题、来源版本/函数、证据级别、独立实现差异及本项目压测收益，不能仅凭架构图断言性能。

## SeaDrive 静态调用链：用于 P1 设计评审

来源固定为上述提交，读取经 GitHub API；以下为行为归纳，不是代码移植。只确认静态锁范围，不声明锁的运行时持有时长。

| 同类问题 | 已追踪来源与锁范围 | 本项目独立设计及区别 |
|---|---|---|
| 每次状态查询重建全树 | [sync-status-tree.c](https://github.com/haiwen/seadrive-fuse/blob/2d2cc7c7d81369bc5904eb030d144ef6d3d0cc5e/src/sync-status-tree.c)：add 按路径插入；del 向上删除空目录；exists 按路径查找。树本身不加锁；sync-mgr 的更新/查询调用者持 paths_lock | SQLite 保持权威；计划维护可重建的逐状态计数及祖先聚合。不能直接采用上游 syncing/synced 两棵路径集合，因为 native 还要表达固定、过期、冲突与本地持久状态。尚未实现或证明收益 |
| 状态查询是否真的很轻 | [sync-mgr.c:4730](https://github.com/haiwen/seadrive-fuse/blob/2d2cc7c7d81369bc5904eb030d144ef6d3d0cc5e/src/sync-mgr.c#L4730)：get_path_sync_status 在 paths_lock 内调用树查询，还可能调用缓存过期判断或 repo_tree_stat_path；锁到 4788 才释放 | 不照搬嵌套锁范围。查询应读取已发布状态快照，文件系统检查在变更处理时完成；重建期间明确状态不可确认，不能返回假同步 |
| 目录读取重复重型元数据查询 | [fuse-ops.c](https://github.com/haiwen/seadrive-fuse/blob/2d2cc7c7d81369bc5904eb030d144ef6d3d0cc5e/src/fuse-ops.c) readdir_repo → [repo-tree.c:302](https://github.com/haiwen/seadrive-fuse/blob/2d2cc7c7d81369bc5904eb030d144ef6d3d0cc5e/src/repo-tree.c#L302) repo_tree_readdir：持 tree 读锁解析路径、遍历并复制该目录全部条目，内部还检查可见性；返回后 FUSE 填充条目 | 参考本地元数据与内容分离。计划按打开的目录句柄生成稳定枚举视图，分页只读取所需条目；仍需定义并发改名/删除的可见性。上游也在锁内遍历整个目录，不能宣称其目录路径为常数耗时 |
| 缓存读等待与句柄保护 | [file-cache-mgr.c:1167](https://github.com/haiwen/seadrive-fuse/blob/2d2cc7c7d81369bc5904eb030d144ef6d3d0cc5e/src/file-cache-mgr.c#L1167)：read 在等待范围就绪之后才取 handle.lock，锁覆盖本地 seek/read；有独立 CachedFile 原子 n_open、cache_lock 和 task_lock；start_cache_task 用按文件任务表防重复，取消考虑手动任务和剩余引用 | native 继续整文件校验后发布，保留已有按版本合并和不可变内容句柄。参考读取者/任务引用分离；将缓存读取与阻塞元数据工作分开调度。上游范围读取和取消阈值属于其协议及对象生命周期，不能直接移植 |

上游仅为架构思路来源；未编译、安装或链接它。P0 已显示本项目本地队列/状态处理的失败基线，P1 的收益必须由相同 native 夹具比较，不能由这份源码阅读推定。
