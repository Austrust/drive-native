# 精确写拒绝自动学习：可证子集与实现增量

- 基线：`codex/fix-7` @ `9c28992`（产品冻结 `9c9aaba` 之后的文档提交）。本轮只追加本目录，不修改 002–012 冻结证据包、`G4-evidence-bundle-012-final` 或 `work/issue7-deb-001-artifacts`。
- 硬约束：不连接 NAS、不读取凭据、不 push/发布；所有结论只来自已归档静态证据与本机隔离测试；真实 NAS 响应、真实 ACL 配置、其他 Drive Server 版本均未实测。
- **结论（窄子集，不是“写拒绝学习已完成”）**：同时满足“服务端业务提交前拒绝”与“可唯一归属到 `(目标节点, action_class)`”的只有一项——**upload 直接响应 `0x9003`，且请求目标是已存在节点（非本地创建）**；只把它学习为 `(node, write_content) = denied, source=observed, revision=当前策略 revision`。其余 upload/remove/rename/mkdir/replace 的权限相关码全部保持原 `MutationPermissionUncertain` hint / `uncertain` / `cleanup_uncertain` / reconcile 路径，不写 `directory_permissions`、不改 pending、不升格 L2、不推导父目录或兄弟路径。

## 1. 缺口与目标

`REPORT.md`（M2）记录的停止线是：mutation 的直接权限响应只作 action-scoped 诊断，保持 uncertain/reconcile；“后续 M3 调查也未把这些码证明为可唯一归属的节点动作拒绝”。本增量只关闭这个缺口里**证据足够**的部分：

1. 先回答哪些 mutation 响应对应“业务提交前 ACL 拒绝”（非提交），以及在 wire 可见字段内能否对应到 `(节点, 动作)`；
2. 只对同时满足两者的子集实现自动学习 denied，其余保持现状；
3. fail-closed：证据不足不学习，不把 unknown 当 allowed，不把服务端 reason 文本或客户端错误标签当证据。

规格来源：`work/plan-5-2-20261104/PLAN-7.md` §3.4（attempt-and-learn）、§6（错误分类与恢复）；契约 `docs/CONTRACTS.md` C3（本轮同步修订）；交付门 `DELIVERY_GATES.md`。

## 2. 证据链

### 2.1 非提交：ACL middleware 在业务 Handle/Commit 之前拒绝

以下地址与原件见 [服务端静态证据](../server-static/REPORT.md) 及其 `dispatch-boundary/`、`acl-boundary/`、`flow/` 目录；本次只引用，不重新提取。

1. [InitRouteMap `0x210c0d–0x210c5e`、`0x210e0b–0x210e5a`、`0x210c98–0x210d74`](../server-static/dispatch-boundary/function-20b460.asm) 把 `remove` 注册 `RemoveACLTest`、`upload` 注册 `CreateACLTest`，`rename` 注册源 Remove 测试与目标 Create 测试。
2. [TestPhaseMiddleware::HandleAfterHeaderRead](../server-static/dispatch-boundary/function-29eb40.asm) 进入 [Test](../server-static/dispatch-boundary/function-29e7b0.asm)，按 action 取测试集合；[TestPhaseSet::Test](../server-static/control-details/function-556600.asm) 遇非零测试结果返回 false。
3. [ACLTest::Test `0x222a20`](../server-static/acl-boundary/function-222a20.asm) 依次调用 `PrepareInfo`（vtable +0x20）、`GetUserCapability`（+0x38）、`PermissionCheck`（+0x28）；任一失败即调 `HandleError`（+0x40）并返回 -1。`HandleCheckFailure`（+0x30）只做诊断日志，不写响应错误。
4. [Router::RouteAfterHeaderRead](../server-static/dispatch-boundary/function-555320.asm) 在 middleware 返回 false 时直接返回；[SyncTask::RunFlowBody `0x2206d7–0x2206e1`](../server-static/flow/function-220610.asm) 收到 false 后转 `DiscardBody`，随后写响应，**跳过 ReadBody、业务 `Handle` 与 Commit**。

因此，**由这条 ACL middleware 路径产生的拒绝是本请求业务提交前的拒绝**。该结论只覆盖这条 middleware 路径及其已列 action，不等价于“任意来源、任意版本、任意复合操作收到相同数字码都未提交”。

### 2.2 码的产生点与能力位

能力结构布局由 core 库 [Capability::ToPObject `0x256ea0`](../server-static/core-capability/function-256ea0.asm) 逐字段确认（`+0 can_preview`、`+1 can_read`、`+2 can_write`、`+3 can_delete`、`+4 can_comment`、`+5 can_share`、`+6 can_encrypt`、`+7 can_rename`、`+8 can_download`、`+9 can_sync`、`+10 can_lock`、`+11 can_auto_lock`）。`ACLTest::GetUserCapability`（[`0x223df0`](../server-static/acl-boundary/function-223df0.asm)）把 `NodeCapability::Get` 结果拷到状态 `+0x340` 起，故 `+0x341/+0x342/+0x343` 分别为 `can_read/can_write/can_delete`。

| 测试 | PermissionCheck 原件 | 失败条件 → 错误码 | HandleError 映射 |
|---|---|---|---|
| Create（upload，含 mkdir/replace 上传阶段） | [`CreateACLTest::PermissionCheck 0x223710`](../server-static/acl-boundary/function-223710.asm)：`0x223731` 读 `+0x341`，`0x22373a` 读 `+0x342` | `can_read=0` → 写 `-3`（`0x2237cf`）；`can_read=1 && can_write=0` → 写 `-4`（`0x22390a`） | [`0x223140`](../server-static/acl-boundary/function-223140.asm)：`-3 → 0x9003`，`-4 → 0x9004`，其余 → `0x7000` |
| Remove（remove，含 rename 源测试） | [`RemoveACLTest::PermissionCheck 0x222c50`](../server-static/acl-boundary/function-222c50.asm)：`0x222c70` 读 `+0x343` | `can_delete=0` → 写 `-5`（`0x222cfd`） | [`0x2232f0`](../server-static/acl-boundary/function-2232f0.asm)：`-5 → 0xa004`，`-3 → 0xa003`，其余 → `0x7000` |

**`0xa003` 没有已证生产者**：全 bundle 中只有 `CreateACLTest::PermissionCheck 0x2237cf` 写 `-3`；`RemoveACLTest::PermissionCheck` 只写 `-5`；`DefaultACLTest::PrepareInfo` 与 `ACLTest::GetUserCapability` 失败写 `-1`（`0x2268e2`/`0x226b96`、`0x22412f`）。`RemoveACLTest::HandleError` 的 `-3 → 0xa003` 分支是防御映射，不能据此把 `0xa003` 归到删除目标。

[立即数扫描 `permission-code-immediates.json`](../server-static/permission-code-immediates.json) 覆盖 **3,593** 个已定义动态函数区间，共 **13** 处四码立即数引用；其中 `0x9003` 只出现 1 处（CreateACLTest::HandleError `0x22329b`），`0xa003` 只出现 1 处（RemoveACLTest::HandleError `0x2234a3`）。`0x9004` 另有 `BackgroundTaskUtil::CopyHandler::CheckPermission`（2）、`UpdateFileInfoHandler::Handle`、`UploadHandler::TestFilter`（2）等产生点；`0xa004` 另有 `RemoveBackgroundTask::PermanentDelete`、`BackgroundTaskUtil::RemoveHandler`、`RemoveHandler::TestFilter` 等产生点。扫描本身声明不覆盖计算出的码和未导出代码，因此“唯一”只在这个范围内成立。

### 2.3 同码歧义：0x9004 / 0xa004 不能学习

- core 库 [`FilterChecker::TestFileFilter 0x2607f0`](../server-static/filter-source/function-2607f0.asm) 在 `0x261879–0x2618af` 把 **profile 只读路径过滤**失败设为 `0x9004`。
- [`UploadHandler::TestFilter 0x4b7da0`](../server-static/filter-boundary/function-4b7da0.asm) 把 FilterChecker 的 `0x9004` 原样保留（其余远端过滤结果 → `0x9005`）。
- [`RemoveHandler::TestFilter 0x41f2b0`](../server-static/filter-boundary/function-41f2b0.asm) 把 FilterChecker 的 `0x9004` 转成 `0xa004`（其余 → `0xa005`）。
- 这些 TestFilter 分支在 upload/remove 的 Commit 之前，但证据只证明“所列过滤分支早于对应 Commit 调用”，未对全部副作用作非提交证明；且 ACL 拒绝与 profile 过滤均写 `error.action=0`（`Response::SetError` 第二参数为 0），wire 上没有字段可区分二者。

所以 `upload 0x9004`、`remove 0xa004` 既可能是 ACL 拒绝也可能是 profile 只读过滤，**不能归属为某节点的 ACL deny**；生产代码也不允许按 `error.action` 或 reason 字符串推断。

### 2.4 唯一归属：请求身份与 ACL 测试对象

- 生产上传是**身份寻址**：[`native_upload.rs`](../../../../crates/drive-core/src/native_upload.rs) 的 `upload_prepared` 对有远程身份的节点使用 `id:<file_id>`，对本地创建使用 `id:<parent_file>/<leaf>`；[`sproto/actions/transfer.rs`](../../../../crates/sproto/src/actions/transfer.rs) 在 `id:` 路径时**不带 `view_id`**（同时带两者会被 4.0.3 拒绝为 `0x7000`）。
- 服务端身份边界见 [能力与身份补充证据](../server-capability/REPORT.md) §请求与响应的身份边界：`view_id` 缺省时 [PathConverterMiddleware `0x287e20`](../server-capability/raw/identity/function-287e20.asm) 才走 `PathConvertHelper::Convert`（[`0x24ca00`](../server-capability/raw/identity/function-24ca00.asm)、[ProcessID `0x24bc60`](../server-capability/raw/identity/function-24bc60.asm)），经 [QueryNodeByPermanentId `0x24ab80`](../server-capability/raw/identity/function-24ab80.asm) 按永久 ID 找到节点，并把 `view_id/node_id/sync_id/file_id/path` 写入 request header（[Convert `0x24cdc0`](../server-capability/raw/identity/function-24cdc0.asm)）。
- ACL middleware 的 [`DefaultACLTest::GetPath 0x222a90`](../server-static/control-details/function-222a90.asm) 读 `header.path`；[DefaultACLTest::PrepareInfo `0x226670`](../server-static/acl-boundary/function-226670.asm) 给 Node 设置该路径；[GetUserCapability `0x223df0`](../server-static/acl-boundary/function-223df0.asm) 对该 Node 构造能力查询；core 库 [LoadAclByPath `0x2f5330`](../server-static/acl-path/function-2f5330.asm)、[DriveAcl::GetCapability `0x25b7a0`](../server-static/acl-path/function-25b7a0.asm) 按父链汇总有效能力。

因此对**已存在节点的 upload**，请求 header 的身份解析与 ACL 测试落在同一个永久节点上，`0x9003`（`can_read=0`）是该节点 ACL Create 测试的提交前拒绝，可归属为 `(node, write_content)`。对**本地创建的 upload**（`id:<parent>/<leaf>`），ACL 测试针对尚不存在的子路径，证据明确“不是独立的 CreateFile/CreateDir primitive”，不能改写成父目录的 CreateFile；**不学习**。mkdir 使用同一条 `upload` action（`is_dir=true`，[`namespace.rs mkdir_at_identity`](../../../../crates/sproto/src/actions/namespace.rs)），同属创建语义，**不学习**。

本次子集仍有的残余边界（写入 §6 未实测项）：ACL 测试用的是转换后的 `header.path`，不是响应身份字段；同一请求内“解析→ACL 求值”的并发改名 ABA 窗口未实测；结论仅限 Drive Server 4.0.3-27892 静态证据。

## 3. 判定表：学习 / 不学习

| action / wire code（直接响应） | 业务提交前拒绝已证 | 唯一归属 (node, action) | 本轮处理 |
|---|---|---|---|
| upload `0x9003`，目标为已存在节点 | 是（2.1/2.2） | 是（2.4） | **学习** `denied(write_content)`，source=observed，revision=当前策略 revision |
| upload `0x9003`，目标为本地创建（含 mkdir） | 是 | 否（子路径 capability 非 CreateFile/CreateDir primitive） | hint/uncertain，不学习 |
| upload `0x9004` | 是（ACL 路径）/ 不等同（过滤路径） | 否（2.3 同码） | hint/uncertain，不学习 |
| remove `0xa003` | 是（若真由 HandleError 产生） | 否（无已证 `-3` 生产者） | hint/uncertain，不学习 |
| remove `0xa004` | 是（ACL 路径）/ 不等同（过滤路径） | 否（2.3 同码） | hint/uncertain，不学习 |
| rename `0x9003`/`0x9004`/`0xa003`/`0xa004` | ACL middleware 可能（源 Remove 测试 + 目标 Create 测试） | 否：响应不标识哪端/哪个测试；客户端 `RenameTarget` 是目的父目录，服务端目标测试是目的路径能力 | 保持既有 generic uncertain/hint，不学习 |
| replace 上传阶段 `0x9003`/`0x9004` | 上传阶段本身先于 replace 清理 | 否：复合语义、客户端目标集合不含“目标节点 write_content” | hint/uncertain，不学习 |
| mutation `0x3002`/`0x3004`（upload/remove/rename） | 未证（handler 内通用转换，可能在业务阶段之后） | 否 | 既有 hint，不变 |
| download `0x3002`/`0x3008` | 既有独立路径（`recv_download` 边界） | 是（当前读请求） | 既有行为不变 |
| `0x9005`/`0xa005` 选择性同步 | 不适用 | 否（非 ACL） | 既有排除，不学习 |

## 4. 实现（最小、可审计）

1. **wire 语义边界**：`crates/sproto/src/error.rs` 新增 typed 变体 `Error::MutationPermissionDenied { code }`，仅由 `mutation_reply_error` 在 **action=`upload` 且 code=`0x9003`** 时产生；**`0x9003` 从原 hint 集合移出并改由该 typed 变体接收，`0x9004` 仍留在 `MutationPermissionUncertain` hint 集合**（其同码歧义未变）；`0x3002/0x3004`、`remove 0xa003/0xa004` 的 hint 集合不变。分类只放在“直接响应 + 已知 action”的边界，并带注释引用 `server-static/REPORT.md`。
2. **请求身份绑定边界**：`crates/drive-runtime/src/worker.rs` 新增纯函数 `mutation_permission_response(kind, node, code)`：
   - `kind=="upload"` 且 `node.version != creations::CREATE_VERSION` → `Response::PermissionDenied { node: node.id, action: WriteContent }`；
   - 创建（`local-create-v1`）、`kind` 为 mkdir/replace/remove/rename、或缺少请求节点 → `MutationPermissionUncertain { code }`（保留现有 hint，不丢诊断）。
   `Command::Upload` 的 typed 错误走该函数；Mkdir/Replace 等其它 mutation 显式映射回 hint；Rename/Remove 的对应分支为防御性（该 action 不产生 typed 变体）。按 T4 独立审阅建议，在该分支加 `debug_assert_eq!(code, 0x9003)` 作纵深防御（仅断言、不改行为；正式保证仍由 sproto 分类边界给出）。
3. **既有学习机制原样复用**：`bridge.rs` upload 循环中已存在的 `Response::PermissionDenied{node,action}` 分支（`crates/drive-runtime/src/bridge.rs:1445`）调用 `write_permissions::refuse_operation`。该函数已实现——`record_db`（`crates/drive-core/src/write_permissions.rs:313`）写 `directory_permissions(state=denied, source=observed, revision=(SELECT revision FROM write_policy))` 并把 `write_policy.revision+1`；journal 从 `inflight/cleanup_inflight` 回 `pending`；history 记 `failed/permission_denied`；随后的调度由 `upload_pending` 的 `targets_denied` 门禁（`bridge.rs:1293`）阻止重发。`profile_changed` → `invalidate_remote_permissions`（`write_permissions.rs:338`，`bridge.rs:567` 调用）、重连 `attach` → `reset_session`（`write_permissions.rs:360`，`bridge.rs:145` 调用）、重新核对（`bridge.rs:1209`）的既有 revision 失效机制不变；成功后 `observe_targets(true)`（`bridge.rs:1429/1441`）按既有规则以 observed/allowed 覆盖。FUSE 写入口以 `WriteContent` 判定（`crates/drive-fs/src/writable.rs:105/117/136`），学到的 denied 会在下一次写尝试前返回 EACCES；该内核路径本轮未做端到端实测。
4. **错误传播完整性**：`native_upload.rs` 同时透传 `MutationPermissionUncertain` 与 `MutationPermissionDenied`（否则 typed 错误会退化为 generic `Uncertain` 并丢失 hint）；`drive-native-setup.rs` 的穷尽 `category()` 把新变体归入 `"server_rejected"`。
5. **不变项**：schema 仍为 25，IPC 仍为 21，无新表/列/迁移；不新增 L2 映射；`wire::Response` 版本与序列化不变（复用既有 `PermissionDenied`/`MutationPermissionUncertain`）；下载、读、删除、改名、复合 replace 的既有路径与门禁未改。`SharedStore::has_permission_denials` / `permission_clear_lease` 的既有语义沿用：新增的写 `denied` 行与读拒绝一样使缓存回复走保守投影，无新机制。

## 5. 测试

测试全部为隔离夹具/纯函数，不连接 NAS。环境：WSL Ubuntu-24.04，`source work/fsearch-perf-runs/env.sh`（cargo/rustc 1.99.1、nproc 12），独立 target `/home/drive-native-test/.cache/p032-writelearn-target`，`CARGO_INCREMENTAL=0`、`-j2`、`--locked --offline`。开跑前宿主 C: 可用 **80,585,555,968 bytes**（满足项目 40GiB 门禁），完成全部测试后 target 占用 **2.7G**（上限 8GiB），宿主 C: 可用 91,616,641,024 bytes。原始 stdout/stderr 与退出码保存在本目录 `check-001.log`、`test-001..006.log`，以及 T4 审阅后 assert-only 改动的复跑日志 `check-002.log`、`test-007*.log`、`test-008.log`、`test-009.log`；`test-005/006` 含测试输出的 NUL 字节，按原始字节保留（本目录 `.gitattributes: *.log -text`）。

```bash
source work/fsearch-perf-runs/env.sh
export CARGO_TARGET_DIR=/home/drive-native-test/.cache/p032-writelearn-target
export CARGO_INCREMENTAL=0
# 1) 全 target 编译（含 example/bin 的穷尽匹配）
cargo check --workspace --all-targets --all-features --locked --offline -j2
# 2) 定向测试
cargo test -p sproto --lib error::tests --locked --offline -j2
cargo test -p drive-core --lib write_permissions::tests --locked --offline -j2
cargo test -p drive-runtime --lib mutation_permission_tests --locked --offline -j2
cargo test -p drive-runtime --lib upload_tests --locked --offline -j2
# 3) 三个 crate 的 lib 回归（default 与 all-features）
cargo test -p sproto -p drive-core -p drive-runtime --lib --locked --offline -j2
cargo test -p sproto -p drive-core -p drive-runtime --lib --all-features --locked --offline -j2
```

本轮实际结果（通过/失败/忽略）：

| 运行 | 命令 | 结果 | 日志 |
|---|---|---|---|
| 全 target 编译 | `cargo check --workspace --all-targets --all-features --locked --offline -j2` | **exit 0**（增量 10.02s；全 target/all-features 通过） | `check-001.log` |
| sproto 分类 | `cargo test -p sproto --lib error::tests` | **8 passed / 0 failed / 0 ignored**（154 filtered） | `test-001.log` |
| drive-core 权限 | `cargo test -p drive-core --lib write_permissions::tests` | **26 passed / 0 failed / 0 ignored**（178 filtered） | `test-002.log` |
| drive-runtime worker 绑定 | `cargo test -p drive-runtime --lib mutation_permission_tests` | **2 passed / 0 failed / 0 ignored**（156 filtered） | `test-003.log` |
| drive-runtime upload 桥 | `cargo test -p drive-runtime --lib upload_tests` | **19 passed / 0 failed / 0 ignored**（139 filtered） | `test-004.log` |
| 三 crate lib 回归（default） | `cargo test -p sproto -p drive-core -p drive-runtime --lib` | **723 passed / 0 failed / 2 ignored**（drive-core 404+1、drive-runtime 157+1、sproto 162+0；71.35s+16.75s+0.52s） | `test-005.log` |
| 三 crate lib 回归（all-features） | 同上 `--all-features` | **733 passed / 0 failed / 2 ignored**（drive-core 408+1、drive-runtime 157+1、sproto 168+0） | `test-006.log` |

过程偏差：首轮 `cargo check` 因新增测试代码访问 `drive_core::Store` 私有字段 `db` 产生 3 处 E0616（仅作者测试代码，产品源码无编译错误）；修复后同一 `check-001.log` 路径被重跑覆盖，**首轮失败原件未保留**，此处如实记录，不伪造成新增证据。产品 `--all-targets --all-features` 最终 exit 0。

T4 审阅后按建议仅增加 `debug_assert_eq!(code, 0x9003)`（assert-only、不改行为），随后复跑（同一独立 target）：

| 运行 | 结果 | 日志 |
|---|---|---|
| `cargo check --workspace --all-targets --all-features` | **exit 0**（24.94s） | `check-002.log` |
| `cargo test -p sproto --lib error::tests` | **8 passed / 0 failed / 0 ignored**（154 filtered） | `test-007a.log` |
| `cargo test -p drive-core --lib write_permissions::tests` | **26 passed / 0 failed / 0 ignored**（178 filtered） | `test-007c.log` |
| `cargo test -p drive-runtime --lib mutation_permission_tests` | **2 passed / 0 failed / 0 ignored**（156 filtered） | `test-008.log` |
| `cargo test -p drive-runtime --lib upload_tests` | **19 passed / 0 failed / 0 ignored**（139 filtered） | `test-009.log` |

过程偏差（复跑）：`test-007.log` 是把两条 `cargo test ... --lib` 错误合并为一个调用产生的 **CLI 用法错误**（`--lib` 不能重复，exit 1），不是产品失败，原件保留；`test-007b.log` 的原始输出被终端进度覆盖丢了汇总行（exit 0、可见 15 个 OK），因此用 `--quiet` 的 `test-007c.log` 作为 clean count。五个复跑均 0 失败。


覆盖矩阵（新增/扩展）：

| 场景 | 测试 | 期望 |
|---|---|---|
| 正例分类（仅 upload/0x9003 typed） | `sproto` `error::tests::only_the_upload_create_acl_code_is_a_definite_mutation_refusal` | 0x9003 upload → `MutationPermissionDenied`；0x9004/0x9005/0xa003/0xa004/rename/其它 action 均不成为既有拒绝 |
| hint 边界不回归 | `sproto` `error::tests::mutation_permission_hints_are_action_scoped_and_never_definite_refusals` | 原 hint 集合保持；绝不出现 `MutationPermissionDenied` |
| 身份绑定正例 | `drive-runtime` `worker::mutation_permission_tests::only_an_existing_node_upload_binds_the_definite_refusal` | 已存在节点 + upload → `PermissionDenied(node, write_content)` |
| 0x9004 过滤歧义/创建/其它 mutation 不学习 | `worker::mutation_permission_tests::creation_and_every_other_mutation_keep_the_uncertain_hint` | 创建、mkdir/replace/remove/rename、缺身份 → hint |
| 正例端到端（桥） | `upload_tests::learned_upload_denial_gates_the_queue_and_recheck_releases_the_same_intent` | 学 `denied(observed)`、revision=当前、journal pending、门禁阻止重发、reset 后同一意图重发成功、成功后 observe allowed |
| 0x9004 歧义端到端（桥） | `upload_tests::ambiguous_upload_permission_hint_never_learns_a_denial` | uncertain + `directory_permissions` 0 行 |
| 取消/传输错误边界 | `upload_tests::transport_loss_during_upload_never_learns_an_acl_denial` | Err、uncertain、0 ACL 行、内容保留 |
| revision / 持久化 / 失效 | `write_permissions` `learned_upload_denial_uses_current_revision_persists_and_expires` | revision+1、DB 重开仍 denied、profile 失效回 unknown 且 revision+1 |
| 绑定校验 fail-closed | `write_permissions` `definite_refusal_must_match_an_authorized_target` | 不匹配目标 → Err、0 行、journal 不动 |
| rename/remove hint 不写行 | `write_permissions` `rename_and_remove_permission_hints_never_write_acl_rows` | 0 行；uncertain 状态保持 |

既有断言未删除、未放宽；`mutation_permission_hints_are_action_scoped...` 只是把 `upload/0x9003` 从 hint 表移出并由新增测试以更强断言覆盖。

## 6. 未实测 / 未闭合项

1. **真实 NAS 未实测**：本轮没有任何真实 `0x9003` 响应、真实 ACL 配置、profile 切换或上传拒绝样本；该码在真实环境是否可达、可达时的节点归属均未观测。静态证据只到 Drive Server 4.0.3-27892。
2. **服务端扫描非全域**：立即数扫描只覆盖 3,593 个已定义动态函数区间，不覆盖计算出的码、未导出/剥离代码和未知版本；“0x9003 唯一产生者”只在这个范围内成立。
3. **身份边界残余**：ACL 测试使用转换后的 `header.path`（不是响应身份字段）；同一请求内解析与 ACL 求值之间的并发改名 ABA 窗口未实测；`id:`、view_id 缺省与 `PathConverterMiddleware` 的中间件顺序未单独证明（本轮依赖生产上传已有真实行为与静态转换路径）。
4. **可学习语义的边界**：学习的是“该 upload 请求在此 ACL 下被提交前拒绝”的 `write_content` deny，而不是“`can_write=false` 已被证明”；它只影响本节点本动作的本地预检与重试门禁，不修改 mode、不推导父目录/兄弟路径、不升格 L2。
5. **未做真实内核/GUI 端到端**：T4 的 G4-013 已独立完成全矩阵/补测（见 §8），但 FUSE 写路径“学到 deny → EACCES”的端到端、真实 worker upload IPC 链（0x9003 穿过真实 worker 到 Bridge）仍未实测；deb 构建/安装/升级由 T5，且本增量不单独归因于任何 G4 分组。
6. **rename/replace/mkdir/remove 仍未学习**：见 §3；`0xa003` 的可达性与 `0x9004`/`0xa004` 的过滤歧义需要服务端运行时证据或响应字段证据才可能升级。
7. **产品暴露边界（T3 发现，Lead 转达）**：当前 mutation history 的权限响应白名单不含 rename 的 `0x9xxx`/`0xa004`（只接受 rename 的 `0x3002/0x3004`）。因此真实 rename 权限拒绝码当前不会落 history；本增量本就不学习 rename，故不影响本子集，也不在本轮放宽该白名单。
8. 未修改 `static/REPORT.md` 的旧结论：该文明确“mutation 可保留 action 限定数字码为诊断，但继续保持 uncertain/cleanup_uncertain 与 reconcile”；本增量以服务端 bundle-002 的非提交证明为新增依据，旧证据不回溯改写。

## 7. 复现

- 静态证据复核：见 `../server-static/COMMANDS.md`、`../server-capability/COMMANDS.md`；本目录不重新下载/解包。
- 代码锚点（当前分支）：`crates/sproto/src/error.rs`（`MutationPermissionDenied` / `mutation_reply_error`）、`crates/drive-runtime/src/worker.rs`（`mutation_permission_response` 及 Upload/Mkdir/Replace/Rename/Remove 分支）、`crates/drive-core/src/native_upload.rs`（typed 透传）、`crates/drive-core/src/write_permissions.rs`（`refuse_operation`/`record_db`/失效）、`crates/drive-runtime/src/bridge.rs`（upload `PermissionDenied` 分支、`targets_denied` 门禁）。
- 测试命令/计数/日志：见 §5；原始日志在本目录 `check-001.log`、`test-001..006.log`、`check-002.log`、`test-007*.log`、`test-008.log`、`test-009.log`，target 与宿主空间记录见 §5 开头。
- 运行环境与门禁：WSL Ubuntu-24.04 + `work/fsearch-perf-runs/env.sh`（cargo/rustc 1.99.1），独立 target `/home/drive-native-test/.cache/p032-writelearn-target`（完成后 2.7G，上限 8GiB）；开跑前宿主 C: 可用 80,585,555,968 bytes（≥40GiB 门禁）。
- 本增量只提交到本地 `codex/fix-7`，不 push；真实 NAS 端到端由 T3/T7 用最终产品执行，不属本报告结论。

## 8. 独立验证（T4 / G4-013）

- 候选：`fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4`（T1 提交）。独立证据包 `work/issue7-validation/G4-evidence-bundle-013-final/`（仓库外），已逐字节复制为 [g4-final-013](../g4-final-013/REPORT.md)，复制核验见 [G4-013 复制核验](../g4-final-013-COPY-VERIFY.md)。权威哈希：`MANIFEST.json` SHA256 `1db81a18f19dbabac8ade2b3168d748ceaa133019fe18a8edb6733bfbb9fc4a2`（300 文件），`SHA256SUMS` SHA256 `5dd00f356b18195f7790df6581d58ac8eb9c2df5d0fe305c088efb92c62a43e4`（299 项）。
- 主矩阵：workspace all-features **985/0/42**、default **975/0/37**（相对 G4-012 的 976/0/42、966/0/37 各 **+9**，与新增测试数一致）。FUSE debug/release 38/36、attr TTL 38、GUI 148 + labels 3 + Nemo/Nautilus 各 10、210k 容量 1/0/0、真实 worker 链 10/0/0、组件替换 8/8、文档前置回归 2/0/0；T1 定向测试单独计 16 项测试执行全通过。
- **012/013 loopback delta（冻结保留 + 校正说明）**：冻结的 012 native-loopback harness 期望把 `upload 0x9003`、`mkdir 0x9003` 判为 `mutation_permission_uncertain`，在 T1 后实际为 typed `MutationPermissionDenied`，故该冻结期望运行 **64/2**（原件保留）；校正 harness 仅改这两处期望 + 增加 4 个对抗用例，运行 **70/0**。逐例比对只有这 2 例 observed/expected 变化，其余 **64 例 observed 与 012 完全一致**；新增 4 例（download 0x9003、download 0xa004、auth 0x9003、connect 0x9003）全部保持 generic `server`，typed 拒绝未泄漏到下载/认证/连接分类。012 旧期望与旧原件不修改，013 用校正期望说明差异。
- T4 对 T1 的独立审阅：**通过**——非提交与唯一归属成立；独立 rescan 复现 `0x9003` 写立即数产生点仍唯一（`0x22329b`），`0x9004` 扩到 4 个 ELF 后共 8 处立即数引用（写 6 处），支持“同码异源不学”；代码边界、fail-closed、不学歧义码成立；并给出本报告 §4.1 笔误与 assert-only 建议（均已处理）。全文见 `g4-final-013/review/T1-REVIEW.md`。
- T4 对 T2 的审阅：结论成立（无可安全接线 L2），但指出 T2 报告/证据不在 `fefb8ee` 提交内；本最终提交已纳入 `l2-source-002/` 报告与仓库外证据包的路径/SHA 记录（见 [l2-source-002/README](../l2-source-002/README.md)）。
- 边界：G4-013 不包含真实 NAS、新 deb 构建/安装、FUSE 写路径 EACCES 端到端；这些保持在 T7/T5 与后续独立步骤。最终打包以 Lead 确认的提交与版本为准，本报告不把 T4 通过计为真实 NAS 或 release 验收。
