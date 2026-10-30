# P0：本地持久性边界与 FUSE 内核基线

2026-10-31，WSL Ubuntu 24.04，普通测试用户UID 1000。产品进程使用原 `.6`，源码 `62be13c9f9f4a2457a3040829f478bf43b1821df`，deb SHA-256 `4a66edf61d1a32f385f6f61f401db3c181b74f80df44cd0411140ae373c73cf4`。运行文件、夹具工具及内核测试程序哈希见 [context.json](context.json)。没有替换现有挂载二进制或写入NAS。

## 可写挂载的实际边界

用 [durability_fixture](../../../crates/drive-core/examples/durability_fixture.rs) 生成全新的三节点合成状态：根目录、lab、4096字节的A.bin。工具来自干净提交71006ad，`--release --locked`构建；测试内容由确定性字节生成，目标哈希与最终目录清单在操作前写入[expected.json](expected.json)。

挂载配置启用本地写入，但没有凭据、没有Unlock，连接地址仅为本机无效实验端点。测试包括：改写文件且保持句柄打开，文件fsync，关闭句柄，临时文件写入/fsync后改名替换，父目录fsync，创建子目录和文件，再正常卸载/启动。每步限定15秒，执行1轮；这是P0边界观察，不计入P2的20轮竞争或P3的30轮故障注入。

| 检查点 | 独立读取本地状态的结果 | 能证明什么 |
|---|---|---|
| write返回、句柄仍打开 | 草稿generation=1、checkpoint=0、dirty=1；内容哈希匹配；dirty_blob为空，上传操作0条 | 修改已进入草稿并可读，尚未形成待上传快照 |
| 文件fsync返回、句柄仍打开 | generation=checkpoint=1；dirty_blob内容哈希匹配；1条pending upload | 此测试路径已完成本地检查点和待上传记录 |
| close之后 | 与fsync后的检查点及上传记录相同 | 正常关闭保留已建立的本地状态 |
| 临时文件替换、子目录创建 | 挂载内容和目录清单符合预先写入的期望 | 普通保存路径在此合成状态下工作 |
| 正常卸载并重新启动 | 独立判定器再次通过；mounted=true、writable=true、connection=locked、pending_operations=3 | 离线本地内容和队列在正常生命周期内保留，没有伪报服务器已提交 |

证据：[write后](after-write.json)、[fsync后](after-fsync.json)、[close后](after-close.json)、[操作执行记录](operations.jsonl)、[卸载前校验](oracle-before-stop.json)、[重启后校验](oracle-after-restart.json)、[重启状态](restart-overview.json)。校验程序为 [verify_fixture.py](../../../scripts/validation/verify_fixture.py)，不导入客户端代码；比较精确命名空间、内容哈希及SQLite quick_check。写入与校验期间没有远端工作进程或其他内容变更。

正常读取草稿或blob可能命中操作系统缓存，这些观察不是物理介质持久性证明。没有执行kill -9、ENOSPC、EIO或断电。特别是目录fsync在本机返回0，但产品仍没有覆盖FUSE fsyncdir，vendor默认返回ENOSYS；不能用调用成功推断目录创建/替换具有完整断电契约。

两个正常停止窗口分别约0.216秒，Result=success，无残留挂载：[首次停止](first-stop.json)、[最终停止](final-stop.json)。状态保留在WSL `~/.local/state/drive-native-p0/durability-001`，未清除未上传合成数据。

## 现有六项内核回归显式执行通过

使用71006ad的隔离源码归档，执行：

```sh
cargo test --release --locked -p drive-fs --test mounted -- --ignored --test-threads=1
```

这六项默认忽略的测试本轮实际执行，6通过、0失败，用时2.10秒；覆盖缓存/版本切换/离线错误、随机写与截断/fsync、打开文件删除和同名重建、原子替换及旧句柄、权限与本地容量限制、嵌套目录改名与正常重启。它们使用合成数据，不连接NAS。构建源码的drive-core/src、drive-fs/src、drive-fs/tests和Cargo.lock与`.6`构建提交相比无变化。

结果见 [内核日志](p0-kernel-boundary-tests.log)、[构建日志](p0-kernel-boundary-build.log)。保留vendor fuser既有5条编译警告；没有把这6项算成全量358项Rust或GUI回归重跑。容量测试也不等于P3存储故障注入矩阵。

## 后续

本轮补齐P0普通本地生命周期边界的实测基础；[持久性契约](../../DURABILITY.md)仍将崩溃、目录fsync和独立断电语义留作后续验证。原高并发夹具停止超时缺陷仍开放，此小夹具正常停止不关闭它。

只有测试夹具和证据新增，运行时代码未改，没有新deb。24小时基线继续运行；独立链路测速尚未取得有效结果。阶段状态以 [DELIVERY_GATES.md](../../../DELIVERY_GATES.md) 为准。执行脚本的LF归一化哈希见 [source-identity.json](source-identity.json)。
