"""Real Qt certificate management against an isolated running test service."""

import sys, time, resource
from pathlib import Path
from unittest.mock import patch
from PySide6.QtWidgets import QApplication, QFileDialog
from PySide6.QtCore import Qt
from PySide6.QtTest import QTest
from drive_native_gui.window import MainWindow

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
app = QApplication([])


def wait(predicate):
    deadline = time.monotonic() + 30
    while not predicate():
        if time.monotonic() > deadline:
            raise RuntimeError("trust GUI deadline")
        app.processEvents()
        QTest.qWait(25)


with patch.object(MainWindow, "start", lambda self: None):
    window = MainWindow()
    window.show()
    app.processEvents()
    QTest.mouseClick(window.trust_settings_button, Qt.LeftButton)
    dialog = window.trust_dialog
    wait(lambda: dialog.current is not None and not dialog.busy)
    if sys.argv[1] == "add":
        with patch.object(QFileDialog, "getOpenFileName", return_value=(sys.argv[2], "")):
            QTest.mouseClick(dialog.editor.add, Qt.LeftButton)
        assert len(dialog.editor.certificates) == 1
    elif sys.argv[1] == "remove":
        assert len(dialog.editor.certificates) == 1
        dialog.editor.table.setCurrentItem(dialog.editor.table.topLevelItem(0))
        QTest.mouseClick(dialog.editor.remove, Qt.LeftButton)
        assert not dialog.editor.certificates
    else:
        raise RuntimeError("invalid trust action")
    app.processEvents()
    assert dialog.grab().save(sys.argv[3])
    QTest.mouseClick(dialog.save, Qt.LeftButton)
    wait(lambda: not dialog.busy and "已保存" in dialog.message.text())
    expected = 1 if sys.argv[1] == "add" else 0
    assert len(dialog.current["settings"]["trusted_ca_pem"]) == expected
    dialog.close()
    window.close()
print("PASS actual Qt trust import/remove -> task-bound D-Bus save")
