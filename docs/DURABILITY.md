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
