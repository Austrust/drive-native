# tests

源码与开发文件。当前状态及构建方法见仓库根 README。

- [cache_gui_live.py](cache_gui_live.py)
- [cache_gui_tests.py](cache_gui_tests.py)
- [backend_upgrade_test.py](backend_upgrade_test.py)：暂停失败、后台身份变化、等待旧进程正常退出、pidfd 生命周期及新后台身份/暂停/挂载确认；超时不强杀。
- [backend_upgrade_live.py](backend_upgrade_live.py)：私有 D-Bus 中实际 Qt 控制器识别旧进程、正常退出/启动新 FUSE、保持暂停及未上传编辑的合成联调；不访问 NAS。
- [pause_handover_test.py](pause_handover_test.py)：首次旧版升级所用任务暂停标记的原子持久化、属主/权限与符号链接校验。
- [conflict_gui_live.py](conflict_gui_live.py)
- [conflict_gui_tests.py](conflict_gui_tests.py)
- [gui_unit_tests.py](gui_unit_tests.py)
- [hidden_dialog_tests.py](hidden_dialog_tests.py)：隐藏设置页的重连、在途分页和已排队定时器回归。
- [lazy_gui_tests.py](lazy_gui_tests.py)：按需展开、关闭/隐藏窗口及在途分页停止，使用合成本地状态。
- [filemanager_provider_tests.py](filemanager_provider_tests.py)：模拟D-Bus、真实GI；分别用 `DRIVE_NATIVE_TEST_FILEMANAGER=nemo` 或 `nautilus` 运行。
- [installed_package_desktop.py](installed_package_desktop.py)
- [installed_host_smoke.py](installed_host_smoke.py)：安装包的真实 Nemo/Nautilus 窗口与本地后台状态刷新测试；仅接受 `multi_mount_fixture` 的无网络合成身份，在独立 D-Bus/Xvfb 中运行，不操作当前用户桌面。
- [keyring_control.py](keyring_control.py)
- [live_gui_test.py](live_gui_test.py)
- [managed_service_test.py](managed_service_test.py)
- [mounted_read_retry_test.py](mounted_read_retry_test.py)
- [multi_mount_tests.py](multi_mount_tests.py)
- [multi_nemo_check.py](multi_nemo_check.py)
- [multi_process_check.py](multi_process_check.py)
- [multi_ui_check.py](multi_ui_check.py)
- [nemo_host_test.py](nemo_host_test.py)
- [nemo_provider_test.py](nemo_provider_test.py)
- [secret_service_test.py](secret_service_test.py)
- [selector_gui_tests.py](selector_gui_tests.py)
- [service_launcher_test.py](service_launcher_test.py)
- [theme_tests.py](theme_tests.py)
- [transfer_gui_live.py](transfer_gui_live.py)
- [transfer_gui_tests.py](transfer_gui_tests.py)
- [trust_gui_live.py](trust_gui_live.py)
- [trust_gui_tests.py](trust_gui_tests.py)
