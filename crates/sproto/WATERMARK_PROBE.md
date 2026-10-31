# 当前事件水位与选定根验证

`get_max_sync_id` 的请求形状已从官方 Windows 调用链恢复，并由官方 Linux 命名符号辅助核对。封套带 `view_id`、`root_node_id`、`root_node_path`、会话及服务器恢复身份；动作另外设置 `path`，值为同步任务的根路径。它使用普通请求命令，读取响应的 unsigned `max_sync_id`。这里不添加 `pull_event` 的 `node_id`、`sync_id` 或目录列表的字符串 `cursor`。

[watermark.rs](src/actions/watermark.rs)只增加显式调用接口。缺字段、错误类型及未绑定根被拒绝；数值零不被猜测为协议错误。收到水位不等于元数据快照已经应用，更不允许直接写 committed cursor。服务器版本、任务模式和子目录适用性需要实测。

当前子目录元数据通过现有 `list(parent)` 精确匹配叶名读取，同时核对 `file_id`、`node_id`、名称、目录类型和删除状态。它没有编造 `query_node` 报文。顶层 `/` 无父列表，当前方法安全拒绝；调用者应保留已有恢复路径。父目录不可读、根被替换、服务器恢复身份变化也不能通过放宽身份判断处理。

## 有界只读探针

[native_watermark_probe.rs](examples/native_watermark_probe.rs)使用独立连接，顺序执行：当前水位、当前选定根元数据、从该水位读取一批作用域事件、再次查询水位。各阶段上限90秒，最多一个连接；不会下载正文、改写NAS、安装快照或提交游标。输出仅耗时、计数、布尔和错误类别，禁止记录会话、路径、身份值或正文。

构建：`cargo build --offline --locked --all-features -p sproto --example native_watermark_probe`。启动时仅传一个非秘密JSON配置文件路径，密码从终端无回显输入，不从命令行或配置读取。测试环境应关闭core dump并限制本地文件权限。

配置包含 `host`、`tls_name`、`username`、`scope`（`view_id`、`node_id`、`path`、`file_id`）及可选 `trusted_ca_pem` 数组；这些连接及身份信息仍应只留本地。TLS主机名和证书链校验保持启用。配置不允许含 `password` 或 `session`。

当前单元回归覆盖上下文字段、游标/版本类型混淆、无效根和选定根的多字段身份变化。真实探针结果由项目发布验证记录维护；不能把单元测试当作真实NAS已兼容。即使一次探针成功，也没有证明持续写入时的原子快照、游标失效恢复或权限变更安全；生产接入仍需枚举及事件fence协调。
