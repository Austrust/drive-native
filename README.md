# Drive Native

开发包与安装说明 — FSearch 提速、故障恢复操作和验证边界：[交付说明](docs/RELEASE_20261103_9.md)。

以下保留此前交付记录；当前验收统一从 [DELIVERY_GATES.md](DELIVERY_GATES.md) 进入。

[`.8` 开发包已发布](https://github.com/Austrust/drive-native/releases/tag/dev20261102.8)，包含[初始化流水线与阶段诊断](docs/INITIALIZATION_PIPELINE.md)、[队列恢复与目录属性优化](docs/ISSUES_20261103.md)和[文件时间写回](docs/FILE_TIMESTAMPS.md)。配套 FSD／worker 使用 IPC20、数据库 schema23；源码为 `416cc41b11fce2774bc8eb1e899e100f3d896b8f`。

用户决定发布后停止目标；已发布经过验证的[包013](docs/validation/native-package-013/REPORT.md)。[真实Lab004](docs/validation/native-lab-004/REPORT.md)通过非空事件、后台缓存保鲜及文件时间写回；[原版FSearch扫描](docs/validation/fsearch-001/REPORT.md)首轮约170→79秒、重扫162→70秒。近本地性能及Personal_Sync完整初始化仍未通过；后续缓存实验和目录等待分支未纳入本包。[Personal_Sync一小时终态](docs/validation/personal-003/REPORT.md)确认非空事件推进，但完整初始化未通过。发布范围见[本版结论](docs/RELEASE_20261102_8.md)。

`.7` 已发布，完成[停止恢复、多挂载界面与初始化修复](docs/ISSUES_20261102.md)及[包验证012](docs/validation/native-package-012/REPORT.md)。冻结源码 `c6f0a80f22180654d696830d7d4c47beaad9a6b5`，schema22／IPC18；751项Rust、96项GUI、debug／release各23项显式内核通过，隔离升级和重装保留未上传修改。[下载开发包](https://github.com/Austrust/drive-native/releases/download/dev20261102.7/drive-native_0.1.0.dev20261102.7_amd64.deb)，六项资产SHA256已核验；详情见[Release](https://github.com/Austrust/drive-native/releases/tag/dev20261102.7)。

历史开发包 `0.1.0~dev20261101.6` 已[发布](https://github.com/Austrust/drive-native/releases/tag/dev20261101.6)，冻结 `7683b36dcbce91b5e6143ef17bb8e97882f0453b`，schema22／IPC18。[冷启动目录预览](docs/INITIALIZATION_PREVIEW.md)让完整接收的目录先显示元数据，正文仍须等全局初始化完成；同时整合[已知目录局部重查](docs/INITIALIZATION_EVENT_EVIDENCE.md)与[目录视图缓存修复](docs/DIRECTORY_VIEW_STAMP.md)。[包验证011](docs/validation/native-package-011/REPORT.md)记录743项Rust、75项GUI，debug／release各20项显式内核测试，以及schema20→22迁移和未上传数据保留。该冻结源码在发布时尚无NAS或配对性能实测；随后短时预览观察另行记录。

历史开发包 `.5` 的[初始化事件依赖等待与有界恢复](docs/CANDIDATE_EVENT_COVERAGE.md)已[发布](https://github.com/Austrust/drive-native/releases/tag/dev20261101.5)。本包本次真实只读初始化未通过；受控重启续扫完成=True，状态查询异常0次。最终0.266秒正常停止且无遗留挂载；未写NAS。实测期间宿主仍有开发活动，远端变更负载未控制，不作与旧版配对性能或长期验收结论。

历史 `.4` SQL修复及真实失败见[原报告](docs/CANDIDATE_QUERY_ORDER.md)与[包验证009](docs/validation/native-package-009/REPORT.md)，不与本轮结果混计。

面向 Linux Mint Cinnamon / Nemo 的 Synology Drive 原生协议按需同步客户端。使用 Rust、FUSE、SQLite、PySide6 和 Nemo 扩展；独立连接 Drive Server，不依赖 WebDAV 或官方客户端进程。

使用与验收范围见[当前交付说明](docs/RELEASE_20261103_9.md)；历史观察不能替代新代码的验证。

## 已实现

当前完整目标见 [GOAL.md](GOAL.md)，**唯一验收状态**见 [DELIVERY_GATES.md](DELIVERY_GATES.md)。native 为唯一主产品线；官方适配冻结为协议参照与备用分支。P0 的 WSL 基线、实验口径与续接位置见 [docs/P0.md](docs/P0.md)。

- 多挂载、选择个人或团队子目录、自定义挂载名称。
- 按需读取、已缓存文件后台更新、固定与释放缓存。
- 本地写入、离线队列、冲突保留和持久恢复。
- Qt 配置界面、Nemo 状态与菜单、systemd 用户服务。
- 分层目录权限过滤、大目录枚举与符号链接过滤。

完整稳定性、部分命名空间竞态及 Mint 桌面验收仍开放；详见 [验证状态](docs/STATUS.md)。

最新诊断结论见 [2026-10-31 性能与同步问题摘要](docs/PERFORMANCE_FINDINGS_20261031.md)。本次仅公开结论，详细证据保留本地；这些诊断不代表产品修复或候选验收完成。

Ubuntu更新包已逐项整合，继续在独立分支修复原生恢复及按需性能；[采用决定](docs/UBUNTU_INTEGRATION.md)。原P0窗口已结束；用户认可的23小时时长已满足，连接异常仍待归因，见[窗口终审](docs/validation/p0-window2-final/REPORT.md)。这不代表新代码已通过长期验收。

[恢复及增量状态基础](docs/validation/incremental-status-001/REPORT.md)已完成组件整合；[批量导入成本回退已修复](docs/validation/status-write-cost-003/REPORT.md)。运行时增量桌面状态尚未接入；不将这些基础模块算作 Personal 或整体性能问题已经解决。

当前生产接线和可靠FD分步接口见[持久初始化](docs/INITIALIZATION_PERSISTENCE.md)。已发布 `.5` 的本次真实只读初始化未通过；受控重启续扫完成=True，状态查询异常0次。最终0.266秒正常停止且无遗留挂载；未写NAS。实测期间宿主仍有开发活动，远端变更负载未控制，不作与旧版配对性能或长期验收结论。 历史基础、owner和SQL修复保留各自验证范围。

## 构建和测试

隔离开发中的[分页只读 WAL 状态快照](docs/STATUS_WAL_SNAPSHOT.md)用于大目录索引初始化，尚未接入桌面；验证状态和调用边界见该文档。

在 Ubuntu 24.04 / Linux 文件系统中构建。准备 Rust 1.89+、Debian control 中列出的构建依赖和独立 Python 虚拟环境；历史包构建使用 Rust 1.99.1。

```sh
cargo test --workspace --all-features --locked
python3 -m venv .venv-build
.venv-build/bin/pip install -r scripts/gui-requirements.lock
export GUI_PYTHON="$PWD/.venv-build/bin/python"
# Git checkout must be clean.
export DRIVE_NATIVE_SOURCE_COMMIT="$(git rev-parse HEAD)"
chmod +x debian/rules debian/drive-native.postinst debian/drive-native.postrm
dpkg-buildpackage -b -us -uc
```

从发布的源码归档构建时，将配套 `SOURCE_COMMIT` 文件保留在解压后的 `source/` 旁边；在该源码目录中改用 `export DRIVE_NATIVE_SOURCE_COMMIT="$(cat ../SOURCE_COMMIT)"`。

安装生成的 `.deb` 后从应用菜单打开 Drive Native。首次配置连接地址、证书服务器名称及账号，选择同步目录。凭据由用户输入，通过 Secret Service 保存；不要将凭据或用户数据库提交到 Git。

## 导航

FSearch 与故障操作 — 目录性能、缓存安全和问题恢复的实现与验证入口：[说明](docs/FS_RECOVERY_20261103.md)。

| 目录 | 内容 |
| --- | --- |
| [crates](crates/) | 原生协议、状态核心、FUSE 与服务 |
| [desktop](desktop/) | Qt GUI、Nemo、GIO 与用户服务 |
| [scripts](scripts/) | 构建、测试及显式运行的研究工具 |
| [debian](debian/) | Debian 打包定义 |
| [vendor](vendor/README.md) | 固定的 fuser 补丁及上游许可 |
| [docs/INITIALIZATION_PREVIEW.md](docs/INITIALIZATION_PREVIEW.md) | 冷启动只读目录预览、ACL撤销、原子转正及待验证边界 |
| [docs/P1_ARCHITECTURE.md](docs/P1_ARCHITECTURE.md) | P1修改契约；通知诊断与部分读取隔离已实现，完整性能验收仍开放 |
| [docs/FUSE_READ_ISOLATION.md](docs/FUSE_READ_ISOLATION.md) | 已打开内容读取隔离的实现、回归与未完成范围 |
| [docs/DOWNLOAD_PUBLICATION.md](docs/DOWNLOAD_PUBLICATION.md) | 原生下载锁外复制、校验与持久化，以及短 Store 发布和 GC 保留 |
| [docs/CACHED_CLOSE_ISOLATION.md](docs/CACHED_CLOSE_ISOLATION.md) | 只读缓存同步与关闭绕开元数据阻塞的路由、持久化边界和内核回归 |
| [docs/FUSE_DIRECTORY_VIEWS.md](docs/FUSE_DIRECTORY_VIEWS.md) | 目录轻量投影、稳定续读句柄与独立验证范围 |
| [docs/STATUS_CHANGE_JOURNAL.md](docs/STATUS_CHANGE_JOURNAL.md) | 事务性状态变化日志基础层；尚未接入增量桌面聚合 |
| [docs/STORE_COMMIT_NOTIFICATIONS.md](docs/STORE_COMMIT_NOTIFICATIONS.md) | 统一 Store guard、已提交语义失效与非阻塞合并唤醒 |
| [docs/INCREMENTAL_STATUS_INDEX.md](docs/INCREMENTAL_STATUS_INDEX.md) | 纯增量聚合与事务性树拥有者；运行订阅、发布屏障和桌面接入待实现 |
| [docs/STATUS_TRANSIENT.md](docs/STATUS_TRANSIENT.md) | 独立瞬态水位、多版本下载与正文核验票据；真实生命周期接入待完成 |
| [docs/STATUS_PROJECTION.md](docs/STATUS_PROJECTION.md) | 同一 Store 边界的提交投影与有界快照；尚未接入桌面 |
| [docs/INITIALIZATION_PROGRESS.md](docs/INITIALIZATION_PROGRESS.md) | 初始化阶段、真实计数、类型化重试原因与 IPC 配对边界 |
| [docs/INITIALIZATION_PERSISTENCE.md](docs/INITIALIZATION_PERSISTENCE.md) | 私有候选事务、票据、写回门禁与生产调度器及待验边界 |
| [docs/SNAPSHOT_CONVERGENCE.md](docs/SNAPSHOT_CONVERGENCE.md) | 扫描期间普通文件变化的局部重查、完整 fence 与保守回退；真实 NAS 性能待验证 |
| [docs/SNAPSHOT_ERROR_CLASSIFICATION.md](docs/SNAPSHOT_ERROR_CLASSIFICATION.md) | 枚举失败的相关竞态验证；保留原一致性围栏 |
| [docs/UPGRADE_RECOVERY.md](docs/UPGRADE_RECOVERY.md) | 暂停、正常升级接管、正常升级与历史迁移验证与保留边界 |
| [docs/STATUS.md](docs/STATUS.md) | 验证范围、已知问题及后续工作 |
| [docs/PROVENANCE.md](docs/PROVENANCE.md) | 源码归档来源和版本关系 |

仓库只保存源码、合成或脱敏测试夹具和维护文档。实验数据库、账号配置、NAS 文件、官方二进制与旧安装包不进入 Git 历史。项目及第三方许可见 [LICENSE](LICENSE)、[debian/copyright](debian/copyright) 和各依赖目录。
