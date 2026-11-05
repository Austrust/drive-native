# 覆盖映射

[历史组件级映射](prior/G4-evidence-bundle-010-blocked/COVERAGE.md)与[本机 TLS 设计说明](harness/native-worker-chain-012/RUNBOOK.md)分别保留。下面只列入口和可证明的边界；实际结果见 REPORT/原始 JSON。

| 用例组 | 实际入口 | 边界 |
|---|---|---|
| 3002/3008 拒绝 | bridge_download_denied_3002 / 3008 | 实际 TLS 服务→worker ELF→IPC→Bridge ensure；只污染目标 Read；再次 ensure 和到期后台刷新不发新 Download |
| profile 三态 | bridge_profile_false_empty / true_empty / true_other_view | 实际 long-poll 响应穿过 worker 与 Bridge；profile-only 不依赖文件事件 |
| 迟到 Read | bridge_late_read_after_profile | 双通道有界门闩；profile 失效先于旧 Download 拒绝发布 |
| generic 四码 | worker_rpc_generic_3004 / 8004 / 9005 / a005 | 实际 worker RPC 分类；不称完整 Bridge 路径，不称 mutation 成功或无部分提交 |
| 210k 容量 | default_budget_admits_210k_node_snapshot_without_holding_store | ignored 功能用例单独执行；不替代 G5 墙钟/每目录门槛 |
| 私有组件更换 | component-policy-handover.py | 旧/新 fsd+worker、私有 D-Bus/FUSE、持久化策略/历史；不是 deb 生命周期 |

主矩阵的 precise suite/name、INDEX0 N/A、ignored/doc 状态与重复执行观测均在逐轮 inventory；不把跨模式或跨候选计数累加成唯一总数。
