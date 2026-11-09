# 验收状态入口

唯一实时验收状态见 [DELIVERY_GATES.md](../DELIVERY_GATES.md)。开发版的修复、性能、安装与剩余范围见 [交付说明](RELEASE_20261103_9.md)。历史实验按各自报告保留；本页不再复制阶段快照。

当前源码候选：`0.1.0~dev20261109.2`（`integration/fixes3`，schema v29、IPC21）；发布说明见 [RELEASE_NOTES.md](RELEASE_NOTES.md)，可复制的验收命令见 [ACCEPTANCE_COMMANDS.md](ACCEPTANCE_COMMANDS.md)。

本地修复分支的证据入口：[索引与桌面历史](FIX_5_2_REPORT.md) · [逐目录权限](FIX_7_REPORT.md) · [DB 热点与增量状态投影（#15/#16/#8）](ISSUES_20261107.md)。

已知 FUSE 限制：[硬链接不支持（#18）](FUSE_HARDLINK.md)。

已知冲突状态问题：[陈旧目录冲突标志自愈（#6 follow-up）](STALE_CONFLICTS.md)。
