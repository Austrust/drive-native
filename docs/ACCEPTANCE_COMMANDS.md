# 性能 issue 验收命令（P032 / issue #34）

本页是性能复核 issue [#1](https://github.com/Austrust/drive-native/issues/1)–[#14](https://github.com/Austrust/drive-native/issues/14) 的可复制验收命令。命令以最终合并树 `integration/fixes2`（`work/integ2/src`）为准。最终验收的验证代码树 = `integration/fixes2` @ `9cb6b1c`（clean checkout；`docs merge 7fd7f2f` 仅文档，不改变代码/二进制），`final-<tag>.log` 首行记录该真实 HEAD。重负载必须在远端 Linux 构建机执行，不要在 WSL 或开发机跑长基准。

## 共同前提

```sh
cd work/integ2/src
export CARGO_TARGET_DIR=/tmp/p032-targets/t0
# 重负载与计时类测试加 flock，避免与其它 agent 的构建/基准互相干扰：
#   flock ~/p032/test.lock cargo test ...
```

- `#[ignore]` 用例不会随普通 `cargo test` 执行，必须显式 `-- --ignored --nocapture`。
- 内核测试（`--test mounted`、`permissions_kernel` 等）需要普通 Linux 用户 + `/dev/fuse`；WSL/容器不满足。FUSE 测试二进制之间并行执行会用错误的 OnceLock 顺序初始化 `DRIVE_NATIVE_FS_DIAGNOSTICS` 采样，必须逐二进制串行并显式传 `DRIVE_NATIVE_FS_DIAGNOSTICS=1`（最终验收脚本已按 `--test-threads=1` 执行；测试隔离 follow-up 见 #24）。
- GUI 测试要求 `GUI_PYTHON` 指向锁定 venv 的 python：`GUI_PYTHON=$HOME/dn/.venv-build/bin/python bash scripts/test-gui.sh`。
- `status_latency_210k` / `namespace_index_budget` / `scale_perf*` 在测试内自建合成夹具，无需外部 fixture 路径；需要数 GiB 临时盘与 `/dev/fuse`（210k 用例）；`LAT_CURVE_DIRS` / `LAT_CURVE_WARM_ITERS` 可控制规模曲线。
- 当前合入状态（2026-11-09）：性能修复 `#21–#33` 的代码提交已全部合入 `integration/fixes2`（含 `#26/#28/#29/#31/#32/#33`；#26/#28 的 verifier 收尾不影响代码树）；`#11/#12` 的 fix 分支分别以 `#31/#32` 身份落地。最终计数以最终验收日志的 SUMMARY 为准，本文件命令可直接在冻结树执行（验证树内容见本页开头）。

## 按 issue

1. **#1 GetOverview 去全局锁**
   ```sh
   cargo test -p drive-runtime --all-features --locked --test desktop overview
   cargo test -p drive-runtime --all-features --release --locked --test status_latency_210k -- --ignored --nocapture status_latency_210k
   LAT_CURVE_DIRS=210000,294000,500000 LAT_CURVE_WARM_ITERS=5 \
     cargo test -p drive-runtime --all-features --release --locked --test status_latency_210k -- --ignored --nocapture overview_scale_curve
   ```
   输出中的 `#8 STAT`/`#8 CURVE` 行给出 overview 分段、持锁与 48 路 mixed 结果。

2. **#2 只读 open / NamespaceIndex 水位**
   ```sh
   cargo test -p drive-core --all-features --locked --test cache_policy
   cargo test -p drive-fs --all-features --locked namespace_index_tests
   cargo test -p drive-core --release --all-features --locked --test namespace_index_budget -- --ignored --nocapture
   ```

3. **#3 并发 opendir / GetStates 不互阻**
   ```sh
   cargo test -p drive-core --all-features --locked --lib directory_scale_tests
   cargo test -p drive-fs --all-features --locked --lib bench_tests
   cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture
   ```

4. **#4 草稿提交 fsync 有界/崩溃一致性**
   ```sh
   cargo test -p drive-fs --all-features --locked draft_and_writable_fsync
   cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture
   ```

5. **#5 增量状态投影（硬预算 / delta 发布 / ACL scoped）**
   ```sh
   cargo test -p drive-runtime --all-features --locked --lib status_cache
   cargo test -p drive-runtime --all-features --locked access_denied_changes_use_the_scoped_acl_refresh
   cargo test -p drive-runtime --all-features --locked --test status_cached_batch
   cargo test -p drive-runtime --release --all-features --locked --lib status_cache -- --ignored --nocapture
   ```

6. **#6 ACL 闭包（#26 已合入）**
   ```sh
   cargo test -p drive-core --all-features --locked access_denied
   cargo test -p drive-fs --all-features --locked --test permissions_kernel -- --ignored --nocapture
   ```

7. **#7 失效 fail-closed**
   ```sh
   cargo test -p drive-runtime --all-features --locked notification_queue
   cargo test -p drive-runtime --all-features --locked first_install_commit_publishes
   cargo test -p drive-fs --all-features --locked namespace_index_tests
   ```

8. **#8 remove barrier（#28 已合入）**
   ```sh
   cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture
   cargo test -p drive-runtime --all-features --locked upload
   ```

9. **#9 GUI 读放大（#29 已合入）**
   ```sh
   GUI_PYTHON=$HOME/dn/.venv-build/bin/python bash scripts/test-gui.sh
   ```

10. **#10 桌面 N+1**
    ```sh
    cargo test -p drive-runtime --all-features --locked --test status_cached_batch
    cargo test -p drive-runtime --all-features --locked --test desktop
    cargo test -p drive-core --all-features --locked desktop_status
    ```

11. **#11 运行时信号/命令背压（#31 已合入）**
    ```sh
    cargo test -p drive-runtime --all-features --locked --lib status_events
    cargo test -p drive-runtime --all-features --locked --lib worker_queue_tests
    cargo test -p drive-runtime --all-features --locked --test status_index_oracle
    ```
    重点断言：空闲 StatusChanged=0、StatusChanged2 带 revision/变更集、>32 命令不中断会话、reset 粘滞与背压常量有界。

12. **#12 存储残留（1M operations / retention / schema v28；#32 已合入）**
    ```sh
    cargo test -p drive-core --all-features --locked --test schema_indexes
    cargo test -p drive-core --all-features --locked --lib retention::tests
    cargo test -p drive-core --release --all-features --locked --test schema_indexes -- --ignored --nocapture v27_upgrade_cost_on_a_294k_node_database
    ```
    `queries_select_the_v28_indexes` 断言 pending_operations/冲突/下载候选命中 operations_countable 等新索引；`retention_tests` 覆盖历史裁剪上界；1M 历史 operations 下 `pending_operations` <1 ms 为 #32 微基准（未入库，debug 实测 103.6 ms 全表扫描 → 约 5 µs 索引扫描）。

13. **#13 远程初始化恢复/池/事件证据**
    ```sh
    cargo test -p sproto --all-features --locked
    cargo test -p drive-core --lib --features native
    cargo test -p sproto --all-features --locked reconcile_listing_tests
    cargo test -p drive-core --lib --features native recorded_page_apply
    ```
    真机 102k/148k 事件数据需专用 NAS 夹具，不在本仓库测试范围内；现场验证入口见 [INITIALIZATION_PIPELINE.md](INITIALIZATION_PIPELINE.md)。

14. **#14 文档一致**
    ```sh
    cargo test --workspace --all-features --locked
    bash scripts/verify.sh
    rg -n '6[.]35|47[7]-532|schema[ ]26|STATE_SCHEMA_VERSION=[2]7|TBD[-]FINAL|c3b62a[1]|229[.]1|877[.]6|0[.]690' docs/
    # 远端无 rg 时用 grep 备用：
    grep -rnE '6[.]35|47[7]-532|schema[ ]26|STATE_SCHEMA_VERSION=[2]7|TBD[-]FINAL|c3b62a[1]' docs/
    ```
    前两条覆盖代码-文档引用不被破坏；grep 只扫描顶层 `docs/*.md`（排除 `docs/validation` 证据文件），用于确认过时口径（无出处的毫秒数、旧持锁行号、旧 schema 当前版表述、旧 v27 当前版、TBD 占位与旧树标识、旧 final1 数字）没有残留；括号写法避免命令匹配本文件自身。当前契约 schema 为 v28（`STATE_SCHEMA_VERSION=28`）。

## 与 issue #8 三树数字的关系

ISSUES_20261107.md 的三树表与 verifier 数字测于 fix-7/#8 分支线（`58724a0` 等），**未含 #2 `3dd03a1`**；RELEASE_20261108_1.md 的 28 次 48 路超时测于 `.1` 树 `f6f48ea`（含 #2）。最终树（`integration/fixes2` @ `9cb6b1c`，release，`latency-clean-driver.log`，loadavg_end 10.88）的同一 210k 验收：48 路 mixed **0 超时**（mixed p95 861.6 ms、max 1.64 s），overview 分段 total p95 **0.708 ms**、lock_hold / lock_wait **0**、read-only calls 40/40、cached 40，GetStates p95 226.95 ms、2000 项目录列举 3.89 ms、classic_timeouts 0；数字均取自 `status_latency_210k` 的 `#8 METRICS` 输出。
