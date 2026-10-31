# 下载正文准备与短提交边界

本批将原生 Bridge 的正文复制、SHA-256 校验和正文/目录 `fsync` 移出共享 Store 锁。此前 `transfer_once` 在锁内完成这些工作，FUSE 元数据查询会随下载校验一起等待。后台并行度和网络协议不变。

[download_publication.rs](../crates/drive-core/src/download_publication.rs) 分三步实现：

1. `Store::prepare_download(ticket)` 在短 Store 边界核对身份、版本和本地修改，记录原 blob、本地版本与 dirty generation，并通过已有打开句柄保留表预留最终 UUID。此阶段不复制正文。
2. `DownloadPreparation::prepare` 在锁外有界读取、复制、校验，写入 `blobs/.drive-download-<12位随机字符>.part`。固定前缀保证该临时名称不是 UUID，现有 GC 只回收普通 UUID 文件，因而不会误删准备中的正文。正文 `fsync` 后安装同目录最终 UUID，再持久化该目录；最后返回不可伪造的 `PreparedDownload`。
3. `Store::publish_prepared_download` 再核对 Store 生命周期、节点权限、身份/版本、dirty draft 和原本地引用，然后事务性发布。只有数据库提交后，新正文才可见。完整旧版及打开旧版的句柄继续有效。

最终 UUID 在准备开始前已登记到原 `ReadHandle` 使用的保留表，因而并发 GC 即使在准备完成、数据库提交之前运行也不会删除它。失败或取消不删除旧正文；未提交的最终文件在保留对象释放后成为普通可回收孤儿。临时文件由 RAII 清理；进程异常退出留下的上述临时残片，在下一次获得 Store 独占 writer 锁后回收。启动清理只删除符合本模块命名规则的普通临时文件，不递归、不跟随符号链接，不触碰未知文件、草稿和待上传正文。

Bridge 在复制块之间检查真实下载需求以及原 RPC 是否被替换，并在提交前重新核对。一个读取者取消不会清除其他读取者、固定任务或旧缓存后台更新的需求。已收到完整 FD 后，仅网络断开不使正文失效；如果 RPC 没有被替换且节点仍匹配，可完成本地发布。暂停关闭网络后也可能完成这类已经收到正文的本地工作，不发起新的网络下载。

旧 `publish_download` / `publish_download_reader` 同步 API 保留，内部复用准备与提交逻辑；持有 Store 调用这些同步 API 的旧调用者仍会等待正文准备。本次只修改原生 Bridge 热路径，不宣称所有正文写入、上传快照或草稿检查点都已隔离。

验证包括 [core 并发与安全回归](../crates/drive-core/tests/download_publication.rs) 和 [真实 Bridge 路径回归](../crates/drive-runtime/src/download_publication_tests.rs)：

- 可控慢 Read 在半写入和 EOF/最终校验前阻塞；另一线程使用 `try_lock` 查询 Store，并运行 GC。
- 准备完成、未提交时再执行 GC；发布后验证旧打开句柄仍读取旧版。
- 新远端版本、同路径不同身份、本地 dirty blob、dirty draft、竞争本地发布和 Store 重开均拒绝迟到准备结果。
- 错误哈希、错误长度、读取失败、取消和 RPC 替换保留旧完整内容；取消后的孤儿可以回收。
- 重开清理遗留临时文件；竞争 writer 不执行清理，符号链接、嵌套目录、未知文件、dirty draft 和 pending 操作保留。

最终 WSL 独立源码/target 回归：core 289 通过、0 失败、1 项既有大规模夹具忽略；runtime lib 83 通过、0 失败、1 项需显式调用的 FUSE 内核取消测试忽略。新增 6 项 core 安全/并发测试与 2 项 runtime 路径/分类测试均通过。所有测试正常退出；使用 1 个编译 job，负载已登记。初轮缺少目录根的测试夹具和次轮暂存子目录影响旧夹具的失败记录均保留；最终采用原 blobs 目录下固定非 UUID 临时文件命名，旧夹具未改。上述同步条件属于确定性并发断言，不是性能基准；NAS、Mint 和最终安装包验收由整体交付门禁记录。

复现：使用独立 `CARGO_TARGET_DIR` 执行 `cargo test --offline --locked --all-features -p drive-core`，然后 `cargo test --offline --locked --all-features -p drive-runtime --lib`。
