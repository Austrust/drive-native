"""Actual Qt settings action on the isolated live service. No NAS credentials."""
import sys,time,resource
from unittest.mock import patch
from PySide6.QtWidgets import QApplication
from PySide6.QtCore import Qt,QPoint
from PySide6.QtTest import QTest
from drive_native_gui.window import MainWindow
resource.setrlimit(resource.RLIMIT_CORE,(0,0))
app=QApplication([])
def wait(predicate):
    deadline=time.monotonic()+30
    while not predicate():
        if time.monotonic()>deadline:raise RuntimeError('cache settings deadline')
        app.processEvents();QTest.qWait(25)
with patch.object(MainWindow,'start',lambda self:None):
    window=MainWindow();window.show();app.processEvents();QTest.mouseClick(window.cache_settings_button,Qt.LeftButton)
    dialog=window.cache_dialog;wait(lambda:dialog.loaded and not dialog.loading)
    assert not dialog.enabled.isChecked()
    QTest.mouseClick(dialog.enabled,Qt.LeftButton,pos=QPoint(10,dialog.enabled.height()//2));assert dialog.enabled.isChecked();dialog.unit.setCurrentText('MiB');dialog.amount.setValue(1)
    assert dialog.save_button.isEnabled();QTest.mouseClick(dialog.save_button,Qt.LeftButton)
    wait(lambda:not dialog.saving and not dialog.dirty)
    dialog.refresh();wait(lambda:not dialog.loading)
    assert dialog.enabled.isChecked() and dialog.amount.value()==1 and dialog.unit.currentText()=='MiB'
    assert dialog.grab().save(sys.argv[1]);dialog.close();window.close()
print('PASS actual Qt cache settings save -> task-bound D-Bus -> persistent policy')
