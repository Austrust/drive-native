# 验收状态入口

唯一实时验收状态见 [DELIVERY_GATES.md](../DELIVERY_GATES.md)。开发版的修复、性能、安装与剩余范围见 [交付说明](RELEASE_20261103_9.md)。历史实验按各自报告保留；本页不再复制阶段快照。

当前发布：`0.1.0~dev20261113.2`（schema v29、IPC21，本版无迁移）；发布说明见 [RELEASE_20261113_2.md](RELEASE_20261113_2.md) 与 [RELEASE_NOTES.md](RELEASE_NOTES.md)，可复制的验收命令见 [ACCEPTANCE_COMMANDS.md](ACCEPTANCE_COMMANDS.md)。

只读路径排除与预览挂载：[按任务正则排除（#50 P1–P3，含 B2 preview 修复）](EXCLUDE_PATTERNS.md)；跨平台移植（#11 stage0）：[Windows / CFAPI 移植路线与 core POSIX 门禁](WINDOWS_PORT.md)（`bash scripts/check-core-posix.sh`，当前为骨架阶段）。

本地修复分支的证据入口：[索引与桌面历史](FIX_5_2_REPORT.md) · [逐目录权限](FIX_7_REPORT.md) · [DB 热点与增量状态投影（#15/#16/#8）](ISSUES_20261107.md)。

已知 FUSE 限制：[硬链接不支持（#18）](FUSE_HARDLINK.md)。

已知冲突状态问题：[陈旧目录冲突标志自愈（#6 follow-up）](STALE_CONFLICTS.md)；已绑定目录被服务器删除后的 [keep-local / keep-remote 恢复（#54）](STALE_CONFLICTS.md#绑定目录的-keep-local--keep-remoteissue-54)。
