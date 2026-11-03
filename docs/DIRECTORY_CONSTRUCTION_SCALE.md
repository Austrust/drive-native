# 目录密集树：构造成本优化

基准源码 `de19ae000e001b8763f4ef1ef3c538e162027a3d` 的既有合成完整 FSearch 测量，在 319679 项／56073 目录下约 66.5 秒首扫，明显高于 102130 项／2066 目录的结果。两种规模同时改变条目数和目录数，不能仅用总耗时除目录数证明全部成本来源；本切片另提供逐阶段合成例程。

本切片仅改权威 `Store::directory_snapshot` 的查询构造，不改变 TTL、失效通知、预览、持有目录 cookie 或正文读取：

- 用一个 prepared 查询读取当前目录与父目录的必要身份，仍检查双方存活及本地／远端 overlay 两条 ACL 祖先关系，不读取无关的完整版本或文件属性。
- count、缺失 inode 分配、条目收集复用 prepared statements；无 ACL 种子时跳过递归 blocked 集合。缓存的是 SQL 执行计划，每次仍读取当前事务，不能复用权限结果。
- 父 inode 已在同一查询中核实时不执行无效 INSERT。映射缺失仍懒分配，不信任 complete hint。原排序、预算预检、最终实际内存检查和事务回滚不变。

## 验证入口与当前状态

```sh
cargo test --release --locked --offline -p drive-core --all-features --lib directory_scale_tests
cargo test --release --locked --offline -p drive-core --all-features --test filesystem_metadata --test native_access
cargo test --release --locked --offline -p drive-fs --test directory_views --test preview_tests --test readdirplus -- --ignored --test-threads=1
```

新增回归覆盖查询缓存后的 ACL/overlay 撤销、缺父／删除／非目录拒绝、inode 预算与失败回滚、UTF-8 名称、local shadow 和 dirty body 不被目录查询打开。隔离 run001 的原夹具将本地创建排在远端同名之后，合法触发 EEXIST：core 350 通过、1 失败、1 ignored；FS lib 28 通过、7 ignored；默认和 TTL=0 两组显式目录／预览／PLUS 内核测试各 10 通过。core 命令在 lib 失败后未继续指定的两个集成测试。修订夹具改为先本地创建再接收远端同名事件，**当前提交尚待复测，不写成全绿**。run001 原源码与失败保留，不回写。

局部成本例程：

```sh
cargo build --release --locked --offline -p drive-core --example directory_construction_scale
./target/release/examples/directory_construction_scale 56007 4 3
```

例程在新临时数据库创建合成树，先安装索引，再测每个目录构造的 identity/count/inode/collect/commit 时长，并断言条目数量、父 inode 与零 inode 写入。对比时将同一例程逐字节移植到基准源码，所有构建完成后串行执行；它不含内核、FSearch、网络或 TTL，不能代替完整扫描。此提交尚无新的计时结果；完整原版 FSearch 和整合后安全策略的性能由后续独立窗口验证。


001 后续构建均退出 0，但微基准的两个工作区共用同一新 target，导致 fixed 的无 feature 例程复用 baseline 产物（SHA256 完全相同，且不含新的 identity SQL）。**这两个例程不能用于性能比较**；没有运行计时，也没有据此作提速声明。实际 FSearch fixture 包含新 SQL，已运行的新增 core 用例也进入测试二进制。后续配对构建必须使用两个独立目标目录，或在保留旧二进制后显式使新目标目录的相关产品 crate 重新构建，再验证源码／二进制身份；不能仅凭 cargo 成功退出断言两个工作区都已重新编译。001 原始结果保留，整体退出码为 1。
