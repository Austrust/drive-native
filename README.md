# Drive Native

面向 Linux Mint Cinnamon / Nemo 的 Synology Drive 原生协议按需同步客户端。使用 Rust、FUSE、SQLite、PySide6 和 Nemo 扩展；独立连接 Drive Server，不依赖 WebDAV 或官方客户端进程。

**开发测试版本，尚未完成日常使用验收。** 当前 main 基线来自 `0.1.0~dev20261030.5+acl2` 的实际构建归档。官方 Linux 引擎适配是另一个项目，不包含在此仓库。

## 已实现

- 多挂载、选择个人或团队子目录、自定义挂载名称。
- 按需读取、已缓存文件后台更新、固定与释放缓存。
- 本地写入、离线队列、冲突保留和持久恢复。
- Qt 配置界面、Nemo 状态与菜单、systemd 用户服务。
- 分层目录权限过滤、大目录枚举与符号链接过滤。

完整稳定性、部分命名空间竞态及 Mint 桌面验收仍开放；详见 [验证状态](docs/STATUS.md)。

## 构建和测试

在 Ubuntu 24.04 / Linux 文件系统中构建。准备 Rust 1.89+、Debian control 中列出的构建依赖和独立 Python 虚拟环境；历史包构建使用 Rust 1.99.1。

```sh
cargo test --workspace --all-features --locked
python3 -m venv .venv-build
.venv-build/bin/pip install -r scripts/gui-requirements.lock
export GUI_PYTHON="$PWD/.venv-build/bin/python"
chmod +x debian/rules debian/drive-native.postinst debian/drive-native.postrm
dpkg-buildpackage -b -us -uc
```

安装生成的 `.deb` 后从应用菜单打开 Drive Native。首次配置连接地址、证书服务器名称及账号，选择同步目录。凭据由用户输入，通过 Secret Service 保存；不要将凭据或用户数据库提交到 Git。

## 导航

| 目录 | 内容 |
| --- | --- |
| [crates](crates/) | 原生协议、状态核心、FUSE 与服务 |
| [desktop](desktop/) | Qt GUI、Nemo、GIO 与用户服务 |
| [scripts](scripts/) | 构建、测试及显式运行的研究工具 |
| [debian](debian/) | Debian 打包定义 |
| [vendor](vendor/README.md) | 固定的 fuser 补丁及上游许可 |
| [docs/STATUS.md](docs/STATUS.md) | 验证范围、已知问题及后续工作 |
| [docs/PROVENANCE.md](docs/PROVENANCE.md) | 源码归档来源和版本关系 |

仓库只保存源码、合成或脱敏测试夹具和维护文档。实验数据库、账号配置、NAS 文件、官方二进制与旧安装包不进入 Git 历史。项目及第三方许可见 [LICENSE](LICENSE)、[debian/copyright](debian/copyright) 和各依赖目录。
