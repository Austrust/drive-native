# P0 部分观察审查：采样连续性与 worker 周期

这是冻结 `.6` 的约2小时22分钟观察副本，**不是24小时完成报告，也不是修复后验证**。六任务各852个样本，迭代0～851；共5112条任务样本和1条观察器启动记录。原服务继续运行，没有重启、改动二进制或写入NAS。

## 连续性与失败

没有JSON解析错误、缺轮、重复轮次、时间倒退或超过15秒的相邻采样间隔。每个任务观察到的FUSE主PID始终相同，定期核对的二进制哈希均属于manifest规定的 `.6`。这只能说明采样覆盖与进程身份，不能由此推断目录读取和内容正确性。

8个失败样本完整保留：MFM协作2、培养1、实验报销1、模拟报销2、我的文件1、Personal_Sync 1。失败样本中挂载标记仍为true；10秒采样不能测出准确恢复时长。GetOverview全部有耗时记录，但它不是FUSE目录或正文读取。

| 任务 | worker PID数量 | GetOverview p95 / 最大，ms | fsd+直接worker RSS最大，MiB |
|---|---:|---:|---:|
| MFM协作 | 18 | 9.18 / 100.68 | 346.20 |
| 培养 | 12 | 8.94 / 58.25 | 19.39 |
| 实验报销 | 19 | 9.21 / 121.37 | 19.61 |
| 模拟报销 | 15 | 9.48 / 114.56 | 17.70 |
| 我的文件 | 13 | 9.97 / 521.78 | 457.67 |
| Personal_Sync | 13 | 12.81 / 4852.14 | 1590.30 |

全部p50/p95/p99/max、失败、RSS/PSS和统计缺口见[summary.json](summary.json)。RSS合计含共享页重复；不包含GUI。窗口有既有探针、基准、编译和上传负载，不能把均值当作空闲CPU验收，也不对内存泄漏作结论。CPU只在相同PID集合且间隔不超过15秒的样本对上累计，其余区间不按零用量处理。

## 450秒线索

观察到的worker PID数量远高于失败采样数。排除首尾不完整区间和无worker的区间后，得到78个存活时间有上下界的区间；其中62个区间包含450秒，多数下界约441～442秒、上界约461～462秒。上下界由前后10秒采样限定，不是进程精确出生/退出时间，原数据见[worker-lifetimes.json](worker-lifetimes.json)。

对应冻结源码的静态路径：

- `Bridge::event_loop` 给 `Command::Wait` 设置450秒RPC截止；[bridge.rs](https://github.com/Austrust/drive-native/blob/62be13c9f9f4a2457a3040829f478bf43b1821df/crates/drive-runtime/src/bridge.rs)。
- `Client::wait_changes` 发出原生long_poll，请求字段timeout为400，底层交换截止也是450秒；[native.rs](https://github.com/Austrust/drive-native/blob/62be13c9f9f4a2457a3040829f478bf43b1821df/crates/sproto/src/actions/native.rs)。字段单位与服务端空闲返回行为需要另行实测。
- `Rpc::call_controlled` 截止后返回TimedOut；`Service::supervise`将其归入network_unavailable并重建worker；[rpc.rs](https://github.com/Austrust/drive-native/blob/62be13c9f9f4a2457a3040829f478bf43b1821df/crates/drive-runtime/src/rpc.rs)、[service.rs](https://github.com/Austrust/drive-native/blob/62be13c9f9f4a2457a3040829f478bf43b1821df/crates/drive-runtime/src/service.rs)。

因此优先假设是“空闲长轮询的截止触发整个worker恢复”，而非把每次worker更换都判为NAS随机掉线。当前样本没有具体RPC结束原因或TLS连接关闭时序，尚未确认因果。下一步应在隔离实例区分外层RPC截止、底层原生截止、服务器主动关闭及正常空闲回复，并参考官方协议行为；不能仅拉长超时或吞掉错误就称为修复。此处没有修改产品行为。

## 证据与复核

- [原始副本](observations.jsonl.gz)：仅本地状态、错误类别和进程资源，不含文件正文或凭据。
- [条件](manifest.json)、[当时的负载登记](planned-loads.jsonl)、[文件哈希](evidence-sha256.json)。本轮curl上传的负载区间在冻结后追补登记于WSL主日志；该副本从未标为纯空闲窗口。
- [审查脚本](p0-review-observation.py)、[worker区间脚本](p0-worker-lifetimes.py)：实际执行源码，路径指向WSL保留的review-001目录。

冻结副本SHA与解压后内容逐项校验；观察器当前status可能晚于文件快照，不能将两者视为原子快照。完成记录缺失且 `complete_24h_record_present=false`，P0继续运行。唯一阶段入口见[DELIVERY_GATES.md](../../../DELIVERY_GATES.md)。
