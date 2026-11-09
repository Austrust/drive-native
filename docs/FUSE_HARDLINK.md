# 硬链接限制

2026-11-09，修复分支`fix/18-link-enotsup`（issue [#18](https://github.com/Austrust/drive-native/issues/18)）。本页记录挂载点内 `link(2)` 的返回码与设计边界；不改变 schema、上传、权限或重命名契约。

## 结论

挂载点不支持硬链接。`link(2)` / `linkat(2)`（`ln`，以及「写临时文件再 link 到目标名」的原子落盘工具）统一返回 `EOPNOTSUPP`（Linux 上与 `ENOTSUP` 是同一个 errno，值 95），不再返回 `EPERM`。调用方据此把「文件系统不支持硬链接」与「权限不足」区分开，并按标准语义降级为 `rename`/复制。

实现位于 `crates/drive-fs/src/lib.rs`：`CacheFilesystem` 覆盖 vendored fuser 的默认 `link`（上游默认返回 `EPERM`，见 `vendor/fuser/CHANGELOG.md`），改为返回 `HARD_LINK_UNSUPPORTED = libc::EOPNOTSUPP`。请求在 handler 内直接拒绝，不进入本地命名空间、写队列或上传队列。

## 与 Synology Drive 远端模型的差异

| 维度 | POSIX 硬链接语义 | Synology Drive 远端模型 |
| --- | --- | --- |
| 节点身份 | 一个 inode 可有多个名字，`nlink` 计数 | 单一路径标识一个节点，无 link 计数 |
| 改名/删除 | 任一名字操作后对象仍被其它名字引用 | 按唯一路径判定；删除即移除该对象 |
| 上传/冲突 | 按对象身份去重 | 按路径与版本同步；复制出的两个名字会分叉 |
| 官方客户端 | `link(2)`（若文件系统支持） | 未发现硬链接 API/操作入口 |

因此挂载点无法把远端节点映射成「多名字单对象」；支持硬链接不是实现工作量问题，而是远端身份模型缺失。

## 设计取舍

- 服务端硬链接：未发现 Synology Drive API/官方客户端暴露，且需要远端 link 身份与计数，当前不可行。
- 本地硬链接 + 两个独立远端文件：本地看似成功，但改名、删除、冲突合并与上传确认语义会崩坏（两个名字内容分叉），不采用。
- 同挂载内 local-only link + 写时复制：技术可行，但需产品确认第二个名字是否上传、上传哪个名字、本机独占如何声明、跨设备与重启行为；本次不实现。
- 服务端 copy：语义是复制而非硬链接，不能冒充 `link(2)` 成功。

## 边界

- 同一挂载内的跨目录 link（源与目标在同一次 FUSE 挂载下）：由本 handler 统一返回 `EOPNOTSUPP`/`ENOTSUP`，不会创建第二个名字。
- 跨挂载/跨文件系统的 link（源或目标不在本挂载所在文件系统）：由内核 VFS 按 POSIX 先返回 `EXDEV`，不会到达本 handler；同样不是 `EPERM`。
- `symlink(2)` 仍返回 `EPERM`（vendored fuser 默认值），不随本 issue 改变；如需标准 errno 另开 issue。
- 只读挂载由 VFS 先返回 `EROFS`，本返回码针对可写挂载。
- 被拒绝的 link 无副作用：目标名不创建、源文件 `nlink` 保持 1、本地状态库不产生记录。
- `unlink`/`rename` 的既有语义不变。

## 验收

默认测试（不挂载内核，锁定 errno 契约）：

```sh
cargo test -p drive-fs --all-features --locked --lib hard_link
```

内核测试（普通 Linux 用户 + `/dev/fuse`，`#[ignore]` 需显式执行）：

```sh
cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture kernel_hard_link_is_rejected_with_operation_not_supported
```

用例在 RW 挂载内写文件后调用 `std::fs::hard_link`，断言 `EOPNOTSUPP`、`ENOTSUP`、`ErrorKind::Unsupported`、目标不存在、源 `nlink == 1`、重复调用返回码一致，并验证 `symlink` 仍为 `EPERM`；结束时 `session.join()` 卸载。

## 工具降级

依赖「两个名字指向同一 inode」的工具应在收到 `EOPNOTSUPP`/`ENOTSUP` 后改用同目录 `rename`（原子替换）或复制。`EPERM` 不应再被当作「不支持硬链接」的可靠信号。
