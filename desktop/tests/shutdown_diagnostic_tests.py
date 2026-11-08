import json
import unittest

from drive_native_gui.shutdown import (
    BLOCKING_REASONS, OVERDUE_SECONDS, STOP_DEADLINE_SECONDS, PHASES,
    ShutdownDeadline, blocking_reason_text, shutdown_text,
)
from drive_native_gui.issue_actions import diagnostic_text


class FakeClock:
    def __init__(self):
        self.now = 1000.0

    def __call__(self):
        return self.now


class ShutdownTextTests(unittest.TestCase):
    def test_every_phase_has_a_label_and_unknown_is_empty(self):
        for phase in PHASES:
            self.assertTrue(shutdown_text({"phase": phase}))
        self.assertEqual(shutdown_text(None), "")
        self.assertEqual(shutdown_text({"phase": "invented"}), "")
        self.assertEqual(shutdown_text("draining_filesystem"), "")

    def test_overdue_notice_is_only_claimed_for_active_stop(self):
        active = {"phase": "draining_filesystem", "stage_elapsed_ms": 31000, "overdue": True}
        text = shutdown_text(active)
        self.assertIn("31 秒", text)
        self.assertIn("耗时超过预期", text)
        self.assertIn("尚未确认停止", text)
        complete = {"phase": "complete", "stage_elapsed_ms": 31000, "overdue": True}
        self.assertNotIn("耗时超过预期", shutdown_text(complete))

    def test_blocking_reason_enum_is_closed(self):
        for reason in BLOCKING_REASONS:
            self.assertTrue(blocking_reason_text({"blocking_reason": reason}))
        self.assertEqual(blocking_reason_text({"blocking_reason": "raw error text"}), "")
        self.assertEqual(blocking_reason_text({}), "")
        text = shutdown_text({"phase": "draining_filesystem", "blocking_reason": "draining_open_files"})
        self.assertIn("阻塞原因：", text)
        self.assertIn(BLOCKING_REASONS["draining_open_files"], text)
        complete = shutdown_text({"phase": "complete", "blocking_reason": "draining_open_files"})
        self.assertNotIn("阻塞原因", complete)

    def test_deadline_notice_never_claims_success_or_kill(self):
        value = {"phase": "checkpointing_local", "elapsed_ms": 91000}
        text = shutdown_text(value, deadline_reached=True)
        self.assertIn(f"已等待超过 {STOP_DEADLINE_SECONDS} 秒", text)
        self.assertIn("不会强制结束进程", text)
        self.assertIn("复制诊断", text)
        self.assertNotIn("已停止", text)
        self.assertEqual(shutdown_text({"phase": "complete"}, deadline_reached=True),
                         shutdown_text({"phase": "complete"}))

    def test_deadline_uses_monotonic_clock_and_backend_offset(self):
        clock = FakeClock()
        deadline = ShutdownDeadline(limit=90, clock=clock)
        self.assertFalse(deadline.update(None))
        self.assertFalse(deadline.update({"phase": "idle"}))
        active = {"phase": "detaching_mount", "elapsed_ms": 20000}
        self.assertFalse(deadline.update(active))
        clock.now += 69.0
        self.assertFalse(deadline.update(active))
        clock.now += 1.5
        self.assertTrue(deadline.update(active))
        self.assertIn("已等待超过 90 秒", deadline.render(active))
        self.assertFalse(deadline.update({"phase": "complete"}))
        self.assertFalse(deadline.update({"phase": "failed"}))

    def test_overdue_constant_matches_backend_stage_budget(self):
        self.assertEqual(OVERDUE_SECONDS, 30)


class DiagnosticWhitelistTests(unittest.TestCase):
    def test_shutdown_and_connection_diagnostics_are_closed_tokens(self):
        overview = {
            "connection": "online",
            "shutdown": {
                "phase": "draining_filesystem", "blocking_reason": "draining_open_files",
                "elapsed_ms": 91000, "stage_elapsed_ms": 31000, "overdue": True,
                "secret": "raw backend text",
            },
            "connection_diagnostics": {
                "load_failed": False, "persistence_failed": True,
                "last_transition": {"phase": "resume", "category": "network",
                                    "recovery": "automatic", "message": "secret"},
                "last_failure": {"phase": "pull/../etc", "category": "Bad Token",
                                 "recovery": "retry"},
            },
            "background_error_kind": "network_unavailable",
            "background_error_since_ms": 123456,
            "sync_state": "checking",
            "issue_count": 3,
            "cache_availability": {"state": "partial", "offline": 2, "pinned": 0,
                                   "total": 5, "checked": True, "extra": "hidden"},
        }
        text = diagnostic_text(overview)
        data = json.loads(text)
        self.assertEqual(data["shutdown"]["phase"], "draining_filesystem")
        self.assertEqual(data["shutdown"]["blocking_reason"], "draining_open_files")
        self.assertEqual(data["shutdown"]["elapsed_ms"], 91000)
        self.assertTrue(data["shutdown"]["overdue"])
        self.assertNotIn("secret", text)
        self.assertEqual(data["connection_diagnostics"]["last_transition"],
                         {"phase": "resume", "category": "network", "recovery": "automatic"})
        self.assertEqual(data["connection_diagnostics"]["last_failure"], {"recovery": "retry"})
        self.assertEqual(data["background_error_kind"], "network_unavailable")
        self.assertEqual(data["background_error_since_ms"], 123456)
        self.assertEqual(data["sync_state"], "checking")
        self.assertEqual(data["issue_count"], 3)
        self.assertEqual(data["cache_availability"]["state"], "partial")
        self.assertNotIn("extra", data["cache_availability"])

    def test_older_backend_without_new_fields_stays_bounded(self):
        data = json.loads(diagnostic_text({"connection": "online", "mounted": True}))
        self.assertNotIn("shutdown", data)
        self.assertNotIn("connection_diagnostics", data)
        self.assertNotIn("cache_availability", data)

    def test_unknown_enum_values_are_replaced(self):
        overview = {
            "shutdown": {"phase": "raw-phase", "blocking_reason": "raw-reason"},
            "connection_diagnostics": {"last_transition": {"phase": "UPPER CASE"}},
            "sync_state": "secret-state",
            "cache_availability": {"state": "nonsense"},
        }
        text = diagnostic_text(overview)
        data = json.loads(text)
        self.assertNotIn("phase", data["shutdown"])
        self.assertEqual(data["shutdown"]["blocking_reason"], "unrecognized")
        self.assertNotIn("connection_diagnostics", data)
        self.assertNotIn("sync_state", data)
        self.assertEqual(data["cache_availability"]["state"], "unrecognized")
        self.assertNotIn("secret-state", text)


if __name__ == '__main__':
    unittest.main()
