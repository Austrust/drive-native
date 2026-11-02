# Drive Native 的 FUSE 本地补丁

输入为 crates.io 发布的 fuser 0.16.0；许可证及原始归属见 [LICENSE.md](LICENSE.md)。[UPSTREAM-MANIFEST.json](UPSTREAM-MANIFEST.json) 保存复制前逐文件 SHA-256。原始 Cargo.toml、依赖版本及功能不升级。按以下顺序在原始 crate 上应用补丁：

1. [drive-native-interrupt.patch](drive-native-interrupt.patch)：修改 `lib.rs`、`request.rs`、`reply.rs`，接入读取取消。
2. [drive-native-session-stop.patch](drive-native-session-stop.patch)：修改 `channel.rs`、`session.rs`，接入独立会话退出唤醒和可返回错误的 join。

上游 request.rs 对 INTERRUPT 返回 ENOSYS；第一个补丁新增 Filesystem::interrupt 回调，默认仍保留上游行为。我们的文件系统用请求唯一 ID 找到待完成读取，给原读取返回 EINTR；找不到时短暂延迟并返回 EAGAIN，由内核处理先后竞争。ReplyEmpty::ignore_interrupt 仅用于此类无回复通知。Linux 的 fuse_read_interrupt 将请求头清零，只填写 len、opcode、unique，因此 INTERRUPT 的 UID 为零；将其加入内核控制消息权限例外不改变普通请求权限。

第二个补丁解决后台会话退出依赖 /dev/fuse 返回 ENODEV 的问题。卸载路径消失后，未关闭的描述符等引用仍可能保留连接。后台接收使用非阻塞设备加 poll，同时监听私有 Unix socket；显式停止先尝试卸载，再关闭唤醒端，退出接收循环并正常销毁文件系统。join_result/join_with_unmount 返回线程错误；旧 join 接口保持兼容。停止不会发送进程信号，也不会操作其他挂载命名空间。回调表示卸载尝试结束，不保证卸载成功，调用方必须复核真实挂载状态。

依据 [Linux 6.8 fs/fuse/dev.c](https://github.com/torvalds/linux/blob/v6.8/fs/fuse/dev.c)：O_NONBLOCK 用于无待读请求时返回 EAGAIN；回复写入路径不检查该标志。应用层负责排空已分发的写入/fsync，停止网络读取，释放 notifier，并在排空后持久化草稿。尚未分发、未确认的内核请求不能被当作成功写入。暂停同步不调用会话停止。

2026-11-02 已静态确认：第二个补丁从清单中原始 channel.rs/session.rs 逐块重放后，与当前两个源文件逐字节相同。运行门禁尚待本批统一验证；不能将补丁重放当作内核或 Mint 验收通过。

验证入口：

```bash
cargo test -p drive-fs --lib shutdown_tests
cargo test -p drive-fs --lib shutdown_tests -- --ignored --test-threads=1
cargo test -p drive-fs --test interrupted -- --ignored --nocapture
```

第二行需普通用户和可用 /dev/fuse，使用合成本地数据，覆盖持有文件/目录/notifier、已接收工作排空、草稿与 pending 保留及下载读取取消。失败保留夹具；只正常关闭测试自己的描述符，不用强杀。原生 NAS 取消联调是独立授权门禁，本次补丁不据此宣称已运行。上游整个平台矩阵和 Mint 实机仍需分别验证。

第三方升级时审查两个补丁、原始哈希、控制报文权限、非阻塞设备回复行为及停止/取消回归。上游提供等价能力后再移除本地补丁，不能只按版本号删除。
