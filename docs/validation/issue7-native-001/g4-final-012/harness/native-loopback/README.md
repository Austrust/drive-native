# Native loopback harness

- [运行说明](RUNBOOK.md) — 准备独立 crate、核对依赖并运行合成回环协议矩阵。
- [覆盖映射](COVERAGE.md) — 对照设计用例、公开 API、预期分类及产品回归入口。
- [Rust harness](main.rs) — 使用公开 Client API 与独立 pstream 对端检验响应分类。
- [crate 准备脚本](prepare.py) — 从冻结源生成外部 manifest 并复制依赖锁。
- [依赖核对脚本](verify-lock.py) — 对照冻结 Cargo.lock 与完整 cargo metadata。
- [manifest 模板](Cargo.toml.in) — 定义外部 harness 的依赖。
