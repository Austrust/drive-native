# P0：冷读取阶段定位与大文件基线

2026-10-31，WSL Ubuntu 24.04。仍运行未修改的 `.6`：源码 `62be13c9f9f4a2457a3040829f478bf43b1821df`，deb SHA-256 `4a66edf61d1a32f385f6f61f401db3c181b74f80df44cd0411140ae373c73cf4`。运行文件身份见 [context.json](context.json)，硬件/系统见 [原始核验](../p0-baseline-audit.json)。使用培养的全新只读隔离副本（666节点）、独立任务/设备身份及透明回环转发；真实NAS证书验证保持开启，原六任务与探针继续运行。没有修改NAS内容或权限。

## 主要发现：重复下载拒绝后的重试占据等待时间

使用与上一轮相同的1,050,854字节文件。准备读取约0.187秒；之后每轮通过ReleaseSpace释放本地缓存并核对blob引用及文件均已移除，预热2次、正式10次。每20ms查询现有本地GetTransfers，保留请求ID、进度与查询耗时，不保存路径。该查询会引入额外开销，因此这组用于诊断，上一轮无阶段采样的30次仍是原性能基线。

10个正式样本均成功，完整读取p50 1.760秒、p95/p99/最大1.836秒；每次都观察到两个不同下载请求ID。典型时间线：

1. 约22ms：第一个请求进入transferring，字节数仍为0。
2. 约43ms：第一个请求从活动列表消失。
3. 约1.45～1.62秒：第二个请求出现，随后传输并完成。

按前一采样状态延续至下一采样估计，每次约1.43～1.59秒没有该文件的活动RPC记录。这里的 `no_rpc` 只表示GetTransfers未返回记录，不是内核或网络级跟踪。部分样本捕获到verifying，跨度约21ms；未捕获的短阶段不等于耗时为零。

结束时累计 **24次下载尝试、11次失败**，最后失败为 `server_rejected / 32772`，未完成下载错误为0：[计数与错误类别](diagnostics-after-phases.json)。准备1次、预热2次和正式10次共13次最终成功，与24次尝试及11次内部失败一致。它支持本组主要等待来自拒绝后的重试，不支持“1.8秒都在校验”的解释。

源码 [download_jobs.rs](../../../crates/drive-core/src/download_jobs.rs) 将首次失败的重试时间设为当前整数秒加2；[Bridge](../../../crates/drive-runtime/src/bridge.rs) 在同一次读取中等待该时间并重新请求，因此实际等待约1～2秒。该策略是已有恢复逻辑，本轮没有改短它。32772的完整触发/清理语义仍未恢复，不能据此断定连接复用、服务器限流或缺少某个确认字段是根因。此前项目 `DOWNLOAD_RECOVERY_REPORT.md` 已记录新会话及Windows字段组合仍会复现，避免重复把这些猜测当成修复。

证据：[预登记](phases-001/manifest.json)、[全部阶段采样](phases-001/samples.jsonl)、[预热](phases-001/warmup.jsonl)、[分位数](phases-001/summary.json)、[状态区间估计](phases-001/phase-summary.json)。GetTransfers不是精确追踪工具；尚未分离服务端处理、协议接收、worker fsync与fsd发布耗时。

## 大文件：约67 MiB，3个正式冷读取样本

预先按培养目录元数据选择64～128MiB范围内最小文件，固定 **70,289,047字节**。初次准备下载后取得完整内容哈希；后续1次预热、3次正式读取均先清除本地完整缓存。认证/元数据温热，NAS端缓存不可控。单次截止120秒，无并发合成负载。所有正式样本哈希一致、远端版本字段未变化；哈希来自此前完整下载，不是独立服务端内容判定器。

| 指标，3样本 | p50 | p95 / p99 / 最大 |
|---|---:|---:|
| 子进程启动（单列） | 23.428ms | 25.953ms |
| open返回 | 9.744ms | 9.944ms |
| 首字节返回 | 4.121s | 4.187s |
| 完整读取 | 4.199s | 4.267s |

三个样本的有效吞吐分别为23.156、15.710、15.963 MiB/s（文件字节数除以完整读取时间，排除子进程启动）。该时间包含整文件下载、核验、持久发布和应用读取，未拆分其中的重复请求。3个样本仅构成初步基线，不提供稳定尾延迟保证。

证据：[条件](large-read-001/manifest.json)、[原始样本](large-read-001/samples.jsonl)、[预热](large-read-001/warmup.jsonl)、[汇总](large-read-001/summary.json)、[首次准备](large-preparation-result.json)。网络读写数量受实验manifest限制，没有启用固定整个目录。

## 链路基线未获得，不能计算80%达标比例

只读查询发现NAS标准iperf3端口5201可建立TCP连接。将Ubuntu源的iperf3及依赖解包到独立用户测试目录，未安装系统包或启动NAS服务；包版本/哈希见 [工具清单](link-tool-packages.json)。

预先登记单流NAS→WSL反向测量：2秒预热、3次各5秒，单次进程截止12秒。实际在预热握手阶段返回 `control socket has closed unexpectedly`，立即停止；**正式轮次0/3，未取得吞吐数值**。见 [manifest](link-001/manifest.json)、[失败原文](link-001/raw--1.json)、[样本](link-001/samples.jsonl)。端口可连接不代表iperf3服务可用，未重复冲击该端口，也未用网卡标称速率或Native自身吞吐冒充可用链路基线。

## 现场与下一步

测试客户端0.218秒正常退出，转发器随后停止，无残留挂载；[退出结果](stop-result.json)。首次缓存准备曾遇到connecting，等待同一客户端完成认证后成功，未重启。源码脚本及LF归一化哈希见 [source-identity.json](source-identity.json)。

24小时观察仍active，至第510轮约85分钟已有5个异常采样，新增模拟报销connecting/network_unavailable，前后采样online且mounted持续为true；[第三份失败快照](../p0-failure-context-003.json)。不以10秒采样推断准确恢复耗时或纯空闲故障率。

本轮只补证据，运行时代码未变、没有新deb。后续需建立有效的独立链路测速路径，以及在协议对照阶段定位32772；不能通过放宽校验或把重试时间直接缩短来宣称问题消失。阶段状态见 [DELIVERY_GATES.md](../../../DELIVERY_GATES.md)。
