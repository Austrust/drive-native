# Issue #7 原生反馈与权限修复证据

- [增量报告](REPORT.md)：实现里程碑、独立失败、修复和最终验证（作者侧）。
- [作者编译记录](CHECKS.json)：每次 cargo check 的退出码、原始日志和字节摘要。
- [早期独立验证原件](g4-through-010/REPORT.md)：G4-006–010 的逐组计数、失败、时钟诊断及环境阻塞。
- [G4-012 独立验证](g4-final-012/REPORT.md)：`9c9aaba` 的最终矩阵、真实worker链、组件数据保留和此前反例原件（历史，期望冻结）。
- [G4-013 独立验证（当前）](g4-final-013/REPORT.md)：T4 对 `fefb8ee` 的独立矩阵/补测、T1/T2 审阅与 012/013 loopback delta；包内 `MANIFEST.json` 与 `SHA256SUMS` 可逐文件复核。
- [G4-013 复制核验](g4-final-013-COPY-VERIFY.md)：本仓库入库时的逐字节复制/哈希/文件数核验（作者侧）。
- [官方客户端静态证据](static/REPORT.md)：直接响应的错误码与分类边界。
- [官方服务端静态证据](server-static/REPORT.md)：提交前 ACL 检查、路径过滤和父目录语义。
- [能力与身份补充证据](server-capability/REPORT.md)：原生能力查询为何尚不能绑定精确节点。
- [空间只读来源复核](l2-static/REPORT.md)：共享配置与根节点能力的组合边界。
- [空间只读来源复核 II](l2-source-002/REPORT.md)：NodeCapability 共享权限注入、评估器边界、profile 只读过滤与给 T3 的实验设计（证据包在仓库外，路径/SHA 见目录 README）。
- [精确写拒绝自动学习增量](write-learning/REPORT.md)：可证窄子集（upload/0x9003/已存在节点）、实现锚点、测试矩阵与未实测项。
- [真实 NAS 挂载健康检查（阻塞）](../real-nas-initial-index-20261105/REPORT.md)：`0.1.0~dev20261105.2` 在 build 17892 的 `initial_index` 断连阻塞、探针证据、归因边界与后续建议。
- [当前功能报告](../../FIX_7_REPORT.md)：用户可观察行为、验收矩阵及未实测项。
