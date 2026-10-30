# P0 持久性契约核验草案

以下是源码检查，不是断电验收承诺。引用构建基线 `.6`，需要独立判定器和故障注入补证。

| 边界 | 现有实现 | 可声明范围/待验 |
|---|---|---|
| write返回 | writable::write → draft_write → FileExt::write_at | 接收进本地写入路径，不代表断电持久或服务器确认 |
| fsync/flush | writable::sync → checkpoint_draft；draft sync_all，建立内容快照和操作记录 | 设计为本地持久边界；逐事务断电/EIO/ENOSPC尚未验证 |
| release/close | release调用sync并移除句柄 | 不能假定应用能够可靠收到所有close后的内核release错误；须单列应用行为 |
| 目录fsync | FUSE实现未覆盖fsyncdir | 未达到显式目录fsync契约，必须测试默认行为并在后续阶段处理 |
| 服务器已确认 | 原生确认及版本核对之后的状态 | 不将本地fsync、传输100%视为服务器已提交；未知提交不得盲重试 |

源码入口：[writable.rs](../crates/drive-fs/src/writable.rs)、[lib.rs](../crates/drive-fs/src/lib.rs)、[drafts.rs](../crates/drive-core/src/drafts.rs)。判定器需比较持久边界前后版本、草稿、日志与命名空间，不仅读取GUI状态。

WSL 合成只读挂载的目录 `os.fsync` 实测返回0，见[内核边界检查](validation/p0-fixed-001/launcher-check.json)。vendor fuser 的默认 fsyncdir 实现回复 ENOSYS，产品未覆盖它。该观察不证明新增目录、改名或删除已持久；仍须实现并验证可写目录契约。

## 已定位的事务边界，尚未注入故障

固定基线源码 `62be13c9f9f4a2457a3040829f478bf43b1821df`。下表是 P3 待建立的注入点清单，不计为已执行轮次；每点适用故障仍须至少30轮。进程崩溃与存储/电源模型分开：kill -9 不能证明断电持久性。

| 点 | 实际源码边界 | 独立判定要求 |
|---|---|---|
| D01 | drafts::begin_draft 创建新草稿、复制旧内容后，file.sync_all 前 | 此前已确认版本仍可读取，未确认草稿不得取代它 |
| D02 | begin_draft 草稿 sync_all 后、drafts 目录 sync_all 前 | 崩溃后的草稿存在性与内容单列，不能假设目录项已持久 |
| D03 | begin_draft 目录同步后、INSERT drafts 前后 | 正文与 token/版本关联一致；允许未引用完整孤儿，不允许记录指向半文件 |
| D04 | editing_file 持久 generation/dirty 更新后、write_at/set_len 前 | 失败不得误报已同步；旧的已确认版本和操作基准保持正确 |
| D05 | draft_write 返回后、checkpoint_draft 的 sync_all 前 | 未 fsync 的新写允许丢失或部分落盘，不能损坏此前已确认完整版本 |
| D06 | checkpoint_draft 的 draft sync_all 后、内容快照提交前 | 已持久草稿可恢复；不能上传更旧快照为最新修改 |
| D07 | write_blob_reader 临时内容写入/校验后、文件 sync_all 前 | 截断、坏哈希、ENOSPC/EIO 均不得发布新版本 |
| D08 | write_blob_reader 文件 sync_all 后、persist 改名之前 | 旧已确认 blob 保持可读；未引用临时内容不能伪装已提交 |
| D09 | persist 后、blobs 目录 sync_all 前后 | 对比正文存在性、完整哈希及后续 SQLite 引用；单测进程终止不足以覆盖此窗口 |
| D10 | publish_download_reader 的版本条件 UPDATE 与 tx.commit 前后 | 发布内容必须仍匹配文件身份/目标版本；保留旧版本或提交完整新版本 |
| D11 | save_local_reader_from_version/commit_local_blob 后、draft checkpoint generation 更新前 | 操作日志、快照和草稿代次可恢复，不能忽略仍有新修改的草稿 |
| D12 | mark_upload_started 的 inflight/upload_paths 事务提交后、网络提交前后 | 未知提交进入核对路径；不因重启自动重复创建或静默覆盖 |
| D13 | confirm_upload_identity 的本地确认事务提交前后 | 服务器身份/确认版本和本地快照一致，较新草稿不被旧确认清除 |
| D14 | receive/apply_next 的收件箱、事件应用及 committed_cursor 提交前后 | 原子推进游标、重复事件幂等；不能以光标已更新但目录尚旧的状态恢复 |

主入口见 [state.rs](../crates/drive-core/src/state.rs) 与 [drafts.rs](../crates/drive-core/src/drafts.rs)。目录操作事务、升级迁移/降级拒绝、数据库损坏恢复将在 P2/P3 补专用场景，不能用上述正文点替代。

独立校验器当前覆盖完整命名空间、内容、指定草稿/操作字段及 quick_check，尚不覆盖 generation/checkpoint、服务器身份或断电后的块持久顺序。准备注入前必须扩展预期 manifest 和采集器。`drafts/` 首次创建的父目录持久性也需要在独立断电模型中核查，当前静态检查不足以作安全承诺。
