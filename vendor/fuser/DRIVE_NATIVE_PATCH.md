# Drive Native 的 FUSE 中断接入补丁

输入为 crates.io 发布的 fuser 0.16.0，许可证及原始归属见 [LICENSE.md](LICENSE.md)。[UPSTREAM-MANIFEST.json](UPSTREAM-MANIFEST.json)保存复制前逐文件 SHA-256；[drive-native-interrupt.patch](drive-native-interrupt.patch)完整记录三个源文件的修改。原始 crate 的 Cargo.toml、依赖和功能不升级。

上游 request.rs 当前对 INTERRUPT 返回 ENOSYS；本地新增 Filesystem::interrupt 回调，默认仍保留上游行为。我们的文件系统使用请求唯一 ID 找到待完成读取，给原始读取返回 EINTR，消耗中断通知；找不到时短暂延迟并返回 EAGAIN，由内核处理先后竞争。ReplyEmpty::ignore_interrupt 只能用于此类无回复通知，不用于普通 I/O。

Linux 内核 fs/fuse/dev.c 的 fuse_read_interrupt 将请求头清零，只填写 len、opcode、unique，故中断 UID 为零。本补丁将 Interrupt 加入与 Forget／Release 同类的内核控制消息例外，普通请求权限判断不变。依据 [Linux 6.8 源码](https://github.com/torvalds/linux/blob/v6.8/fs/fuse/dev.c)；这不是允许其他用户读取挂载。

本项目验证入口为 `cargo test -p drive-fs --test interrupted -- --ignored --nocapture`，以及原生 NAS 的 native_cancel_roundtrip；需普通用户、FUSE 和显式授权测试环境。上游自身的整个平台测试矩阵未据此通过。第三方更新时重新审查三个改动、原始哈希、内核控制报文权限和实际内核取消回归；上游提供等价能力后移除本地补丁，不能只按版本号删除。
