# RPM 打包与 Fedora/RHEL 整合

本文记录 `drive-native` 的 RPM 打包方式、目标发行版与实测基线、依赖映射、构建/安装流程及未验证范围。当前状态：**开发版打包实验（0.1.0~dev20261118.1）**，RPM 是新增交付格式，未经过 Fedora/RHEL 发行流程，**未签名、未在真实 Fedora/RHEL 主机安装验证**。唯一验收状态仍是 [DELIVERY_GATES](../DELIVERY_GATES.md)；.deb 与桌面验收记录见 [UBUNTU_INTEGRATION.md](UBUNTU_INTEGRATION.md)。

## 1. 目标发行版与实测基线

RPM 由 Ubuntu 24.04 构建机上的 `rpmbuild 4.18.2` 组装，但载荷来自 Ubuntu 构建的同一 staging 树，因此运行时下限由该树里的 ELF 决定，而不是由 rpmbuild 版本决定。实测（workstation202611，Ubuntu 24.04.5，glibc 2.39）：

| 载荷 | 实测 glibc 符号下限 | 依据 |
|---|---:|---|
| `drive-native-fsd` | **2.39** | `pidfd_getpid` / `pidfd_spawnp` |
| `drive-native-worker`、`drive-native-setup` | 2.38 | `__isoc23_sscanf` |
| `libdrive-native-probe.so`（GIO 模块） | 2.33 | 非版本化符号扫描 |
| GUI onedir（178 个 ELF；整棵 staging 182 个） | 2.38 | bundled libpython3.12 等 |
| **整包实际要求** | **glibc ≥ 2.39** | 取各载荷上限 |

由此确定：

- **支持：Fedora 40+ 与 RHEL 10+（x86_64）**。Fedora 40 / RHEL 10 的 glibc 均为 2.39，是下限；更新的 Fedora 向后兼容。
- **不支持：RHEL 8（glibc 2.28）、RHEL 9（glibc 2.34）**，也不承诺任何更老的发行版。EL8/EL9 上表现为启动时 GLIBC 符号版本错误。
- 同一构建机的 `.deb` 面向 Ubuntu 24.04 / Mint 22 兼容环境，不受 RPM 目标限制影响；不要据此宣称"通用 Linux"支持。
- 包为 `x86_64` 单架构；未做 aarch64 交叉打包。

## 2. 与 .deb 的关系：共用一条 staging 链

RPM **不重新编译**，也**不复用 debian/ 的控制文件**。`.deb` 与 `.rpm` 共用同一套构建与布局脚本：

```
scripts/build-linux.sh  <build>          cargo release bins + gio 模块 + PyInstaller GUI
        │
scripts/stage-linux.sh  <build> <stage>  安装树（/usr/lib/drive-native、/usr/bin、...）
        │
        ├── debian/rules (dh) ──────────► .deb
        └── scripts/build-rpm.sh <out> ─► 重定位 gio 模块 ─► rpmbuild -bb ─► .rpm
```

好处是安装布局、文档内容（`stage-docs.py` 生成 `SOURCE_COMMIT`/`BUILD_VERSION`）、GIO 模块和 GUI bundle 只有一处定义，不会出现 deb 与 rpm 布局漂移。`packaging/rpm/drive-native.spec` 是"预 staging 模式"：

- 无 `Source`、无 `BuildRequires`、`%prep`/`%build` 为空，`rpmbuild` 只执行 `%install` 拷贝与打包；
- staging 树由 `--define '_stagedir <绝对路径>'` 传入，默认 `%{_topdir}/stage`；
- `%check` 校验 `BUILD_VERSION` 与 spec `Version` 一致、`SOURCE_COMMIT` 为 40 位十六进制，因此 **必须先 bump `debian/changelog`**（`stage-docs.py` 从 changelog 取版本）。

RPM 打包路径**不重复运行测试**：`debian/rules` 的 `dh_auto_test`（全量 `cargo test` + GUI 回归）仍是发布测试门禁；`scripts/build-rpm.sh` 只做构建、打包与内容自检。

## 3. 构建

### 3.1 前置条件

- Ubuntu 24.04 构建机（与 .deb 相同）：Rust 1.89+、`debian/control` 的构建依赖、`pkg-config`、`libglib2.0-dev`；
- `rpm` / `rpmbuild`（Ubuntu 24.04 自带 4.18.2）、`rpm2cpio`、`cpio`、`binutils`（`strip`/`objdump`）；
- GUI 构建虚拟环境：`bash scripts/prepare-gui-build.sh "$PWD/.venv-build"`（PySide6 + PyInstaller，安装期不需要）。

### 3.2 命令

```sh
cd /path/to/drive-native            # git 工作树
bash scripts/prepare-gui-build.sh "$PWD/.venv-build"   # 首次

GUI_PYTHON="$PWD/.venv-build/bin/python" \
DRIVE_NATIVE_SOURCE_COMMIT="$(git rev-parse HEAD)" \
bash scripts/build-rpm.sh /absolute/output/dir
```

`build-rpm.sh` 依次：

1. 校验 `GUI_PYTHON`、40 位十六进制 `DRIVE_NATIVE_SOURCE_COMMIT`，以及 spec `Version` 与 `debian/changelog` 版本一致；
2. `bash scripts/build-linux.sh <out>/build`；
3. `bash scripts/stage-linux.sh <out>/build <out>/stage`；
4. 把 gio 模块从 `/usr/lib/x86_64-linux-gnu/gio/modules` 重定位到 `/usr/lib64/gio/modules`（spec `%install` 对裸 staging 树也会兜底）；
5. `rpmbuild -bb`，`_topdir=<out>/rpmbuild`、`_stagedir=<out>/stage`，完整日志在 `<out>/rpm-build.log`；
6. 复制 `.rpm` 到输出目录并打印 `RPM_NEVRA`、`RPM_SHA256`、`RPM_FILE`、`SOURCE_COMMIT`、`BUILD_VERSION`、`GITHUB_ASSET_NAME_HINT`（`~`→`.`，仅供 Release 上传命名参考）。

产物文件名固定为 `drive-native-<Version>-<Release>.x86_64.rpm`：

```
drive-native-0.1.0~dev20261118.1-1.x86_64.rpm
```

RPM 目标 `Release: 1` 不带 `%{?dist}`：包从 Ubuntu 构建机直接产出并作为独立资产分发，不进入 Fedora/RHEL 构建系统，跨主机构建时 EVR 保持一致。NEVRA 为 `drive-native-0.1.0~dev20261118.1-1.x86_64`。

spec 固定 `%global _binary_payload w19.xzdio`，不继承构建机的 rpm 宏：Ubuntu 24.04 的 rpm 默认 gzip/9，同一 staging 树实测 **73,526,191 B**；改为 xz/19 后 **49,213,525 B（−33%）**，`rpm -K` digests OK、`PAYLOADCOMPRESSOR=xz FLAGS=19`、`rpm2cpio | cpio -t` 正常。固定压缩宏也保证包体不随构建机 rpm 配置漂移。

### 3.3 构建脚本的自动检查

`build-rpm.sh` 在 `rpmbuild` 成功后检查并在不满足时失败：

- NEVRA 与 `drive-native-<Version>-1.x86_64` 一致；
- 打包文件不含 Ubuntu multiarch 路径（`x86_64-linux-gnu`）；
- 安装树身份：`/usr/share/doc/drive-native/SOURCE_COMMIT` 等于传入的 source commit，`BUILD_VERSION` 等于 spec `Version`；
- `Requires` 不含 GUI bundle 泄漏（`libQt*`/`PySide*`/`shiboken`/`libpython*`/`python3dist`）；
- 必需文件存在：`/usr/bin/drive-native`、`/usr/lib/drive-native/gui/drive-native-gui`、`/usr/lib64/gio/modules/libdrive-native-probe.so`、`/usr/lib/systemd/user/drive-native.service` 等；
- 显式 `Requires` 均出现在包的依赖中。

## 4. 安装布局

与 [stage-linux.sh](../scripts/stage-linux.sh) 的 deb 布局一致，唯一差异是 gio 模块目录：

| 路径 | 内容 |
|---|---|
| `/usr/lib/drive-native/{drive-native-fsd,drive-native-worker,drive-native-setup}` | 原生二进制（已 strip） |
| `/usr/lib/drive-native/gui/` | PyInstaller onedir GUI bundle（**不 strip**） |
| `/usr/bin/drive-native` | `exec /usr/lib/drive-native/gui/drive-native-gui "$@"` 包装脚本 |
| `/usr/lib64/gio/modules/libdrive-native-probe.so` | GIO volume/metadata 探针（已 strip；deb 位于 `/usr/lib/x86_64-linux-gnu/gio/modules`） |
| `/usr/share/nemo-python/extensions/drive_native_nemo.py` | Nemo 状态图标/菜单扩展 |
| `/usr/share/nautilus-python/extensions/drive_native_nautilus.py` | Nautilus 4 扩展 |
| `/usr/share/drive-native/filemanager/drive_native_provider.py` | 两个文件管理器共用的异步 provider |
| `/usr/share/icons/hicolor/scalable/{emblems,apps}/` | 6 个状态 emblem + `drive-native.svg` |
| `/usr/share/applications/drive-native.desktop` | 桌面入口 |
| `/usr/lib/systemd/user/{drive-native,drive-native-unlock,drive-native@,drive-native-unlock@}.service` | 用户单元（**只安装，不启用**） |
| `/usr/share/doc/drive-native/` | `LICENSE`、`sproto-LICENSE`、构建提交/版本、文档、`gui-dependencies.txt`、`gui-requirements.lock` |

`rpm -qlp` 的条目数（实测 692）大于 staging 文件数（322）：其中 373 条是 `%files` 的实际载荷（322 个文件 + GUI 目录递归带来的子目录条目），另 319 条是 rpmbuild 按 RPM 常规行为为包内 ELF 生成的 `/usr/lib/.build-id` 链接。build-id 链接只新增符号链接，不修改 GUI bundle 文件本身。

### 4.1 GIO 模块重定位与缓存

`stage-linux.sh` 在 Ubuntu 构建机按 `pkg-config --variable=giomoduledir gio-2.0` 安装模块到 multiarch 路径。RPM 目标 Fedora/RHEL 的 x86_64 GIO 模块目录是 `/usr/lib64/gio/modules`，因此 spec 与 `build-rpm.sh` 都做重定位，并在 `%files` 中以目标路径声明。

包**不携带** `giomodule.cache`（与 deb 同理，共享目录的缓存由维护脚本生成）：

- `%post`：`command -v gio-querymodules` 守卫后执行 `gio-querymodules /usr/lib64/gio/modules`；
- `%postun`：仅 `$1 -eq 0`（最终卸载，非升级）时刷新，与 deb `postrm` 的 `remove|purge|disappear` guard 对应。

### 4.2 systemd 用户单元

四个用户单元只安装到 `/usr/lib/systemd/user/`，**安装/升级/卸载均不启用、不启动**，与 `debian/rules` 里 `override_dh_installsystemduser` 的语义一致：GUI 在账号配置完成后才由用户显式启用单元；升级还需要 FSD/worker/GUI 成对切换。卸载不会删除用户配置、缓存或待写内容。

### 4.3 strip 策略

`.deb` 用 `dh_strip -Xusr/lib/drive-native/gui` 保留 GUI bundle 原样；RPM 采用等效做法：

- `%global __os_install_post %{nil}` 关闭 rpmbuild 自动 post 脚本（strip、brp-*），`%global debug_package %{nil}` 不生成 `-debuginfo`；
- `%install` 只对 3 个原生二进制和 gio 模块执行 `strip --strip-unneeded`；
- PyInstaller bundle（含私有 Qt/Python 库）**完全不改**，避免破坏 PyInstaller 运行时与 deb 的产物体积/行为对齐。

## 5. 依赖策略与 Fedora/RHEL 映射

依赖分三层：

1. **自动依赖**：`rpmbuild` 默认的 ELF 依赖生成器对原生二进制与 gio 模块生成 soname capability，例如 `libc.so.6(GLIBC_2.39)(64bit)`、`libgcc_s.so.1(GCC_3.0)(64bit)`、`libgio-2.0.so.0()(64bit)`。这些由目标系统的 glibc/glib2 等包提供，跨 Fedora/RHEL 稳定。
2. **显式 `Requires`**：覆盖 GUI bundle 与脚本/数据依赖（bundle 被排除出自动扫描）。
3. **`Recommends`**：可选桌面集成，包不存在时 dnf 只忽略弱依赖。

### 5.1 排除 GUI bundle 的自动扫描

```spec
%global __requires_exclude_from ^/usr/lib/drive-native/gui/.*$
%global __provides_exclude_from ^/usr/lib/drive-native/gui/.*$
```

PyInstaller bundle 内的 Qt/Python/shiboken 私有库不进入 `Requires`/`Provides`：`Requires: libQt6Core.so.6()(64bit)` 在 Fedora/RHEL 上要么无解，要么会把另一套 Qt 拉进事务；`Provides: libQt6Core.so.6()` 则会与系统 Qt 冲突。GUI 运行时真正需要的**系统**库在 spec 里显式声明。`build-rpm.sh` 会再次断言 `Requires`/`Provides` 中没有 GUI bundle 泄漏。

### 5.2 实测 ELF NEEDED → 目标包映射

判据是 staging 树里**没有被 GUI bundle 自带**的 SONAME。实测（322 文件 / 191.73 MiB；GUI onedir 内 178 个 ELF）：

- 原生二进制 NEEDED：`libgcc_s.so.1`、`libm.so.6`、`libc.so.6`、`ld-linux-x86-64.so.2`；
- GIO 探针 NEEDED：`libgio-2.0.so.0`、`libgobject-2.0.so.0`、`libglib-2.0.so.0`、`libc.so.6`；
- GUI bundle 自带 288 个文件，覆盖其大部分库栈（Qt6/PySide6、libX11、libxkbcommon、libfontconfig、libdbus-1、xcb-util-*、libz 等）。**整棵 staging 中不在 bundle 内的 NEEDED 只有**：`libc`/`libm`/`libdl`/`libpthread`/`libresolv`（glibc）、`libgcc_s`、`libgio`/`libgobject`/`libglib`、`libEGL.so.1`、`libGL.so.1`、`libdrm.so.2`、`libwayland-client/cursor/egl`、`libxcb.so.1`。

必需依赖（实测非 bundle）：

| 实测 NEEDED | 来源 | Fedora/RHEL 包 | 方式 |
|---|---|---|---|
| libc/libm/libdl/libpthread/libresolv、ld-linux | 原生二进制、gio 模块、GUI launcher | `glibc` | 显式 + 自动 soname capability |
| `libgcc_s.so.1` | 原生二进制、GUI bundle 内的 Rust 扩展 | `libgcc` | 显式 + 自动 soname capability |
| `libgio-2.0.so.0`、`libgobject-2.0.so.0`、`libglib-2.0.so.0` | GIO 模块 | `glib2` | 显式 + 自动 soname capability |
| `libEGL.so.1` | GUI bundle（Qt EGL/QSG） | `mesa-libEGL` | 显式 |
| `libGL.so.1` | GUI bundle（Qt OpenGL） | `mesa-libGL` | 显式 |
| `libdrm.so.2` | GUI bundle（EGL/GBM 路径） | `libdrm` | 显式 |
| `libwayland-client/cursor/egl.so.*` | GUI bundle（Qt Wayland 平台插件） | `libwayland-client`、`libwayland-cursor`、`libwayland-egl` | 显式 |
| `libxcb.so.1` | GUI bundle（Qt xcb 平台插件） | `libxcb` | 显式 |

桌面运行时契约（deb 显式清单中、实测已被 bundle 自带的库；保留声明，避免 PyInstaller 停止打包时静默缺库）：

| 库 | 实测 | Fedora/RHEL 包 |
|---|---|---|
| libX11/libXext/libXrender/libXi/libSM/libICE | bundle 内 | 同名包 |
| libxkbcommon-x11（及 libxkbcommon） | bundle 内 | `libxkbcommon-x11` |
| xcb-util-cursor/wm/image/keysyms/renderutil | bundle 内 | 同名包 |
| libfontconfig | bundle 内 | `fontconfig` |
| libdbus-1.so.3 | bundle 内 | `dbus-libs` |

非 ELF 的功能依赖保持与 deb 相同：`fuse3`（`/dev/fuse` 与 `fusermount3`）、`systemd`（用户单元）、`ca-certificates`（TLS 根证书）、`hicolor-icon-theme`（图标主题目录）、`python3-gobject`（Nemo/Nautilus Python 扩展）、`gvfs`/`gvfs-fuse`（GIO 卷/挂载后端）。

弱依赖映射：

| 可选能力 | Fedora/RHEL 包 | 说明 |
|---|---|---|
| 凭据钥匙串 | `gnome-keyring` | Secret Service 后端；缺失时 GUI 走降级路径 |
| 中日韩字体 | `google-noto-sans-cjk-fonts` | 替代 deb 的 `fonts-noto-cjk` |
| Nemo 扩展运行时 | `nemo-python` | Fedora 提供；RHEL 10 需 EPEL/第三方源，缺失不影响主程序 |
| Nautilus 扩展运行时 | `nautilus-python` | EPEL 10 提供 4.0.x |

### 5.3 不照搬 deb 包名

| deb 依赖 | RPM 处理 |
|---|---|
| `python3-gi` | `Requires: python3-gobject` |
| `libglib2.0-bin`（`gio-querymodules`） | 合并到 `Requires: glib2` |
| `libegl1` / `libgl1` | `mesa-libEGL` / `mesa-libGL` |
| `libfontconfig1` | `fontconfig` |
| `libdbus-1-3` | `dbus-libs` |
| `libxkbcommon-x11-0` | `libxkbcommon-x11` |
| `libxcb-cursor0` | `xcb-util-cursor` |
| `libxcb-icccm4` | `xcb-util-wm` |
| `libxcb-image0` | `xcb-util-image` |
| `libxcb-keysyms1` | `xcb-util-keysyms` |
| `libxcb-render-util0` | `xcb-util-renderutil` |
| `libxcb-xinerama0` / `libxcb-shape0` | `libxcb`（扩展在同一包内） |
| `dbus-user-session` | 不映射；Fedora/RHEL 默认提供会话总线 |
| `nemo-python \| python3-nautilus` | 改为 `Recommends: nemo-python`、`nautilus-python` |
| `fonts-noto-cjk` | `Recommends: google-noto-sans-cjk-fonts` |
| `${shlibs:Depends}` | 由 RPM 自动 soname capability 生成（GUI bundle 除外） |

## 6. 安装、升级与卸载

```sh
# 校验后再安装（包未签名）
sha256sum -c SHA256SUMS          # 发布资产清单
sudo dnf install ./drive-native-0.1.0~dev20261118.1-1.x86_64.rpm
# 或
sudo rpm -Uvh ./drive-native-0.1.0~dev20261118.1-1.x86_64.rpm

rpm -q drive-native              # NEVRA
rpm -ql drive-native | head      # 文件清单
```

安装后从应用菜单打开 Drive Native，首次配置连接地址、证书服务器名称与账号，并选择同步目录。凭据保存到 Secret Service（`gnome-keyring` 等后端）。

升级：保存并关闭挂载内正在编辑的文件，暂停后安装新包；FSD/worker/GUI 必须成对切换。卸载：

```sh
sudo dnf remove drive-native
```

卸载只刷新 GIO 模块缓存，不删除用户配置、用户数据库、缓存或待写内容；用户 systemd 单元从未被包安装启用，需要时由用户自行 `systemctl --user disable`。

## 7. 限制与未验证项

- **未签名**：没有 GPG 签名，也没有经过 Fedora/RHEL 签名流程；下载后请用发布资产 `SHA256SUMS` 校验。
- **未在真实 Fedora/RHEL 主机安装/升级/卸载**：本版只做了构建机上的 `rpmspec -P` 与 `rpmbuild` 打包（结果见 §8），未做 dnf 事务、`rpm -V`、SELinux、真实桌面与真实 NAS 验收。
- **不是发行版包**：未进入 Fedora Koji/Copr 或 RHEL 构建系统，不提供 debuginfo、源 RPM、updateinfo 或自动更新通道。
- **GUI 系统库依赖是实测 ELF 分析 + 发行版名称映射，而非逐发行版安装验证**：按 staging 树区分「bundle 自带」与「必须由系统提供」（§5.2），各发行版 Qt/GIO 版本差异仍可能改变实际加载路径；若目标机 GUI 启动报缺库，请以 `ldd /usr/lib/drive-native/gui/drive-native-gui` 与 bundle 内 `ldd` 结果为准并反馈。
- **RHEL 8/9 不支持**（glibc 2.39 下限）；aarch64 未打包；未测试 Wayland-only 会话（Qt xcb 依赖仍是硬依赖）。
- **RPM 构建不重复跑测试**：测试门禁仍由 `debian/rules` 的 `dh_auto_test` 与发布验证承担（见 [RELEASE_20261118_1.md](RELEASE_20261118_1.md)）。
- `nemo-python` / `nautilus-python` 只在 Fedora（及 EPEL 10 的 nautilus-python）提供；其他发行版需自行处理，不影响主程序与 CLI/FUSE 功能。

## 8. 远端构建验证记录

验证环境：`workstation202611`（Ubuntu 24.04.5，32 核 / 125 GiB，glibc 2.39，rpm/rpmbuild 4.18.2，Rust 1.99.1）。

| 步骤 | 命令 | 结果 |
|---|---|---|
| 共享 staging | `scripts/build-linux.sh` + `scripts/stage-linux.sh` | 通过；322 文件 / 191.73 MiB（冷 target 约 44 s） |
| spec 解析 | `rpmspec -P packaging/rpm/drive-native.spec`（rpm 4.18.2） | 通过：RC=0，stderr 0 行，NEVRA `drive-native-0.1.0~dev20261118.1-1.x86_64` |
| rpmbuild（gzip/9 对照） | `rpmbuild -bb`（`_topdir`/`_stagedir`） | 通过：`%install`/`%check` 全绿、无 unpackaged 文件；`drive-native-0.1.0~dev20261118.1-1.x86_64.rpm` **73,526,191 B** |
| rpmbuild（发布配置 `w19.xzdio`） | 同上，同一 staging 树 | 通过：**49,213,525 B（−33%）**；`PAYLOADCOMPRESSOR=xz FLAGS=19`；`rpm -K` digests OK；`rpm2cpio | cpio -t` 692 条 |
| 依赖实测 | `rpm -qpR` | 自动 ELF 依赖含 `libc.so.6(GLIBC_2.39)(64bit)`（fsd 的 pidfd 符号）；显式 Requires 全部展开；无 bundled Qt/Python 依赖；`libdrive-native-probe.so()(64bit)` 为 GIO 模块自身 provide |
| Fedora/RHEL 实机安装 | — | **未验证** |

> 本表只记录构建机上的打包结果；不代表安装/桌面/NAS 验收。发布构建会从最终 release commit 重新执行 `scripts/build-rpm.sh`，实际 `RPM_SHA256`、NEVRA 与体积以发布资产 `SHA256SUMS` / `VALIDATION.json` 为准。

### 8.1 冒烟中发现并修复的封装脚本问题

首次真实全流程冒烟时 rpmbuild 本身全绿，但 `scripts/build-rpm.sh` 的封装层自检有两处误报，已修复：

1. `rpm -qp ... | grep -q` 在 `set -o pipefail` 下会因 grep 提前退出、rpm 收到 SIGPIPE 而被误判为失败（文件清单/依赖检查可能报 missing）。现在先把查询输出捕获到变量，再用 here-string 过滤，不再有 SIGPIPE。
2. multiarch 泄漏检查原先匹配任意 `x86_64-linux-gnu` 字符串，被 PyInstaller bundle 里的合法文件名（如 `_cffi_backend.cpython-312-x86_64-linux-gnu.so`）误伤；现在只匹配包内路径前缀 `^/usr/lib/x86_64-linux-gnu/`。

这两处只影响构建脚本的校验输出，不影响 spec 或包内容；最终发布构建使用修复后的脚本。

## 9. 维护流程

1. 修改版本：`packaging/rpm/drive-native.spec` 的 `Version`、`debian/changelog` 顶部条目（`stage-docs.py` 用 changelog 版本生成 `BUILD_VERSION`）；
2. 同步更新 spec `%changelog`；
3. 按 §3.2 在构建机执行 `scripts/build-rpm.sh`；
4. 用 `sha256sum` 与 `rpm -qp --requires/--provides` 复核后随发布资产上传；
5. 若调整 staging 布局（`stage-linux.sh`）或 GUI 依赖（`scripts/gui-requirements.lock`），同步更新 spec `%files` 与 §5 映射表。
