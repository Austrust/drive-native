"""Notification policy for mount problems and merged sync-completion notices.

The center owns *policy*; delivery belongs to the injected host.  The host only
needs ``available()`` and ``show_message(title, body, *, warning=False,
target=None)``; ``target`` is the mount slot behind the notice, which delivery
layers that can address individual notices (freedesktop D-Bus) use to route a
click back to the right mount.  A host without per-notice identity (the tray's
``messageClicked``) can ignore it: :meth:`NotificationCenter.clicked` is also
called without a target and then activates the mount behind the most recent
notification.
"""
import time
from PySide6.QtCore import QObject

PROBLEM_KINDS = ("login", "attention", "failing")
SUCCESS_ACTIONS = frozenset(("upload", "download"))
MERGE_MS = 30 * 60 * 1000
PERSISTENT_FAILURE_MS = 60 * 1000
DEFAULT_TITLE = "挂载"
PROBLEM_TEXT = {
    "login": ("需要登录", "打开 Drive Native 解锁后继续同步。"),
    "attention": ("需要处理", "打开 Drive Native 查看同步问题。"),
    "failing": ("持续同步失败", "同步已失败超过 1 分钟。打开 Drive Native 查看诊断。"),
}


class NotificationCenter(QObject):
    """Decide when a desktop notification is useful; never talk to D-Bus."""

    def __init__(self, host, activate, *, parent=None, now_ms=None):
        super().__init__(parent)
        self.host = host
        self.activate = activate
        # Tests inject a clock; production uses epoch milliseconds because the
        # backend reports background_error_since_ms in epoch milliseconds.
        self.now_ms = now_ms or (lambda: int(time.time() * 1000))
        self.enabled = {"problems": True, "success": False}
        self.episodes = {}
        self.last_problem = {}
        self.success_cursors = {}
        self.last_target = None
        self.log = []

    def set_enabled(self, settings=None, *, problems=None, success=None):
        if isinstance(settings, dict):
            problems = settings.get("problems", problems)
            success = settings.get("success", success)
        if problems is not None:
            self.enabled["problems"] = bool(problems)
        if success is not None:
            self.enabled["success"] = bool(success)
        if not self.enabled["success"]:
            # Re-enabling starts from a fresh baseline, never a backlog.
            self.success_cursors.clear()

    def enabled_success(self):
        return self.enabled["success"]

    def forget(self, slot):
        """Drop episode/cursor state after a task is removed or replaced."""
        self.episodes.pop(slot, None)
        self.success_cursors.pop(slot, None)
        for key in [key for key in self.last_problem if key[0] == slot]:
            self.last_problem.pop(key, None)
        if self.last_target == slot:
            self.last_target = None

    def observe_overview(self, slot, title, overview):
        if not isinstance(overview, dict):
            return
        kinds = self._conditions(overview)
        previous = self.episodes.get(slot, frozenset())
        now = self.now_ms()
        for kind in PROBLEM_KINDS:
            if kind not in kinds:
                continue
            last = self.last_problem.get((slot, kind))
            if kind not in previous:
                self._problem(slot, title or DEFAULT_TITLE, kind)
            elif last is not None and now - last >= MERGE_MS:
                # The same problem is still present.  While it stays present,
                # remind at most once per merge window instead of going silent
                # forever after a single notification was merged away.
                self._problem(slot, title or DEFAULT_TITLE, kind)
        if kinds:
            self.episodes[slot] = kinds
        else:
            self.episodes.pop(slot, None)

    def _conditions(self, overview):
        kinds = set()
        connection = overview.get("connection")
        if connection == "locked":
            kinds.add("login")
        if connection == "needs_attention" or overview.get("error"):
            kinds.add("attention")
        since = overview.get("background_error_since_ms")
        now = self.now_ms()
        if (overview.get("background_error") and type(since) is int
                and 0 <= since <= now and now - since >= PERSISTENT_FAILURE_MS):
            kinds.add("failing")
        return kinds

    def _problem(self, slot, title, kind):
        if not self.enabled["problems"]:
            return
        now = self.now_ms()
        key = (slot, kind)
        last = self.last_problem.get(key)
        if last is not None and 0 <= now - last < MERGE_MS:
            # A recent notification for the same mount and kind is still
            # relevant; merge recurring episodes instead of repeating it.
            return
        self.last_problem[key] = now
        label, body = PROBLEM_TEXT[kind]
        self._notify(slot, f"「{title}」{label}", body,
                     warning=kind in ("attention", "failing"), kind=kind)

    def observe_activity(self, slot, title, records):
        """Merge one GetRecentActivity page into success notifications."""
        if not self.enabled["success"] or not isinstance(records, (list, tuple)):
            return
        rows = [row for row in records if isinstance(row, dict) and type(row.get("id")) is int]
        if not rows:
            return
        newest = max(row["id"] for row in rows)
        cursor = self.success_cursors.get(slot)
        self.success_cursors[slot] = newest if cursor is None else max(cursor, newest)
        if cursor is None:
            # The first page is the baseline; historical rows are not "new".
            return
        fresh = [row for row in rows if row["id"] > cursor
                 and row.get("action") in SUCCESS_ACTIONS and row.get("result") == "ok"]
        if not fresh:
            return
        self._notify(slot, f"「{title or DEFAULT_TITLE}」同步完成",
                     f"已完成 {len(fresh)} 项同步。", warning=False, kind="success")

    def _notify(self, slot, title, body, *, warning, kind):
        available = False
        if self.host is not None:
            try:
                available = bool(self.host.available())
            except (AttributeError, RuntimeError):
                available = False
        shown = False
        if available:
            try:
                self.host.show_message(title, body, warning=warning, target=slot)
                shown = True
            except (AttributeError, RuntimeError):
                shown = False
        if shown:
            self.last_target = slot
        self.log.append({"slot": slot, "kind": kind, "title": title, "body": body,
                         "warning": warning, "shown": shown, "at": self.now_ms()})

    def clicked(self, target=None):
        """Activate the mount behind a clicked notification.

        Delivery layers that can identify individual notices pass their target
        explicitly; the tray's ``messageClicked`` has no payload and falls back
        to the most recent notice.  Policy (merging, enabled kinds) is untouched
        either way.
        """
        if target is None:
            target = self.last_target
        if target is not None and self.activate is not None:
            self.activate(target)
        return target
