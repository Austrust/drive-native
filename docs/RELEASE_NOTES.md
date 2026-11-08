# 0.1.0~dev20261108.1：上传终态、数据库热点与统一桌面

本版把 #14/#17 的未确认上传终态与背景错误清理、#15/#16 的本地 SQLite 索引与去全局锁读、#8 的已提交状态投影、#2 的统一窗口/托盘/持久同步历史与 #5 的 NamespaceIndex 精简投影合入主线（`8646766` 之后的候选发布版）。schema27、IPC21：FSD 与 worker 必须成对更新，旧程序不能读取新 schema。正目录项 entry TTL 恒 0，attr 默认 1000 ms。

未确认上传现在最多核对五次（10/30/120/300 秒退避）后进入可解释终态；本地删除会作废对应的未确认创建；权限失败不再写成粘性 background_error。问题与下载候选查询使用 v27 节点索引与 keyset 分页，GetIssues 移出全局 Store 锁；GetStates 由已提交状态投影的只读 WAL 快照提供。桌面端统一到一个管理窗口，保留托盘活动与持久同步历史，后台接管的 schema 期望同步到 27。

workstation202611 在 `544adb1` 树上验证（该树含 #2 `3dd03a1`、不含 #21–#30 后续修复；以下计数与 48 路为 `.1` 树口径）：workspace all-features 1097 通过 / 0 失败 / 52 忽略，default 1087 / 0 / 47；GUI 24 套 / 257 项通过；`scripts/verify.sh` 退出码 0，lab `--simulate` 报告 LocalComplete。210k 合成树 GetStates p95 312.9 ms（SLO 500 ms），2000 项目录列举 4.07 ms（Store）/0.727 ms（FUSE），持锁 2 秒时 GetStates/GetIssues 310/314 ms。48 路 mixed 负载仍有 28 次超时（SLO 0），原因是 GetOverview 的问题/缓存统计仍在全局锁内；#21 已将 GetOverview 迁移到只读连接。最终树（`integration/fixes2` @ `9cb6b1c`，release，clean checkout；`latency-clean-driver.log`，loadavg_end 10.88）48 路 mixed 超时 **0**（mixed p95 861.6 ms、max 1.64 s），overview total p95 0.708 ms、lock hold 0 ms、read-only calls 40/40，8/8 SLO PASS；全量 sweep（`final-final2-9cb6b1c.log`，12 phase 全部 `PHASE-EXIT=0`）SUMMARY **passed=2359 / failed=0 / ignored=114 / failed_binaries=0**：workspace all-features **1176 通过 / 0 失败 / 59 忽略**、default **1166 / 0 / 54**、GUI **24 套 / 276 项**、kernel-mounted 串行 **7/7**、`scripts/verify.sh` 退出码 0（`LocalComplete`）。本版不宣称 #8 全量达标。本版不宣称 #8 全量达标。

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
