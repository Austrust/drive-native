"""Real GUI action against an already running isolated NAS fixture, no credentials."""
import sys, time, resource
from unittest.mock import patch
from PySide6.QtWidgets import QApplication
from PySide6.QtCore import Qt
from PySide6.QtTest import QTest
from drive_native_gui.window import MainWindow
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
app=QApplication([])
def wait(predicate):
    deadline=time.monotonic()+30
    while not predicate():
        if time.monotonic()>deadline:raise RuntimeError('GUI conflict test deadline')
        app.processEvents();QTest.qWait(25)
with patch.object(MainWindow,'start',lambda self: None):
    window=MainWindow();window.show();app.processEvents()
    QTest.mouseClick(window.issues_button,Qt.LeftButton)
    dialog=window.issues_dialog
    wait(lambda: len(dialog.rows)==1 and not dialog.loading)
    item=dialog.table.topLevelItem(0)
    dialog.table.setCurrentItem(item)
    assert dialog.keep.isEnabled(),dialog.message.text()
    if len(sys.argv)>2:
        assert next(iter(dialog.rows.values()))['reason']==sys.argv[2]
        if sys.argv[2]=='directory_creation_conflict':assert '整个本地文件夹' in dialog.explanation.text()
        if sys.argv[2]=='creation_conflict':assert '同名远端条目保持不变' in dialog.explanation.text()
    dialog.name.setText(sys.argv[3] if len(sys.argv)>3 else 'A (local conflict).doc')
    app.processEvents()
    assert dialog.grab().save(sys.argv[1])
    QTest.mouseClick(dialog.keep,Qt.LeftButton)
    wait(lambda:not dialog.busy and not dialog.loading and not dialog.rows)
    assert not dialog.keep.isEnabled()
    dialog.close();window.close()
print('PASS actual Qt button -> D-Bus KeepBoth -> refreshed empty issues')
