"""Notification policy: episodes, merging, no-tray bookkeeping and click targeting."""
import unittest
from PySide6.QtWidgets import QApplication
from drive_native_gui.notifications import NotificationCenter

APP = QApplication.instance() or QApplication([])
START = 1_900_000_000_000


class Host:
    def __init__(self, available=True):
        self.is_available = available
        self.messages = []

    def available(self):
        return self.is_available

    def show_message(self, title, body, *, warning=False, target=None):
        self.messages.append({"title": title, "body": body,
                              "warning": warning, "target": target})


class Clock:
    def __init__(self, value=START):
        self.value = value

    def __call__(self):
        return self.value

    def advance(self, seconds):
        self.value += int(seconds * 1000)


def overview(**extra):
    value = dict(task_id='task', connection='online', mounted=True, pending_operations=0,
                 background_error=None, background_error_since_ms=None, error=None,
                 queue_health={'namespace_uncertain': 0, 'upload_uncertain': 0})
    value.update(extra)
    return value


class NotificationTests(unittest.TestCase):
    def make(self, available=True):
        self.host = Host(available)
        self.clock = Clock()
        self.activated = []
        return NotificationCenter(self.host, self.activated.append, now_ms=self.clock)

    def test_locked_mount_notifies_once_per_episode_and_merges_for_thirty_minutes(self):
        center = self.make()
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 1)
        message = self.host.messages[0]
        self.assertIn('Archive', message['title'])
        self.assertIn('需要登录', message['title'])
        self.assertFalse(message['warning'])
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 1)
        # Condition clears, then returns: the same problem is merged, not repeated.
        center.observe_overview('slot', 'Archive', overview(connection='online'))
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 1)
        self.clock.advance(31 * 60)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 2)

    def test_persistent_problem_reminds_at_most_once_every_thirty_minutes(self):
        center = self.make()
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 1)
        # The problem never clears; the merge window only defers the reminder.
        self.clock.advance(31 * 60)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 2)
        self.assertEqual([entry['kind'] for entry in center.log], ['login', 'login'])
        # A short while later it stays silent until the next window elapses.
        self.clock.advance(60)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 2)

    def test_attention_conditions_use_a_neutral_body(self):
        center = self.make()
        center.observe_overview('one', 'One', overview(connection='needs_attention'))
        center.observe_overview('two', 'Two', overview(connection='online', error='ssh://user:secret@host'))
        self.assertEqual(len(self.host.messages), 2)
        for message in self.host.messages:
            self.assertIn('需要处理', message['title'])
            self.assertNotIn('ssh://', message['body'])
            self.assertNotIn('secret', message['body'])
            self.assertTrue(message['warning'])

    def test_persistent_background_failure_waits_sixty_seconds(self):
        center = self.make()
        since = self.clock.value
        center.observe_overview('slot', 'Archive', overview(
            background_error='upload stopped; local snapshots retained', background_error_since_ms=since))
        self.clock.advance(59)
        center.observe_overview('slot', 'Archive', overview(
            background_error='upload stopped; local snapshots retained', background_error_since_ms=since))
        self.assertEqual(self.host.messages, [])
        self.clock.advance(2)
        center.observe_overview('slot', 'Archive', overview(
            background_error='upload stopped; local snapshots retained', background_error_since_ms=since))
        self.assertEqual(len(self.host.messages), 1)
        self.assertIn('持续同步失败', self.host.messages[0]['title'])
        self.assertTrue(self.host.messages[0]['warning'])
        center.observe_overview('slot', 'Archive', overview(
            background_error='upload stopped; local snapshots retained', background_error_since_ms=since))
        self.assertEqual(len(self.host.messages), 1)

    def test_different_mounts_and_kinds_are_not_merged_together(self):
        center = self.make()
        center.observe_overview('one', 'One', overview(connection='locked'))
        center.observe_overview('one', 'One', overview(connection='needs_attention'))
        center.observe_overview('two', 'Two', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 3)
        self.assertEqual([entry['kind'] for entry in center.log], ['login', 'attention', 'login'])

    def test_no_tray_only_records_state_and_click_is_a_no_op(self):
        center = self.make(available=False)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(self.host.messages, [])
        self.assertFalse(center.log[-1]['shown'])
        self.assertIsNone(center.clicked())
        self.assertEqual(self.activated, [])

    def test_click_activates_the_mount_behind_the_latest_notice(self):
        center = self.make()
        center.observe_overview('one', 'One', overview(connection='locked'))
        center.observe_overview('two', 'Two', overview(connection='locked'))
        self.assertEqual(center.clicked(), 'two')
        self.assertEqual(self.activated, ['two'])

    def test_disabled_problem_notifications_still_track_the_episode(self):
        center = self.make()
        center.set_enabled(problems=False)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(self.host.messages, [])
        center.set_enabled(problems=True)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(self.host.messages, [])
        center.observe_overview('slot', 'Archive', overview(connection='online'))
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 1)

    def test_success_notifications_are_off_by_default_and_merged_per_mount(self):
        center = self.make()
        rows = [{'id': 1, 'action': 'upload', 'result': 'ok'},
                {'id': 2, 'action': 'download', 'result': 'ok'}]
        center.observe_activity('slot', 'Archive', rows)
        self.assertEqual(self.host.messages, [])
        center.set_enabled(success=True)
        # First page is the baseline: history is not replayed as new successes.
        center.observe_activity('slot', 'Archive', rows)
        self.assertEqual(self.host.messages, [])
        center.observe_activity('slot', 'Archive', rows + [
            {'id': 3, 'action': 'upload', 'result': 'ok'},
            {'id': 4, 'action': 'download', 'result': 'ok'},
            {'id': 5, 'action': 'delete', 'result': 'ok'},
            {'id': 6, 'action': 'upload', 'result': 'failed'}])
        self.assertEqual(len(self.host.messages), 1)
        self.assertIn('Archive', self.host.messages[0]['title'])
        self.assertIn('已完成 2 项同步', self.host.messages[0]['body'])
        self.assertFalse(self.host.messages[0]['warning'])
        center.observe_activity('slot', 'Archive', rows + [{'id': 3, 'action': 'upload', 'result': 'ok'}])
        self.assertEqual(len(self.host.messages), 1)

    def test_disabling_success_resets_the_baseline(self):
        center = self.make()
        center.set_enabled(success=True)
        center.observe_activity('slot', 'Archive', [{'id': 10, 'action': 'upload', 'result': 'ok'}])
        center.set_enabled(success=False)
        self.assertEqual(center.success_cursors, {})
        center.set_enabled(success=True)
        center.observe_activity('slot', 'Archive', [{'id': 10, 'action': 'upload', 'result': 'ok'}])
        self.assertEqual(self.host.messages, [])
        center.observe_activity('slot', 'Archive', [{'id': 10, 'action': 'upload', 'result': 'ok'},
                                                    {'id': 11, 'action': 'upload', 'result': 'ok'}])
        self.assertIn('已完成 1 项同步', self.host.messages[0]['body'])

    def test_forget_resets_a_replaced_task(self):
        center = self.make()
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        center.forget('slot')
        self.assertEqual(center.clicked(), None)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(len(self.host.messages), 2)

    def test_missing_since_or_bad_fields_do_not_notify(self):
        center = self.make()
        center.observe_overview('slot', 'Archive', overview(background_error='stalled'))
        center.observe_overview('slot', 'Archive', overview(background_error='stalled', background_error_since_ms='soon'))
        center.observe_overview('slot', 'Archive', overview(background_error='stalled', background_error_since_ms=self.clock.value + 60_000))
        self.assertEqual(self.host.messages, [])
        center.observe_overview('slot', 'Archive', None)


if __name__ == '__main__':
    unittest.main()
