# 原版 FSearch 完整扫描复放

使用未修改的 FSearch 0.3.1 扫描本项目真实 FUSE 挂载，并核对它保存的索引。入口是 [benchmark-fsearch-full.py](benchmark-fsearch-full.py)，合成挂载来自 [fsearch_fixture.rs](../../crates/drive-fs/examples/fsearch_fixture.rs)。[fsearch-index-audit.c](fsearch-index-audit.c) 只调用同版上游库读取已保存索引，不遍历挂载、不生成替代索引。

这是独立测试工具，不安装到客户端，也不读取用户配置、连接 NAS 或启动原生 worker。完整结果、数据库和原始 TSV 保存在仓库外；公开报告仅引用脱敏聚合与源码哈希。实际结果和发布门禁在 [DELIVERY_GATES.md](../../DELIVERY_GATES.md) 维护，此页不把工具存在当作验收通过。

## 构建未修改的上游

在 Linux 普通测试用户下准备 Meson、Ninja、C 编译器、pkg-config、GTK3/GIO 开发库、PCRE2、ICU、gettext、itstool、xmllint、Xvfb、xauth、会话 D-Bus、Python3 和 FUSE。只在新的测试目录构建，不改用户现有 FSearch 安装或配置。

```sh
# 在本仓库根执行；OUT 必须是仓库外不存在的新目录。
REPO="$PWD"
OUT="/absolute/new-fsearch-validation"
mkdir -m 700 "$OUT"
curl -fL https://codeload.github.com/cboxdoerfer/fsearch/tar.gz/refs/tags/0.3.1 \
  -o "$OUT/fsearch-0.3.1.tar.gz"
printf '%s  %s\n' \
  b16ab75556d841bf858633710d71c92f35d34362614b8584b0a5b71690a72c39 \
  "$OUT/fsearch-0.3.1.tar.gz" | sha256sum -c -
tar -xzf "$OUT/fsearch-0.3.1.tar.gz" -C "$OUT"
meson setup "$OUT/upstream-build" "$OUT/fsearch-0.3.1" --buildtype=release
ninja -C "$OUT/upstream-build" -j2 src/fsearch
cc -O2 -I"$OUT/fsearch-0.3.1/src" -I"$OUT/upstream-build" \
  -I"$OUT/upstream-build/src" "$REPO/scripts/validation/fsearch-index-audit.c" \
  "$OUT/upstream-build/src/libfsearch.a" \
  $(pkg-config --cflags --libs gio-unix-2.0 gtk+-3.0 libpcre2-8 icu-uc) \
  -lm -o "$OUT/fsearch-index-audit"
"$OUT/upstream-build/src/fsearch" --version
```

上游源码保持原样；审计器是单独的可执行文件。固定归档 SHA256 后，还应记录实际编译器、依赖版本、源码清单和两个二进制的 SHA256。这个 C 审计器与 0.3.1 内部库接口配对，不是所有 FSearch 版本的通用格式读取器。

## 构建配对夹具

准备两个独立源码目录及 target。基线使用待比较的旧提交；只复制新版的同一份 `fsearch_fixture.rs`，旧产品代码不变。记录两个源提交、完整文件清单和被复制夹具的相同哈希。

```sh
BASELINE="/absolute/baseline-checkout"
FIXED="/absolute/fixed-checkout"
mkdir -p "$BASELINE/crates/drive-fs/examples"
cp "$FIXED/crates/drive-fs/examples/fsearch_fixture.rs" \
  "$BASELINE/crates/drive-fs/examples/fsearch_fixture.rs"
(cd "$BASELINE" && CARGO_TARGET_DIR="$OUT/baseline-target" \
  cargo build --release --locked -j2 -p drive-fs --example fsearch_fixture)
(cd "$FIXED" && CARGO_TARGET_DIR="$OUT/fixed-target" \
  cargo rustc --release --locked -j2 -p drive-fs --example fsearch_fixture \
  -- --cfg drive_fs_diagnostics)
```

诊断 cfg 只启用夹具读取聚合的接口。普通计时运行仍将 `DRIVE_NATIVE_FS_DIAGNOSTICS` 设为 0；不要将采样或编译与正式比较混在一起。

## 完整扫描与判定

确认没有竞争编译、NAS压测或其他扫描后执行：

```sh
python3 scripts/validation/benchmark-fsearch-full.py \
  --run "$OUT/scans-001" --exclusive-window-confirmed \
  --baseline "$OUT/baseline-target/release/examples/fsearch_fixture" \
  --fixed "$OUT/fixed-target/release/examples/fsearch_fixture" \
  --fsearch "$OUT/upstream-build/src/fsearch" \
  --audit "$OUT/fsearch-index-audit" --rounds 3
```

默认每个独立夹具含 100,064 个文件和 2,066 个目录：浅层多目录、20,000文件宽目录、64层深目录。每轮 baseline/fixed/local 顺序交替，分别做 first/repeat；本地目录是独立参照，不是提供给 FUSE 测试的镜像。`--groups`、`--files`、`--wide` 可缩小工具冒烟范围，改变规模必须重新记录，不能混算。

first 是新挂载、新配置下的首次扫描，**没有清空宿主页面缓存**；repeat 使用同一挂载和配置。工具用私有 XDG、会话 D-Bus 和隐藏 Xvfb 启动原版 `--update-database`。每次都检查 CLI 返回、保存失败警告、数据库保存签名更新，再逐项核对真实索引的相对路径、类型、大小和 mtime。目录 size 按 FSearch 的索引格式为 0。源码实际保存路径写临时文件并替换原文件，因此即使相同内容的数据库哈希不变，保存签名仍应变化。CLI 返回 0 本身不足以通过。

18次扫描全部通过才可汇总三轮 first/repeat。每次必须无缺项、额外项、重复项、属性差异及正文 hydration；失败保留现场和 `summary.json`，不得跳过后继续报告全通过。正常退出夹具与挂载也要记录。宿主环境和负载窗口由执行者另行登记。

`--diagnostics` 只用于另建 run，记录固定阶段的进程内聚合；它与正式计时分开。`client_children_maxrss_kib_so_far` 是进程累计观察到的子进程最大RSS，不能当作本次扫描独立峰值；fixture RSS是扫描结束样本，不是全程峰值。

这套基准不自动覆盖取消、并行挂载、ACL撤销、目录预览、远端事件或官方客户端对照。相关功能需另行实测并引用既有内核回归；不能将本地目录参考或WSL跨系统路径冒充同机官方按需同步客户端。

单独做本地编辑功能验证时，夹具可显式使用 `fuse-writable` 模式，再通过挂载执行 rename/delete/create；普通基准仍固定使用只读 `fuse`。两种模式均只有拒绝网络的 hydration stub，不连接 NAS。可写模式需要另外构建并记录源码/二进制哈希；不能将原只读夹具的 EROFS 当作产品故障或跳过编辑检查。
