"""Actionable recovery uses local asynchronous API contracts, never NAS fixtures."""
import json
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QTimer, QCoreApplication, QEvent, Qt
from PySide6.QtWidgets import QApplication
from drive_native_gui.issues import IssuesDialog
from drive_native_gui.issue_actions import parent_candidates, diagnostic_text, NEXT_STEPS, recheck_message
from drive_native_gui.window import MainWindow

app = QApplication.instance() or QApplication([])


class Bus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ":1.synthetic"
    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.calls = []
    def call(self, method, args=(), callback=None):
        self.calls.append((method, list(args), callback))
    def respond(self, method, data=None, error=None):
        index = next(i for i, row in enumerate(self.calls) if row[0] == method and row[2] is not None)
        _, _, callback = self.calls.pop(index)
        callback(data, error)


class RecoveryTests(unittest.TestCase):
    def setUp(self):
        self.bus = Bus()
        self.dialog = IssuesDialog(self.bus)
        self.dialog.show(); app.processEvents()
        self.dialog.timer.stop(); self.bus.calls.clear()
        self.dialog.loading = False; self.dialog.context_loading = False
        self.context = dict(schema=1, connection="online", mounted=True, mount="/tmp/synthetic-mount", task_id="synthetic-task", pending_operations=3,
                            recovery_actions={"recheck_issues": True})
        self.dialog.update_context(self.context)
        self.row = dict(id="file", token="private-token", path="gone/parent/file.txt", reason="commit_uncertain", can_keep_both=False, can_keep_directory=False)
        self.populate(self.row)

    def populate(self, row):
        self.dialog.received("", self.dialog.generation, {"entries": [row], "next": None}, None)
        self.dialog.table.setCurrentItem(self.dialog.table.topLevelItem(0))

    def tearDown(self):
        self.dialog.close(); self.dialog.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)

    def test_every_non_keep_reason_has_specific_next_step_and_safe_actions(self):
        for reason in NEXT_STEPS:
            self.populate(dict(self.row, reason=reason))
            self.assertIn(NEXT_STEPS[reason], self.dialog.explanation.text())
            self.assertFalse(self.dialog.keep.isEnabled())
            self.assertTrue(self.dialog.copy_details.isEnabled())
            self.assertTrue(self.dialog.pause_sync.isEnabled())
        self.assertFalse(any(c[0] in ("KeepBoth", "KeepDirectoryBoth", "Resume", "ReleaseSpace") for c in self.bus.calls))

    def test_recheck_is_async_not_resolution_or_automatic_resume(self):
        heartbeat = []
        self.dialog.request_recheck()
        QTimer.singleShot(0, lambda: heartbeat.append(True)); app.processEvents()
        self.assertEqual(heartbeat, [True]); self.assertTrue(self.dialog.busy)
        self.assertEqual(sum(c[0] == "RecheckIssues" for c in self.bus.calls), 1)
        self.dialog.request_recheck()
        self.assertEqual(sum(c[0] == "RecheckIssues" for c in self.bus.calls), 1)
        self.bus.respond("RecheckIssues", {"schema": 1, "accepted": True, "paused_preserved": True, "message_code": "queued"})
        self.assertIn("尚未确认", self.dialog.message.text())
        self.assertIn("file", self.dialog.rows)
        self.assertFalse(any(c[0] == "Resume" for c in self.bus.calls))

    def test_paused_reply_and_legacy_capability_cannot_resume_task(self):
        self.dialog.update_context(dict(self.context, connection="paused"))
        self.assertFalse(self.dialog.pause_sync.isEnabled())
        self.dialog.request_recheck()
        self.bus.respond("RecheckIssues", {"schema": 1, "accepted": False, "paused_preserved": True, "message_code": "paused"})
        self.assertIn("暂停状态保持", self.dialog.message.text())
        self.dialog.update_context({"connection": "online"})
        self.assertFalse(self.dialog.recheck.isEnabled())
        self.assertIn("未提供", self.dialog.recheck.toolTip())
        self.dialog.request_recheck()
        self.assertFalse(any(c[0] in ("RecheckIssues", "Resume") for c in self.bus.calls))

    def test_busy_recheck_reply_is_retryable_and_never_claims_resolution(self):
        self.dialog.request_recheck()
        self.bus.respond("RecheckIssues", {"schema": 1, "accepted": False, "paused_preserved": True, "message_code": "busy"})
        self.assertFalse(self.dialog.busy)
        self.assertTrue(self.dialog.recheck.isEnabled())
        self.assertIn("本次未安排核对", self.dialog.message.text())
        self.assertIn("稍后重试", self.dialog.message.text())
        self.assertIn("file", self.dialog.rows)
        self.assertFalse(any(c[0] in ("Resume", "KeepBoth", "KeepDirectoryBoth") for c in self.bus.calls))

    def test_recheck_failure_preserves_issue_and_does_not_claim_success(self):
        self.dialog.request_recheck()
        self.bus.respond("RecheckIssues", None, "service_changed")
        self.assertIn("未确认", self.dialog.message.text())
        self.assertIn("file", self.dialog.rows)
        self.assertFalse(self.dialog.busy)

    def test_pause_changes_view_only_after_backend_ack(self):
        self.dialog.pause_task()
        self.assertEqual(self.dialog.overview["connection"], "online")
        self.bus.respond("Pause", None, "local_storage_error")
        self.assertEqual(self.dialog.overview["connection"], "online")
        self.dialog.pause_task(); self.bus.respond("Pause")
        self.assertEqual(self.dialog.overview["connection"], "paused")
        self.assertIn("缓存和未上传修改保留", self.dialog.message.text())

    def test_parent_lookup_falls_back_without_stat_or_content_download(self):
        with patch("drive_native_gui.issues.QDesktopServices.openUrl", return_value=True) as opened, patch("pathlib.Path.stat", side_effect=AssertionError("GUI must not stat FUSE")):
            self.dialog.open_available_parent()
            self.assertEqual(self.bus.calls[-1][:2], ("GetDirectory", ["gone/parent", 0]))
            self.bus.respond("GetDirectory", None, "permission_denied"); app.processEvents()
            self.assertEqual(self.bus.calls[-1][:2], ("GetDirectory", ["gone", 0]))
            self.bus.respond("GetDirectory", {"path": "gone", "entries": []})
            self.assertEqual(opened.call_args.args[0].toLocalFile(), "/tmp/synthetic-mount/gone")
        self.assertFalse(any(c[0] in ("Download", "RetryDownloads") for c in self.bus.calls))

    def test_parent_reply_after_hide_or_service_change_never_opens(self):
        with patch("drive_native_gui.issues.QDesktopServices.openUrl") as opened:
            self.dialog.open_available_parent(); self.dialog.hide()
            self.bus.respond("GetDirectory", {"path": "gone/parent", "entries": []})
            opened.assert_not_called()
            self.dialog.show(); self.dialog.timer.stop(); self.dialog.loading = False
            self.dialog.open_available_parent()
            self.bus.respond("GetDirectory", None, "service_changed")
            app.processEvents(); opened.assert_not_called()

    def test_preview_pending_is_not_an_accessible_empty_parent(self):
        with patch("drive_native_gui.issues.QDesktopServices.openUrl") as opened:
            self.dialog.open_available_parent()
            self.bus.respond("GetDirectory", {"path": "gone/parent", "entries": [], "preview_only": True, "directory_state": "pending"})
            app.processEvents(); opened.assert_not_called()
            self.assertEqual(self.bus.calls[-1][:2], ("GetDirectory", ["gone", 0]))

    def test_parent_timeout_is_bounded_and_allows_user_retry(self):
        with patch("drive_native_gui.issues.QDesktopServices.openUrl") as opened:
            self.dialog.open_available_parent()
            self.bus.respond("GetDirectory", None, "org.freedesktop.DBus.Error.NoReply")
            app.processEvents(); opened.assert_not_called()
            self.assertFalse(self.dialog.busy)
            self.assertTrue(self.dialog.open_parent.isEnabled())
            self.assertIn("超时", self.dialog.message.text())
            self.assertFalse(any(c[0] == "GetDirectory" for c in self.bus.calls))

    def test_recheck_reply_must_match_acceptance_and_pause_contract(self):
        for body in ({}, None, {"schema": 1, "message_code": "queued", "accepted": False, "paused_preserved": True},
                     {"schema": 1, "message_code": "queued", "accepted": True, "paused_preserved": False}):
            self.assertIn("未返回可识别", recheck_message(body))
        for code in ("login_required", "settings_required", "stopping"):
            self.assertNotIn("未返回可识别", recheck_message(dict(schema=1, message_code=code, accepted=False, paused_preserved=True)))

    def test_parent_paths_are_bounded_and_cannot_escape_mount(self):
        for path in ("/etc/passwd", "../escape", "a/../b", "a//b", "a\\b", "a/./b", "a\0b", ""):
            self.assertEqual(parent_candidates(path), [])
        values = parent_candidates("/".join(["a"] * 200))
        self.assertLessEqual(len(values), 33); self.assertEqual(values[-1], "")

    def test_copy_diagnostic_uses_closed_whitelist_not_raw_issue_or_overview(self):
        self.dialog.update_context(dict(self.context, username="secret-user", password="secret-password", upload_block="private-path", error="secret_token", queue_health={"namespace_uncertain": 1, "upload_uncertain": True, "private": "secret"}))
        self.dialog.copy_diagnostic(); value = QApplication.clipboard().text()
        for text in ("secret", "private-token", "gone", "private-path", "synthetic-mount", "synthetic-task"):
            self.assertNotIn(text, value)
        data = json.loads(value)
        self.assertEqual(data["queue_health"], {"namespace_uncertain": 1})
        self.assertTrue(data["upload_block_present"])
        self.assertEqual(data["error"], "unrecognized")

    def test_attention_explains_actual_next_step_without_generic_relogin(self):
        self.dialog.update_context(dict(self.context, connection="needs_attention", error="native_index_capacity"))
        self.assertIn("复制诊断", self.dialog.context_notice.text())
        self.assertIn("反复登录不能", self.dialog.context_notice.text())
        self.dialog.update_context(dict(self.context, connection="needs_attention", error="native_root_unavailable"))
        self.assertIn("同名新目录不能", self.dialog.context_notice.text())

    def test_untrusted_labels_are_plain_text(self):
        value = "<img src='file:///never-open'/>"
        self.dialog.update_context(dict(self.context, upload_block=value))
        self.populate(dict(self.row, path=value))
        self.assertEqual(self.dialog.context_notice.textFormat(), Qt.PlainText)
        self.assertEqual(self.dialog.explanation.textFormat(), Qt.PlainText)
        self.assertIn(value, self.dialog.context_notice.text())

    def test_online_upload_block_visible_and_opens_problem_centre(self):
        with patch.object(MainWindow, "start", lambda _: None), patch("drive_native_gui.window.ClientBus", Bus):
            window = MainWindow()
            try:
                window.show(); app.processEvents(); window.refresh_timer.stop()
                value = dict(self.context, upload_block="dependent operation", scope_path="/Lab")
                window.received_overview(value, None)
                self.assertEqual(window.connection_value.text(), "已连接")
                self.assertFalse(window.upload_block_label.isHidden())
                self.assertEqual(window.upload_block_label.textFormat(), Qt.PlainText)
                self.assertFalse(window.upload_block_action.isHidden())
                window.upload_block_action.click()
                self.assertIsNotNone(window.issues_dialog)
                self.assertIn("dependent operation", window.issues_dialog.context_notice.text())
                self.assertTrue(window.issues_dialog.isVisible())
                window.received_overview(dict(value, upload_block=None), None)
                self.assertTrue(window.upload_block_label.isHidden())
                self.assertTrue(window.upload_block_action.isHidden())
            finally:
                window.close(); window.deleteLater()
                QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)


    def test_exhausted_commit_uncertain_keeps_recheck_and_blocks_resolution(self):
        self.populate(dict(self.row, reconcile_attempts=5, reconcile_exhausted=True))
        item = self.dialog.table.topLevelItem(0)
        self.assertIn("已自动只读核对 5 次仍未确认", item.text(1))
        self.assertIn("自动重试已暂停", item.text(1))
        self.assertIn(NEXT_STEPS["commit_uncertain"], self.dialog.explanation.text())
        self.assertTrue(self.dialog.recheck.isEnabled())
        self.assertFalse(self.dialog.keep.isEnabled())
        self.dialog.keep_both()
        self.dialog.request_recheck()
        self.assertEqual(sum(c[0] == "RecheckIssues" for c in self.bus.calls), 1)
        self.bus.respond("RecheckIssues", {"schema": 1, "accepted": True, "paused_preserved": True, "message_code": "queued"})
        self.assertFalse(any(c[0] in ("KeepBoth", "KeepDirectoryBoth", "Resume", "ReleaseSpace") for c in self.bus.calls))
        self.assertIn("file", self.dialog.rows)

    def test_exhausted_fields_never_invent_a_state_or_action(self):
        for extra in ({"reconcile_exhausted": True, "reconcile_attempts": None},
                      {"reconcile_exhausted": "true", "reconcile_attempts": 5},
                      {"reconcile_exhausted": True, "reconcile_attempts": -1},
                      {"reconcile_exhausted": True, "reconcile_attempts": True},
                      {"reconcile_exhausted": False, "reconcile_attempts": 5}):
            self.populate(dict(self.row, **extra))
            self.assertEqual(self.dialog.table.topLevelItem(0).text(1), "服务器执行结果尚未确认")
        self.assertFalse(self.dialog.keep.isEnabled())
        self.assertFalse(any(c[0] in ("KeepBoth", "KeepDirectoryBoth") for c in self.bus.calls))

    def test_diagnostic_includes_reconcile_budget_without_identity(self):
        self.populate(dict(self.row, reconcile_attempts=5, reconcile_exhausted=True))
        self.dialog.copy_diagnostic(); value = QApplication.clipboard().text()
        data = json.loads(value)
        self.assertEqual(data["reconcile_attempts"], 5)
        self.assertTrue(data["reconcile_exhausted"])
        for text in ("private-token", "gone", "synthetic-mount", "synthetic-task"):
            self.assertNotIn(text, value)

    def test_diagnostic_ignores_invalid_reconcile_fields(self):
        for extra in ({"reconcile_exhausted": "yes", "reconcile_attempts": True},
                      {"reconcile_exhausted": None, "reconcile_attempts": -1},
                      {"reconcile_exhausted": 1, "reconcile_attempts": 1 << 32}):
            self.populate(dict(self.row, **extra))
            data = json.loads(diagnostic_text(self.dialog.overview, self.dialog.selected_row()))
            self.assertNotIn("reconcile_exhausted", data)
            self.assertNotIn("reconcile_attempts", data)


    def test_initial_message_never_claims_no_issues(self):
        dialog = IssuesDialog(self.bus)
        try:
            self.assertFalse(dialog.loaded)
            self.assertIn("正在检查", dialog.message.text())
            self.assertNotIn("没有需要处理", dialog.message.text())
        finally:
            dialog.deleteLater()

    def test_empty_page_before_verification_settles_stays_checking(self):
        self.dialog.rows.clear()
        self.dialog.loaded = False
        self.dialog.update_context(dict(self.context, initialization={"active": True, "phase": "catching_up"}))
        self.assertIn("正在检查", self.dialog.message.text())
        self.dialog.received("", self.dialog.generation, {"entries": [], "next": None}, None)
        self.assertIn("正在检查", self.dialog.message.text())
        self.assertNotIn("没有需要处理", self.dialog.message.text())

    def test_loaded_online_empty_page_is_the_only_no_issue_claim(self):
        self.dialog.rows.clear()
        self.dialog.update_context(dict(self.context, initialization={"active": False}))
        self.dialog.received("", self.dialog.generation, {"entries": [], "next": None}, None)
        self.assertEqual(self.dialog.message.text(), "目前没有需要处理的同步问题。")

    def test_offline_empty_page_explains_verification_is_incomplete(self):
        self.dialog.rows.clear()
        self.dialog.update_context(dict(self.context, connection="offline"))
        self.assertIn("尚未完成问题核对", self.dialog.message.text())
        self.dialog.received("", self.dialog.generation, {"entries": [], "next": None}, None)
        self.assertIn("尚未完成问题核对", self.dialog.message.text())
        self.assertIn("离线", self.dialog.message.text())

    def test_diagnostic_includes_shutdown_and_connection_diagnostics(self):
        self.dialog.update_context(dict(
            self.context,
            shutdown={"phase": "draining_filesystem", "blocking_reason": "draining_open_files",
                      "elapsed_ms": 92000, "stage_elapsed_ms": 33000, "overdue": True,
                      "raw": "secret"},
            connection_diagnostics={"load_failed": False, "persistence_failed": True,
                                    "last_transition": {"phase": "resume", "category": "network",
                                                        "recovery": "automatic", "message": "secret"}},
            background_error_kind="network_unavailable",
            sync_state="checking",
            issue_count=2,
            cache_availability={"state": "partial", "offline": 1, "pinned": 0,
                                "total": 3, "checked": True, "secret": "x"},
        ))
        self.dialog.copy_diagnostic(); value = QApplication.clipboard().text()
        data = json.loads(value)
        self.assertEqual(data["shutdown"]["blocking_reason"], "draining_open_files")
        self.assertTrue(data["shutdown"]["overdue"])
        self.assertEqual(data["connection_diagnostics"]["last_transition"],
                         {"phase": "resume", "category": "network", "recovery": "automatic"})
        self.assertEqual(data["background_error_kind"], "network_unavailable")
        self.assertEqual(data["sync_state"], "checking")
        self.assertEqual(data["issue_count"], 2)
        self.assertEqual(data["cache_availability"]["state"], "partial")
        self.assertNotIn("secret", value)
        for text in ("private-token", "gone", "synthetic-mount", "synthetic-task"):
            self.assertNotIn(text, value)


    def test_context_overview_is_coalesced_per_second(self):
        self.bus.calls.clear()
        self.dialog.context_loading = False
        self.dialog.context_queried = 0.0
        overviews = 0
        for _ in range(5):
            self.dialog.schedule_context()
            if any(call[0] == "GetOverview" for call in self.bus.calls):
                self.bus.respond("GetOverview", self.context)
                overviews += 1
        self.assertEqual(overviews, 1, "a refresh burst must issue one capability GetOverview")


if __name__ == "__main__":
    unittest.main()
