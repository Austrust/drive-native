# P0：Personal_Sync、培养恢复及小文件冷读取

2026-10-31，WSL Ubuntu 24.04，运行 `.6` 未修改产品代码。源码 `62be13c9f9f4a2457a3040829f478bf43b1821df`，deb SHA-256 `4a66edf61d1a32f385f6f61f401db3c181b74f80df44cd0411140ae373c73cf4`；fsd 身份及任务规模见 [Personal_Sync 条件](personal/context.json)、[培养条件](training/context.json)。系统/硬件/NAS条件沿用 [P0 身份核验](../p0-baseline-audit.json)，NAS 版本未重新查询。

## 恢复：两组均自动恢复，Personal_Sync 两次超过 3 秒

从原只读挂载分别备份到全新 SQLite 与缓存目录；原六个挂载及两个探针一直运行。Personal_Sync 为212,484节点，培养666节点。两个隔离任务使用不同任务和设备身份，通过各自回环字节转发器访问NAS；保持真实服务器名与CA校验，未写NAS或改变权限。培养没有已有缓存，先只读下载一个1,050,854字节文件；首次准备读取约0.201秒，随后才开始恢复实验。准备脚本最初查询了不存在的 size 列，在网络读取前失败；修正为 node_metadata 联表后执行，未把失败尝试当作成功下载。

沿用 [MFM 方法](../p0-recovery-001/REPORT.md)：每组预登记30次、15秒超时、50ms轮询，每次中断自有连接而保留fsd和转发监听器。恢复要求新worker PID、增加的连接计数及online；故障后读取预先打开的完整缓存并校验哈希。两个组顺序测量，另一个隔离实例空闲。文件打开与读者进程启动不计入缓存读取，连接恢复包含本地控制/D-Bus开销。

| 测量项 | 样本 | p50 | p95 | p99 / 最大 | 功能失败 |
|---|---:|---:|---:|---:|---:|
| Personal_Sync 恢复 | 30 | 1.351s | 3.202s | 4.059s | 0 |
| 培养恢复 | 30 | 1.174s | 1.229s | 2.134s | 0 |
| Personal_Sync 预开缓存读取 | 30 | 1.764ms | 2.163ms | 2.254ms | 0 |
| 培养预开缓存读取 | 30 | 1.910ms | 2.365ms | 2.536ms | 0 |
| Personal_Sync 根目录枚举 | 30 | 10.575ms | 13.420ms | 13.467ms | 0 |
| 培养根目录枚举 | 30 | 2.435ms | 3.182ms | 3.367ms | 0 |

Personal_Sync 缓存1,005,024字节，根目录25项；培养缓存1,050,854字节，根目录1项。这里不验证2000项目录目标，两个目录规模也不构成公平性能A/B。60次采样均保持mounted，内容均匹配故障前版本；不证明服务器版本收敛或事件无遗漏。

Personal_Sync 的零基 trial 12/23 超过3秒，时间线显示约1.02秒进入connecting，分别在4.06/3.20秒返回online。`Service::supervise` 在online前包含认证、附着和事件pull；现有采样不能区分其中哪一步慢。不得直接归因于索引重建或用缩短退避冒充根因修复。培养这组30次均满足3秒，但不是全部环境的恢复保证。

证据：[Personal原始样本](personal/recovery/samples.jsonl)、[汇总](personal/recovery/summary.json)、[预登记](personal/recovery/manifest.json)；[培养原始样本](training/recovery/samples.jsonl)、[汇总](training/recovery/summary.json)、[预登记](training/recovery/manifest.json)。仅显式断开，不模拟静默丢包、长时间离线、休眠或服务器重启。

## 约 1 MiB 冷读取：30 次成功，1 次超过 2 秒

只在培养隔离实例重复读取上述文件。每次通过产品 D-Bus ReleaseSpace 清除该文件的本地完整缓存，确认 SQLite blob 为NULL且之前的blob路径不存在，才发起读取；不直接修改数据库，不删除NAS文件。认证和元数据保持温热，NAS端缓存不可控。完成2次预热和30个正式样本，预先规定15秒超时、首个功能失败停止；全部样本保留。

| 自进程内部计时，30样本 | p50 | p95 | p99 / 最大 |
|---|---:|---:|---:|
| 子进程启动（单列） | 22.911ms | 25.106ms | 26.053ms |
| open返回 | 12.484ms | 13.683ms | 13.961ms |
| 首字节返回 | 1.757s | 1.813s | 2.042s |
| 完整读取完成 | 1.759s | 1.815s | 2.044s |

30次完整哈希及字节数均匹配先前已下载的完整版本，远端版本字段前后不变；它不是独立服务端内容判定器。trial 8超过2秒，因此不能声明所有未缓存小文件读取都满足目标。首字节等待包含整文件原生下载、校验、持久发布与调度；不能把总耗时等同于哈希计算时间。worker使用异步任务派发，原生下载通过连接池获得通道；尚未分段测量实际锁等待、服务器响应和fsync耗时。

证据：[条件](training/network-read/manifest.json)、[预热](training/network-read/warmup.jsonl)、[30样本](training/network-read/samples.jsonl)、[分位数](training/network-read/summary.json)。链路未加延迟/丢包/限速，使用透明回环转发；其他六任务、探针及空闲Personal副本运行。不将这组结果替代大文件链路吞吐或官方A/B。

## 现场、源码及下一步

两个测试实例分别在0.218/0.217秒正常退出，转发器随后停止，无残留挂载：[Personal退出](personal/stop-result.json)、[培养退出](training/stop-result.json)。状态保留。结束时原六服务和24小时观察服务均active；观察第445轮、约74分钟，累计4个失败采样，未完成24小时。

[执行脚本身份](source-identity.json)记录LF归一化SHA-256；转发工具来自提交be72923。脚本副本仅用于追溯具体实验，不作为通用生产配置工具。本轮未改运行时、未重跑全量产品回归、没有新deb；这仍是P0测量补充。继续补大文件/链路基线与下载阶段诊断，P0是否退出只见 [DELIVERY_GATES.md](../../../DELIVERY_GATES.md)。
