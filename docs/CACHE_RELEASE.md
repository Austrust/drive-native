# 缓存释放（释放空间）语义与耗时

本文描述 GUI、Nemo 扩展与 IPC 调用方共用的内容缓存释放行为，涉及 issue #19 的
三个问题：大目录释放没有进度、释放后 `protected_bytes` 残留不可回收、以及
`online_only` 文件释放返回假成功。

## 释放做什么

释放空间只删除本地缓存 body（`blobs/` 中的内容文件）并清除节点上的
`blob`/`local_version` 指针，**不会删除 NAS 上的文件，也不会创建远端操作**。
以下内容受保护、不会被释放：

- 固定（pinned）的节点；
- 有未同步修改（dirty）或冲突（conflict）的节点，以及冲突恢复行保留的 body；
- 有打开句柄的节点；
- 正在上传/下载、或处于 `pending`/`inflight`/`uncertain`/`conflict` 操作中的 body；
- 命名空间替换暂存（`namespace_ops`）引用的 body。

## 结果分类

一次释放对每个文件给出三种结果之一：

| 字段 | 含义 |
| --- | --- |
| `released` | 有本地 body，已删除并清除指针。 |
| `skipped` | 已经是 `online_only`、没有本地 body。**这不是成功释放**，调用方不应把它计入已回收。 |
| `protected_or_failed` | 受保护、打开中、写入中或释放失败，本地 body 保留。 |

释放针对的是节点本身；对目录调用会递归覆盖其下所有文件。重复释放同一目录时，
第二次起文件都会进入 `skipped`，`reclaimed_bytes` 为 0。

## IPC 接口

所有方法都返回 `schema: 1` 的 JSON 字符串。

### 同步接口（兼容旧客户端）

```
ReleaseSpace(s path) -> s json
```

一次性完成整个目录并返回：

```json
{"schema":1,"released":["<node-id>"],"skipped":[],"protected_or_failed":[],
 "reclaimed_bytes":1055}
```

大目录会阻塞到全部文件处理完才返回。实测中 7,676 个离线条目、1.88 GiB 的目录
一次释放需要约 9–10 分钟（issue #19），期间调用方无法区分“正在工作”和“卡死”。
新调用方应改用下面的后台任务接口。

### 后台任务接口

```
BeginReleaseSpace(as paths) -> s json    # {"schema":1,"job_id":"release-1","total":7676}
GetReleaseJob(s job_id)     -> s json    # 轮询进度/结果
CancelReleaseSpace(s job_id) -> s json   # 请求取消
```

`BeginReleaseSpace` 立即返回并在后台线程处理。同一服务同时只允许一个释放任务，
重入会返回 D-Bus 错误（`a release job is already in progress`），而不是排队等待。

`GetReleaseJob` 返回的进度快照：

```json
{"schema":1,"job_id":"release-1","available":true,"total":7676,"done":1200,
 "released_count":1150,"skipped_count":50,"protected_count":0,
 "reclaimed_bytes":268435456,"finished":false,"cancelled":false,"error":null}
```

- `finished: true` 时额外包含 `released`、`skipped`、`protected_or_failed` 数组；
  运行中不返回大数组，避免轮询放大 D-Bus 消息。
- 建议每 250 ms 轮询一次；GUI 与 Nemo 扩展即按此频率更新“正在释放 n/m”和已回收字节。
- job 只保存在内存中：服务重启后未知 `job_id` 返回
  `{"schema":1,"job_id":"...","available":false}`，调用方应提示失败并允许重新发起，
  不要把它当作成功。

`CancelReleaseSpace` 置位取消标志，返回 `cancel_requested: true/false`。已经释放
的文件不会回滚：最终快照为 `finished: true, cancelled: true`，`done` /
`released_count` / `reclaimed_bytes` 对应已提交的前缀。取消在批处理边界生效，
因此可能多完成一个文件才停下。

### 进度与 GC 的关系

释放过程中每 32 个文件批量执行一次 `collect_garbage()`，并累计回收字节；结束时
再执行一次。这样既不会对每个文件都做一次目录扫描，也不会让大目录把 Store 锁
长时间占住。`reclaimed_bytes` 是单调累计值，取消后仍保留已回收量。

## 释放后的残留与 limit_bytes

`collect_garbage()` 的 keep 集不再保留 `deleted=1` 节点的 `blob` 指针，因此远端
删除后残留的 body 会在下一次释放/GC 时被回收；`cache_usage` 的
`reclaimable_bytes` 随之为正、`protected_bytes` 回落。GC 前会检查该 blob 是否
仍被 `deleted=0` 节点或其它引用持有，多节点共享同一 blob 时只清理失效指针，
不会误删 live 节点的 body；`deleted=1` 且 `conflict=1` 的恢复行继续保留 body。

journal 引用同样是 keep 集的来源：`operations` 中 `pending`/`inflight`/`uncertain`/
`conflict` 的 `blob`、未提交 `namespace_ops` replace 的 `body.blob`、以及打开句柄
都在删除前加入保护。`drafts` 与 `cache_releases` 表没有 `blob` 列，不可能直接钉住
`blobs/` 中的 body 文件：草稿 body 位于 `drafts/` 目录并由 dirty/open 规则保留，
`cache_releases` 只记录远端所有权交接。测试
`garbage_collection_keeps_a_deleted_nodes_blob_while_an_upload_operation_references_it`、
`garbage_collection_keeps_a_deleted_nodes_blob_while_a_namespace_replace_references_it`
与 `draft_and_cache_release_journals_carry_no_body_pointer` 覆盖这些边界：deleted
节点与未完成 upload/namespace replace 引用同一 blob 时 GC 回收 0，日志项退役后的
下一次 GC 才回收；不会出现“清理 deleted 指针时误删 live 共享 body”。

`GetOverview` 与 `GetCacheSettings` 也会追加 `release_job` 只读摘要
（`null` 或 `{job_id,total,done,reclaimed_bytes,finished,cancelled}`），
供其他客户端显示进度；它是 additive 字段，`schema` 仍为 1，旧 GUI 忽略即可，
后台任务本身仍是内存态、不随服务重启保留。

`limit_bytes = 0` 表示**不做自动淘汰**：缓存不会因为超过上限而自动 trim，会一直
增长到用户手动释放或显式设置上限。需要自动回收时在 GUI 缓存设置中填入正整数
上限（例如 20 GiB）；设置为 0 时仍可手动释放，只是后台 cache loop 不会自动淘汰。

## 验证入口

```sh
cargo test -p drive-core --lib garbage_collection
cargo test -p drive-core --test cache_policy release
cargo test -p drive-runtime --test release_job
cargo test -p drive-runtime --lib release_job_tests
GUI_PYTHON=... bash scripts/test-gui.sh   # 包含 release_gui_tests
```
