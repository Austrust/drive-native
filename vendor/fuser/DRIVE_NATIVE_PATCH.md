# Drive Native 的 FUSE 本地补丁

输入为 crates.io 发布的 fuser 0.16.0；许可证及原始归属见 [LICENSE.md](LICENSE.md)。[UPSTREAM-MANIFEST.json](UPSTREAM-MANIFEST.json) 保存复制前逐文件 SHA-256。原始 Cargo.toml、依赖版本及功能不升级。按以下顺序在原始 crate 上应用补丁：

1. [drive-native-interrupt.patch](drive-native-interrupt.patch)：修改 `lib.rs`、`request.rs`、`reply.rs`，接入读取取消。
2. [drive-native-session-stop.patch](drive-native-session-stop.patch)：修改 `channel.rs`、`session.rs`，接入独立会话退出唤醒和可返回错误的 join。
3. [drive-native-metadata-cache.patch](drive-native-metadata-cache.patch)：在前两个补丁之上增加 READDIRPLUS 协商、无 lookup 引用的 PLUS 目录项、独立 entry/attr TTL 回复及保留通知错误的接口。

上游 request.rs 对 INTERRUPT 返回 ENOSYS；第一个补丁新增 Filesystem::interrupt 回调，默认仍保留上游行为。我们的文件系统用请求唯一 ID 找到待完成读取，给原读取返回 EINTR；找不到时短暂延迟并返回 EAGAIN，由内核处理先后竞争。ReplyEmpty::ignore_interrupt 仅用于此类无回复通知。Linux 的 fuse_read_interrupt 将请求头清零，只填写 len、opcode、unique，因此 INTERRUPT 的 UID 为零；将其加入内核控制消息权限例外不改变普通请求权限。

第二个补丁解决后台会话退出依赖 /dev/fuse 返回 ENODEV 的问题。卸载路径消失后，未关闭的描述符等引用仍可能保留连接。后台接收使用非阻塞设备加 poll，同时监听私有 Unix socket；显式停止先尝试卸载，再关闭唤醒端，退出接收循环并正常销毁文件系统。join_result/join_with_unmount 返回线程错误；旧 join 接口保持兼容。停止不会发送进程信号，也不会操作其他挂载命名空间。回调表示卸载尝试结束，不保证卸载成功，调用方必须复核真实挂载状态。

依据 [Linux 6.8 fs/fuse/dev.c](https://github.com/torvalds/linux/blob/v6.8/fs/fuse/dev.c)：O_NONBLOCK 用于无待读请求时返回 EAGAIN；回复写入路径不检查该标志。应用层负责排空已分发的写入/fsync，停止网络读取，释放 notifier，并在排空后持久化草稿。尚未分发、未确认的内核请求不能被当作成功写入。暂停同步不调用会话停止。

2026-11-02 已静态确认：第二个补丁从清单中原始 channel.rs/session.rs 逐块重放后，与当前两个源文件逐字节相同。运行门禁尚待本批统一验证；不能将补丁重放当作内核或 Mint 验收通过。

第三个补丁来自本项目的元数据性能分支及安全整合修订；未升级 fuser。`enable_readdirplus` 协商内核能力；`add_without_lookup` 令 entry 的 nodeid 为零但保留目录项 inode/name/type，供已有目录视图续读，不授予属性缓存。`entry_with_timeouts` 和 `add_with_timeouts` 分离名字与属性有效期。应用策略仍将 entry TTL 固定为零；存在正属性 TTL 不表示名字可缓存，也不能绕过预览目录的权限核对。

`inval_entry_strict` / `inval_inode_strict` 保留通知失败供应用关闭缓存准入；接口可用不等于通知已成功执行。补丁同时保留性能分支的 `inc_epoch` 通知接口；当前安全策略不依赖它启用正 entry TTL，不宣称旧内核支持该能力。

安全整合首轮内核回归发现 `ReplyEntry::entry_with_timeouts` 将内部 `Response::new_entry(attr_ttl, entry_ttl)` 的两个参数传反，导致预期 entry=0、attr=30 秒实际成为名字缓存 30 秒。本补丁修正顺序，并新增两个经真实 `ReplySender` 序列化的不等 TTL ABI 回归，同时覆盖普通 entry 和 PLUS 的秒及纳秒字段。PLUS 原构造参数顺序不同，其现有传递未发现反转。两项内核改名测试另核验 Store 中旧路径已消失，避免把未生效的本地改名误判为内核缓存问题。

2026-11-03 来源核验：从 `416cc41b11fce2774bc8eb1e899e100f3d896b8f` 的 20 个源码文件反向撤销前两个补丁，逐个匹配原始清单哈希，再顺序重放三个补丁；最终 20 个源码文件均与当前仓库 LF 内容一致。原两补丁未改。此结果只验证来源和重放；参数修复后的统一内核、ABI 与应用回归待新的隔离快照执行，不能把重放结果作为运行验收。

验证入口：

```bash
cargo test -p drive-fs --lib shutdown_tests
cargo test -p drive-fs --lib shutdown_tests -- --ignored --test-threads=1
cargo test -p drive-fs --test interrupted -- --ignored --nocapture
```

第二行需普通用户和可用 /dev/fuse，使用合成本地数据，覆盖持有文件/目录/notifier、已接收工作排空、草稿与 pending 保留及下载读取取消。失败保留夹具；只正常关闭测试自己的描述符，不用强杀。原生 NAS 取消联调是独立授权门禁，本次补丁不据此宣称已运行。上游整个平台矩阵和 Mint 实机仍需分别验证。

第三方升级时审查三个补丁、原始哈希、控制报文权限、非阻塞设备回复行为、名字/属性 TTL 的 ABI 顺序、通知错误传播及停止/取消回归。上游提供等价能力后再移除本地补丁，不能只按版本号删除。
