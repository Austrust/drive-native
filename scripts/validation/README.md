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
