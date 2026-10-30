# 官方 Linux 长轮询：通道超时与整个请求截止的区别

2026-10-31，静态核验用户提供的官方Linux ELF。**该构建的长轮询把600传给通道超时；缓冲读取层用秒数判断无数据进展的时间，并在读到数据后刷新计时点。它与native当前450秒的整个请求截止不是相同语义。** 本报告只记录客户端静态证据；后续[MFM诊断003](../p0-wait-deadline-003/REPORT.md)补充了服务器秒数预算与本地450秒超时的真实记录，尚未完成产品修复验证。

## 构建和适用范围

`cloud-drive-daemon` SHA-256：`ea01d634762060424f4a5fd3fccb3f5c0ba7c10a81e90e144706fb2b8406314a`，读取原始文件重新校验通过。应用INFO为4.2.1、build 20062；同目录解包来源的Debian control版本为8.2.1-20062，两种原始版本标识均保留，未自行统一。版本文件哈希见[context.json](context.json)。

只读提取11个函数、两个虚表及一个PLT导入映射；未执行、安装或链接官方二进制，没有新增NAS连接。结论限定于此ELF和以下调用链，不提供跨版本ABI承诺，也未穷举官方进程所有更外层的超时策略。

## 可追溯调用链

| 步骤 | 静态位置及证据 | 确认内容 |
|---|---|---|
| 构造long_poll | [LongPoller::LongPoll](protocol/00.asm)，0x5cd9e、0x5d415～0x5d41d | 报文字段timeout写入400；调用Request的额外整数参数为600。本次客户端静态范围未核验400的服务器端单位；后续服务器证据见003 |
| 转交参数 | [Request简化重载](protocol/01.asm)，0x734e4、0x73522、0x73534；[完整重载](protocol/02.asm)，0x72072、0x72172～0x7219b | 参数经寄存器和栈传到PerformRequestAndReceiveResponse；没有把400和600混为同一字段 |
| 设置通道预算 | [PerformRequestAndReceiveResponse](protocol/03.asm)，0x6c0c2～0x6c0cc、0x6c14d～0x6c158；[Channel虚表](channel-vtable.json) | 虚表偏移0x40是SetTimeout，发送/接收前设置传入值，结束路径恢复60 |
| 保存预算 | [Channel::SetTimeout](protocol/04.asm)，0x2b7b60～0x2b7b6c；[BufferedIOBase::setTimeout](protocol/07.asm)，0x318ed0 | 通道字段和缓冲I/O对象的+0x18字段保存相同值 |
| 保活不结束请求 | [RecvResponseFromChannel](protocol/06.asm)，0x6b858～0x6b8af、0x6b8b5～0x6b904 | 比较type与keep_alive；相等时恢复对象并返回接收循环。保活帧不是最终通知 |
| 读取实现 | [Channel::Read](protocol/05.asm)，0x2b88fb～0x2b8908；[BufferedIO虚表](buffer-vtable.json)；[BufferedIO::read](buffer/02.asm)，0x319e13、0x319e9b | 缓冲读取转入wait_and_read；虚表偏移0x28映射read，未仅凭函数名猜测虚调用目标 |
| 有进展就刷新 | [wait_and_read](buffer/00.asm)，0x319bad、0x319c20～0x319c3a、0x319cb0～0x319cbe | 开始时取得时间；正数读取后重新保存当前时间；等待无就绪数据时以保存的时间检查超时 |
| 单位为秒 | [functionTimeout](buffer/01.asm)，0x3191d9～0x319204；[PLT映射](time-plt.json) | 0x1e5e0解析到C库time；当前time减去保存值，与预算比较。该处为秒数、严格大于比较，检测粒度和系统时钟会影响实际触发时刻 |

这些证据支持“此通道读取层使用600秒进展超时”，不支持“官方每600秒重连”或“官方长轮询最多600秒”的说法。一次调用中持续收到数据，可以反复刷新读取计时点；更外层是否另有限制未在本次范围内确认。

## 与native当前实现的差异

源码基线 `1285aa08832dfb4f706a5bd013dae7e0f8163598`：

- [sproto channel.rs](../../../crates/sproto/src/channel.rs)的`request_with_timeout`用一次Tokio timeout包住整个send＋recv；`recv_single`跳过keep_alive时不会刷新这次450秒截止。
- [原生wait_changes](../../../crates/sproto/src/actions/native.rs)仍发送timeout=400，传输截止为450秒；[桥接事件循环](../../../crates/drive-runtime/src/bridge.rs)的外层RPC也为450秒。
- 因而，即使底层还收到保活/数据，绝对截止依然可能结束等待。此处是源码确认的行为差异；尚未观察到真实失败前的保活帧，不能把“可能”提升为该NAS断连的已证实原因。

此前[pidfd退出](../p0-worker-exit-001/REPORT.md)给出约450秒周期；[截止对照001](../p0-wait-deadline-001/REPORT.md)未复现；[MFM诊断002](../p0-wait-deadline-002/REPORT.md)在初始化门槛前失败。后续必须分别取得初始化/退出卡点与长轮询失败时的阶段证据。若最终需要调整，也应同时审查原生进展计时、外层RPC等待及取消边界，不能只把450替换成600。

业务代码未复制，产品运行时未改动，没有新增安装包。研究记录用于P0故障定位与后续实现评审，阶段状态见[DELIVERY_GATES.md](../../../DELIVERY_GATES.md)。提取方法为符号边界内x86-64反汇编、局部LEA常量传播、ELF重定位和虚表槽定位；不是完整反编译。实际脚本见[extract.py](extract.py)，全部产物见[SHA256SUMS](SHA256SUMS)。
