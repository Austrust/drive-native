"""Cooperative handover does not restart a process with unfinished local work."""
import os
import subprocess
import unittest
from PySide6.QtCore import QCoreApplication, QEvent, QObject, Signal
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
        self.upgrade.pidfd = None
        self.upgrade.deleteLater()

    def test_parent_destruction_closes_controller_pidfd(self):
        parent = QObject()
        controller = BackendUpgrade(self.bus, self.launcher, parent, save_pause=lambda: None)
        fd = os.pidfd_open(os.getpid())
        controller.pidfd = fd
        parent.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        with self.assertRaises(OSError):
            os.fstat(fd)

    def test_replacing_pidfd_releases_previous_handle(self):
        first, second = os.pidfd_open(os.getpid()), os.pidfd_open(os.getpid())
        self.upgrade.pidfd = first
        self.upgrade.pidfd = second
        with self.assertRaises(OSError):
            os.fstat(first)
        os.fstat(second)

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
            self.upgrade.begin({"task_id": "synthetic"})
            self.bus.calls[0][1](None, None)
            self.upgrade._wait_exit()
            self.assertEqual(self.launcher.starts, 0)
            process.terminate()
            process.wait()
            self.upgrade._wait_exit()
            self.assertEqual(self.launcher.starts, 1)
            self.launcher.completed.emit(None)
            self.assertEqual(self.results, [])
            self.bus.owner = ":synthetic.2"
            self.upgrade._verify_installed_owner = lambda owner, callback: callback(owner, True)
            self.upgrade._confirm_started()
            self.assertEqual(self.bus.calls[-1][0], "GetOverview")
            self.bus.calls[-1][1]({"task_id": "synthetic", "connection": "paused", "mounted": True}, None)
            self.assertEqual(self.results, [None])
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait()

    def confirmation(self):
        self.upgrade.active = True
        self.upgrade.intent_profile = {"task_id": "synthetic"}
        self.launcher.completed.emit(None)
        self.assertEqual(self.results, [])
        self.bus.owner = ":synthetic.2"
        self.upgrade._confirm_started()
        self.assertEqual(self.bus.calls[-1][0], "GetOverview")
        return self.bus.calls[-1][1]

    def test_spawn_success_does_not_confirm_missing_backend(self):
        self.upgrade.active = True
        self.launcher.completed.emit(None)
        self.assertEqual(self.results, [])
        self.upgrade.elapsed = 239
        self.upgrade._wait_exit()
        self.assertTrue(self.results[0])

    def test_new_owner_with_wrong_task_is_not_confirmed(self):
        self.confirmation()({"task_id": "other", "connection": "paused", "mounted": True}, None)
        self.assertTrue(self.results[0])

    def test_new_owner_must_be_paused_and_mounted(self):
        for state, mounted in [("online", True), ("paused", False)]:
            with self.subTest(state=state, mounted=mounted):
                self.upgrade.confirm_pending = False
                self.bus.owner = ":synthetic.1"
                self.confirmation()({"task_id": "synthetic", "connection": state, "mounted": mounted}, None)
                self.assertEqual(self.results, [])

    def test_unmatched_binary_or_replaced_owner_does_not_confirm(self):
        received = self.confirmation()
        checks = []
        self.upgrade._verify_installed_owner = lambda owner, callback: checks.append((owner, callback))
        received({"task_id": "synthetic", "connection": "paused", "mounted": True}, None)
        self.assertEqual(self.results, [])
        owner, callback = checks[0]
        callback(owner, False)
        self.assertEqual(self.results, [])
        self.bus.owner = ":synthetic.3"
        callback(owner, True)
        self.assertEqual(self.results, [])


    def test_stage_progress_is_conservative_and_additive(self):
        stages = []
        self.upgrade.stage_changed.connect(stages.append)
        self.upgrade.begin()
        self.assertEqual(stages, ["pausing"])
        self.bus.calls[0][1](None, None)
        self.assertEqual(stages, ["pausing", "persisting_pause", "stopping", "waiting_exit"])
        self.assertEqual(self.bus.calls[1][0], "StopMount")

    def test_version_mismatch_blocks_confirmation(self):
        received = self.confirmation()
        received({"task_id": "synthetic", "connection": "paused", "mounted": True,
                  "version": {"build": "0.1.0", "state_schema": 25, "ipc": 21}}, None)
        self.assertTrue(self.results[0])
        self.assertIn("版本", self.results[0])
        self.assertEqual(self.upgrade.stage, "failed")

    def test_missing_version_does_not_block_confirmation(self):
        received = self.confirmation()
        self.upgrade._verify_installed_owner = lambda owner, callback: callback(owner, True)
        received({"task_id": "synthetic", "connection": "paused", "mounted": True}, None)
        self.assertEqual(self.results, [None])
        self.assertIsNone(self.upgrade.version)

    def test_optional_backend_stage_is_displayed_before_completion(self):
        received = self.confirmation()
        self.upgrade._verify_installed_owner = lambda owner, callback: callback(owner, True)
        stages = []
        self.upgrade.stage_changed.connect(stages.append)
        received({"task_id": "synthetic", "connection": "paused", "mounted": True,
                  "upgrade_stage": "waiting_exit",
                  "version": {"build": "dev", "state_schema": 26, "ipc": 21}}, None)
        self.assertTrue(self.upgrade.stage_supported)
        self.assertIn("waiting_exit", stages)
        self.assertEqual(self.results, [None])

    def test_version_and_upgrade_helpers_fail_closed(self):
        from drive_native_gui.upgrade import (
            available_upgrades, normalize_version, stage_text, version_state, version_summary,
        )
        self.assertEqual(normalize_version({"build": "0.2", "state_schema": 26, "ipc": 21}),
                         {"build": "0.2", "state_schema": 26, "ipc": 21})
        self.assertIsNone(normalize_version(None))
        self.assertIsNone(normalize_version({"build": "", "state_schema": 0, "ipc": None}))
        self.assertEqual(normalize_version({"state_schema": 77}),
                         {"build": None, "state_schema": 77, "ipc": None})
        self.assertEqual(version_state(None), "missing")
        self.assertEqual(version_state({"state_schema": 26, "ipc": 21}), "ok")
        self.assertEqual(version_state({"state_schema": 25, "ipc": 21}), "mismatch")
        self.assertEqual(version_summary({"build": "dev", "state_schema": 26, "ipc": 21}),
                         "dev · 状态库 v26 · IPC v21")
        self.assertEqual(stage_text("waiting_exit"), "正在等待旧后台退出")
        self.assertEqual(stage_text("invented"), "")

        class U:
            def __init__(self, available):
                self.available = available

        self.assertEqual(available_upgrades([U(True), U(False), None]), [0])
        self.assertEqual(available_upgrades({"a": U(True), "b": U(False)}), ["a"])


if __name__ == "__main__":
    unittest.main()
