# 0.1.0~dev20261031.4：目录视图、缓存关闭与升级恢复

本批源码整合[稳定目录视图](FUSE_DIRECTORY_VIEWS.md)、[完整缓存只读关闭隔离](CACHED_CLOSE_ISOLATION.md)、[初始化阶段进度](INITIALIZATION_PROGRESS.md)和[暂停与后台升级](UPGRADE_RECOVERY.md)。组件回归已完成；本快照尚待最终包构建与安装验证，可下载状态以 GitHub Releases 为准。

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
