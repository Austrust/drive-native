# P0 退出分段短测：正常结束，原失败仍开放

2026-10-31，WSL Ubuntu24.04。只读MFM副本初始化完成后，按预先约定等待10秒再停止。实际停止0.218067秒，systemd success、退出0、没有残留挂载。本轮没有复现[004的停止超时](../p0-wait-deadline-004/REPORT.md)，不能因此关闭该缺陷。

## 方法及身份

一次性源码基于`f1d17ec3d79ab59b9e2c9cebeee77895d8e439d4`，沿用600秒原生绝对截止、650秒RPC截止以及请求timeout=400；新增supervisor/uploads/cache join、worker进程等待和FUSE join的起止标记，没有改变退出顺序或锁。原六个`.6`冻结实例及产品源码不变。

fsd SHA-256 `7308011668634638a82be30cf0edbdc6e0df49425ccec0df9ede21e6179d4995`，worker `b7452c657708bbf5e8b23618d5036e5b93edf905ffcb89c345fad24b3b86ae17`。构建再次通过132项协议单元测试、2项文档测试，1项文档测试忽略；未重跑全产品/GUI。

MFM副本91,033节点；snapshot RPC84.344100秒，本地快照安装5.274243秒。这些仅是阶段耗时，不等价于官方同工作量性能比较。没有NAS内容或权限修改。

## 已观察到的退出顺序

[完整日志](final-trace.jsonl)显示显式停止后：shutdown返回，等待中的RPC报NotConnected，worker等待结束，supervisor/uploads/cache各自join完成，最后FUSE join完成。由日志单调时间差得到tasks join约5.861ms、FUSE join约3.578ms；systemd退出观察约218ms，测量起点和采样粒度不同。

本次改为枚举FSD所有线程的children，再收集这些直接子进程的线程等待/FD计数；较004只读主线程children更完整，但仍不声称覆盖任意深度的进程树。没有输出进程参数、凭据或正文。

短测是在第一次长等待期间停止；004是在483秒通知后进入下一次等待时停止，时序不同。因此这里仅建立“新增阶段记录可运行且此短路径正常”的证据。下一步应沿004的响应后停止边界使用同一组标记复现，不能把当前短测正常视为延长截止已经解决退出问题。

导出时原六个FSD PID及`.6`哈希保持，24h基线约3.88小时、第1398轮、16个失败采样。阶段状态仍见[DELIVERY_GATES.md](../../../DELIVERY_GATES.md)。本轮无产品改动或新包；开发发布`20261031.1`与本诊断分开。

证据：[manifest](manifest.json)、[结果](result.json)、[检查记录](checks.json)、[初始状态](initial.jsonl)、[停止前记录](trace.jsonl)、[完整记录](final-trace.jsonl)、[校验清单](SHA256SUMS)。
