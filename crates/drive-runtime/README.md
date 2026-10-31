# drive-runtime

源码与开发文件。当前状态及构建方法见仓库根 README。

- [Cargo.toml](Cargo.toml)
- [examples](examples/)
- [src](src/)
- [tests](tests/)

[桌面分页对比工具](examples/desktop_page_bench.rs)在2000子项合成目录上串行交替比较旧完整记录路径和数据库分页路径，核对响应一致并保留32对样本；不访问NAS或FUSE，不代表整目录性能验收。使用 `cargo run --locked -p drive-runtime --example desktop_page_bench`（本轮为debug构建）。
