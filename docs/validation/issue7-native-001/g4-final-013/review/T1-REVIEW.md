# T1（精确写拒绝自动学习）独立审阅

被审提交：`fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4`（父 `9c28992`，产品树等于已冻结的
`9c9aaba`）。审阅者未修改产品源码，也未复用作者的 target/构建产物；所有产品测试在独立快照 +
独立空 target 中执行（见 REPORT §快照与 §G4 矩阵）。

## 1. 变更范围

`9c28992..fefb8ee` 共 18 个文件：产品代码 6 个（`sproto/error.rs`、`drive-runtime/worker.rs`、
`drive-runtime/upload_tests.rs`、`drive-core/native_upload.rs`、`drive-core/write_permissions.rs`、
`drive-runtime/bin/drive-native-setup.rs`）+ 文档 3 个 + 新证据目录 `write-learning/`。
没有 schema/IPC/协议版本变化；`bridge.rs` 未被修改（复用既有 `refuse_operation`/`targets_denied`）。
旧冻结证据（`server-static/`、`static/`、`l2-static/`、`server-capability/`、002–012 包）未被改动。

## 2. “非提交 + 唯一归属”论证复核

### 2.1 非提交（ACL middleware 在业务 Commit 之前拒绝）

对照 server-static 冻结原件逐点复核，**论证成立（限于该 middleware 路径）**：

1. `InitRouteMap` 把 `upload` 注册 `CreateACLTest`（`dispatch-boundary/function-20b460.asm` 0x210e0b–0x210e5a）。
2. `TestPhaseMiddleware::HandleAfterHeaderRead` → `TestPhaseSet::Test` → `ACLTest::Test`；失败调
   `HandleError` 并返回非零（`dispatch-boundary/function-29eb40.asm`、`function-29e7b0.asm`、
   `control-details/function-556600.asm`、`acl-boundary/function-222a20.asm`）。
3. `Router::RouteAfterHeaderRead` 在 middleware 返回 false 时直接返回；`SyncTask::RunFlowBody` 在
   0x2206d7–0x2206e1 转 `DiscardBody`，跳过 `ReadBody`/业务 `Handle`/Commit
   （`dispatch-boundary/function-555320.asm`、`flow/function-220610.asm`）。
4. 码产生点复核：`CreateACLTest::PermissionCheck`（0x223710）在 `0x223731` 读状态 `+0x341`
   （can_read），为 0 时走 `0x223770` 分支并在 `0x2237cf` 写 `-3`；`CreateACLTest::HandleError`
   （0x223140）在 `0x223166` 比较 `-3`、`0x22329b` 写 `0x9003`；`-4` 分支写 `0x9004`。
   与 T1 报告一致。

报告已显式披露的边界（我确认这些披露存在且恰当）：middleware 与 `PathConverterMiddleware` 的先后、
同一请求内解析/ACL 求值的 ABA 窗口、仅限 Drive Server 4.0.3-27892 静态证据。**这些边界不推翻
“该 middleware 路径的 0x9003 是提交前拒绝”，但限定其外推。**

### 2.2 唯一产生者（独立重扫，强于作者证据）

作者扫描（`server-static/permission-code-immediates.json`）只覆盖 `syncd` 的 3,593 个动态符号函数区间。
本轮用**独立工具**（capstone 线性扫描四个公开 ELF 的全部 PF_X PT_LOAD 段）重扫
（`environment/server-immediates-013.json`）：

- 复现作者 syncd 计数：`0x9003` 1 处、`0x9004` 7 处、`0xa003` 1 处、`0xa004` 4 处，共 13 处；
  `0x9003` 命中地址 `0x22329b` 正是 `CreateACLTest::HandleError`。
- 扩大到四个 ELF 后，`0x9003` 的**写立即数产生点仍只有 1 处**（syncd 的 0x22329b）；
  `libsynosyncserver.so` 的 `0x1bda99 sub esi,0x9003` 是错误码分派/比较消费者（已读上下文确认），
  不是产生点。
- `0x9004` 共 8 处立即数引用（syncd 7 + core 库 `FilterChecker::TestFileFilter 0x261879`），
  其中写立即数 6 处；与 T1“0x9004 同码异源、不能学习”的结论一致。
- 扫描边界同作者声明并更大：线性扫描仍不覆盖计算出的码、未解包组件与其它版本。

结论：在可获得的公开静态证据内，“`upload` 直接响应 0x9003 的唯一产生者是 Create ACL 前置测试”
成立，且比作者原扫描多覆盖了三个库（未发现反例）。

### 2.3 归属（node, write_content）

代码路径复核：

- 生产上传对已有远程身份节点使用 `id:<file_id>`（`native_upload.rs` / `sproto` identity 寻址），
  `PathConverterMiddleware` 在 view_id 缺省时把永久 ID 解析结果写入 request header；ACL 测试使用
  转换后的 `header.path`。因此请求身份与 ACL 测试对象在**正常路径**上是同一节点。
- `upload_targets(&job.node)`：已存在节点 → `[(node, WriteContent)]`；本地创建 → `[(parent, CreateFile)]`
  （`write_permissions.rs:473`）。`refuse_operation` 只接受请求 target 集合内的 `(node, action)`，
  不匹配即返回错误（fail-closed）。
- `mutation_permission_response(kind, node, code)`：仅 `kind=="upload"` 且
  `node.version != CREATE_VERSION` 才产出 `Response::PermissionDenied{node, WriteContent}`；创建（含
  mkdir）、replace/remove/rename、缺节点身份全部回 `MutationPermissionUncertain`。
- bridge upload 循环只在 `!reconcile` 时调用 `refuse_operation`；reconcile 模式不学习（保守）。
- 学习语义是“该节点的 upload 请求被提交前拒绝”，不是“can_write=false 已被证明”；报告 §6.4 已如实
  写明。方向是保守拒绝（最多让 FUSE 写提前 EACCES），不会把 unknown 当 allowed。

### 2.4 sproto 分类边界

- 分类唯一入口 `mutation_reply_error(action, error)`，正常路径仅 `transfer.rs:496` 的
  `upload_prepared` 与 protocol-research 的 `namespace_request(action,...)` 调用；
  `0x9003 && action=="upload"` → `MutationPermissionDenied`，0x9004/0xa003/0xa004/0x3002/0x3004 保持
  原 hint 集合，0x9005/0xa005/0x3008 等保持普通 `Server`，宽整数码保持 `Decode`（wire 矩阵实测见 §3）。
- 因为 mkdir 复用 `upload` wire action，`Client::mkdir_native` 在 0x9003 时也会在 sproto 层拿到 typed
  变体；worker 把 mkdir 显式映射回 hint（`mutation_permission_response("mkdir", ...)`）。这是**预期
  行为**，但要求任何非 worker 的 sproto 使用者不得把 typed 变体直接当成节点归属的确定拒绝；错误类型
  的文档注释已声明“the code alone does not name a node: the caller must bind the current request
  identity”，方向上正确。

## 3. 对抗测试（独立于作者测试）

1. **wire 边界矩阵（外部 crate，公开 `sproto::Client`，本机 127.0.0.1 合成 pstream）**：
   在 012 冻结 harness 基础上，除两处**预期行为变化**外保持全部期望不变，另加 4 个对抗用例
   （download 0x9003/0xa004、auth/connect 阶段 0x9003 必须仍是 generic `server`）。
   冻结期望运行：64/2；校正期望运行：70/0，且逐 case 比对证明**唯一变化**就是
   `upload_reply_9003`、`mkdir_reply_9003` 两例（从 hint 变为 typed definite），其余 64 例 observed
   与 012 完全一致。新增 4 例证明 typed 分类没有泄漏到 download 或握手阶段。
2. **bridge 学习/不学习**：重跑作者的隔离 IPC 夹具测试（`upload_tests` 3 个新用例 +
   2 个 worker 绑定用例 + 3 个 core 学习/失效/防误绑用例 + sproto 分类用例），在独立 target 中执行，
   计数单列于 REPORT §T1 定向。这些用例覆盖：学到 denied → journal pending → `targets_denied`
   阻止重发 → reset 后同一意图重发成功；0x9004 不写 ACL 行；传输丢失不学；revision/持久化/profile
   失效；`(node,action)` 不匹配时不写行、journal 不动。
3. **代码层反例审阅**（找不到测试入口的路径用静态反例分析）：任何 typed
   `MutationPermissionDenied` 只能来自 `mutation_permission_response` 或 sproto 分类；`Response::
   PermissionDenied` 另有下载 Read 路径（bridge download handler 校验 action==Read 且 node 相符），
   两条路径的 (node,action) 语义不重叠；namespace 路径的 PermissionDenied 分支仍是防御性（该 action
   不产生 typed 变体）。

## 4. 发现

未发现需要修改产品代码的缺陷。以下为观察/建议（均不阻塞）：

1. **冻结 012 native-loopback harness 期望过期（预期内的行为变化）**：作者未运行全矩阵，因此没有
   更新这份外部 harness；T4 必须记录并校正期望（本包已做，见 COVERAGE §口径差异 1）。打包/冻结时
   应把该 delta 写进验收说明，避免误读为回归。
2. **T1 REPORT 一处笔误**：§4.1 写“`0x9004` 从原 hint 集合移除但仍保留为
   `MutationPermissionUncertain`”，实际代码是**把 `0x9003` 移出 hint 集合**，`0x9004` 一直保留在
   hint 集合中（wire 矩阵与 `mutation_reply_error` 源码均如此）。仅文档错误，不影响行为；建议作者
   在冻结前更正（审阅者不改作者文件）。
3. **纵深防御建议（非缺陷）**：`mutation_permission_response` 不校验 `code == 0x9003`
   （typed 生产者的唯一性由 sproto 层保证，当前不可利用）；若以后新增 typed 生产者，建议在该函数或
   typed 构造点断言码值，避免语义漂移。
4. **未改变的行为**：学到的 `(node, write_content)=denied` 只影响本节点本动作的本地预检与重试门禁；不写
   L2、不推导父目录/兄弟、不改 mode、不改变下载/删除/改名路径。与契约文档（`CONTRACTS.md` C3）
   的本轮修订一致（已核对）。

## 5. 结论

T1 增量**通过审阅与独立测试**：证据链（非提交+唯一归属）、typed 分类边界、worker 绑定、
bridge 学习/失效/fail-closed 均成立；未发现放宽旧断言、把 unknown 当 allowed、学习歧义码或把
rename/creation 误归属的实现。残余边界（真实 NAS 0x9003 可达性、真实 worker upload IPC 链、FUSE
EACCES 端到端、其它版本/未解包组件）未实测，均在报告中如实标注，不进入本轮通过计数。

详细计数与失败/未实测清单见 [REPORT.md](../REPORT.md) 与 `evidence/g4-013/inventory-001.json`。
