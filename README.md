# Drive Native

面向 **Linux Mint 22.x Cinnamon / Nemo** 的群晖 Drive 按需同步客户端。使用 Rust、FUSE、SQLite、Qt/PySide6 和 Nemo/GIO 扩展实现，直接以原生协议连接 Drive Server，不依赖 WebDAV，也不依赖官方客户端进程。

当前状态：**开发版（Pre-release），持续开发中**。最新开发包为 [`0.1.0~dev20261113.1`](https://github.com/Austrust/drive-native/releases/tag/dev20261113.1)，数据库 schema **29**、IPC **21**（本版无迁移）；发布源码提交即 `dev20261113.1` tag（候选包 `SOURCE_COMMIT` 记录实际 SHA）。本版是开发包，不是 rc1；真实 NAS 复核、真实 dpkg 安装／升级与 Mint／Cinnamon 桌面验收仍未完成。注意排除规则默认行为已变为「本地保留、不同步」，需要旧行为时在 profile 中设置 `"exclude_mode": "hidden"`，详见[本版说明](docs/RELEASE_20261113_1.md)。

## 核心能力

- **按需同步**：目录按需列出、文件按需下载；已缓存文件在后台保鲜；支持固定与释放缓存，大目录释放走可取消的后台任务并显示进度。
- **多挂载**：可挂载个人空间或团队空间下的任意子目录，自定义挂载名称，多个挂载并行工作。
- **本地写入**：写入走离线队列，支持草稿持久化、冲突保留与重启/断线恢复。
- **桌面集成**：Qt 配置界面、Nemo 状态图标与右键菜单、systemd 用户服务、GIO 元数据探针、托盘不可用时回退 freedesktop 系统通知。
- **权限与规模**：分层目录权限（ACL）过滤、大目录枚举、符号链接过滤，以及面向 20 万级以上节点的分页与增量视图。
- **可观测与可恢复**：各阶段诊断计数、失败降级与自动恢复、数据库 schema 迁移前自动备份。

## 本版性能

`0.1.0~dev20261109.1` 合入 #21–#34 的并发、锁与存储修复。在 32 核 / 125 GiB 构建机、210k–500k 节点合成夹具与 lab FUSE 上实测：

| 指标 | 本版 | 修复前 |
|---|---:|---:|
| 48 路混合负载超时 | **0**（p95 861.6 ms，max 1.64 s） | 28 次 |
| GetOverview 总耗时 p95 / 持锁 | **0.708 ms / 0 ms** | 持锁可达 13 s |
| 210k 树 GetStates p95 | **226.95 ms** | 312.9 ms |
| 只读 open 的 WAL 写入 / NSI 重建 | **0 / 0** | 每次 open 一次 fsync + 整代重建 |
| 100 MiB 顺序写 | **4 row changes / 18 fsync** | 3200 / 817 |
| 10k remove 积压的 barrier | **0.016 ms** | 236 ms |
| ACL lookup p95（50k denied） | **21.3 µs** | 95.6 ms |
| 1M 行 pending_operations | **0.0014 ms** | 46.3 ms |
| schema v27→v28 升级（294k 节点） | **614 ms**（含 169 ms 备份） | — |

以上数字来自合成夹具与 lab FUSE，**未覆盖真实 NAS、真实桌面会话与长期数据安全**；完整口径与 before/after 见[本版发布说明](docs/RELEASE_20261109_1.md)与 PR #35–#49。

## 架构与目录

```
crates/
  drive-core/     状态存储、SQLite schema、同步与初始化核心
  drive-fs/       FUSE 文件系统与 namespace 索引
  drive-runtime/  后台服务 fsd、worker、bridge、桌面/状态 IPC
  sproto/         群晖 Drive 原生协议实现
desktop/          Qt/PySide6 GUI、Nemo/Nautilus 扩展、GIO 探针
scripts/          构建、测试、打包与显式运行的研究工具
debian/           Debian/.deb 打包定义
vendor/           固定的 fuser 补丁与上游许可
docs/             设计契约、验收状态、历史报告与发布说明
```

关键文档：

- 验收目录与阶段状态：[DELIVERY_GATES.md](DELIVERY_GATES.md) · 完整目标：[GOAL.md](GOAL.md)
- 实现契约与版本边界：[docs/CONTRACTS.md](docs/CONTRACTS.md)
- 当前验证范围与已知问题：[docs/STATUS.md](docs/STATUS.md)
- 缓存释放语义、进度与 `limit_bytes=0`：[docs/CACHE_RELEASE.md](docs/CACHE_RELEASE.md)
- 最新性能修复证据：[docs/RELEASE_NOTES.md](docs/RELEASE_NOTES.md) · [docs/PERFORMANCE.md](docs/PERFORMANCE.md) · [docs/ISSUES_20261107.md](docs/ISSUES_20261107.md)
- 历史发布与实验记录：各版 `docs/RELEASE_*.md`、`docs/validation/`

## 构建与测试

在 Ubuntu 24.04 / Mint 兼容环境准备 Rust 1.89+、`debian/control` 列出的构建依赖，以及独立 Python 虚拟环境：

```sh
cargo test --workspace --all-features --locked

python3 -m venv .venv-build
.venv-build/bin/pip install -r scripts/gui-requirements.lock
GUI_PYTHON="$PWD/.venv-build/bin/python" bash scripts/test-gui.sh

# 构建 .deb（git checkout 需干净）
export DRIVE_NATIVE_SOURCE_COMMIT="$(git rev-parse HEAD)"
chmod +x debian/rules debian/drive-native.postinst debian/drive-native.postrm
dpkg-buildpackage -b -us -uc
```

从发布的源码归档构建时，把配套 `SOURCE_COMMIT` 放在解压目录旁，并用 `export DRIVE_NATIVE_SOURCE_COMMIT="$(cat ../SOURCE_COMMIT)"`。kernel FUSE 测试套件需显式串行执行并设置 `DRIVE_NATIVE_FS_DIAGNOSTICS=1`（测试隔离要求，非产品行为）。

安装生成的 `.deb` 后从应用菜单打开 Drive Native，首次配置连接地址、证书服务器名称与账号，并选择同步目录。凭据由用户输入并保存到 Secret Service；不要将凭据或用户数据库提交到 Git。仓库只保存源码、合成或脱敏测试夹具与维护文档，实验数据库、NAS 文件、官方二进制与旧安装包不进入 Git 历史。

## 未来计划

按验收阶段推进，完整门槛与证据要求以 [DELIVERY_GATES.md](DELIVERY_GATES.md) 为准：

1. **真实 NAS 验收**：在真实账号与目录上完成根 scope 完整初始化、逐目录 ACL、长时间运行与数据安全核验（当前性能与正确性结论均为合成夹具/lab FUSE 口径）。
2. **正确性与故障自愈**：关闭已命名的命名空间竞态，补齐事务点、独立断电模型与损坏恢复的故障注入矩阵。
3. **普通目录全量同步模式**：候选版要求同时支持按需与普通本地目录两种可写模式；本地变更监视、溢出恢复、停机期间修改核对、远端原子发布与两模式统一任务状态仍待实现。
4. **桌面验收**：Mint 22 / Cinnamon / Nemo、HiDPI、键盘导航、休眠恢复与真实桌面端到端流程。
5. **候选发布**：冻结构建 7×24 稳定性、安装/升级/卸载与恢复，通过后才进入 rc1。
6. **本版遗留的性能 follow-up**：cache trim 单次扫描、idle prune 基准、status 指标接入 diagnostics、真实 FUSE ACL 复测、GUI 端到端、`sync_issues` 1M 行复测等；不阻塞本开发版，但应在候选版前关闭。

## 发布与校验

开发包与发布说明见 [GitHub Releases](https://github.com/Austrust/drive-native/releases)。每个发布包含源码归档、`.deb`、`RELEASE_NOTES.md`、`SHA256SUMS`、`SOURCE_COMMIT` 与 `VALIDATION.json`，建议下载后先校验：

```sh
sha256sum -c SHA256SUMS
```

## 许可

项目许可见 [LICENSE](LICENSE)；第三方依赖与许可声明见 [debian/copyright](debian/copyright) 及各依赖目录。
