# 验收命令（P032 / integration/fixes3 批次）

本页是性能复核 issue [#1](https://github.com/Austrust/drive-native/issues/1)–[#14](https://github.com/Austrust/drive-native/issues/14) 与本批修复（#6/#8/#9 TB1/#14/#18/#19/#2 2a/#50 P1-P3（含 B2）/#11 stage0/#53）的可复制验收命令。命令以最终合并树 `integration/fixes3`（`work/integ3/src`）为准；最终验收的验证代码树 = `integration/fixes3` 冻结 HEAD（`final-<tag>.log` 首行记录该真实 HEAD；本页编写时的复核树为 `aa36099`，其后又合入 #50 P3/B2、#14 补漏、#11 stage0 与 #8 门禁稳健性，task-21 已并入，最终以冻结日志为准）。重负载必须在远端 Linux 构建机执行，不要在 WSL 或开发机跑长基准。

## 共同前提

```sh
cd work/integ3/src
export CARGO_TARGET_DIR=/tmp/p032-targets/t0
# 重负载与计时类测试加 flock，避免与其它 agent 的构建/基准互相干扰：
#   flock ~/p032/test.lock cargo test ...
```

- `#[ignore]` 用例不会随普通 `cargo test` 执行，必须显式 `-- --ignored --nocapture`；lib 内 ignored 用例也可用 `-- --include-ignored --test-threads=1` 连同普通用例一起跑（见第 24 条）。
- 内核测试（`--test mounted`、`permissions_kernel` 等）需要普通 Linux 用户 + `/dev/fuse`；WSL/容器不满足。FUSE 测试二进制之间并行执行会用错误的 OnceLock 顺序初始化 `DRIVE_NATIVE_FS_DIAGNOSTICS` 采样，必须逐二进制串行并显式传 `DRIVE_NATIVE_FS_DIAGNOSTICS=1`（最终验收脚本已按 `--test-threads=1` 执行；测试隔离 follow-up 见 #24）。
- GUI 测试要求 `GUI_PYTHON` 指向锁定 venv 的 python：`GUI_PYTHON=$HOME/dn/.venv-build/bin/python bash scripts/test-gui.sh`。
- `status_latency_210k` / `namespace_index_budget` / `scale_perf*` 在测试内自建合成夹具，无需外部 fixture 路径；需要数 GiB 临时盘与 `/dev/fuse`（210k 用例）；`LAT_CURVE_DIRS` / `LAT_CURVE_WARM_ITERS` 可控制规模曲线。
- 当前合入状态（2026-11-09 增量）：性能修复 `#21–#33` 已合入 `integration/fixes2`；本批 `#14/#19/#18/#53/#8/#6/#2 2a/#50 P1-P3（含 B2 preview 修复）/#9 TB1/#11 stage0` 已合入 `integration/fixes3`（`22ba761` 之后；`aa36099` 复核后再增量合入 #50 P3/B2、#14 补漏、#11 stage0 与 #8 门禁稳健性）。task-21（abandoned 残留 pending op）已并入（`42580b5`）。#9 TB2（`commit_diagnostics` 消费面）仍在单独分支，随 replay commit 合入后补验收命令。最终计数以最终验收日志的 SUMMARY 为准，本文件命令可直接在冻结树执行（验证树内容见本页开头）。

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
    前两条覆盖代码-文档引用不被破坏；grep 只扫描顶层 `docs/*.md`（排除 `docs/validation` 证据文件），用于确认过时口径（无出处的毫秒数、旧持锁行号、旧 schema 当前版表述、旧 v27 当前版、TBD 占位与旧树标识、旧 final1 数字）没有残留；括号写法避免命令匹配本文件自身。当前契约 schema 为 v29（`STATE_SCHEMA_VERSION=29`）。

15. **#9 TB1 提交结局抽象与 schema v29 迁移**
    ```sh
    cargo test -p drive-core --all-features --locked --test schema_v29
    ```
    重点断言：v28 真实结构（operations/namespace_ops/transfers 的 pending/inflight/uncertain/committed 行）升级到 29 后 outcome/outcome_category/last_error/next_retry_at 回填正确、生成 `state-before-v29-*` 备份、二次 open 幂等、`upload_reconcile_operations`/`pending_operations` 查询不受新列影响；`commit_diagnostics` 视图、`recovery_runs`、`platform_state_changes`、`nodes.in_sync` 只建结构，消费逻辑由 TB2/TB3 接入。

16. **#14 未确认上传 abandoned 终态与放弃入口**
    ```sh
    cargo test -p drive-core --all-features --locked --test abandoned_upload
    cargo test -p drive-runtime --all-features --locked --test desktop abandon
    GUI_PYTHON=$HOME/dn/.venv-build/bin/python desktop/tests/issue_recovery_gui_tests.py
    ```
    重点断言：预算耗尽后 op 进入 `abandoned`、`pending_operations` 不再计入、GetIssues 带 `can_abandon_upload`、AbandonUpload 只删本地文件且不改服务器条目、RecheckIssues 重新武装、重启后终态保持；绑定/仍在上传的条目拒绝放弃。

17. **#19 缓存释放后台作业与 GC**
    ```sh
    cargo test -p drive-runtime --all-features --locked --test release_job
    GUI_PYTHON=$HOME/dn/.venv-build/bin/python desktop/tests/release_gui_tests.py
    ```
    重点断言：`BeginReleaseSpace`/`GetReleaseJob`/`CancelReleaseSpace` 契约、skipped（已在线）不报假成功、protected 不下发释放、每 32 个文件触发 GC、服务重启后未知 job 返回 `available:false`、同步 `ReleaseSpace` 新增 `skipped` 且形状兼容。

18. **#18 FUSE 硬链接拒绝码**
    ```sh
    cargo test -p drive-fs --all-features --locked --lib hard_link
    DRIVE_NATIVE_FS_DIAGNOSTICS=1 cargo test -p drive-fs --all-features --locked --test mounted -- --ignored --nocapture kernel_hard_link_is_rejected_with_operation_not_supported
    ```
    内核用例需要普通 Linux 用户 + `/dev/fuse`；断言 `EOPNOTSUPP`（等于 `ENOTSUP`）、目标不存在、源 `nlink==1`、二次调用同样返回、符号链接仍为 `EPERM`，并在卸载后确认 Store 中无目标记录。

19. **#53 已提交水位推进后的 overview 刷新**
    ```sh
    cargo test -p drive-runtime --all-features --locked --lib status_watch
    GUI_PYTHON=$HOME/dn/.venv-build/bin/python desktop/tests/unified_manager_tests.py
    ```
    重点断言：`commit_revision` 前进触发 `overview_changed`、GUI 重查 GetOverview 后陈旧托盘 attention 被清除，纯语义水位变化不伪造其它计数。

20. **#8 初始化期间桌面读响应性（仅测试验收，无运行时变化）**
    ```sh
    cargo test -p drive-runtime --all-features --locked --lib initialization_owner::tests
    cargo test -p drive-runtime --all-features --locked --test desktop initialization_remains_available
    ```
    用例自建 210k 合成夹具并在共享构建机上对负载敏感；p95 门槛与分段统计以用例输出为准。门禁稳健性调整：并发查询证据改为单调计数（attempts_throughout/DirectoryCommitted），并把 delayed directory / FIFO 启动超时预算放宽到 60s/30s；该批只增/调整测试，不改变运行时行为。

21. **#6 已绑定目录 stale conflict 自愈**
    ```sh
    cargo test -p drive-core --all-features --locked --test conflicts -- stale
    ```
    重点断言：mkdir/rename 确认成功后与快照安装时清除已绑定目录的陈旧 conflict；远端删除、本地脏内容、未终态操作等真实冲突保留。

22. **#2 2a freedesktop 系统通知回退**
    ```sh
    GUI_PYTHON=$HOME/dn/.venv-build/bin/python desktop/tests/notification_dbus_tests.py
    ```
    覆盖托盘优先、托盘缺失/失败回退 freedesktop、jeepney/会话总线缺失安静降级、ActionInvoked 点击路由、通知 id 复用与 32 条映射淘汰。`scripts/test-gui.sh` 会一次性跑全部 26 个 GUI 模块，脚本没有单模块参数；单模块按上面直接执行，所需环境变量见 `scripts/test-gui.sh`。

23. **#50 P1/P2 按任务正则排除**
    ```sh
    cargo test -p drive-core --all-features --locked --lib exclude_tests
    GUI_PYTHON=$HOME/dn/.venv-build/bin/python desktop/tests/gui_unit_tests.py
    ```
    重点断言：初始化候选按规则整棵子树剪枝、祖先规则排除后代、root 永不被排除、非法正则报出 pattern 与编译错误、GUI 保存时用等价校验；FUSE 读路径过滤（P3）与 preview 挂载（B2）见第 24 条；上传/下载/缓存调度防御与规则变更后的存量清理属阶段 4。

24. **#50 P3 读路径隐藏与 B2 preview 挂载**
    ```sh
    cargo test -p drive-core --all-features --locked --lib exclude_index_tests
    DRIVE_NATIVE_FS_DIAGNOSTICS=1 cargo test -p drive-fs --all-features --locked --lib exclude_read_tests -- --ignored --nocapture
    cargo test -p drive-runtime --all-features --locked --lib mount_exclusion -- --include-ignored --test-threads=1
    ```
    重点断言：排除路径在 FUSE 读路径返回 `ENOENT`（lookup/getattr/readdir(plus)/open/opendir），`readdir(plus)` 不显示且分页 cookie 保持原位置（允许空洞）；事件与恢复快照在入口剪枝；GetDirectory/GetStates 过滤自身与子树；未配置规则时保持零成本快路径；带规则的任务在初始化 preview 阶段挂载成功（空索引），初始化完成/快照安装后索引重建、排除路径不可见。`exclude_read_tests` 与 `mount_exclusion` 是内核用例，需普通 Linux 用户 + `/dev/fuse`；FUSE 测试二进制之间串行执行。

25. **#11 stage0 core POSIX 门禁**
    ```sh
    bash scripts/check-core-posix.sh
    bash scripts/check-core-posix.sh --list
    bash scripts/check-core-posix.sh --strict || true
    ```
    默认模式扫描 `crates/drive-core` 的 POSIX 依赖，新增未列入 allowlist 的命中即失败（`scripts/verify.sh` 已接入，发生在 `--tests-only` 早退之前）；`--list` 输出扫描文件/命中/allowlist 清单；`--strict` 当前会把 allowlist 项也报为违规，仅用于收紧基线，不接入 CI。当前基线：63 个文件、108 处命中全部在 allowlist（0 violations）。迁移计划与三接口见 [WINDOWS_PORT.md](WINDOWS_PORT.md)。

26. **#50 P4 写路径/调度防御与规则变更本地清理**
    `sh
    DRIVE_NATIVE_FS_DIAGNOSTICS=1 cargo test -p drive-fs --all-features --locked --lib exclude_write_tests -- --ignored --nocapture
    cargo test -p drive-core --all-features --locked --lib exclude_purge_tests
    cargo test -p drive-core --all-features --locked --lib exclude_index_tests
    `
    重点断言：排除路径的 create/mkdir/unlink/rmdir/rename/write/setattr 返回 ENOENT 且不落库、不留 pending 操作；上传/下载/缓存与 namespace 调度跳过被排除对象；规则变更或重启把已导入/已缓存子树本地隐藏并回收正文，断言不发远端删除、移除规则后可 restore；恢复快照 install 前 purge 排除节点，历史行不会被 tombstone 标成 conflict（conflict=0/dirty NULL/operations 0）。

27. **#10 恢复分类与自动恢复**
    `sh
    cargo test -p drive-runtime --all-features --locked --lib recovery
    cargo test -p drive-core --all-features --locked --test recovery_runs
    cargo test -p drive-runtime --all-features --locked --test desktop -- recovery
    `
    重点断言：rescan_later / merge_mode / 游标缺口在 3 次自动恢复内完成（2/5/10 秒或 0/5/15 秒退避），耗尽才 
eeds_attention；GetOverview/GetIssues 的 ecovery{state,cause,attempts,next_retry_at,revision,attention} 能区分重试中/恢复中/需人工；ecovery_revision 推进触发 overview 刷新；IPC 21 不变。

28. **#11 step1 文件身份契约**
    `sh
    cargo test -p drive-core --all-features --locked --lib file_identity
    bash scripts/check-core-posix.sh
    `
    重点断言：PlatformFileId 往返/根=1/未知映射返回 None 且不写库/adapter 只读镜像 s_inodes；行为与 ino 语义不变；POSIX 门禁仍 0 violations。
## 与 issue #8 三树数字的关系

ISSUES_20261107.md 的三树表与 verifier 数字测于 fix-7/#8 分支线（`58724a0` 等），**未含 #2 `3dd03a1`**；RELEASE_20261108_1.md 的 28 次 48 路超时测于 `.1` 树 `f6f48ea`（含 #2）。最终树（`integration/fixes2` @ `9cb6b1c`，release，`latency-clean-driver.log`，loadavg_end 10.88）的同一 210k 验收：48 路 mixed **0 超时**（mixed p95 861.6 ms、max 1.64 s），overview 分段 total p95 **0.708 ms**、lock_hold / lock_wait **0**、read-only calls 40/40、cached 40，GetStates p95 226.95 ms、2000 项目录列举 3.89 ms、classic_timeouts 0；数字均取自 `status_latency_210k` 的 `#8 METRICS` 输出。
