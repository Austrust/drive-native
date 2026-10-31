# 状态索引：分页只读 WAL 初始化

本模块用于下一阶段增量状态索引的初始化，不改变当前 desktop 路径，不代表已完成大目录或 NAS 验收。[索引契约](INCREMENTAL_STATUS_INDEX.md)负责发布与追赶规则；这里仅提供一个可取消、有限资源的历史元数据快照。

## 接口与边界

`SharedStore::try_begin_status_snapshot(SnapshotBudget)` 在 Store 忙时立即返回 `Busy`。获取 guard 后，读取该连接的 TEMP journal watermark、SharedStore verified/revision 和主数据库路径；打开只读 main 连接，检查 WAL 模式，执行 `BEGIN DEFERRED` 后实际读取 `main.nodes`，再释放 guard。未提交事务、poison 和 unverified 边界均拒绝建立快照。

SQLite 的 `BEGIN DEFERRED` 本身只改变自动提交状态；首次数据库读取才建立读事务。固定 main 表的读取不能替换成 `SELECT 1`、TEMP journal 查询或仅 `BEGIN`。WAL 读事务维持同一个 end mark，后续提交不会改变这次读取结果。依据：[事务语义](https://www.sqlite.org/lang_transaction.html)、[WAL 快照隔离](https://www.sqlite.org/isolation.html)、[WAL end mark](https://www.sqlite.org/wal.html)。代码不依赖可选的 SQLite snapshot 扩展。

返回的 `StatusSnapshotReader` 自己持有只读连接，不持 Store 锁；`next_page` 按稳定文件 ID 做键集分页，重用现有 Projector 的可见性、双父 ACL、继承状态和错误投影。新连接不能读取 writer 的 TEMP journal。每页仅返回实际扫描数、可见数和逻辑计费；全部读完后 `finish` 才交付完整 atoms、初始游标与 source stamp。部分结果不可当作已发布索引。

source 包含主库规范路径、该 writer 的 journal epoch、捕获时的 commit verified/revision。路径相同不等于同一 Store 生命周期。完成快照仍是历史视图：调用方必须从捕获游标连续读取并应用 delta，处理 epoch/gap，以及提交后尚未消费的查询失效；不能用快照完成或旧 commit revision 宣称已同步。此接口沿用单 writer 所有权，不支持绕过 writer lock 改库、替换数据库文件。

## 资源和生命周期

初始化默认最多 500,000 个源节点、512 MiB 累计逻辑计费、64,000,000 次投影工作及 120 秒，每页 512 项。节点上限与 IndexOwner 的默认值一致，避免使用 delta 的 100,000 节点预算导致 21 万节点必然失败。默认不是所有名称长度/目录深度组合都能容纳的承诺；超限必须报告具体预算失败，不无限重试同一配置。

配置硬上限为 500,000 节点、1 GiB 逻辑计费、128,000,000 次工作、300 秒、每页 4096 项。计费累计每页的 ID、raw/祖先/输出暂存，重复祖先也重复计费；不是只计算返回 atoms，也不声称等于进程 RSS。页级 raw 缓存用完释放，已完成 atoms 保存在 reader 中。时间包含页间暂停，每页和投影步骤检查；超限或查询错误关闭事务并丢弃已累积输出。

`cancel`、`Drop`、失败、`finish` 均结束读事务。持有 WAL 读事务会延迟 checkpoint，因此未来后台 owner 必须持续驱动并在暂停、关闭、任务更换及超时时丢弃 reader。core 不暗启线程：如果调用方把 reader 长期闲置，单靠本接口不会自动关闭；严格的无人驱动 TTL 由后续 owner 任务实现。短边界仍包含本地连接打开/首次读取的 I/O，不能宣称硬实时。

## 验证状态

2026-10-31，在独立 WSL Ubuntu 24.04 源码副本、target 和临时数据库中验证：

- `status-wal-snapshot-002`：全部 core 测试 256 通过、0 失败、1 项容量用例默认忽略；随后显式运行该容量用例通过。初次专项为 12 项通过。
- `status-wal-snapshot-003`：最终专项 13 通过、0 失败、1 项容量用例默认忽略；新增失败后 reader 仍在作用域时可成功 TRUNCATE checkpoint 的测试，并将空表后续写入改成单行 SQL，避免依赖全量安装的 journal 策略。相对上一轮生产逻辑只调整了短锁注释，没有再次全跑 core。
- 容量用例实际分页读取 210,001 个短 ID 合成节点，累计逻辑计费 294,453,888 bytes、1,470,415 次工作；每页之间可获取 Store。通过默认预算，不是实际 NAS 名称/层次/缓存分布的容量承诺，也不是无竞争性能结论。

专项覆盖写入期间多页快照、快照后 delta、空表首次读取、回滚、取消/析构及失败释放 checkpoint、只读与 TEMP 隔离、journal gap、Store 重开 epoch、忙/未提交边界、前缀不可发布、累计预算和页间超时。容量用例保留显式 ignored 标记，普通测试不自动产生 21 万节点负载。

首次 `status-wal-snapshot-001` 在新 target 编译 aws-lc-sys 时，读取宿主文件系统头文件返回 `Cannot allocate memory`，未运行新测试；保留失败日志。随后核对 WSL 可用内存约 28 GiB，并以 1 job 重试通过，不把它写成已定位的环境根因。后两轮负载已登记并正常结束。详细构建日志及源码校验值留在本地隔离 run，不进入公开仓库。

本模块尚未接入桌面 owner、严格 TTL 调度及发布屏障，没有修改基线、NAS 或桌面运行路径；不改变产品验收门槛。
