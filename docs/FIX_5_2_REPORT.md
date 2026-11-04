# Issue #5 / #2 实现与验收报告

状态：实现已集成；G4 独立验证进行中，G5 配对基准未实测。不能据此关闭性能验收。

基线 `codex/fs-recovery-safe-9 @ d555944`；#5/#2 集成分支 `codex/fix-5-2 @ ed1fabf`。
随后 #7 在 `codex/fix-7` 单线实现；共同候选及结果以本报告最终验证章节为准。
规格为项目 `work/plan-5-2-20261104/PLAN.md` 和 `PLAN-ULTRA.md`。

## 实现范围

NamespaceIndex 是 Store 的只读元数据投影。LOOKUP、GETATTR、OPENDIR 和目录续读可从内存快照回答；
每个索引回复持有 projection 读租约，与 Store guard 发布的版本绑定。atime、inode 分配、未提交边界、
Store 替换均进入独立版本域，通知丢失不能把旧索引变成当前权威。
脏草稿、未链接对象、预览、索引 miss 或不可信代数沿原 SQL 路径处理。
索引容量或构建时间超限时整代降级 SQL；有界目录 LRU 懒加载未实现，契约允许安全全量回退。

正目录项 entry TTL 恒 0；attr TTL 默认 1000 ms、可设 0。fuser 的两种 TTL 参数顺序修复保留，
不以空文件镜像、放宽 TTL、排除挂载或减少正确性核对取得性能数字。
详细接口、限制、计数与 G3 权限叠加见 [冻结契约](CONTRACTS.md)。

同步历史使用 schema24 的 `sync_history`，以 durable operation/transfer 身份合并重试，
记录上传确认、下载发布、命名空间确认与远端事件；元数据查询不会制造传输成功。
四个 JSON IPC 支持分页、挂载/结果/名称筛选、近期活动、身份解析和独立清理。
默认保留 30 天 / 10,000 条；历史清理不删除缓存、草稿或队列。

桌面使用同一主窗口中的挂载树与页面；历史及托盘活动按所属服务保存记录身份。
从历史打开文件会重新解析当前身份和路径；在线未缓存内容显示下载过程，离线无内容不假装已打开。
托盘提供汇总、近期历史、全体暂停/恢复和两种退出方式；全体操作保留单挂载手动暂停意图。

## 阶段与提交

- G0：协调者亲自冻结契约 `f730025`；后续补充共享投影租约和跨流接线。
- G1：A/B/C/D 各自 worktree、文件所有权和独立 target，仅 cargo check。
- G2：固定 A→B→C→D 合入 `ed1fabf`。作者里程碑、原件与首次编译失败见 [G2 记录](validation/fix-all-001/G2_REPORT.md)。
- G3：不与 G1 并行，schema25 权限；见 [#7 报告](FIX_7_REPORT.md)。
- G4：不写产品代码的独立代理，在不可变源码归档和独立 target 上运行矩阵。
- G5：等待 G4 结果后独占窗口，原版 FSearch 与三套相同命名空间配对。

原有 bus.py 未提交改动的 stash `5dbece9e64d201a800df5332faf6ce99cc61829f` 保留；对应功能已比较整合。
没有 push、发布、安装到生产、连接 NAS 或读取凭据。

## 正确性与性能证据

G2 全量 `cargo check --workspace --all-targets --all-features --locked --offline -j2` 通过。
G3 冻结编译及失败原件见 [G3 记录](validation/fix-all-001/G3_IMPLEMENTATION.md)。编译通过不算测试通过。

G4 通过/失败/忽略计数、逐矩阵结果与失败原件：待独立运行完成。

| 树 | 候选首扫目标 | 候选重扫目标 | `.9` / local / 候选配对结果 |
|---|---:|---:|---|
| 102,130 条目 / 2,066 目录 | ≤2.0 s | ≤1.6 s | 未实测 |
| 319,679 条目 / 56,073 目录 | ≤13 s | ≤12 s | 未实测 |
| 已保存真实清单 327,750 条目 / 56,554 目录 | ≤15 s | 记录实测值 | 未实测 |

每组要求三轮首扫/重扫，保存索引逐项比较路径、类型、size、mtime，missing/extra/duplicate/mismatch=0，
hydration=0，结束无残留挂载。真实清单仅复用既有 manifest，不重扫生产目录；local 用保留尺寸和时间的稀疏镜像。
性能表还须报告窗口、宿主负载、请求数、每目录成本、用户态索引时间和 client/fixture CPU。
无法由这些计数直接分离的纯内核/调度时间应标注不可辨识，不以重叠 span 相减冒充归因。

## 验收对应与边界

| 验收项 | 实现/验证入口 | 当前证据 |
|---|---|---|
| 名称重验、脏属性、通知丢失与降级 | entry_revalidation、namespace_index、publication、目录夹具 | G4 待结果 |
| INDEX=0 / attr TTL=0 兼容 | 独立矩阵 | G4 待结果 |
| 三树真值、hydration、卸载 | G5 配对 harness | 未实测 |
| 同窗切换、单实例、托盘、混合暂停 | unified_manager / history / pause GUI suites | G4 待结果 |
| 历史重启、重试、分页/筛选/清理、身份解析 | core history、runtime desktop、GUI history | G4 待结果 |
| 真实 Mint 面板/Nemo/HiDPI | 外部桌面验收 | 未实测 |

FOPEN_CACHE_DIR / io_uring 等可选实验未启用。真实 NAS、真实用户队列、生产初始化、裸机性能及长期运行均未实测。
本轮不扩展普通目录全量同步或既有 P0–P6 门槛，也不把历史 `.9` 结果算作新实现通过。
