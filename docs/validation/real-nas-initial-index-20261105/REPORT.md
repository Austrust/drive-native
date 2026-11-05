# 真实 NAS 只读挂载健康检查：`initial_index` 阻塞（2026-11-05）

状态：**阻塞发现**。新开发包 `0.1.0~dev20261105.2` 在真实 NAS（build 17892）上 selector 登录/目录列表正常，但 fsd `initial_index` 两次均失败（`native_transport` / `native_response_timeout`，TLS `UnexpectedEof: peer closed connection without TLS close_notify`），`nodes=0`，无残留挂载。真实 NAS 的逐目录 ACL 与端到端验收因此**未完成**；Issue #7 保持开放。

本轮只做**只读**健康检查与既有证据归因；没有在 NAS 上写文件、改权限或运行生产任务，不记录凭据，不 push。**未找到根因**；现有证据只支持“长初始化序列中被服务端/网络中断，产品缺少每请求重试与断点续拉”，不支持“服务端版本差异/会话限制/协议字段语义”任一具体归因。

## 1. 目标、环境与产品

| 项 | 值 |
|---|---|
| 目标服务器 | 真实 NAS `[redacted-test-host]`，TLS 名 `[redacted-test-tls-name]`，Synology Drive Server build 17892（与本项目 2026-11-01 只读核验同一台） |
| 账号 | 测试账号，非管理员；凭据由 Lead 掌握，本文与仓库不记录 |
| 被测产品 | 本地开发 deb `0.1.0~dev20261105.2`；源码行为 = 产品提交 `fefb8ee` + T4 审阅要求的 `debug_assert_eq!(code,0x9003)`（assert-only）；**无初始化/重试/协议改动** |
| 操作 | 只读挂载健康检查：selector 登录与目录列表；fsd `initial_index`；无 NAS 写操作、未改权限、未安装到生产宿主 |
| 观察者 | Lead（fsd 日志与隔离探针）；T1 只写本报告与文档，不连接 NAS |

## 2. 结果

| 阶段 | 观测 | 结论 |
|---|---|---|
| selector 登录 + 列表 | 登录成功；team root list 约 **0.03s**，`folders=18`；personal root `folders=3` | 凭据、TLS、视图/根元数据与基础 list 通路正常 |
| fsd `initial_index` 第 1 次 | 失败；错误类 `native_transport` / `native_response_timeout`；TLS 层 `UnexpectedEof: peer closed connection without TLS close_notify`；`nodes=0`；无残留挂载 | 初始化元数据接收被中断，未产生节点、未提交 |
| fsd `initial_index` 第 2 次 | 同上（两次均在 `initial_index` 阶段失败） | 可重复的阻塞；不是一次偶发 |

失败发生在初始化候选接收阶段（`nodes=0`），不是挂载/权限判定阶段；没有产生可验收的挂载状态。

## 3. 隔离只读探针（Lead，仓库外调试副本）

探针在隔离调试副本中运行，只读、不写 NAS；调试副本与凭据均不入库（本报告只记录计数与分类）。

| 探针 | 观测 |
|---|---|
| `Client.list_dir` 小请求、单连接 | **200/200 全过** |
| `pull_event`（action=`pull_event`, recursive, offset=0）cursor=0 | 常成功：约 **516 events**，`next_sync_id=2000`，耗时约 0.5–2.3s |
| 紧接 cursor=2000 的 `pull_event` | 常被服务端**直接断 TLS**（UnexpectedEof），约 0.4–1.1s；重试常失败，偶发单次成功（曾 `0→2000→4000` 两页连过） |
| `list_dir` ×5 与 `pull_event` 混跑 | list 5/5 全过；pull cursor=0 过；随后 cursor=2000 连续 3 次失败 |
| 子目录 scope `/Drive`（watermark S0 路径） | 可取得 S0 并进入 enumerate；10s 时约 **948 目录 / 8351 节点**，随后在约 **1000 次 list 请求**内被断连 |
| 根 scope `/`（legacy seed 路径） | 走 `collect_snapshot_seed` 从 cursor 0 逐页 `pull_event`（本报告 §4） |

探针支持的关键事实：**小请求与首屏正常，连接中断集中出现在大响应/长序列/大量请求之后**；子目录 watermark 路径能推进到枚举，说明 S0/根查询本身可用，但同样在长序列中被断连。

## 4. 产品侧数据流与失败传播（代码锚点）

- 挂载根 scope 为 `/`：`native_snapshot.rs` 在 `scope.path=="/"` 时**不使用** `current_snapshot_seed`（`snapshot_candidate_with_observer` 的 `if seed.is_none() && self.scope.path != "/"`，`native_snapshot.rs:66-77`），而是走 legacy `collect_snapshot_seed`（`:81-92`，定义在 `:497-599`）：从 `EventCursor(0)` 开始逐页 `scoped_events`，以“`next == cursor` 的静止批”作为 fence/边界（`:585-596`）。
- `scoped_events` → `Client::pull_events`（`native_root_guard.rs:33-69`；`sproto/src/actions/native.rs:120-138`）：请求 `action=pull_event`，携带 `view_id/node_id/path`、`sync_id=watch.cursor`、`recursive=true`、`offset=0`、`file/mtime/execbit=true`；`parse_events` 校验 `next_sync_id` 不倒退、事件 `sync_id` 不超过 next（`native.rs:234` 起）。
- 失败传播：`collect_snapshot_seed` 对 `fetch` 返回的 transport/TLS 错误**不重试**，直接向上返回；`recovery_snapshot_with_observer` 的 3 次尝试只覆盖 `Error::SnapshotChanged`（枚举冲突），不覆盖 `pull_event`/`list_dir` 的传输中断（`native_snapshot.rs:344-357`）。枚举阶段 `list_dir` 失败同样直接返回（除已知 12290 父不可列的特例）。
- 对照：非根子目录可用 watermark（`get_max_sync_id` S0 + 当前根查询，`native_snapshot_watermark.rs`）；`docs/SNAPSHOT_WATERMARK.md` 的**兼容边界**明确“路径 `/` 尚无本轮验证的当前根元数据查询，保留历史 seed 路径”，且 2026-11-01 的真实只读核验只覆盖“选定个人子目录”，不覆盖根。

因此，一旦服务端在 `pull_event`/`list_dir` 的某一页主动断连，根 scope 的 `initial_index` 直接失败；这与探针观察一致。中断点的协议语义未定，本轮不做推断。

## 5. 协议实现对照（静态证据，仅用于排除“产品臆造分页”（不构成根因）

对已归档的官方 Linux 客户端 `cloud-drive-daemon`（4.2.1 build 20062，SHA-256 `ea01d634762060424f4a5fd3fccb3f5c0ba7c10a81e90e144706fb2b8406314a`，仓库内既有静态证据，只读反汇编、未执行）复核 `PullEventHandler::PullEventFromServer`：

- 官方请求 action 为 `pull_event`，请求字段包含 `sync_id`、`path`、`node_id`、`recursive`、`offset` 等；
- 请求 `sync_id` 的取值来自调用方事件内容里的 `next_sync_id`（即上一轮返回的 next 作为下一次请求 cursor）。这与产品 `collect_snapshot_seed` 的“next 作为下一页 sync_id”方向一致，**不支持**“next=2000 就不能作为 sync_id”这一猜测。
- 其余请求字段的形状/名称差异（例如文件选项细节）本轮受时间盒限制未完成对照，**不作为任何失败原因**。

该静态对照只说明“按 next_sync_id 续拉”是官方模式，未证明服务端断连原因。

## 6. 归因边界（必须区别已证/未证）

**已证（本轮证据支持）：**
1. 真实 NAS 的 TLS、账号、selector 与普通 `list_dir` 通路可用（selector list 0.03s；探针小请求 200/200）。
2. 在真实 NAS 的长初始化序列中，服务端/网络会主动中断连接（TLS `UnexpectedEof`，无 close_notify）；中断集中在大响应/长序列/大量请求之后，且可重复。
3. 产品当前对 `pull_event`/`list_dir` 的单页失败没有每请求重试或断点续拉，初始化候选因此无法完成；`nodes=0`、无残留挂载。
4. 失败不是 T1 窄子集修复引入的：`.2` 产品行为等于 `fefb8ee` + assert-only，初始化路径未改；G4-013 合成矩阵通过（但合成矩阵不能替代真实 NAS）。

**未证（不得写成本轮结论）：**
- 服务端版本差异、会话/连接数/时间限制、大响应分页语义、网络中间设备、账号/共享配置——均无直接证据（中断只有 TLS EOF，没有服务端错误码/响应）。
- 具体是哪一页/哪一次请求先触发断连；是否与 `pull_event` 的 `offset`/`file` 字段形状有关。
- 旧版本/旧账号/其他网络下是否同样失败（未做对照）。

## 7. 与项目既有记录的关系：既有集成缺口在 #7 候选上的暴露

以下项目自身记录此前已多次标注真实 NAS 初始化/续扫未完成，说明这是**既有集成缺口**，不是 #7 窄子集修复引入的已证回归：

- [生产持久初始化调度接入](../../../../../outputs/drive-native/initialization-owner-20261101/REPORT.md)：未关闭边界明确“仍无真实 NAS……真实 NAS 初始化／续扫……继续开放”。
- [development-20261101-3](../../../../../outputs/drive-native/delivery/development-20261101-3/RELEASE_NOTES.md)：断连续扫仅有合成本地回归，“尚未做完整真实 NAS 续扫”。
- [development-20261101-4](../../../../../outputs/drive-native/delivery/development-20261101-4/RELEASE_NOTES.md)：**“真实 `.3` 冷初始化曾失败。”** 新包“真实只读初始化尚未通过”。
- [Personal_Sync package013 终态观察](../../../../../work/initialization-pipeline-8/docs/validation/personal-003/REPORT.md)：真实 NAS 3600 秒预算内未 fence/提交完整检查点；控制断网窗口外 `list` 响应超时 7 次、transport 失败 6 次。

## 8. 对 Issue #7 验收状态的影响

- #7 核心修复（根权限不扩整盘、父目录自身权限不误限制孩子、明确读拒绝、精确写拒绝窄子集学习）与独立 **G4-013** 矩阵仍成立；本文不改写这些结论。
- 真实 NAS 的**逐目录 ACL / 端到端验收与挂载健康检查**因 `initial_index` 阻塞**未完成**；#7 完整验收保持开放，不能以合成矩阵或本文替代。
- `.2` 开发包版本与 changelog 不因本文改变；本文只追加证据与状态。

## 9. 后续建议（待下轮验证，本轮不改产品代码）

| 优先级 | 建议 | 代价/边界 |
|---|---|---|
| P0 | 为 `list_dir`/`pull_event` 增加**每请求有限重试 + 断点续拉**（同一 cursor/同一页重试；seed/enumerate 均覆盖；超时/取消/断连分类；有界 backoff），再用真实 NAS 复测 | 不改服务端语义；需防重试放大与重复事件；schema/IPC 可不变；需合成故障注入（断连点、重复页、半页）与真实 NAS 验证 |
| P1 | 根 scope 的 `watermark S0 + 当前根查询` 可行性研究（需证明 `/` 的当前根元数据查询/身份回包；否则保留 legacy seed 并在 P0 后重测） | 需协议证据与真实 NAS 只读探针；失败时必须回退而非伪造根 |
| P2 | 环境对照：旧版本（如 `.3`/`.8`）、旧账号/网络、同只读夹具下复跑，区分服务端/网络因素 | 需管理员与旧包；不能写生产共享 |
| — | 不建议在根因未定时修改 `pull_event` 字段形状或分页语义 | 冻结的官方客户端对照未完成；投机性改动会扩大风险 |

## 10. 未实测 / 边界

- 本轮 T1 未连接 NAS；本文事实来自 Lead 的只读健康检查与仓库外隔离探针；原始 fsd/journal 日志由 Lead 保存，本文只记录其分类与计数，凭据/PII 不入库。
- 未做服务端抓包、未取得服务端错误码/响应；未在旧版本、其他账号/网络、其他 build 上对照；未做真实 NAS 写操作。
- 未验证 FUSE 挂载后的读写行为（初始化未完成）；未验证普通目录全量同步；G5/Mint/7×24 等原门槛不变。
- 本文是“阻塞发现与证据边界”，不是根因报告；在补充对照与 P0 鲁棒性改动前的任何“已修复”声明都不成立。

## 后续（Round 2）

本轮阻塞后已做根因收敛与最小修复：测量表明团队根 legacy seed 在每请求重试下可完成（76 页/102,014 事件/215s/25 次断连恢复），已实现有界重试+断点续拉并加确定性测试；个人空间超大根仍为残余。见[根因收敛与最小修复报告](REPORT-R2-RESILIENCE-20261105.md)。
