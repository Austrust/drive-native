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
