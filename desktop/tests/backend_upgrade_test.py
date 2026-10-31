"""Cooperative handover does not restart a process with unfinished local work."""
import os
import subprocess
import unittest
from PySide6.QtCore import QCoreApplication, QObject, Signal
from drive_native_gui.upgrade import BackendUpgrade

app = QCoreApplication([])


class Bus(QObject):
    availability = Signal(bool)
    owner = ":synthetic.1"

    def __init__(self):
        super().__init__()
        self.calls = []

    def call(self, method, callback):
        self.calls.append((method, callback))


class Launcher(QObject):
    completed = Signal(object)

    def __init__(self):
        super().__init__()
        self.starts = 0

    def start(self):
        self.starts += 1


class UpgradeTest(unittest.TestCase):
    def setUp(self):
        self.bus, self.launcher = Bus(), Launcher()
        self.upgrade = BackendUpgrade(self.bus, self.launcher, save_pause=lambda: None)
        self.upgrade.check = lambda: None
        self.upgrade.available = True
        self.upgrade.owner = self.bus.owner
        self.results = []
        self.upgrade.finished.connect(self.results.append)

    def tearDown(self):
        self.upgrade.timer.stop()
        self.upgrade.detect_timer.stop()
        if self.upgrade.pidfd is not None:
            os.close(self.upgrade.pidfd)
        self.upgrade.deleteLater()

    def test_failed_pause_never_stops_or_starts(self):
        self.upgrade.begin()
        self.assertEqual(self.bus.calls[0][0], "Pause")
        self.bus.calls[0][1](None, "write_failed")
        self.assertEqual(len(self.bus.calls), 1)
        self.assertEqual(self.launcher.starts, 0)
        self.assertTrue(self.results[0])

    def test_owner_change_does_not_stop_replacement(self):
        self.upgrade.begin()
        self.bus.owner = ":synthetic.2"
        self.bus.calls[0][1](None, None)
        self.assertEqual(len(self.bus.calls), 1)
        self.assertEqual(self.launcher.starts, 0)

    def test_failed_startup_intent_does_not_stop_old_owner(self):
        def fail():
            raise OSError("synthetic disk full")
        self.upgrade.save_pause = fail
        self.upgrade.begin()
        self.bus.calls[0][1](None, None)
        self.assertEqual(len(self.bus.calls), 1)
        self.assertEqual(self.launcher.starts, 0)

    def test_busy_process_times_out_without_being_killed(self):
        process = subprocess.Popen(["/bin/sleep", "30"])
        try:
            self.upgrade.pidfd = os.pidfd_open(process.pid)
            self.upgrade.begin()
            self.bus.calls[0][1](None, None)
            self.assertEqual(self.bus.calls[1][0], "StopMount")
            self.upgrade.elapsed = 239
            self.upgrade._wait_exit()
            self.assertIsNone(process.poll())
            self.assertEqual(self.launcher.starts, 0)
            self.assertTrue(self.results[0])
        finally:
            process.terminate()
            process.wait()

    def test_new_backend_starts_only_after_old_pid_exits(self):
        process = subprocess.Popen(["/bin/sleep", "30"])
        try:
            self.upgrade.pidfd = os.pidfd_open(process.pid)
            self.upgrade.begin()
            self.bus.calls[0][1](None, None)
            self.upgrade._wait_exit()
            self.assertEqual(self.launcher.starts, 0)
            process.terminate()
            process.wait()
            self.upgrade._wait_exit()
            self.assertEqual(self.launcher.starts, 1)
            self.launcher.completed.emit(None)
            self.assertEqual(self.results, [None])
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait()


if __name__ == "__main__":
    unittest.main()
