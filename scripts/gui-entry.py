"""Frozen GUI entry point; no account material is embedded in the build."""

import sys

if sys.argv[1:] == ["--unlock-service"]:
    from drive_native_gui.startup import main

    raise SystemExit(main())
elif sys.argv[1:] == ["--verify-runtime"]:
    import json
    import secretstorage
    from PySide6 import QtCore, QtDBus, QtWidgets

    app = QtWidgets.QApplication([sys.argv[0]])
    print(
        json.dumps(
            {
                "passed": True,
                "qt": QtCore.qVersion(),
                "dbus": bool(QtDBus.QDBusConnection),
                "secretstorage_imported": bool(secretstorage),
                "account_accessed": False,
            }
        )
    )
else:
    from drive_native_gui.__main__ import main

    raise SystemExit(main())
