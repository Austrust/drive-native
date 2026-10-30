import sys
import resource
from PySide6.QtCore import QCoreApplication
from PySide6.QtWidgets import QApplication
import os
import argparse


def main():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    QCoreApplication.setOrganizationName("DriveNative")
    QCoreApplication.setApplicationName("Drive Native")
    parser = argparse.ArgumentParser()
    parser.add_argument("--task")
    parser.add_argument("--new-name")
    args = parser.parse_args()
    if args.task is not None:
        instance = "" if args.task == "legacy" else args.task
        from .config import config_path
        config_path(instance)
        os.environ["DRIVE_NATIVE_INSTANCE"] = instance
        if args.new_name:
            os.environ["DRIVE_NATIVE_NEW_NAME"] = args.new_name
    app = QApplication(sys.argv)
    from .theme import install_theme
    install_theme(app)
    if args.task is not None or os.environ.get("DRIVE_NATIVE_INSTANCE"):
        from .window import MainWindow
        window = MainWindow()
    else:
        from .manager import MountManager
        window = MountManager()
    window.show()
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
