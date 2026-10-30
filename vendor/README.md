# 固定的第三方源码

状态：开发中。仅在锁定依赖缺少所需平台能力时保留小范围本地补丁；不自动追踪上游分支。

| 目录 | 来源、用途与下一入口 |
| --- | --- |
| [fuser](fuser/README.md) | crates.io 0.16.0，MIT；补充 FUSE_INTERRUPT 分发和无响应通知消耗，用于实际读取取消。修改范围、上游哈希与回归见 [补丁说明](fuser/DRIVE_NATIVE_PATCH.md) |

workspace Cargo.lock 与 patch 配置共同固定该输入；Debian 构建准备脚本包含此目录。原始第三方文档保留原貌，不代表其所有平台已在本项目验证。
