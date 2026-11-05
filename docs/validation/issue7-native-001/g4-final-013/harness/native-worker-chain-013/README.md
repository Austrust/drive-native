# 原生服务与 worker 链验证入口

- 复现说明 — 说明独立构建、运行、验收范围和证据保留：[RUNBOOK.md](RUNBOOK.md)。
- 验证实现 — 组织本地 TLS 服务、真实 worker 与 Bridge：[main.rs](main.rs)。
- 协议编解码 — 提供独立有界 pstream 对端：[codec.rs](codec.rs)。
- 准备入口 — 从冻结候选生成外部 crate：[prepare.py](prepare.py)。
- 依赖检查 — 核对解析依赖与候选锁文件：[verify-lock.py](verify-lock.py)。
- 构建入口 — 保存各准备与构建阶段的原始输出：[build.py](build.py)。
- 运行入口 — 分别运行用例并保存超时与清理结果：[run.py](run.py)。
- 空间门禁 — 调用协调者冻结的宿主空间检查函数：[space.py](space.py)。
