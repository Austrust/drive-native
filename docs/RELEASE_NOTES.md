# 0.1.0~dev20261109.2：上传终态处理、缓存释放与 schema v29（候选）

本批把 #14 未确认上传预算耗尽后的 `abandoned` 终态与「放弃本地文件」入口、#19 缓存释放后台作业与 GC 回收、#18 FUSE 硬链接拒绝语义、#53 后台提交推进后的托盘/导航计数刷新、#8 初始化期间桌面读响应性验收、#6 已绑定目录 stale conflict 标志自愈、#9 TB1 的 schema v29 诊断列与迁移回填、#2 2a 托盘不可用时的 freedesktop 系统通知回退、#50 的按任务正则排除（P1/P2 初始化剪枝、P3 读路径隐藏、B2 preview 挂载修复与 P4 写路径/调度防御与本地清理）、#11 stage0 的 core POSIX 门禁与 Windows 移植骨架、#11 step1 的文件身份契约、#6/issue #54 的已绑定目录恢复入口、#9 TB2 的提交诊断消费面、#10 的自动恢复分类、#14 残留 pending 上传补漏、#8 门禁稳健性合入 `integration/fixes3`（`22ba761` 之后的候选树）。schema29、IPC21：FSD 与 worker 必须成对更新，旧程序不能读取新 schema；升级前会自动 `VACUUM INTO` `state-before-v29-*.sqlite` 备份。

- **#14**：未确认上传在 5 次核对后进入 `abandoned` 暂停态，不再无限占用 `pending_operations`；GetIssues 提供「放弃本地文件」（只删本地、不改服务器条目）与重新核对入口，overview 用 `paused_uploads` 单独提示。放弃或本地删除时会一并作废该节点残留的 pending/uncertain 上传操作，避免遗留计数。
- **#19**：释放空间改为后台作业，带进度、取消与 released/skipped/protected 分类；每 32 个文件触发一次 GC 回收已删除节点的正文；已在线文件不再报假成功。
- **#18**：FUSE `link(2)` 现在返回 `EOPNOTSUPP`（Linux 上等同 `ENOTSUP`）而非 `EPERM`，工具可按标准语义降级为复制/改名；硬链接仍不支持，符号链接行为不变，详见 [FUSE_HARDLINK.md](FUSE_HARDLINK.md)。
- **#53**：后台提交/安装推进已提交状态水位时，现在会刷新 overview，托盘「需要检查」徽标与导航计数不再过期。
- **#8**：新增初始化期间桌面读响应性验收测试（210k 合成库、并发查询），本批无运行时行为变化；门禁稳健性调整为单调计数证据（attempts_throughout/DirectoryCommitted）并放宽并发用例超时预算（delayed directory 60s、FIFO 30s）；#8 全量门槛仍以 final-validate 为准。
- **#6**：已绑定目录在快照安装、mkdir/rename 确认成功后自动清除残留的 conflict 标志；真实冲突（远端删除、本地脏内容、未终态操作等）保留，规则见 [STALE_CONFLICTS.md](STALE_CONFLICTS.md)。
- **#9 TB1**：schema v29 只追加 `outcome`/诊断列、`schema_migrations`/`recovery_runs`/`platform_state_changes` 表与 `commit_diagnostics` 视图，并对历史 uncertain/inflight 行做幂等回填；本批只建抽象，不改变提交行为。
- **#2 2a**：托盘不可用或 showMessage 失败时，通过 freedesktop D-Bus 发送系统通知；jeepney/会话总线缺失时安静降级，不影响主流程。
- **#50 P1/P2**：profile 新增可选 `exclude_patterns`，初始化导入对匹配路径整棵子树剪枝，非法正则拒绝并报出 pattern 与编译错误，GUI 保存时做同样校验；详见 [EXCLUDE_PATTERNS.md](EXCLUDE_PATTERNS.md)。FUSE 读路径过滤（P3）与初始化 preview 挂载修复（B2）已在本批，见下；上传/下载/缓存调度防御与规则变更后的存量清理属阶段 4。
- **#50 P3**：配置了 `exclude_patterns` 的任务，读路径对匹配路径整棵子树返回 `ENOENT`（lookup/getattr/readdir(plus)/open/opendir），readdir 不显示且分页游标保持稳定；GetDirectory/GetStates 同样过滤，远端事件与恢复快照在入口剪枝；未配置规则时保持零成本快路径，详见 [EXCLUDE_PATTERNS.md](EXCLUDE_PATTERNS.md)。
- **#50 B2**：初始化 preview 阶段（committed root 尚未出现）带排除规则挂载不再因索引刷新失败导致后台退出；preview 阶段使用空索引，初始化完成或恢复快照安装后由 bridge 自动重建，排除路径随后保持不可见。
- **#50 P4**：写路径（create/mkdir/unlink/rmdir/rename/write/setattr）对排除路径返回 `ENOENT` 且不落库、不留 pending 操作；上传/下载/缓存与 namespace 调度入口跳过被排除对象；规则变更或重启时把已导入/已缓存的排除子树在本地隐藏并回收正文（复用 #19 GC，清 staging 与 `upload_paths`/`upload_origins`，绝不向远端提交删除），移除规则后重启可恢复本地视图；恢复快照 install 前先 purge 排除节点，避免 tombstone 把历史行标成冲突。索引重建按事件门控，详见 [EXCLUDE_PATTERNS.md](EXCLUDE_PATTERNS.md)。
- **#11 stage0**：新增 `scripts/check-core-posix.sh`，对 `drive-core` 的 POSIX 依赖做可审计门禁（默认模式已接入 `scripts/verify.sh`，当前基线为 63 个文件、108 处命中全部列在窄 allowlist；`--strict` 用于收紧基线）；同时新增 [WINDOWS_PORT.md](WINDOWS_PORT.md) 的 FileIdentity/StatePush/LocalChangeSource 三接口与去 POSIX 迁移计划，本阶段不改 Rust/schema/运行行为。
- **#6（issue #54）**：已绑定目录被远端删除后，GetIssues 仍以 `bound_directory_conflict` 展示，并提供 KeepDirectoryLocal / KeepDirectoryRemote（D-Bus additive 方法）把本地内容恢复为可继续同步的目录，规则见 [STALE_CONFLICTS.md](STALE_CONFLICTS.md)；keep-remote 仅在没有本地内容时可用。
- **#9 TB2**：五条写路径（upload/rename/mkdir/remove/replace）在同一事务写 `outcome`/`outcome_category`/`last_error`/`next_retry_at`；GetOverview 新增 additive `commit_diagnostics{open_unknown,needs_attention,items}`，abandoned 行带终态原因且不计入 open/needs_attention；旧 v29 库上的 `commit_diagnostics` 视图会在打开时自动修复（B1）。
- **#10**：rescan_later / merge_mode / 事件游标缺口归为 RecoveryRequired，bridge 单飞恢复循环最多 3 次（2/5/10 秒或 0/5/15 秒退避）自动重建快照或重新初始化，只有耗尽、认证/证书/根失效才需要人工；GetOverview/GetIssues 新增 additive `recovery{state,cause,attempts,next_retry_at,revision,attention}`，`recovery_revision` 推进触发 overview 刷新；无 schema 迁移、IPC 21 不变。
- **#11 step1**：新增 platform file identity 契约骨架（`PlatformFileId` + `FileIdentitySource` + Linux `fs_inodes` adapter，行为与 ino 语义不变），为 Windows CFAPI adapter 与 v30 `platform_file_ids` 留出替换点。

验证状态：本批候选树为 `integration/fixes3`（当前 HEAD `7336a54`，含 #50 阶段4、#11 step1、#9 TB2、#10、#6/issue #54、#19 M2）；`final-validate b14` 门禁进行中，正式计数、48 路与性能结论待其完成后回填，本文不宣称最终验收通过。发布前必修项已全部并入（#14 残留 pending op、#9 TB2 B1 视图修复、#8 门禁稳健性）。

以下保留此前发布记录。

# 0.1.0~dev20261108.1：上传终态、数据库热点与统一桌面

本版把 #14/#17 的未确认上传终态与背景错误清理、#15/#16 的本地 SQLite 索引与去全局锁读、#8 的已提交状态投影、#2 的统一窗口/托盘/持久同步历史与 #5 的 NamespaceIndex 精简投影合入主线（`8646766` 之后的候选发布版）。schema27、IPC21：FSD 与 worker 必须成对更新，旧程序不能读取新 schema。正目录项 entry TTL 恒 0，attr 默认 1000 ms。

未确认上传现在最多核对五次（10/30/120/300 秒退避）后进入可解释终态；本地删除会作废对应的未确认创建；权限失败不再写成粘性 background_error。问题与下载候选查询使用 v27 节点索引与 keyset 分页，GetIssues 移出全局 Store 锁；GetStates 由已提交状态投影的只读 WAL 快照提供。桌面端统一到一个管理窗口，保留托盘活动与持久同步历史，后台接管的 schema 期望同步到 27。

workstation202611 在 `544adb1` 树上验证（该树含 #2 `3dd03a1`、不含 #21–#30 后续修复；以下计数与 48 路为 `.1` 树口径）：workspace all-features 1097 通过 / 0 失败 / 52 忽略，default 1087 / 0 / 47；GUI 24 套 / 257 项通过；`scripts/verify.sh` 退出码 0，lab `--simulate` 报告 LocalComplete。210k 合成树 GetStates p95 312.9 ms（SLO 500 ms），2000 项目录列举 4.07 ms（Store）/0.727 ms（FUSE），持锁 2 秒时 GetStates/GetIssues 310/314 ms。48 路 mixed 负载仍有 28 次超时（SLO 0），原因是 GetOverview 的问题/缓存统计仍在全局锁内；#21 已将 GetOverview 迁移到只读连接。最终树（`integration/fixes2` @ `9cb6b1c`，release，clean checkout；`latency-clean-driver.log`，loadavg_end 10.88）48 路 mixed 超时 **0**（mixed p95 861.6 ms、max 1.64 s），overview total p95 0.708 ms、lock hold 0 ms、read-only calls 40/40，8/8 SLO PASS；全量 sweep（`final-final2-9cb6b1c.log`，12 phase 全部 `PHASE-EXIT=0`）SUMMARY **passed=2359 / failed=0 / ignored=114 / failed_binaries=0**：workspace all-features **1176 通过 / 0 失败 / 59 忽略**、default **1166 / 0 / 54**、GUI **24 套 / 276 项**、kernel-mounted 串行 **7/7**、`scripts/verify.sh` 退出码 0（`LocalComplete`）。本版不宣称 #8 全量达标。

#2、#5、#8、#14、#17 保持 open（#15/#16 已由作者关闭）；真实 NAS ACL、真实 dpkg 安装/升级和 Mint 桌面验收未执行。本包为开发版，包生成、实际 ELF/GUI 内容与摘要以同交付的 `REPORT.md`、`SHA256SUMS` 和 `VALIDATION.json` 为准。

以下保留此前发布记录。

# 0.1.0~dev20261105.1：目录权限、读取拒绝与持久同步历史

本地开发版修正“根目录不可写导致所有子目录被整盘只读”的判断：用户只读策略与逐目录、逐动作权限分开，未知权限允许尝试并等待服务端裁决。已确认的读取拒绝返回 EACCES，阻止后台自动重试；权限重新核对、重连和原生 profile 变更会使旧观测失效。取消最后一个读取需求时清理其重试意图，保留独立固定、缓存和其他读取需求。

旧任务的 `writable=false` 保持“用户设为只读”。需要恢复按远端权限判断时，在“读写设置”选择“按远端目录权限同步”，保存后停止并重新启动该挂载；“重新核对”不会替用户改变只读设置。正常更新保留配置、数据库、正文缓存和未上传修改，操作步骤见[升级恢复](UPGRADE_RECOVERY.md)。本次没有修改用户现有任务或安装到宿主。

本版同时包含统一窗口、托盘与持久同步历史，以及 NamespaceIndex 路径。schema24为历史、25为权限，IPC21要求FSD与worker成对更新；旧程序拒绝读取新schema，不可只替换一个组件。正目录项entry TTL恒0、attr默认1000ms。

产品冻结 `9c9aaba` 的独立工作区测试all-features为976/0/42、default为966/0/37；debug/release显式FUSE各38项、GUI148项、210k容量1项、真实本机worker链10项、组件数据保留8项通过。完整计数、失败原件与来源边界见[#7报告](FIX_7_REPORT.md)。这份说明属于构建源码；包生成、实际打包ELF/GUI及文件摘要以同交付的 `REPORT.md` 和 `SHA256SUMS` 为准。配套 `drive-native-source.tar.gz` 提供离线完整证据，不依赖未发布提交的网页。

真实NAS ACL、精确写拒绝自动学习、独立空间限制来源、Mint和真实dpkg生命周期未实测或未完成，#7完整验收仍开放。新候选没有重跑性能基准；此前[#5配对基准](FIX_5_2_REPORT.md)未达到近本地目标。此包不关闭普通目录全量同步或长期验收门槛，本轮不push、不发布。

以下保留此前发布记录。

# 0.1.0~dev20261102.7：停止恢复、统一窗口与初始化局部性能

[包验证012](validation/native-package-012/REPORT.md)记录冻结源码、实际构建、安装和只读NAS范围。FUSE停止可唤醒并排空本地写入；GUI统一多挂载、单实例和托盘；相同身份目录更新保留发现记录并重新核对覆盖。751项Rust、96项GUI、debug/release各23项显式内核，隔离暂停升级及重装保留两份未上传修改通过。

真实个人目录约2.083秒可浏览根元数据，600秒内完整初始化未完成；两次响应超时后继续发现目录，最终正常停止。局部合成事务245.040→33.390ms，不能换算为整树NAS速度。事件接收／应用序号均0，非空事件恢复未验收。普通目录全量同步、持久活动历史、Mint及最终验收继续开放。

以下保留此前版本发布记录。

# 0.1.0~dev20261101.6：冷启动目录预览

当前开发源码已接入[只读目录预览](INITIALIZATION_PREVIEW.md)。选定根经过验证后可挂载；完整直接列表持久接收后可浏览元数据，未完成目录明确暂不可用。预览文件正文和本地修改仍不可用，直到全局候选通过原有fence并完成原子转正；同一挂载随后使用正常按需读取。warm任务继续原权威树、缓存和草稿路径。

本批还整合[原生事件证据与已知目录局部重查](INITIALIZATION_EVENT_EVIDENCE.md)、[目录视图缓存修复](DIRECTORY_VIEW_STAMP.md)。预览包含持续权限拒绝、根撤销、孤立草稿保护、受预算约束的inode及旧句柄失效。未知位置仍可能全树回退，不等于初始化已收敛。

本开发包已[发布](https://github.com/Austrust/drive-native/releases/tag/dev20261101.6)，冻结`7683b36dcbce91b5e6143ef17bb8e97882f0453b`，schema22／IPC18。[包验证011](validation/native-package-011/REPORT.md)记录743项Rust、75项GUI，debug／release各20项显式内核测试通过；隔离Ubuntu完成schema20→22迁移、暂停接管、备份、旧ELF拒绝和同版／purge重装，保留两份pending编辑及dirty草稿。独立Nemo／Nautilus各10项通过。

001普通Root复核误屏蔽预览与旧kernel mode断言失败已修复，原记录保留，001未执行到新preview target。[源码002](validation/progressive-preview-002/REPORT.md)独立通过，不累加到完整包总数。构建时`dh_clean`清理了未编译的`.orig`审计清单，依据冻结归档恢复后完整源码核验通过；升级前置因该差异拒绝启动，内核前置另曾因缺commit参数拒绝启动，修正后才运行，两次均未启动相应测试。没有降低身份核验要求。

本轮未进行新源码NAS或配对性能实测。升级未覆盖旧初始化候选／未知提交，也未观察或强制构造非空WAL。组件须成对更新，正常升级保留数据不清缓存，流程见[升级恢复](UPGRADE_RECOVERY.md)。

普通目录全量同步、Mint完整桌面、配对性能、最终数据安全及7×24门槛保持开放；本包目标仍为开发版，不是rc1。已发布 `.5` 的失败初始化与验证见[包010](validation/native-package-010/REPORT.md)。以下保留历史版本发布说明，版本号对应各自当时状态。

---

# 0.1.0~dev20261101.1：当前水位起点与局部扫描收敛

选定子目录通过[原生当前水位与当前根查询](SNAPSHOT_WATERMARK.md)开始完整枚举，不再为了起点必须读取全部旧历史。Personal 子目录的真实只读协议验证通过；顶层 `/` 与已知父列表权限拒绝保留历史路径。

[局部收敛](SNAPSHOT_CONVERGENCE.md)让普通文件变化只重查相关目录，再核对后续事件；目录结构、权限或身份不明确仍安全回退。没有去掉 fence，没有提前写已提交游标，没有把局部缺项当全域删除。新增水位与局部协调进度，保留此前目录、缓存读取和下载校验锁隔离修复。

schema18 不变，IPC 升至15，必须成对更新 FSD/worker。保存并关闭挂载文件、暂停后安装，再在 GUI 选择“应用后台更新”；确认挂载与暂停状态后恢复。正常升级不清缓存。完整构建、只读初始化和安装验收见发行包 `VALIDATION.json`。

本包仍为开发版。逐目录提前可见、跨重启续扫[尚未实现](INITIALIZATION_INCREMENTAL_PLAN.md)，普通目录全量同步、全部 MFM/Personal 故障、Mint 和最终长期数据安全门槛保持开放。

---

# 0.1.0~dev20261031.5：原生恢复与快照导入修复

同一次原生恢复复用已完成的事件 seed 边界，继续枚举并核对事件，不跨重连冒用旧边界。完整快照采用事务内一次失效与 prepared statements 复用；[固定配对测试](validation/status-write-cost-003/REPORT.md)确认 21 万节点导入耗时改善，保留前两失败窗口和增量写入开销。

[下载正文准备](DOWNLOAD_PUBLICATION.md)将复制、校验和正文持久化移出共享元数据锁，提交时重新核对文件身份、版本及本地修改；保留完整旧版和并发 GC 保护。暂停后已经收到的完整正文仍可能完成本地落盘，不发起新的网络下载。此项通过确定性并发和数据保护回归，不替代真实 NAS 下载性能验收。

本批还包含有界 WAL 读取和带核验票据的瞬态状态基础，**没有切换当前桌面状态查询路径**。这些模块不等于 Nemo 全树性能或 Personal 初始化已经验收。MFM 真实连接、普通目录全同步、Mint 实机和最终数据安全/长期关卡继续开放。最终包构建与安装验证见发行附件 `VALIDATION.json`。

schema 18、IPC 14 不变。保存并关闭挂载中的文件，暂停任务后安装新包；在新版 GUI 中使用“应用后台更新”，确认挂载与暂停状态后再恢复。正常升级保留配置、数据库、缓存和未上传草稿，不要求清缓存。完整步骤见[升级恢复](UPGRADE_RECOVERY.md)。本包为开发版，不是 rc1。

---

# 0.1.0~dev20261031.4：目录视图、缓存关闭与升级恢复

本批整合[稳定目录视图](FUSE_DIRECTORY_VIEWS.md)、[完整缓存只读关闭隔离](CACHED_CLOSE_ISOLATION.md)、[初始化阶段进度](INITIALIZATION_PROGRESS.md)和[暂停与后台升级](UPGRADE_RECOVERY.md)。420项Rust、71项GUI、13项debug/release显式FUSE、安装升级与桌面验证完成，见[包验证及保留失败](validation/native-package-004/REPORT.md)；可下载状态以GitHub Releases为准。

schema 18 不变，fsd/worker 使用 IPC 14。新版 GUI 的“应用后台更新”会先持久保存暂停意图，正常停止旧任务，再确认新后台身份、挂载与暂停状态；正常升级保留数据库、缓存和未上传草稿。操作步骤和失败恢复见升级文档。

目录修复的固定对比已改善单目录展开；全树状态和 48 路混合目标仍未通过。新增阶段显示不等于 Personal_Sync 初始化问题已解决。普通目录全量同步、Mint 实机、完整故障与长期验收继续开放；本包是开发构建，不是候选版。

---

# 0.1.0~dev20261031.3：通知诊断与已打开内容读取隔离

本包已发布，389项Rust、9项显式FUSE（debug/release）、49项GUI及隔离安装生命周期通过，见[发布验证](validation/native-package-003/REPORT.md)。新增[接收进展驱动的通知期限](NOTIFICATION_IDLE.md)、[退出前连接诊断](CONNECTION_DIAGNOSTICS.md)及[已打开文件读取隔离](FUSE_READ_ISOLATION.md)。最后一项避免元数据线程全部等待Store时拖住已有缓存/草稿的读取，不代表新打开文件、大目录及48路混合性能已经达标。

schema18不变，IPC升级为13。升级或回退前正常停止所升级任务并退出旧GUI，再成对更换fsd/worker；不能把新worker和仍运行的旧fsd混用。保留用户数据库、缓存、草稿及未上传数据。隔离的原P0观察实例不替换。

三范围有界只读NAS观察完成，本轮没有覆盖>450秒的真实通知；Personal_Sync连续变化通知的推进语义及约59秒停止耗时仍需调查。真正普通目录全量同步、Mint实机、故障注入和七日验收未完成。本包为开发版本，不是rc1。安装及回退沿用下方流程，包名替换为本版本；实时可下载状态以GitHub Releases为准。

---

# 0.1.0~dev20261031.2：桌面与目录分页开发构建

本轮整合 GIO 枚举器回收修复、Nemo/Nautilus 共用状态扩展、仅状态改变时刷新图标、GUI 按需展开目录及隐藏页面停止查询、SQLite 目录分页和登录错误分类。逐项来源及实际验证见 [整合记录](UBUNTU_INTEGRATION.md)。数据库 schema 18、IPC 12 不变。

本版本仍是开发构建。MFM 断连、Personal_Sync 恢复、FUSE 队列与真正全量同步未在本批修复；不能据此宣称日常或候选验收完成。发布安装包及构建验证以 GitHub Releases 为准。

## 选择文件管理器集成

一个包包含两个扩展入口，共用本地状态实现。依赖使用 `nemo-python | python3-nautilus`，相应 GI 由绑定包带入；APT 不会根据当前桌面自动选择第二项。请明确选择实际使用的文件管理器绑定：

```sh
# Linux Mint Cinnamon / Nemo
sudo apt install nemo-python ./drive-native_0.1.0~dev20261031.2_amd64.deb
# Ubuntu 24.04 GNOME / Nautilus
sudo apt install python3-nautilus ./drive-native_0.1.0~dev20261031.2_amd64.deb
```

升级前正常停止需要升级的挂载并退出旧 GUI，保留未上传内容及状态备份。安装不启动挂载、不结束用户文件管理器；已运行的文件管理器需要在方便时重新启动才能加载新扩展。原冻结基线使用独立路径，保持原观察窗口。普通卸载和清除包配置均不删除用户数据。

---

# 0.1.0~dev20261031.1：P0 基线开发构建

本轮改善安装包中的说明文档与源码可追溯性：主要说明可在 `/usr/share/doc/drive-native/` 离线阅读，未随包附带的证据链接指向构建对应的Git提交；`SOURCE_COMMIT` 和 `BUILD_VERSION` 标识安装内容。原始测试数据、实验数据库和账号配置不进入deb。

运行时、GUI和Nemo行为与 `.6` 相同，数据库schema 18、IPC 12保持不变。本版本没有宣称解决已测出的性能问题，也不表示P0或候选验收通过。开发工具、合成夹具和脱敏基线证据随源码归档提供；发布产物中的 `validation/checks.json` 记录该次构建实际完成的检查。

## 已知范围与风险

- [固定本地基线](P0.md)暴露出大目录/状态/缓存并发访问超时，以及约21万节点任务高内存占用。
- 真实只读断连试验中MFM和Personal_Sync存在恢复超过3秒的样本；本地缓存和目录在相应试验中仍可访问。
- 重复冷读取可能收到服务器32772拒绝并等待重试，其完整协议语义未恢复。
- [持久性边界](DURABILITY.md)只补齐普通生命周期观察；目录fsync、崩溃、断电和命名空间竞争仍有待验证项。
- Mint实机完整桌面/休眠及七日冻结验收尚未完成。阶段状态只见 [DELIVERY_GATES.md](../DELIVERY_GATES.md)。

## 升级与回退

升级前在界面正常停止各挂载，确认应用中打开的文件已保存，再退出旧GUI。保留任务配置、数据库、缓存和草稿的完整备份，尤其是未上传内容；不要直接删除状态目录。

用发行版软件安装器安装新deb，或执行 `sudo apt install ./drive-native_0.1.0~dev20261031.1_amd64.deb`，再启动GUI与挂载。安装过程不自动启动用户挂载。可用 `dpkg-query -W drive-native` 以及包内 `BUILD_VERSION`、`SOURCE_COMMIT` 核对身份。

本轮没有schema迁移。若需回退，正常停止新后台并退出GUI后安装保留的 `.6` deb（apt命令使用 `--allow-downgrades`）；不要在后台仍运行时交换文件或恢复数据库。回退后的缓存/草稿仍需保持，不能把普通卸载当作清空用户数据。

24小时基线观察继续使用冻结的 `.6` 独立实例，不因新开发包而重计或拼接运行时间。
