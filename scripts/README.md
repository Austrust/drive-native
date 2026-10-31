# scripts

源码与开发文件。当前状态及构建方法见仓库根 README。

- [analyze_server_elf.py](analyze_server_elf.py)
- [build-linux.sh](build-linux.sh)
- [directory_author_probe.py](directory_author_probe.py)
- [dsm_auth_probe.py](dsm_auth_probe.py)
- [extract_contracts.py](extract_contracts.py)
- [extract_windows_conflict.py](extract_windows_conflict.py)
- [extract_windows_contract.py](extract_windows_contract.py)
- [extract_windows_directory_author.py](extract_windows_directory_author.py)
- [extract_windows_download.py](extract_windows_download.py)
- [extract_windows_namespace_paths.py](extract_windows_namespace_paths.py)
- [extract_windows_root_paths.py](extract_windows_root_paths.py)
- [extract_windows_snapshot.py](extract_windows_snapshot.py)
- [gui-entry.py](gui-entry.py)
- [gui-requirements.lock](gui-requirements.lock)
- [native_probe.py](native_probe.py)
- [office_roundtrip.py](office_roundtrip.py)
- [package.py](package.py)
- [prepare-debian-source.py](prepare-debian-source.py)
- [prepare-gui-build.sh](prepare-gui-build.sh)
- [source_snapshot.py](source_snapshot.py)
- [stage-linux.sh](stage-linux.sh)
- [stage-docs.py](stage-docs.py)：安装可离线阅读的说明，并将外部证据链接固定到构建提交。
- [test_prepare_debian_source.py](test_prepare_debian_source.py)
- [test-gui.sh](test-gui.sh)：私有配置下逐进程运行合成 GUI 回归，纳入 debhelper 测试门禁；真实桌面与 NAS 验证仍单独执行。
- [verify-runtime.sh](verify-runtime.sh)
- [verify.sh](verify.sh)

## 实验脚本

[validation](validation/README.md) 提供 P0 基线、固定夹具、断连探针和独立校验工具；验证状态见仓库根 DELIVERY_GATES.md。

`native_*`、`*_probe` 及 examples 中的真实联调程序是历史实验工具，其中部分会写入测试目录。主机已替换为不可用示例地址；必须阅读工具的测试范围并配置隔离账号后显式运行。普通客户端不依赖这些脚本，自动测试不运行真实 NAS 写入。
