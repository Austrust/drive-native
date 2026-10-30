"""Actual Qt transfer controls on an isolated native service; no credentials."""

import sys, time, resource
from unittest.mock import patch
from PySide6.QtWidgets import QApplication
from PySide6.QtCore import Qt
from PySide6.QtTest import QTest
from drive_native_gui.window import MainWindow

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
app = QApplication([])


def wait(test):
    end = time.monotonic() + 35
    while not test():
        if time.monotonic() > end:
            raise RuntimeError("transfer GUI deadline")
        app.processEvents()
        QTest.qWait(20)


with patch.object(MainWindow, "start", lambda self: None):
    window = MainWindow()
    window.show()
    app.processEvents()
    window.show_transfers()
    d = window.transfers_dialog
    wait(lambda: d.current is not None)
    if sys.argv[1] == "configure":
        d.download.setValue(64)
        d.upload.setValue(32)
        QTest.mouseClick(d.save, Qt.LeftButton)
        wait(lambda: not d.saving)
        assert d.current and d.current["limits"] == {"download_bps": 65536, "upload_bps": 32768}
    else:
        wait(
            lambda: d.table.topLevelItemCount() > 0
            and any(
                d.table.itemWidget(d.table.topLevelItem(i), 2) is not None
                and 0 < d.table.itemWidget(d.table.topLevelItem(i), 2).value() < 1000
                for i in range(d.table.topLevelItemCount())
            )
        )
        assert d.grab().save(sys.argv[2])
        if sys.argv[1] == "change":
            d.download.setValue(128)
            QTest.mouseClick(d.save, Qt.LeftButton)
            wait(lambda: not d.saving)
            assert d.current and d.current["limits"]["download_bps"] == 131072
    d.close()
    window.close()
print("PASS actual Qt transfer " + sys.argv[1])
