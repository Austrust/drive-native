# drive_native_gui

源码与开发文件。当前状态及构建方法见仓库根 README。

- [__init__.py](__init__.py)
- [__main__.py](__main__.py)
- [bus.py](bus.py)
- [cache_settings.py](cache_settings.py)
- [config.py](config.py)
- [credentials.py](credentials.py)
- [issues.py](issues.py)
- [lifecycle.py](lifecycle.py)
- [upgrade.py](upgrade.py)：识别仍运行的旧后台，经暂停和正常退出后接管；不强杀进程。
- [pause_handover.py](pause_handover.py)：旧版暂停尚不持久化时，通过任务启动标记把暂停意图交给新后台；不直接写运行中的状态库。
- [manager.py](manager.py)
- [setup_client.py](setup_client.py)
- [startup.py](startup.py)
- [theme.py](theme.py)
- [transfers.py](transfers.py)
- [trust.py](trust.py)
- [window.py](window.py)
- [wizard.py](wizard.py)
