"""Synthetic multi-task navigation and closure, never launches a mount or NAS."""
import os
import tempfile
import json
import time
import unittest
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent, Qt
from PySide6.QtWidgets import QApplication, QMainWindow
from drive_native_gui.manager import MountManager
from drive_native_gui.window import MainWindow
from drive_native_gui import config
from drive_native_gui.shutdown import shutdown_text

APP = QApplication.instance() or QApplication([])
A, B = 'a' * 32, 'b' * 32


class Bus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'
    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent); self.instance = instance; self.calls = []; self.waiting = []
    def call(self, method, args=(), callback=None):
        self.calls.append(method)
        if callback: self.waiting.append((method, callback))


class UnifiedTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name, 'DRIVE_NATIVE_INSTANCE': B})
        self.env.start()
        self.records = [(slot, {'schema': 1, 'task_id': 'task-' + slot, 'mount': self.tmp.name + '/' + slot,
                               'scope': {'path': '/'}, 'host': 'fixture', 'username': 'synthetic'}) for slot in [A, B]]
        self.patches = [patch('drive_native_gui.manager.profile_records', lambda: list(self.records)),
                        patch('drive_native_gui.manager.ClientBus', Bus),
                        patch('drive_native_gui.window.ClientBus', Bus),
                        patch.object(MainWindow, 'start', lambda _: None)]
        for p in self.patches: p.start()
        self.window = MountManager(); self.window.show()
        APP.processEvents()

    def tearDown(self):
        self.window.tray.icon.hide(); self.window.hide(); self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        for p in reversed(self.patches): p.stop()
        self.env.stop(); self.tmp.cleanup()

    def complete_overview(self, slot, state='online'):
        method, callback = self.window.buses[slot].waiting.pop(0)
        self.assertEqual(method, 'GetOverview')
        callback({'task_id': 'task-' + slot, 'connection': state, 'mounted': True}, None)

    def test_switching_is_embedded_reuses_pages_and_never_changes_environment(self):
        self.window.activate_task(A); first = self.window.pages[A]
        self.window.activate_task(B); second = self.window.pages[B]
        self.window.activate_task(A)
        self.assertIs(self.window.stack.currentWidget(), first)
        self.assertFalse(first.isWindow()); self.assertFalse(second.isWindow())
        self.assertEqual([w for w in APP.topLevelWidgets() if isinstance(w, QMainWindow)], [self.window])
        self.assertEqual(first.bus.instance, A); self.assertEqual(second.bus.instance, B)
        self.assertEqual(first.launcher.unit, config.service_unit(A))
        self.assertEqual(second.launcher.unit, config.service_unit(B))
        self.assertEqual(os.environ['DRIVE_NATIVE_INSTANCE'], B)
        self.assertNotIn('GetDirectory', first.bus.calls)

    def test_online_uncertain_queue_is_visible_in_manager_and_tray(self):
        method, callback = self.window.buses[A].waiting.pop(0)
        self.assertEqual(method, 'GetOverview')
        callback({'task_id': 'task-' + A, 'connection': 'online', 'mounted': True,
                  'queue_health': {'namespace_uncertain': 1, 'upload_uncertain': 0}}, None)
        self.complete_overview(B)
        self.window.render_progress(A, {'active': True, 'phase': 'waiting'})
        self.assertIn('已连接', self.window.items[A].text(1))
        self.assertIn('1 项提交待确认', self.window.items[A].text(1))
        self.assertNotIn('提交待确认', self.window.items[B].text(1))
        self.assertIn('需要检查', self.window.tray.icon.toolTip())
        self.assertFalse(any(name in ('Resume', 'Pause', 'KeepBoth')
                             for bus in self.window.buses.values() for name in bus.calls))

    def test_explicit_wizard_and_config_do_not_follow_other_task_environment(self):
        from drive_native_gui.wizard import SetupWizard
        a = dict(self.records[0][1]); b = dict(self.records[1][1])
        config.save_profile(a, A); config.save_profile(b, B)
        self.window.activate_task(A)
        page = self.window.pages[A]
        page.start = MainWindow.start.__get__(page)
        # Read via exact task API, with environment deliberately selecting B.
        self.assertEqual(config.load_profile(page.instance), a)
        wizard = SetupWizard(page, instance=A, new_name='One')
        self.assertEqual(wizard.instance, A)
        self.assertEqual(Path(wizard.mount.text()).name, 'One')
        backup = config.archive_profile(a['task_id'], A)
        self.assertEqual(config.load_profile(B), b)
        config.restore_profile(backup, A)
        self.assertEqual(config.load_profile(A), a)
        wizard.reject()

    def test_stalled_one_task_does_not_block_other_task_or_duplicate_requests(self):
        for _ in range(8): self.window.refresh_status(A)
        self.assertEqual(self.window.buses[A].calls.count('GetOverview'), 1)
        self.complete_overview(B, 'paused')
        self.assertEqual(self.window.items[B].text(1), '已暂停')
        self.assertEqual(self.window.status_pending, {A})

    def test_replaced_task_ignores_old_response(self):
        callback = self.window.buses[A].waiting[0][1]
        self.records[0] = (A, dict(self.records[0][1], task_id='replacement'))
        self.window.refresh_catalog()
        callback({'task_id': 'task-' + A, 'connection': 'online', 'mounted': True}, None)
        self.assertNotIn(A, self.window.overviews)
        self.assertNotEqual(self.window.items[A].text(1), '已连接')

    def test_pause_routes_to_selected_task_and_does_not_restore_other_paused_task(self):
        self.complete_overview(A, 'online'); self.complete_overview(B, 'paused')
        self.window.tray.pause(A)
        self.assertEqual(self.window.buses[A].calls[-1], 'Pause')
        self.assertNotIn('Resume', self.window.buses[B].calls)
        self.window.tray.pause(B)
        self.assertEqual(self.window.buses[B].calls[-1], 'Resume')

    def test_close_with_tray_hides_without_stopping_and_without_tray_stays_reachable(self):
        with patch.object(self.window.tray, 'available', return_value=True):
            self.window.close()
        self.assertFalse(self.window.isVisible())
        for bus in self.window.buses.values(): self.assertNotIn('StopMount', bus.calls)
        self.window.show()
        with patch.object(self.window.tray, 'available', return_value=False), patch.object(QApplication, 'quit') as quit_app:
            self.window.close(); quit_app.assert_not_called()
        self.assertTrue(self.window.isVisible())
        self.assertTrue(self.window.exit_button.isVisible())
        for bus in self.window.buses.values(): self.assertNotIn('StopMount', bus.calls)

    def test_tree_and_five_tabs_keep_history_in_same_window(self):
        from PySide6.QtWidgets import QTreeWidget
        self.assertIsInstance(self.window.navigation, QTreeWidget)
        self.assertEqual(self.window.navigation.topLevelItem(1).text(0), '我的挂载')
        self.window.activate_task(A)
        page = self.window.pages[A]
        self.assertEqual([page.tabs.tabText(i) for i in range(page.tabs.count())],
                         ['概览', '文件', '同步历史', '同步问题', '当前挂载设置'])
        self.window.show_history(A)
        self.assertIs(self.window.stack.currentWidget(), page)
        self.assertIs(page.tabs.currentWidget(), page.history_page)
        self.window.show_history()
        self.assertIs(self.window.stack.currentWidget(), self.window.all_page)
        self.assertIs(self.window.all_tabs.currentWidget(), self.window.history_page)
        self.assertEqual([w for w in APP.topLevelWidgets() if isinstance(w, QMainWindow)], [self.window])

    def test_pause_all_only_resumes_tasks_it_paused_and_preserves_manual_pause(self):
        self.complete_overview(A, 'online'); self.complete_overview(B, 'paused')
        tray = self.window.tray
        tray.pause_all()
        self.assertIn('Pause', self.window.buses[A].calls)
        self.assertNotIn('Pause', self.window.buses[B].calls)
        callback = [cb for method, cb in self.window.buses[A].waiting if method == 'Pause'][-1]
        callback(None, None)
        self.window.overviews[A]['connection'] = 'paused'
        tray.resume_all()
        self.assertIn('Resume', self.window.buses[A].calls)
        self.assertNotIn('Resume', self.window.buses[B].calls)

    def test_late_bulk_pause_reply_cannot_reclaim_manual_pause_from_any_page(self):
        self.complete_overview(A, 'online'); self.complete_overview(B, 'paused')
        self.window.activate_task(A)
        page = self.window.pages[A]
        page.overview = self.window.overviews[A]
        for origin in ('manager', 'mount', 'issues'):
            with self.subTest(origin=origin):
                self.window.overviews[A]['connection'] = 'online'
                self.window.items[A].setData(1, Qt.UserRole, 'online')
                self.window.tray.pause_all()
                bulk_callback = [cb for method, cb in self.window.buses[A].waiting if method == 'Pause'][-1]
                if origin == 'manager':
                    self.window.table.setCurrentItem(self.window.items[A]); self.window.pause()
                elif origin == 'mount':
                    page.toggle_pause()
                else:
                    page.show_issues(); page.issues_dialog.pause_task()
                    self.assertIn('Pause', page.bus.calls)
                bulk_callback(None, None)
                self.assertNotIn(A, self.window.tray.paused_by_all)
                self.window.overviews[A]['connection'] = 'paused'
                before = self.window.buses[A].calls.count('Resume')
                self.window.tray.resume_all()
                self.assertEqual(self.window.buses[A].calls.count('Resume'), before)

    def test_catalog_replacement_restarts_visible_history_and_ignores_old_reply(self):
        self.window.show_history()
        old_bus = self.window.buses[A]
        old_reply = [cb for method, cb in old_bus.waiting if method == 'GetSyncHistory'][-1]
        self.records[0] = (A, dict(self.records[0][1], task_id='replacement'))
        self.window.refresh_catalog()
        new_bus = self.window.buses[A]
        self.assertIsNot(new_bus, old_bus)
        new_reply = [cb for method, cb in new_bus.waiting if method == 'GetSyncHistory'][-1]
        row = {'id': 1, 'seq': 1, 'completed_at': 1793750400, 'name': 'new.txt',
               'path_at_event': 'new.txt', 'action': 'upload', 'result': 'ok'}
        old_reply({'records': [dict(row, name='stale.txt')], 'next_cursor': None}, None)
        self.assertIn(A, self.window.history_page.pending)
        new_reply({'records': [row], 'next_cursor': None}, None)
        self.assertNotIn(A, self.window.history_page.pending)
        self.assertEqual([record['name'] for _, record in self.window.history_page.model.records], ['new.txt'])

    def test_tray_menu_has_history_settings_and_two_exit_choices(self):
        self.window.tray.rebuild_menu()
        labels = [action.text() for action in self.window.tray.menu.actions()]
        for title in ('同步历史', '全部暂停', '全局设置', '退出界面，继续同步', '停止全部同步并退出'):
            self.assertIn(title, labels)
        self.complete_overview(A); self.complete_overview(B, 'paused')
        self.assertIn('2 个挂载', self.window.tray.icon.toolTip())
        self.assertIn('个问题', self.window.tray.icon.toolTip())

    def test_stop_all_request_is_not_completion_and_failed_stop_keeps_ui(self):
        from PySide6.QtWidgets import QMessageBox
        tray = self.window.tray
        def capture(slot, task, bus):
            tray.stop_identity_pending.discard(slot)
            tray.stop_accepted.add(slot)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes), \
                patch.object(tray, 'capture_stop_identity', capture), patch.object(QApplication, 'quit') as quit_app:
            tray.stop_all_and_quit()
            tray.poll_stopping(); quit_app.assert_not_called()
            self.window.shutdown_watchers[A].value = {'phase': 'complete'}
            self.window.shutdown_watchers[A].mounted = False
            tray.poll_stopping(); quit_app.assert_not_called()
            self.window.shutdown_watchers[B].value = {'phase': 'failed'}
            tray.poll_stopping(); quit_app.assert_not_called()
            self.assertFalse(tray.stopping)
            self.assertTrue(self.window.isVisible())

    def test_stop_all_waits_for_every_mount_and_ui_exit_does_not_stop(self):
        tray = self.window.tray
        tray.stopping = True
        tray.stop_requested = {slot: (p['task_id'], self.window.buses[slot]) for slot, p in self.window.records.items()}
        with patch.object(QApplication, 'quit') as quit_app:
            for slot in (A, B):
                self.window.shutdown_watchers[slot].value = {'phase': 'complete'}
                self.window.shutdown_watchers[slot].mounted = False
            tray.poll_stopping(); quit_app.assert_called_once()
        for bus in self.window.buses.values(): self.assertNotIn('StopMount', bus.calls)

    def test_stop_all_owner_loss_is_not_process_or_unmount_confirmation(self):
        from PySide6.QtWidgets import QMessageBox
        for bus in self.window.buses.values(): bus.owner = ''
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes), patch.object(QApplication, 'quit') as quit_app:
            self.window.tray.stop_all_and_quit()
            quit_app.assert_not_called()
        self.assertEqual(set(self.window.tray.stop_requested), {A, B})
        self.assertTrue(self.window.isVisible())
        self.assertIn('尚未确认正常停止', self.window.message.text())

    def test_tray_disappearance_restores_window(self):
        self.window.hide()
        with patch.object(self.window.tray, 'available', return_value=False):
            self.window.tray.ensure_reachable()
        self.assertTrue(self.window.isVisible())

    def test_current_activity_only_queries_while_visible_and_checks_identity(self):
        panel = self.window.tray.activity
        panel.refresh()
        self.assertNotIn('GetTransfers', self.window.buses[A].calls)
        panel.show(); APP.processEvents()
        cb = [cb for method, cb in self.window.buses[A].waiting if method == 'GetTransfers'][-1]
        cb({'task_id': 'wrong', 'entries': [{'path': 'SHOULD-NOT-DISPLAY'}]}, None)
        self.assertEqual(panel.table.topLevelItem(0).text(2), '状态暂不可用')
        panel.hide()
        cb2 = [cb for method, cb in self.window.buses[B].waiting if method == 'GetTransfers'][-1]
        before = panel.table.topLevelItemCount()
        cb2({'task_id': 'task-' + B, 'entries': []}, None)
        self.assertEqual(panel.table.topLevelItemCount(), before)
        self.assertFalse(panel.timer.isActive())

    def test_development_launcher_sets_child_instance_without_mutating_environment(self):
        from drive_native_gui.lifecycle import ServiceLauncher
        from PySide6.QtCore import QProcess
        captured = []
        def detached(process):
            captured.append((process.processEnvironment().value('DRIVE_NATIVE_INSTANCE'), process.arguments()))
            return True
        with patch('drive_native_gui.lifecycle.binary', return_value='/synthetic/fsd'), patch('drive_native_gui.lifecycle._start_detached', detached):
            for slot in (A, ''):
                launcher = ServiceLauncher(self.window, managed=False, instance=slot)
                launcher.start()
        self.assertEqual(captured[0], (A, ['--config', str(config.config_path(A))]))
        self.assertEqual(captured[1], ('', ['--config', str(config.config_path(''))]))
        self.assertEqual(os.environ['DRIVE_NATIVE_INSTANCE'], B)

    def test_real_detached_child_receives_own_config_and_instance(self):
        from drive_native_gui.lifecycle import ServiceLauncher
        executable = Path(self.tmp.name) / 'synthetic-child'
        output = Path(self.tmp.name) / 'child-result.json'
        executable.write_text("#!/usr/bin/python3\nimport os,sys,json\nfrom pathlib import Path\nPath(os.environ['DRIVE_NATIVE_TEST_CHILD_RESULT']).write_text(json.dumps([os.environ.get('DRIVE_NATIVE_INSTANCE'),sys.argv[1:]]))\n")
        executable.chmod(0o700)
        completed = []
        with patch.dict(os.environ, {'DRIVE_NATIVE_TEST_CHILD_RESULT': str(output)}), patch('drive_native_gui.lifecycle.binary', return_value=str(executable)):
            launcher = ServiceLauncher(self.window, managed=False, instance=A)
            launcher.completed.connect(completed.append); launcher.start()
        deadline = time.monotonic() + 5
        while not output.exists() and time.monotonic() < deadline:
            APP.processEvents(); time.sleep(.005)
        self.assertEqual(completed, [None])
        self.assertEqual(json.loads(output.read_text()), [A, ['--config', str(config.config_path(A))]])
        self.assertEqual(os.environ['DRIVE_NATIVE_INSTANCE'], B)

    def test_current_activity_stalled_task_does_not_delay_other_refresh(self):
        panel = self.window.tray.activity; panel.show(); APP.processEvents()
        callbacks = [cb for method, cb in self.window.buses[B].waiting if method == 'GetTransfers']
        callbacks[-1]({'task_id': 'task-' + B, 'entries': []}, None)
        panel.refresh()
        self.assertEqual(self.window.buses[A].calls.count('GetTransfers'), 1)
        self.assertEqual(self.window.buses[B].calls.count('GetTransfers'), 2)

    def test_shutdown_query_progresses_while_overview_is_stalled_and_rejects_wrong_task(self):
        watcher = self.window.shutdown_watchers[A]
        watcher.refresh()
        self.assertIn(A, self.window.status_pending)
        callback = [cb for method, cb in self.window.buses[A].waiting if method == 'GetShutdown'][-1]
        callback({'task_id': 'wrong', 'shutdown': {'phase': 'complete'}, 'mounted': False}, None)
        self.assertIsNone(watcher.value)
        watcher.refresh()
        callback = [cb for method, cb in self.window.buses[A].waiting if method == 'GetShutdown'][-1]
        callback({'task_id': 'task-' + A, 'shutdown': {'phase': 'checkpointing_local', 'stage_elapsed_ms': 31000, 'overdue': True}, 'mounted': False}, None)
        self.assertIn('正在保存本地状态', self.window.items[A].text(1))
        self.assertIn('尚未确认停止', self.window.items[A].text(1))
        self.assertIn(A, self.window.status_pending)

    def test_manager_palette_change_updates_embedded_page_stylesheet(self):
        from drive_native_gui.theme import palette_for
        self.window.activate_task(A)
        original = APP.palette()
        try:
            APP.setPalette(palette_for(True)); self.window.apply_theme()
            self.assertIn('#a8b2c2', self.window.pages[A].styleSheet())
            APP.setPalette(palette_for(False)); self.window.apply_theme()
            self.assertIn('#626d7c', self.window.pages[A].styleSheet())
        finally:
            APP.setPalette(original); self.window.apply_theme()

    def test_own_directory_switch_survives_absent_catalog_and_preserves_new_login(self):
        self.window.activate_task(A); page = self.window.pages[A]
        old = dict(self.records[0][1]); page.profile = old
        # The marker is set only after this page successfully archives its profile.
        page.profile_transition = old['task_id']; page.profile = None
        self.records = [(slot, p) for slot, p in self.records if slot != A]
        self.window.refresh_catalog()
        self.assertIs(self.window.pages[A], page)
        replacement = dict(old, task_id='new-task')
        with patch.object(page, 'start_service'):
            page.configured(replacement, 'synthetic-input')
        self.records.append((A, replacement)); self.window.refresh_catalog()
        self.assertIs(self.window.pages[A], page)
        self.assertEqual(page.pending_password, 'synthetic-input')
        self.assertEqual(page.profile['task_id'], 'new-task')

    def test_own_replacement_without_catalog_gap_keeps_page_but_external_change_does_not(self):
        self.window.activate_task(A); page = self.window.pages[A]
        old = dict(self.records[0][1]); page.profile = old; old_bus = self.window.buses[A]
        page.profile_transition = old['task_id']
        replacement = dict(old, task_id='own-replacement')
        with patch.object(page, 'start_service'): page.configured(replacement, 'synthetic-input')
        self.records[0] = (A, replacement); self.window.refresh_catalog()
        self.assertIs(self.window.pages[A], page)
        self.assertIsNot(self.window.buses[A], old_bus)
        self.assertIsNone(page.catalog_replacement)
        self.records[0] = (A, dict(old, task_id='external-replacement'))
        self.window.refresh_catalog()
        self.assertIsNot(self.window.pages.get(A), page)

    def test_external_replacement_during_own_gap_and_external_deletion_are_not_retained(self):
        self.window.activate_task(A); page = self.window.pages[A]
        old = dict(self.records[0][1]); page.profile_transition = old['task_id']; page.profile = None
        self.records = [(slot, p) for slot, p in self.records if slot != A]; self.window.refresh_catalog()
        self.records.append((A, dict(old, task_id='external-replacement'))); self.window.refresh_catalog()
        self.assertIsNot(self.window.pages.get(A), page)
        self.window.activate_task(B); other = self.window.pages[B]
        other.profile = dict(self.records[0][1])
        self.records = [(slot, p) for slot, p in self.records if slot != B]; self.window.refresh_catalog()
        self.assertNotIn(B, self.window.pages)

    def test_actual_archive_and_cancelled_wizard_keeps_embedded_page(self):
        from PySide6.QtCore import QProcess
        from PySide6.QtWidgets import QMessageBox
        class StopController(QObject):
            finished = Signal(int, object)
            errorOccurred = Signal(object)
            NormalExit = QProcess.NormalExit
            def start(self, *_args): pass
            def kill(self): pass
        old = dict(self.records[0][1], tls_name='fixture')
        self.records[0] = (A, old); config.save_profile(old, A)
        self.window.activate_task(A); page = self.window.pages[A]
        page.profile = old; page.overview = dict(old, pending_operations=0)
        page.launcher.managed = True
        with patch('drive_native_gui.window.QProcess', StopController), patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes), patch.object(page, 'start_service'):
            page.choose_other_folder()
            page.folder_switch_process.finished.emit(0, QProcess.NormalExit)
            self.assertIsNone(config.load_profile(A))
            self.assertEqual(page.profile_transition, old['task_id'])
            self.records = [(slot, p) for slot, p in self.records if slot != A]
            self.window.refresh_catalog()
            self.assertIs(self.window.pages[A], page)
            self.assertTrue(page.wizard.isVisible())
            page.wizard.reject()
            self.records.append((A, old)); self.window.refresh_catalog()
            self.assertIs(self.window.pages[A], page)
            self.assertEqual(config.load_profile(A), old)
            self.assertIsNone(page.profile_transition)

    def test_shutdown_uses_real_stage_and_does_not_claim_timeout_is_complete(self):
        self.assertEqual(shutdown_text(None), '')
        self.assertEqual(shutdown_text({'phase': 'invented'}), '')
        text = shutdown_text({'phase': 'draining_filesystem', 'stage_elapsed_ms': 31000, 'overdue': True})
        self.assertIn('31 秒', text); self.assertIn('尚未确认停止', text)
        self.assertNotIn('31 秒', shutdown_text({'phase': 'complete', 'stage_elapsed_ms': True}))


if __name__ == '__main__': unittest.main()
