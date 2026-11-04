# G0–G2 集成记录

2026-11-04，基线 `d555944`。本阶段只运行编译检查，测试、基准、NAS 均未运行。

G0 契约提交 `f730025`，后续协调者补充投影租约与原子 token 交接。原语义通知不覆盖 atime，因此 G2 增加独立 projection generation 和回复租约，不改变原通知含义。

| 流 | 作者提交（按序） | 编译结果 |
|---|---|---|
| A | fc970bb, a6850d8 | core 全 features / all targets check 成功 |
| B | 7640651, 1c9b51b, 3f425e0, c24de25 | G1 check 失败：仅四个约定的 G2 接口缺失；G2 已补齐 |
| C | f02780d, 4d7a83f, 11f6f11 | workspace 全 features / all targets check 成功 |
| D | 5ac3523, d1e6fc3, 4e09f92, 75a146c | 未运行 GUI 或 Python 验证 |

协调者在 `codex/fix-5-2` 按 A → B → C → D cherry-pick，保留各里程碑。然后接入 SharedStore 投影租约、预览转正可恢复的空索引 holder、下载发布事务历史、上传准备失败分类和 GUI 新测试入口。
原有未提交 bus.py 改动保存为 stash `5dbece9e64d201a800df5332faf6ce99cc61829f`；集成 GUI 保留四个同名封装，另修正严格 JSON 参数和 busy 处理。stash 原件保留，不盲目重复应用。

## 编译证据

基线独立 target：`cargo check --workspace --all-features --locked --offline -j2`，exit 0，5m14s。

集成独立 target `/home/drive-native-test/.cache/ultra-integration-target`，命令：

```sh
cargo check --workspace --all-targets --all-features --locked --offline -j2
```

1. [首次失败原件](g2-check-001.log)：exit 1，新 `namespace_index_tests.rs` 的 draft_write 少传 append 参数；未执行测试。
2. [修正后原件](g2-check-002.log)：exit 0，14.56s；现有 unused/dead-code 警告保留，未放宽测试断言。

日志是本地证据。后续 G3/G4/G5 结果分别记录，不把本次编译成功当作行为或性能通过。
