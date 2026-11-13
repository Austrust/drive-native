# 已打开文件的读取隔离

2026-10-31，独立分支`codex/fuse-read-isolation`，沿用当前公开开发代码及通知诊断修复。此批实现P1 A的一部分，不改变原基线、NAS权限、原生提交规则或数据库schema18。

> **2026-11-13 复核补充（issue #67 §3.4）**：本文的「已打开文件读取进入独立四线程队列、读路径不取 Store 写锁」结论在 lab FUSE 上复测成立（1/16/64 个慢 cache-miss 读下 4 KiB cache-hit READ p95 12–15 µs 平坦，数据 lane 内无队头）。但请注意：这**不是** #59 的 metadata 隔离证明——残余串行点在共享 Store 写锁，metadata 目标不能靠本隔离或 `max_background` 调参验收。复核细节见 [FUSE_REQUEST_ISOLATION.md](FUSE_REQUEST_ISOLATION.md) 的「#67 §3.4 复核」。

## 修改

已打开Cached/Draft的读取进入独立的四线程、256项有界队列；元数据和原生下载等待仍各有独立队列。读取在短暂取得句柄内容引用后释放句柄表，再进行文件IO。没有通过放大属性TTL或返回空正文绕过等待。

打开注册、草稿提升、创建及删除/替换的句柄屏障使用独立更新锁；它可以等待Store，但缓存读取不取得这把锁。Store与FUSE句柄表分开取得，不在句柄表锁内查询数据库、校验版本或进行文件属性IO。原有不可变内容引用、取消响应、草稿checkpoint、同名替换和删除后旧句柄保留机制继续使用。

新打开的句柄会加入同版本已创建的草稿，包括草稿刚建立、首次写入尚未发生的间隔；旧版本不会被绑到其他版本草稿。这补上将表锁移出数据库操作后必须保持的注册/提升衔接。

## 已验证范围

- 新内核回归先在未修改的父版本上复现：四个元数据线程全部等待Store时，预先打开的缓存读取超过测试的2秒等待界限。解除屏障后正常清理，不强杀挂载。
- 修复版在同一屏障下完成完整缓存和可写草稿读取，内容校验通过。测试另验证新句柄能看到同版本草稿的首次写入。
- 完整Rust389项通过、0失败、10项默认忽略；其中9项内核测试随后显式通过，剩余1项为文档示例。覆盖随机写、截断、fsync、旧版本、删除后句柄、原子替换、信号取消及下载退避期间取消。
- GUI49项在10个独立Qt进程通过。代码修改未增加安装、账户或网络操作。

验证过程保留失败记录：第一次加入草稿注册检查时，提前关闭额外文件导致异步release占用测试屏障，测试入口等待超时；修改为保持句柄到解除屏障后再关闭。另修正WSL缺少rg及将runtime内核测试误写成独立target的脚本入口；未将未执行项计入通过。

以上是该先行构建的受控锁依赖回归。后续已实现[稳定目录视图](FUSE_DIRECTORY_VIEWS.md)和[只读关闭隔离](CACHED_CLOSE_ISOLATION.md)，安装包及固定混合测量见[`.4`验证](validation/native-package-004/REPORT.md)。新打开文件仍经过Store，创建/草稿准备仍可能等待更新锁；增量状态、内存和全部退出边界未完成，最终要求见[验收门槛](../DELIVERY_GATES.md)。

## #67 §3.4 复核（2026-11-13，lab FUSE）

装置：真实内核 FUSE 挂载（普通用户 + `/dev/fuse`）、合成夹具（root + `d000` + 64×256 KiB cached + 96 个慢 cache-miss 文件），test-only `SlowHydrator` 每次 hydrate 睡 1500 ms 后返回 EIO，使内核 READ 在 sleep 期间保持 in-flight；`with_metadata_ttl_ms(0)` 强制 stat/readdir 走 FUSE。用例 `control → slow1 → slow16 → slow64 → store_hold`，每 case 10 s。integration（`acd32bf`）n=3、base（`a9b97f3`）n=2，取每 rep 中位。

- **数据 lane 隔离成立（H2）**：control 4 KiB cache-hit READ p95 在 base 15 µs / integration 12 µs；存在 1、16、64 个在飞慢数据请求时 cache-hit 小读 p95 仍 12–15 µs 平坦，数据 lane 内无残余队头。慢 hydrate 不拖慢 cache-hit 读，本文的队列/句柄表分离是有效的。
- **前台慢读不抬升 metadata（H1）**：slow16/slow64 下 metadata stat p95 与 control 同量级（base 37/30 µs、integration 45/62 µs），且 `max_background=1` 决定性对照下 slow64 仍有 67 个请求在飞、stat p95 30 µs。→ 本文的读隔离不能、也不需要靠调大内核 background 配额来支撑；`max_background` 只约束后台请求（`FR_BACKGROUND`）。
- **残余串行点不在数据 lane，而在共享 Store 写锁（H3）**：持 Store 4 s、metadata 预算 300 ms 时，integration 36/36 返回可重试 `EAGAIN`、p50≈300.4 ms；base 无界阻塞 4.001 s、0 EAGAIN；两棵树最坏尾都 ~4 s（integration 3.905 s），且 integration 服务端 lookup 直方图 max 仅 300.371 ms → 尾巴在 handler 之外（内核侧）。这属于 metadata 路径的 Store 持有期问题（#23/#24），不是已打开文件读取隔离的回归。
- **稳态无回归（H4）**：control stat p95/p99 base 51/171 µs vs integration 47/171 µs、readdir 131 vs 117 µs、4 KiB READ 15 vs 12 µs，均在 µs 级噪声内。

可复现命令（本分支提交了证据 harness `crates/drive-fs/tests/perf_isolation_probe.rs`，test-only `#[ignore]`）：

```sh
DRIVE_NATIVE_FS_DIAGNOSTICS=1 \
  cargo test -p drive-fs --release --all-features --locked \
  --test perf_isolation_probe -- --ignored --nocapture --test-threads=1
```

限制：lab 合成夹具 + 单一挂载，不代表真实 NAS/桌面端到端（base n=2，integration n=3）；~4 s 尾巴尚未用内核 tracepoint 定位；WILLNEED 后台 readahead 压力实验（`perf_stream_probe.rs`）已编译但未运行；不可把本复核当作 #59 AC1/AC2 已通过的证据（见 FUSE_REQUEST_ISOLATION.md 修正）。
