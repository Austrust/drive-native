# drive_native_gui

源码与开发文件。当前状态及构建方法见仓库根 README。

[统一主窗口与托盘](UNIFIED_WINDOW.md)说明多挂载界面、任务隔离、轻量停止诊断及持久历史；实测状态见[实现报告](../../docs/FIX_5_2_REPORT.md)。

- [__init__.py](__init__.py)
- [__main__.py](__main__.py)
- [bus.py](bus.py)
- [cache_settings.py](cache_settings.py)
- [config.py](config.py)
- [credentials.py](credentials.py)
- [issues.py](issues.py)
- [history.py](history.py)：持久历史、按服务保存身份及当前目标解析。
- [permissions.py](permissions.py)：权限四态与带 revision 的读写策略设置。
- [lifecycle.py](lifecycle.py)
- [upgrade.py](upgrade.py)：识别仍运行的旧后台，经暂停和正常退出后接管；不强杀进程。
- [pause_handover.py](pause_handover.py)：旧版暂停尚不持久化时，通过任务启动标记把暂停意图交给新后台；不直接写运行中的状态库。
- [manager.py](manager.py)：单主窗口与按需嵌入的任务页面。
- [tray.py](tray.py)：当前活动与托盘入口。
- [instance.py](instance.py)：配置根隔离的会话 GUI 激活。
- [shutdown.py](shutdown.py)：不依赖 Overview 的停止阶段查询。
- [setup_client.py](setup_client.py)
- [startup.py](startup.py)
- [theme.py](theme.py)
- [transfers.py](transfers.py)
- [trust.py](trust.py)
- [window.py](window.py)
- [wizard.py](wizard.py)
