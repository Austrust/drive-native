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
    app = QApplication(sys.argv)
    from .theme import install_theme
    install_theme(app)
    from .instance import GuiInstance
    from .manager import MountManager
    task = args.task or os.environ.get("DRIVE_NATIVE_INSTANCE", "")
    instance = GuiInstance(app)
    try:
        if not instance.claim_or_activate(task, args.new_name or ""):
            return 0
    except (ValueError, RuntimeError) as error:
        from PySide6.QtWidgets import QMessageBox
        QMessageBox.warning(None, "Drive Native", str(error))
        return 1
    window = MountManager()
    window.activate(task, args.new_name or "")
    instance.bind(window.activate)
    window.show()
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
