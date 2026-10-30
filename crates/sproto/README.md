# sproto：本地审查分支

[身份删除](src/actions/namespace.rs)已接入正常同步核心，保留源版本条件，事件核对支持 watch 内父目录移动；路径删除保留供研究对照。独立改名身份语法被当前 NAS 拒绝；九场景、未读回执及静态依据见[报告](../../NAMESPACE_IDENTITY_REPORT.md)。

[目录身份请求](src/actions/namespace.rs)复用已有 upload 字段及严格身份路径校验；[命名空间实验屏障](src/actions/namespace_observer.rs)用于预检后竞态，普通请求不安装。该阶段未改正常删除／改名；当前身份删除见上方入口。Windows 拆解、真实 FUSE 和未读目录回执结果见[报告](../../DIRECTORY_IDENTITY_REPORT.md)。

[身份路径上传](src/actions/transfer.rs)根据 Windows／ELF 调用链去掉 `view_id` 后发送 `id:` 路径，已接入正常文件核心；新增 task-local [上传屏障](src/actions/upload_observer.rs)用于真实预检后竞态，普通请求不安装。六场景及服务器重建反例见[身份上传报告](../../UPLOAD_IDENTITY_REPORT.md)，此前静态研究见[根身份报告](../../ROOT_IDENTITY_REPORT.md)。

[分页研究观察器](src/actions/list_observer.rs)用于明确作用域内的真实页间修改实验；独立任务隔离、有界等待，默认调用不暂停。[七组真实结果](../../SNAPSHOT_PAGING_REPORT.md)覆盖分页错误和只能由事件边界发现的页内改名。
[上传实验](src/actions/transfer.rs)增加有界独立观察屏障：正文发送后不读回执，观察服务器提交后丢弃原连接。仅用于明确启用的协议实验；普通上传路径不变。真实创建／更新／原子保存结果见[未读回执报告](../../UPLOAD_WIRE_RECOVERY_REPORT.md)。
新增[正文计量与共享限速](src/flow.rs)，接入实际原生读写，进度完成与校验／服务器回执分开；元数据不受正文额度限制。真实并发下载、上传和后台更新见[传输报告](../../TRANSFER_REPORT.md)。

新增[额外信任校验](src/trust.rs)及[测试夹具入口](tests/README.md)：默认系统信任链，额外公钥 CA 不关闭主机名验证。[实验 TLS 中继](src/trust_relay.rs)只在 protocol-research 下使用；真实结果见[证书报告](../../TRUST_REPORT.md)。

原生 `list` 已按官方同步枚举参数改为 `merge_local=0`，并校验分页边界、总数、重复身份和零身份；真实 1,002 空文件测试及原始响应见[快照恢复报告](../../SNAPSHOT_RECOVERY_REPORT.md)。

新增事件提交会话指纹解析与 `Client::commit_fingerprint`，仅输出带域分隔 SHA-256，不输出认证令牌。Windows 静态依据及双会话真实核实见[目录恢复报告](../../DIRECTORY_RECOVERY_REPORT.md)；匿名事件保持可用，冲突作者字段拒绝解析。

新增 [namespace.rs](src/actions/namespace.rs)：目录创建、改名、删除和丢回执结果核对，真实 NAS 双会话测试已通过。具体边界见 [命名空间报告](../../NAMESPACE_REPORT.md)；尚未接入 FUSE 本地操作队列。

后续新增 [mutations.rs](src/actions/mutations.rs) 的上传结果核对、结构化 CursorReset 和 [丢回执验收](examples/native_recovery_roundtrip.rs)。实测范围与剩余写入语义见 [恢复报告](../../RECOVERY_REPORT.md)。

状态：真实 NAS 的认证、个人根、长轮询／事件、流式上传下载与后台刷新已做部分验证；不是完整同步 SDK。最新结果见 [真实联调报告](../../LIVE_REPORT.md)。

上游为 [m1guelpf/sproto](https://github.com/m1guelpf/sproto/tree/c683cc946a9b999fa0d7e18aabc7a5e657b5a8a6)，固定提交 `c683cc946a9b999fa0d7e18aabc7a5e657b5a8a6`，许可见 [LICENSE](LICENSE)。本包保留上游的包作者与许可信息。

本次改动：

- 下载写入同目录临时文件；完整接收后原子替换，空文件也会落盘。元数据响应和未恢复的下载 continuation 返回错误。
- `download_checked` 检查完整内容 MD4、身份、响应版本和长度，再原子发布；提供 SHA-256 给本地内容库。任意历史版本、BinaryEx、refer/dedup 尚未完整验证。
- 连接池在请求取消或失败时丢弃连接，仅完整成功的请求才可复用。
- 多个二进制段可在 codec 层依次读取；不将任意多段拼接冒充完整文件。
- 解码限制嵌套深度 32、每帧元数据节点 100000、字符串总量 16 MiB；拒绝重复键与不完整 BinaryEx。
- 列表 continuation 检查后续错误、元数据冲突及总预算；目录字段缺失时明确报错。`DirectoryListing` 区分未变化与空快照。
- 平台改为 `linux`，默认 TLS 验证，支持额外 DER CA 及连接地址／证书身份分离。连接、普通请求和长轮询使用独立超时。
- 认证前获取恢复标识，并补齐 agent/header，修复真实服务器返回的 `client version too low`。
- [native.rs](src/actions/native.rs) 提供真实个人根、长轮询与事件 API，包含脱敏样本重放。
- `protocol-research` 仍包含离线构造器，同时开放实验流式上传；这些写操作不代表生产发布关卡已通过。

目录 parser 当前采用严格字段要求。真实目录、文件、符号链接、删除标记的不同字段形态仍需录制样本校准，不能将拒绝样本默认为不存在或执行删除。

`download_to` 是底层流式接口，失败时调用者的 sink 可能已有部分数据，不能直接将其作为已发布文件。路径接口 `download` 使用临时文件保护旧内容，但两者都尚未校验群晖原生哈希。

设备 UUID 应由后续账户服务持久保存。2FA、会话续期、完整能力协商和钥匙环仍待实现。[native_roundtrip](examples/native_roundtrip.rs) 是会连接 NAS 并写入显式实验 run 的双会话测试，凭据只从无回显终端输入。

后续阅读：[源码](src)、[协议证据](../../evidence/README.md)。

[native_download_reuse.rs](examples/native_download_reuse.rs) 是 `protocol-research` 下的只读 NAS 探针，比较既有实验文件在同一／新会话以及两个 Windows 请求字段组合中的再次下载。结果均仍触发 32772，正式请求未切换到研究分支；见 [下载恢复报告](../../DOWNLOAD_RECOVERY_REPORT.md)。

该下载探针进一步增加官方 ProtocolBuilder 根身份／server_id.restore_id 上下文，真实结果仍交替返回 32772；正式下载未启用研究上下文。静态证据与五条件结果见 [GIO_METADATA_REPORT.md](../../GIO_METADATA_REPORT.md) 的下载协议补充。
