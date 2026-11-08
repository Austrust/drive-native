# 性能 issue 验收命令（P032 / issue #34）

本页是性能复核 issue [#1](https://github.com/Austrust/drive-native/issues/1)–[#14](https://github.com/Austrust/drive-native/issues/14) 的可复制验收命令。命令以最终合并树 `integration/fixes2`（`work/integ2/src`）为准；重负载必须在远端 Linux 构建机执行，不要在 WSL 或开发机跑长基准。

## 共同前提

```sh
cd work/integ2/src
export CARGO_TARGET_DIR=/tmp/p032-targets/t0
# 重负载与计时类测试加 flock，避免与其它 agent 的构建/基准互相干扰：
#   flock ~/p032/test.lock cargo test ...
```

- `#[ignore]` 用例不会随普通 `cargo test` 执行，必须显式 `-- --ignored --nocapture`。
- 内核测试（`--test mounted`、`permissions_kernel` 等）需要普通 Linux 用户 + `/dev/fuse`；WSL/容器不满足。
- GUI 测试要求 `GUI_PYTHON` 指向锁定 venv 的 python：`GUI_PYTHON=$HOME/dn/.venv-build/bin/python bash scripts/test-gui.sh`。
- `status_latency_210k` / `namespace_index_budget` / `scale_perf*` 在测试内自建合成夹具，无需外部 fixture 路径；需要数 GiB 临时盘与 `/dev/fuse`（210k 用例）。
- 当前合入状态：`#21/#22/#23/#24/#25/#27/#30` 已在 `integration/fixes2`；`#33` 已合入 `684c20a`（两个条件提交 `641c5bb`/`34e335d` 待重合并）；`#26/#28` 尚未完成；`#29` 的 fix 分支未合入最终树；`#11/#12` 尚无 fix 分支。未合入项的命令在其合入后启用。

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
   cargo test -p drive-fs --all-features --locked namespace_index
   cargo test -p drive-core --release --all-features --locked --test namespace_index_budget -- --ignored --nocapture
   ```

3. **#3 并发 opendir / GetStates 不互阻**
   ```sh
   cargo test -p drive-core --all-features --locked filesystem_directory_scale
   cargo test -p drive-fs --all-features --locked --lib directory_bench
   cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture
   ```

4. **#4 草稿提交 fsync 有界/崩溃一致性**
   ```sh
   cargo test -p drive-core --all-features --locked --lib draft_fsync
   cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture
   ```

5. **#5 增量状态投影（硬预算 / delta 发布 / ACL scoped）**
   ```sh
   cargo test -p drive-runtime --all-features --locked --lib status_cache
   cargo test -p drive-runtime --all-features --locked --test status_cached_batch
   cargo test -p drive-runtime --all-features --release --locked --lib status_cache -- --ignored --nocapture
   ```

6. **#6 ACL 闭包（实现合入后启用）**
   ```sh
   cargo test -p drive-core --all-features --locked access_denied
   cargo test -p drive-fs --all-features --locked --test permissions_kernel -- --ignored --nocapture
   ```

7. **#7 失效 fail-closed**
   ```sh
   cargo test -p drive-runtime --all-features --locked notification_queue
   cargo test -p drive-runtime --all-features --locked first_install_commit_publishes
   cargo test -p drive-fs --all-features --locked publication
   ```

8. **#8 remove barrier（实现合入后启用）**
   ```sh
   cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture
   cargo test -p drive-runtime --all-features --locked upload
   ```

9. **#9 GUI 读放大（fix 分支合入后启用）**
   ```sh
   GUI_PYTHON=$HOME/dn/.venv-build/bin/python bash scripts/test-gui.sh
   ```

10. **#10 桌面 N+1**
    ```sh
    cargo test -p drive-runtime --all-features --locked --test status_cached_batch
    cargo test -p drive-runtime --all-features --locked --test desktop
    cargo test -p drive-core --all-features --locked desktop_status
    ```

11. **#11 运行时信号/命令背压**：尚无 fix 分支，暂无可执行验收命令（实现合入后补）。

12. **#12 存储残留（1M operations / retention / schema v28）**：尚无 fix 分支；schema v28 随 #32 落地后补迁移与 EXPLAIN 验收命令。

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
    rg -n "6\.35|477-532|487-532|schema 26" docs/
    ```
    第一条覆盖代码-文档引用不被破坏；grep 用于确认过时口径（6.35 ms、旧持锁行号、schema 26 当前版）没有残留。

## 与 issue #8 三树数字的关系

ISSUES_20261107.md 的三树表与 verifier 数字测于 fix-7/#8 分支线（`58724a0` 等），**未含 #2 `3dd03a1`**；RELEASE_20261108_1.md 的 28 次 48 路超时测于 `.1` 树 `f6f48ea`（含 #2）。最终合并树的 48 路与 overview 分段数字以 `status_latency_210k` 的 `#8 STAT` 输出为准，并在本次文档修订中保留 **TBD-FINAL** 占位，待最终测量回填。
