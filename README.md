# Drive Native

面向 Linux Mint Cinnamon / Nemo 的 Synology Drive 原生协议按需同步客户端。使用 Rust、FUSE、SQLite、PySide6 和 Nemo 扩展；独立连接 Drive Server，不依赖 WebDAV 或官方客户端进程。

**开发测试版本，尚未完成日常使用验收。** `0.1.0~dev20261101.1` 加入[原生当前水位起点](docs/SNAPSHOT_WATERMARK.md)和[普通文件变化局部收敛](docs/SNAPSHOT_CONVERGENCE.md)，保留此前按需、目录与下载锁隔离修复。[本轮验证结论](docs/validation/native-package-006/REPORT.md)及发行包 `VALIDATION.json` 记录真实只读初始化、安装与保留失败；下载见 [GitHub Releases](https://github.com/Austrust/drive-native/releases)。逐目录提前显示、跨重启续扫与普通目录全量同步仍未实现，最终数据安全/长期验收继续保留。

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

本开发分支新增[持久初始化候选基础](docs/INITIALIZATION_PERSISTENCE.md)及类型化重试原因，SQLite schema 为19、内部 IPC 为16。候选API和写回门禁已进入源码，生产初始化调度器尚未接入；逐目录提前显示与跨重启自动续扫仍不可用。本轮变更待统一验证，尚无新安装包交付。

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

| 目录 | 内容 |
| --- | --- |
| [crates](crates/) | 原生协议、状态核心、FUSE 与服务 |
| [desktop](desktop/) | Qt GUI、Nemo、GIO 与用户服务 |
| [scripts](scripts/) | 构建、测试及显式运行的研究工具 |
| [debian](debian/) | Debian 打包定义 |
| [vendor](vendor/README.md) | 固定的 fuser 补丁及上游许可 |
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
| [docs/INITIALIZATION_PERSISTENCE.md](docs/INITIALIZATION_PERSISTENCE.md) | 私有候选事务、票据、写回门禁与尚未接入的生产调度器 |
| [docs/SNAPSHOT_CONVERGENCE.md](docs/SNAPSHOT_CONVERGENCE.md) | 扫描期间普通文件变化的局部重查、完整 fence 与保守回退；真实 NAS 性能待验证 |
| [docs/SNAPSHOT_ERROR_CLASSIFICATION.md](docs/SNAPSHOT_ERROR_CLASSIFICATION.md) | 枚举失败的相关竞态验证；保留原一致性围栏 |
| [docs/UPGRADE_RECOVERY.md](docs/UPGRADE_RECOVERY.md) | 暂停、正常升级接管、已发布包验证与schema19迁移待验边界 |
| [docs/STATUS.md](docs/STATUS.md) | 验证范围、已知问题及后续工作 |
| [docs/PROVENANCE.md](docs/PROVENANCE.md) | 源码归档来源和版本关系 |

仓库只保存源码、合成或脱敏测试夹具和维护文档。实验数据库、账号配置、NAS 文件、官方二进制与旧安装包不进入 Git 历史。项目及第三方许可见 [LICENSE](LICENSE)、[debian/copyright](debian/copyright) 和各依赖目录。
