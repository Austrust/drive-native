"""GUI activation over an isolated session bus; no backend calls."""
import os
import sys
import time
import tempfile
import unittest
from unittest.mock import patch
from PySide6.QtCore import QProcess
from PySide6.QtWidgets import QApplication
from drive_native_gui.instance import GuiInstance, validate_activation

APP = QApplication.instance() or QApplication([])


class InstanceTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name})
        self.env.start()
        self.guard = GuiInstance(APP)
    def tearDown(self):
        if self.guard.primary and self.guard.bus.isConnected():
            self.guard.bus.unregisterObject('/org/drivenative/Gui1')
            self.guard.bus.unregisterService(self.guard.name)
        self.guard.deleteLater(); self.env.stop(); self.tmp.cleanup()

    def test_strict_activation_and_startup_queue(self):
        calls = []
        self.assertTrue(self.guard.Activate('a' * 32, 'Documents'))
        self.assertFalse(self.guard.Activate('../escape', ''))
        self.assertFalse(self.guard.Activate('a' * 32, '../escape'))
        self.assertFalse(self.guard.Activate('', 'unexpected'))
        self.guard.bind(lambda *args: calls.append(args))
        self.assertEqual(calls, [('a' * 32, 'Documents')])
        self.guard.Activate('legacy', '')
        self.assertEqual(calls[-1], ('legacy', ''))

    def test_configuration_roots_have_distinct_gui_service_names(self):
        first = self.guard.name
        with patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name + '/other'}):
            other = GuiInstance(APP)
        self.assertNotEqual(first, other.name)
        other.deleteLater()

    def test_second_process_activates_existing_gui_without_new_manager(self):
        if not self.guard.bus.isConnected():
            self.skipTest('Run under dbus-run-session for real activation coverage')
        calls = []
        self.guard.bind(lambda *args: calls.append(args))
        self.assertTrue(self.guard.claim_or_activate())
        process = QProcess()
        code = "from PySide6.QtWidgets import QApplication; from drive_native_gui.instance import GuiInstance; a=QApplication([]); g=GuiInstance(a); assert g.claim_or_activate('b'*32, '') is False"
        process.start(sys.executable, ['-c', code])
        deadline = time.monotonic() + 8
        while process.state() != QProcess.NotRunning and time.monotonic() < deadline:
            APP.processEvents(); time.sleep(.005)
        if process.state() != QProcess.NotRunning:
            process.kill(); process.waitForFinished(1000)
            self.fail('Activation subprocess timed out')
        self.assertEqual(process.exitCode(), 0, bytes(process.readAllStandardError()).decode())
        self.assertEqual(calls, [('b' * 32, '')])


if __name__ == '__main__': unittest.main()
