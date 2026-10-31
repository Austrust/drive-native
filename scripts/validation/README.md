# 独立验证工具

这些脚本不导入客户端代码。当前提供静止、隔离夹具的内容/命名空间/指定草稿和操作记录比较；不是已经完成的故障注入体系。

```sh
python3 scripts/validation/test_verify_fixture.py
python3 scripts/validation/verify_fixture.py --root /path/to/quiescent-fixture --manifest expected.json --state /path/to/captured-state
```

预期 manifest 必须由测试驱动在动作前生成并保存，禁止从待判定的客户端状态反推期望。运行时停止隔离夹具的变更，或使用一致的冻结副本；SQLite 只读事务不能自动提供正文文件的一致快照。不要对用户正在编辑的挂载给出故障通过结论。

schema 1 字段：`files` 为相对路径→SHA-256，`directories` 为精确相对目录列表；可选 `drafts` 指定 node/token/base/dirty/sha256，`operations` 指定 id/node/kind/base/state，`exact_operation_ids` 检查多余或丢失操作。状态库列名来自 schema 18；类型与值按 manifest 原样比较。输出 passed/errors，失败返回非零。示例夹具见单元测试。

验证范围：缺失/额外路径、内容变化、缺失状态、指定草稿/操作不符、SQLite quick_check、符号链接边界。不验证服务端身份、远端提交、草稿 generation/checkpoint 或完整服务器收敛。断电、EIO、ENOSPC 和事务点重复注入尚未运行。

2026-10-31：WSL 11/11 单元测试通过；Windows 10 通过，1 个符号链接测试因创建权限跳过。这里的数量是判定器测试，不计入产品 Rust/GUI 回归或 P3 故障轮次。

## 固定规模本地性能夹具

`performance_fixture` 示例通过已有 Store 接口建立精确 210,000 节点、2,000 子项目录与 1 MiB 完整缓存。所有名字及内容均为确定性合成数据。每次使用全新目录；生产状态不得传给此流程。

```sh
cargo build --release --locked -p drive-core --features native-research --example performance_fixture
python3 scripts/validation/prepare_benchmark.py \
  --builder target/release/examples/performance_fixture \
  --fsd /usr/lib/drive-native/drive-native-fsd \
  --runtime-commit <installed-source-commit> \
  --output /absolute/existing-parent/new-fixture --start
# 等待输出的 unit 启动且 GetOverview.mounted=true 后，显式选择测量：
python3 scripts/validation/benchmark_local.py \
  --config /absolute/existing-parent/new-fixture/config.json \
  --output /absolute/existing-parent/new-fixture/run-directory --phase directory
```

需要发行版 Python 的 GI/Gio（python3-gi）、用户 D-Bus、FUSE 和 systemd 用户会话。测试服务不提供凭据，网络保持 locked；停止时对输出的唯一 unit 执行 `systemctl --user stop <unit>`。保留状态和结果，工具不会删除它们。

分项 `directory/status/warm_read/disk_read/held_read` 各预热2次、测30次；`mixed_open/mixed_held` 各48路、5轮、每类预热2次，15秒超时。held_read 的句柄在计时/屏障前取得；其余读取包含打开和关闭。进程启动另列。warm/disk 分组用 mincore 确认 backing blob 页驻留，后者先对单个合成 blob 使用 DONTNEED；不清理系统全局缓存，也不声称控制虚拟磁盘或硬件缓存。

运行会显著占用 CPU，必须登记对其他实验的干扰；单文件成功/超时、setup失败全部保留。已完成的固定夹具基线见 [报告](../../docs/validation/p0-fixed-001/REPORT.md)。初次实测使用项目本地启动适配脚本；通用 `prepare_benchmark.py` 随后在全新夹具中实际通过启动、2000项目录及缓存哈希检查，见 [launcher-check.json](../../docs/validation/p0-fixed-001/launcher-check.json)。

停止服务后还必须检查 `systemctl --user show <unit> -p Result -p ExecMainStatus`，不能只看 stop 命令退出码。此次高并发固定夹具在停止时被 systemd 超时强杀，原始状态保留为失败证据；通用启动工具另建的短测服务正常停止。

## 自有连接中断器

`byte_relay.py` 仅监听回环地址，将字节原样转发给显式指定的上游。它不终止 TLS、不保存凭据或正文；客户端必须继续验证真实服务器的证书名称与 CA。只用于已隔离的只读测试实例，不改变生产任务配置或系统全局网络。

```sh
mkdir -m 700 /private/new-relay
python3 scripts/validation/byte_relay.py --state /private/new-relay --upstream-host <NAS地址>
python3 scripts/validation/test_byte_relay.py
```

就绪后在状态目录写入 `relay-ready.json`。同用户通过权限0600的 `relay.sock` 发送 `status\n` 查询计数，或发送 `drop\n` 中断本工具持有的连接；监听器继续运行。仅 `aborted_streams` 非零不够证明恢复成功，还须检查连接代次、客户端恢复状态和实际本地读。停止客户端后再停止转发器，检查退出结果。残留 socket 不自动覆盖，避免误接管其他测试。

本机回环测试已在WSL通过1项，覆盖透明回传和实际连接关闭。真实NAS基线使用其固定参数前身，实际执行脚本、哈希及全部样本见[恢复报告](../../docs/validation/p0-recovery-001/REPORT.md)。它只模拟显式断开，不能替代丢包、限速、长时间不可达或服务器重启实验。

## 小型持久性边界夹具

`cargo run --release --locked -p drive-core --example durability_fixture -- /absolute/new-fixture` 仅生成三节点合成SQLite状态及确定性内容，拒绝覆盖已有目录，不启动网络或挂载。与210k性能夹具分开使用。

使用`.6`服务启动本地可写挂载、保持网络锁定，检查write/fsync、普通保存和正常卸载/重启的实际脚本、预先登记的期望及独立校验结果见[边界报告](../../docs/validation/p0-durability-001/REPORT.md)。该流程1轮通过；6项既有mounted内核测试也显式执行通过。它不替代故障注入、真正磁盘满/EIO或独立断电测试。

## 独立D-Bus诊断探针

[watch_probe.py](watch_probe.py)读取显式测试目录中的catalog.json（数组，每项含instance、bus、name、mount），监听服务归属与StatusChanged，200ms合并后每任务至多一个由信号触发的本地GetOverview查询；不会在信号回调遍历FUSE或访问NAS。保留既有10秒诊断检查，不把它当产品同步机制。日志8MiB轮换，保留三份历史；只记录选定状态字段，不记录正文或认证数据。该工具不随普通客户端自动运行。

[test_watch_probe_signals.py](test_watch_probe_signals.py)在当前用户会话总线上创建独立模拟服务，验证离线/恢复信号、1000次突发合并及字段过滤，无NAS或真实挂载。Linux需python3-gi、gdbus、stdbuf及会话D-Bus。必须指定不存在的输出目录，保留既有证据：

```sh
python3 scripts/validation/test_watch_probe_signals.py --probe scripts/validation/watch_probe.py --output-dir /tmp/drive-probe-signals-test-new
```

实际监测通过`python3 scripts/validation/watch_probe.py --state-dir <隔离测试目录>`显式启动。合并查询可能看不到短于通知/查询延迟的状态，不等于持久错误事件历史；worker退出原因应在清理前记录，不能从探针未报错反推零重连。
