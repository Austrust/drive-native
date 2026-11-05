# 原生 TLS 服务到实际 worker 的独立补测设计

本目录是非产品 harness。编译、端口监听和运行均须由协调者在主矩阵结束后安排。本说明记录设计与复现入口；本目录作者未编译、未执行，不能据此宣称通过。实际状态由各独立运行目录的 `build.json`、`results.json`、`case.json` 和日志决定。

本 012 副本保留原 `main.rs`、`codec.rs` 和全部验收断言。协调者已批准：完整 012 主矩阵退出后，可用 `--reuse-completed-run` 顺序复用其同候选 target；入口验证主矩阵成功、源码路径/commit/target 相等和补测前 ELF 哈希清单。此时每命令仍需宿主 C 至少 10 GiB；另建 target 仍需 40 GiB。新增信号清理只影响独立 Python driver，确保中断会清理自己创建的子进程组并停止进入后续用例。原 413 harness 和失败原件保持不变。

## 可测路径

服务端仅绑定 `127.0.0.1:6690`，必须先独占绑定成功才启动真实 worker。端口占用时返回 blocked，不连接占用者、不停止其它进程、不修改网络配置。worker 的公开 Login 没有端口或 TLS 模式字段，不能使用随机端口或关闭证书验证。

本地服务完成原生 `encrypt_channel` 升级、验证合成身份的认证、池连接与动作响应。TLS 只使用候选仓库的公开合成 CA/server/server-key 夹具，验证 `localhost`；不调用带上游转发的 `trust_relay`。服务端无外连代码。所有身份、路径、响应、Store 和正文元数据均为合成数据。

实际产品路径为公开 `WorkerProcess::start` → 未修改 worker ELF → `sproto::Client` → native adapter → IPC/Rpc → 公开 `Bridge`。首次真实 `Bridge::pull()` 接收合成文件事件并产生 Node.version，避免手造私有 VersionToken。初始 cursor 为 `native-v1:77`，文件事件推进到 `native-v1:78`。

## 用例设计

以下是设计映射，执行结果独立记录。前六项到 Bridge/Store，后四项到实际 worker 的 RPC 响应，禁止相加称为十项完整 Bridge 链。

| case.name | 入口与预期 |
|---|---|
| `bridge_download_denied_3002` | 原生 download 3002 → `Bridge::ensure` EACCES；仅目标 Read=Deny，相邻 Read=Allow、父 Read/目标 WriteContent=Unknown，无正文发布。再次 ensure 精确要求 EACCES；到实际退避时间后调用 refresh_cached；两次之后原生 download 计数均须仍为一。 |
| `bridge_download_denied_3008` | 同上，原生拒绝码为 3008。 |
| `bridge_profile_false_empty` | 空 changes、profile=false；`event_loop` 到第二次 long_poll，目标/根拒绝及 sibling Allow 保留，revision 不变，digest 正确续传。 |
| `bridge_profile_true_empty` | 空 changes、profile=true；同一真实链清除远端证据至 Unknown，revision 加一，用户 ExplicitReadOnly 保留。 |
| `bridge_profile_true_other_view` | changes 只含其它 view；仍由独立 profile 位触发上述失效。 |
| `bridge_late_read_after_profile` | 下载已在途但服务端不回复；另一条 TLS 通道完成 profile 通知并到第二次订阅；确认 revision 加一后释放 3002。请求返回 EACCES，但目标 Read 保持 Unknown、无旧失败 gate 重新写入、无正文发布。 |
| `worker_rpc_generic_3004` | 直接 `worker.rpc.call(Download)`；只能得到 Failed/native_server_code:12292，无 descriptor；不调用 Bridge 下载应用。 |
| `worker_rpc_generic_8004` | 同上，普通错误码 32772。 |
| `worker_rpc_generic_9005` | 同上，普通错误码 36869。 |
| `worker_rpc_generic_a005` | 同上，普通错误码 40965。 |

下载拒绝不会被假定成 `retry_at=None`：首次失败可保留退避时间，harness 保存实际值并校验 ServerRejected、无 generic server_code、attempts=1。`downloads_needed()` 本身不是 ACL 筛选接口，本 harness 不据此宣称后台调度被拦截。

**已识别的待实测边界：** 候选公开 `Bridge::ensure` 的第二次 Read=Deny 门禁与到期后台重发，不能仅从 Store 字段证明。静态路径可能先返回 EAGAIN，或退避到期后重发；上述精确二次 ensure 和后台不发包断言按协调者要求保留。harness 在立即重试不符合预期时仍收集后台结果，随后同时报告失败；不改成接受 EAGAIN 或只检查 ACL 字段。退避观察至少经过三秒单调时间并到达记录的 wall deadline，八秒仍未到达则明确失败，不作时钟归因。

profile 用例的文件事件只用于事先构造 Node；通知前后 pull 均为空，不能把 bootstrap 事件称为 profile 通知产生的事件。第二次订阅是应用完成的同步屏障，不靠等待固定时间猜测。迟到场景用两个独立通道与明确门闩控制先后。

## 授权后的构建与运行

只在协调者授权且主矩阵结束后执行。`source` 指向不可变候选归档，`target` 是本补测独占的 Linux target，`worker` 是同候选实际 worker ELF。不得共享仍在使用的 G4 target。

```bash
python3 "$harness/build.py" \
  --source "$source" --candidate "$candidate" \
  --target "$target" --output "$new_build_directory" \
  --host-space-helper "$frozen_host_space_helper"
```

`build.py` 依次调用 [prepare.py](prepare.py)、离线 Cargo metadata、[verify-lock.py](verify-lock.py)、离线 locked build。每阶段单独保留 stdout、stderr、退出码、时间和 SHA；失败停止后续阶段，保留 `build.json`。超时仅终止本阶段独立进程组。首次外部 crate metadata 可调整其根包锁记录，候选 Cargo.lock 保持不变；任何 registry name/version/source/checksum 漂移均拒绝。不得下载新包。target 可为空，或由协调者按候选锁文件只复制 registry 依赖；不得复制整棵 target 或旧产品包。

```bash
python3 "$harness/run.py" \
  --binary "$built_harness_elf" --worker "$worker" \
  --crate "$new_build_directory/native-worker-chain-crate" \
  --candidate "$candidate" --output "$new_run_directory" \
  --host-space-helper "$frozen_host_space_helper"
```

可用 `--case NAME` 选择单项，重复名称在运行前拒绝。默认完整设计矩阵。构建入口不运行验证；运行入口不构建。执行前 source、worker 来源与候选 hash 的一致性由协调者冻结记录证明，不以 ELF 文件名代替来源证明。

`--host-space-helper` 指向协调者冻结的 `run-g4.py`，只加载其无参 `host_c_space()`，不调用 main。记录 helper SHA 和原始 stdout/stderr/命令/字节数；新 target 构建开始前要求宿主 C 至少 40 GiB，每阶段命令及每个运行用例前至少 10 GiB。未知值或低于门槛立即停止后续工作；不以 guest df 代替宿主可用空间。已由外层准备 registry target 时，外层也必须在复制前满足 40 GiB 门禁。

每项单独进程组、45 秒外部上限；正常路径关闭 RPC、stop/wait worker、释放服务端 socket 并 join 线程。异常/超时只终止本项创建的进程组，原始 stdout/stderr 和结果仍保存；有残留即失败。端口 blocked 计 ignored，不能计通过，整体退出非零。所有目录要求新建，保留旧运行原件，不删除数据库或日志。

## 证据与限制

`results.json` 保存两只 ELF 的前后 hash、候选、harness/证书 hash、各项命令、日志 hash、通过/失败/忽略计数与清理结果。`case.json` 保存动作计数、仅合成协议阶段、Store 前后权限和失败信息。断言失败时尽力保留清理后的权限快照，原错误不被快照失败覆盖。

入小型证据包可收录本目录源码/文档、build/results/case JSON、stdout/stderr、依赖 metadata、lock 和公开合成证书。不要打包 ELF、target、Store 数据库、tmp；只记录 ELF 原件路径及 SHA。不能只保留 CalledProcessError 而遗漏对应 stdout/stderr。

这是本地合成协议兼容性与应用路径验证；不证明真实 NAS 的错误码来源、ACL 配置/继承、服务器部分提交、服务端能力字段、FUSE 内核行为、GUI 或安装升级。原有函数投影→UNIX IPC 回归及 invalid-CA worker 握手与本组分列，不能冒充本组实际链。
