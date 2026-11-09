"""Synthetic multi-task navigation and closure, never launches a mount or NAS."""
import os
import tempfile
import json
import time
import unittest
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent, Qt
from PySide6.QtGui import QShortcut
from PySide6.QtWidgets import QApplication, QMainWindow
from drive_native_gui.manager import MountManager
from drive_native_gui.window import MainWindow
from drive_native_gui import config
from drive_native_gui.shutdown import shutdown_text

APP = QApplication.instance() or QApplication([])
A, B = 'a' * 32, 'b' * 32


class Bus(QObject):
    changed = Signal()
    changed2 = Signal(object)
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

    def test_gui_settings_storage_defaults_owner_only_and_corrupt_fallback(self):
        path = config.gui_settings_path()
        # Constructing the window only reads global settings; it never writes at startup.
        self.assertFalse(path.exists())
        self.assertEqual(config.load_gui_settings(), config.default_gui_settings())
        path.parent.mkdir(parents=True, mode=0o700, exist_ok=True)
        path.write_text('{not json', encoding='utf-8')
        self.assertEqual(config.load_gui_settings(), config.default_gui_settings())
        config.save_gui_settings({'schema': 1, 'notifications': {'problems': False, 'success': True}})
        self.assertEqual(path.stat().st_mode & 0o077, 0)
        text = path.read_text(encoding='utf-8')
        self.assertNotIn('password', text)
        self.assertEqual(json.loads(text),
                         {'schema': 1, 'notifications': {'problems': False, 'success': True}})
        self.assertEqual(config.load_gui_settings(),
                         {'schema': 1, 'notifications': {'problems': False, 'success': True}})

    def test_global_settings_page_persists_toggles_and_updates_the_notification_center(self):
        page = self.window.settings_page
        self.assertIs(self.window.stack.widget(1), page)
        self.assertTrue(self.window.problems_check.isChecked())
        self.assertIn('默认关闭', self.window.success_check.text())
        self.assertFalse(self.window.success_check.isChecked())
        self.assertFalse(self.window.success_timer.isActive())
        self.assertTrue(self.window.open_history_button.accessibleName())
        self.window.success_check.setChecked(True)
        self.window.problems_check.setChecked(False)
        self.assertEqual(config.load_gui_settings(),
                         {'schema': 1, 'notifications': {'problems': False, 'success': True}})
        self.assertTrue(self.window.success_timer.isActive())
        self.assertFalse(self.window.notifications.enabled['problems'])
        self.assertTrue(self.window.notifications.enabled['success'])
        self.assertTrue(config.gui_settings_path().is_file())
        self.window.open_history_button.click()
        self.assertIs(self.window.stack.currentWidget(), self.window.all_page)
        self.assertIs(self.window.all_tabs.currentWidget(), self.window.history_page)
        # A damaged file degrades to defaults instead of blocking the next window.
        config.gui_settings_path().write_text('{broken', encoding='utf-8')
        self.assertEqual(config.load_gui_settings(), config.default_gui_settings())

    def test_all_mounts_table_shows_activity_and_problem_columns(self):
        headers = [self.window.table.headerItem().text(column) for column in range(6)]
        self.assertEqual(headers, ['挂载名称', '状态', '同步活动', '问题', '远端目录', '本地位置'])
        self.assertEqual(self.window.items[A].text(2), '—')
        method, callback = self.window.buses[A].waiting.pop(0)
        self.assertEqual(method, 'GetOverview')
        callback({'task_id': 'task-' + A, 'connection': 'online', 'mounted': True,
                  'pending_operations': 2, 'download_errors': 1,
                  'queue_health': {'namespace_uncertain': 0, 'upload_uncertain': 0}}, None)
        self.assertEqual(self.window.items[A].text(2), '2 项待同步')
        self.assertEqual(self.window.items[A].text(3), '1 项待处理')
        self.complete_overview(B)
        self.assertEqual(self.window.items[B].text(2), '无')
        self.assertEqual(self.window.items[B].text(3), '无')
        # Unavailable status falls back to placeholders, never to invented values.
        self.window.refresh_status(A)
        method, callback = self.window.buses[A].waiting.pop(0)
        self.assertEqual(method, 'GetOverview')
        callback(None, 'service_unavailable')
        self.assertEqual(self.window.items[A].text(2), '—')
        self.assertEqual(self.window.items[A].text(3), '—')

    def test_mount_row_menu_exposes_management_stop_and_history(self):
        self.complete_overview(A)
        menu = self.window.mount_menu(A)
        self.assertEqual([action.text() for action in menu.actions()],
                         ['打开管理', '打开文件夹', '暂停同步', '停止此挂载并卸载…', '同步历史'])
        self.window.overviews[A]['connection'] = 'paused'
        paused_menu = self.window.mount_menu(A)
        actions = {action.text(): action for action in paused_menu.actions()}
        self.assertIn('恢复同步', actions)
        with patch.object(self.window, 'stop_task') as stop:
            actions['停止此挂载并卸载…'].trigger()
        stop.assert_called_once_with(A)
        actions['同步历史'].trigger()
        self.assertIs(self.window.stack.currentWidget(), self.window.pages[A])
        self.assertIs(self.window.pages[A].tabs.currentWidget(), self.window.pages[A].history_page)

    def test_stop_confirmation_distinguishes_pause_from_unmount(self):
        from PySide6.QtWidgets import QMessageBox
        self.complete_overview(A)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Cancel) as question:
            self.assertFalse(self.window.stop_task(A))
        text = question.call_args.args[2]
        self.assertIn('暂停同步（保留挂载）', text)
        self.assertIn('停止并卸载（结束挂载和服务）', text)
        self.assertIn('不会删除缓存、草稿或远端文件', text)
        self.assertIn('不会强制结束进程', text)
        self.assertNotIn('StopMount', self.window.buses[A].calls)
        self.assertIn('已取消停止', self.window.message.text())

    def test_stop_task_reports_real_shutdown_stage_and_never_forces_kill(self):
        from PySide6.QtWidgets import QMessageBox
        self.complete_overview(A)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.assertTrue(self.window.stop_task(A))
        self.assertIn('StopMount', self.window.buses[A].calls)
        self.assertIn(A, self.window.stop_pending)
        self.assertIn('正在正常停止', self.window.message.text())
        self.assertIn('不会强制结束', self.window.message.text())
        watcher = self.window.shutdown_watchers[A]
        watcher.value = {'phase': 'detaching_mount', 'stage_elapsed_ms': 2000}
        watcher.mounted = True
        self.window.poll_stop_tasks()
        self.assertIn('正在卸载目录', self.window.items[A].text(1))
        watcher.value = {'phase': 'complete'}
        watcher.mounted = False
        self.window.poll_stop_tasks()
        self.assertNotIn(A, self.window.stop_pending)
        self.assertFalse(self.window.stop_timer.isActive())
        self.assertIn('已停止并卸载', self.window.message.text())
        for bus in self.window.buses.values():
            self.assertNotIn('Terminate', bus.calls)
            self.assertNotIn('Kill', bus.calls)

    def test_page_stop_button_starts_one_confirmed_stop_and_restores_its_button(self):
        from PySide6.QtWidgets import QMessageBox
        self.complete_overview(A)
        self.window.activate_task(A)
        page = self.window.pages[A]
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes) as question:
            page.stop_and_unmount()
        # The page confirmed once; the manager must not ask a second time.
        question.assert_called_once()
        self.assertIn('StopMount', self.window.buses[A].calls)
        self.assertIn(A, self.window.stop_pending)
        self.assertFalse(page.stop_mount.isEnabled())
        self.assertIn('正在停止并卸载', page.stop_mount.text())
        self.window.shutdown_watchers[A].value = {'phase': 'complete'}
        self.window.shutdown_watchers[A].mounted = False
        self.window.poll_stop_tasks()
        self.assertTrue(page.stop_mount.isEnabled())
        self.assertEqual(page.stop_mount.text(), '停止此挂载并卸载…')
        self.assertIn('已停止并卸载', page.banner.text())

    def test_stop_timeout_and_failure_explain_blocking_reason_without_force_kill(self):
        from PySide6.QtWidgets import QMessageBox
        self.complete_overview(A)
        self.complete_overview(B)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.assertTrue(self.window.stop_task(B))
        self.window.stop_pending[B]['deadline'] = time.monotonic() - 1
        self.window.poll_stop_tasks()
        self.assertNotIn(B, self.window.stop_pending)
        timeout = self.window.message.text()
        self.assertIn('超时', timeout)
        self.assertIn('不会强制结束后台', timeout)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.assertTrue(self.window.stop_task(A))
        self.window.shutdown_watchers[A].value = {'phase': 'failed'}
        self.window.shutdown_watchers[A].mounted = True
        self.window.poll_stop_tasks()
        self.assertNotIn(A, self.window.stop_pending)
        failure = self.window.message.text()
        self.assertIn('停止未确认', failure)
        self.assertIn('不会强制结束后台', failure)
        for bus in self.window.buses.values():
            self.assertNotIn('Terminate', bus.calls)

    def test_tray_menu_offers_stop_unmount_for_each_mount(self):
        self.window.tray.rebuild_menu()
        submenus = [action.menu() for action in self.window.tray.menu.actions() if action.menu() is not None]
        labels = [action.text() for menu in submenus for action in menu.actions()]
        self.assertIn('停止此挂载并卸载…', labels)
        with patch.object(self.window, 'stop_task') as stop:
            for menu in submenus:
                for action in menu.actions():
                    if action.text() == '停止此挂载并卸载…':
                        action.trigger()
        stop.assert_any_call(A)
        stop.assert_any_call(B)

    def test_stopping_one_mount_leaves_the_other_mount_running(self):
        from PySide6.QtWidgets import QMessageBox
        self.complete_overview(A)
        self.complete_overview(B)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.assertTrue(self.window.stop_task(A))
        self.assertEqual(self.window.buses[A].calls.count('StopMount'), 1)
        self.assertNotIn('StopMount', self.window.buses[B].calls)
        self.assertEqual(list(self.window.stop_pending), [A])
        self.assertNotIn('正在停止', self.window.items[B].text(1))
        self.window.shutdown_watchers[A].value = {'phase': 'complete'}
        self.window.shutdown_watchers[A].mounted = False
        self.window.poll_stop_tasks()
        self.assertIn('已停止并卸载', self.window.message.text())
        # A stop is scoped to one mount and stays cooperative: it never frees
        # cache or forgets history, and the sibling mount keeps its page.
        destructive = ('Terminate', 'Kill', 'ReleaseSpace', 'ClearSyncHistory',
                       'RemoveCache', 'ClearCache')
        for slot, bus in self.window.buses.items():
            for method in bus.calls:
                self.assertNotIn(method, destructive, slot)
        self.assertIn(B, self.window.records)
        self.window.activate_task(B)
        self.assertIs(self.window.stack.currentWidget(), self.window.pages[B])
        self.assertTrue(self.window.pages[B].stop_mount.isEnabled())
        self.assertTrue(self.window.isVisible())

    def test_complete_phase_with_mount_still_present_is_not_reported_as_unmounted(self):
        from PySide6.QtWidgets import QMessageBox
        self.complete_overview(A)
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            self.assertTrue(self.window.stop_task(A))
        watcher = self.window.shutdown_watchers[A]
        # "complete" alone must not turn a stop with a still-present mount into
        # a successful unmount; the UI keeps waiting instead of claiming success.
        watcher.value = {'phase': 'complete'}
        watcher.mounted = True
        self.window.poll_stop_tasks()
        self.assertIn(A, self.window.stop_pending)
        self.assertTrue(self.window.stop_timer.isActive())
        self.assertNotIn('已停止并卸载', self.window.message.text())

    def test_notification_click_reopens_the_mount_and_missing_tray_stays_silent(self):
        method, callback = self.window.buses[A].waiting.pop(0)
        self.assertEqual(method, 'GetOverview')
        with patch.object(self.window.tray, 'available', return_value=True), \
                patch.object(self.window.tray, 'show_message') as show:
            callback({'task_id': 'task-' + A, 'connection': 'locked', 'mounted': True}, None)
        show.assert_called_once()
        self.assertIn('需要登录', show.call_args.args[0])
        self.assertEqual(self.window.notifications.last_target, A)
        self.window.hide()
        # Qt's tray messageClicked has no payload: the center remembers the target mount.
        self.window.tray.icon.messageClicked.emit()
        self.assertTrue(self.window.isVisible())
        self.assertIs(self.window.stack.currentWidget(), self.window.pages[A])
        with patch.object(self.window.tray, 'available', return_value=False), \
                patch.object(self.window.tray, 'show_message') as silent, \
                patch.object(self.window.freedesktop_notifications, 'available',
                             return_value=False):
            self.window.notifications.observe_overview('other', 'Other', {'connection': 'locked'})
        silent.assert_not_called()
        self.assertFalse(self.window.notifications.log[-1]['shown'])

    def test_missing_tray_falls_back_to_system_notifications_with_click_target(self):
        method, callback = self.window.buses[B].waiting.pop(0)
        self.assertEqual(method, 'GetOverview')
        with patch.object(self.window.tray, 'available', return_value=False), \
                patch.object(self.window.freedesktop_notifications, 'available',
                             return_value=True), \
                patch.object(self.window.freedesktop_notifications, 'show_message') as fallback:
            callback({'task_id': 'task-' + B, 'connection': 'locked', 'mounted': True}, None)
        fallback.assert_called_once()
        self.assertIn('需要登录', fallback.call_args.args[0])
        # The click target travels with the delivered notice so a click on an
        # older system notification reopens its own mount, not the newest one.
        self.assertEqual(fallback.call_args.kwargs.get('target'), B)
        self.assertEqual(self.window.notifications.last_target, B)
        self.window.hide()
        self.window.freedesktop_notifications.clicked.emit(B)
        self.assertTrue(self.window.isVisible())
        self.assertIs(self.window.stack.currentWidget(), self.window.pages[B])

    def test_keyboard_shortcuts_switch_tabs_and_navigation_enter_opens_mounts(self):
        self.window.activate_task(A)
        self.assertEqual(self.window.activate_tab(1), 'mount')
        self.assertEqual(self.window.pages[A].tabs.currentIndex(), 0)
        self.window.shortcuts[1].activated.emit()
        self.assertEqual(self.window.pages[A].tabs.currentIndex(), 1)
        self.assertEqual(self.window.activate_tab(3), 'mount')
        self.assertEqual(self.window.pages[A].tabs.currentIndex(), 2)
        self.window.activate('')
        self.assertEqual(self.window.activate_tab(1), 'overview')
        self.assertIs(self.window.all_tabs.currentWidget(), self.window.all_tabs.widget(0))
        self.assertEqual(self.window.activate_tab(3), 'history')
        self.assertIs(self.window.all_tabs.currentWidget(), self.window.history_page)
        self.window.show_settings()
        self.assertEqual(self.window.activate_tab(2), 'settings')
        self.assertIs(self.window.stack.currentWidget(), self.window.settings_page)
        keys = {shortcut.key().toString() for shortcut in self.window.findChildren(QShortcut)}
        for key in ('Ctrl+1', 'Ctrl+5', 'Ctrl+,', 'F6'):
            self.assertIn(key, keys)
        self.window.activate_task(A)
        self.assertIs(self.window.page_focus_target(), self.window.pages[A].tabs)
        self.window.activate('')
        self.assertIs(self.window.page_focus_target(), self.window.all_tabs)
        self.window.show_settings()
        self.assertIs(self.window.page_focus_target(), self.window.problems_check)
        self.window.open_navigation_item(self.window.nav_items[('task', B)])
        self.assertIs(self.window.stack.currentWidget(), self.window.pages[B])


    # Issue #31: the mount catalog only re-queries overview when the change set
    # says the aggregate changed (or the revision stream had a gap/reset).
    def drain_overviews(self):
        for slot in (A, B):
            bus = self.window.buses[slot]
            while bus.waiting:
                method, callback = bus.waiting.pop(0)
                if method == 'GetOverview':
                    callback({'task_id': 'task-' + slot, 'connection': 'online',
                              'mounted': True}, None)

    def test_status_only_event_does_not_query_overview(self):
        from drive_native_gui.bus import StatusEvent
        self.drain_overviews()
        slot = A
        bus = self.window.buses[slot]
        bus.calls.clear()
        bus.waiting.clear()
        manager = self.window
        manager.status2_seen[slot] = True
        manager.status_epoch[slot] = 'e1'
        manager.status_revision[slot] = 7
        baseline = manager.overview_queries
        for revision in range(8, 13):
            bus.changed2.emit(StatusEvent(
                revision - 1, revision, 'e1', False, False,
                [{'path': self.tmp.name + '/' + slot + '/x', 'directory': False,
                  'kind': 'status'}]))
        APP.processEvents()
        self.assertEqual(manager.overview_queries, baseline)
        self.assertNotIn('GetOverview', bus.calls)

    def test_overview_changed_event_queries_one_overview(self):
        from drive_native_gui.bus import StatusEvent
        self.drain_overviews()
        slot = A
        bus = self.window.buses[slot]
        bus.calls.clear()
        manager = self.window
        manager.status2_seen[slot] = True
        manager.status_epoch[slot] = 'e1'
        manager.status_revision[slot] = 7
        manager.status_queried[slot] = 0.0
        bus.changed2.emit(StatusEvent(7, 8, 'e1', False, True, []))
        APP.processEvents()
        self.assertIn('GetOverview', bus.calls)

    def test_status_gap_falls_back_to_overview(self):
        from drive_native_gui.bus import StatusEvent
        self.drain_overviews()
        slot = B
        bus = self.window.buses[slot]
        bus.calls.clear()
        manager = self.window
        manager.status2_seen[slot] = True
        manager.status_epoch[slot] = 'e1'
        manager.status_revision[slot] = 7
        manager.status_queried[slot] = 0.0
        bus.changed2.emit(StatusEvent(9, 10, 'e1', False, False, []))
        APP.processEvents()
        self.assertIn('GetOverview', bus.calls)

    def test_legacy_changed_signal_is_ignored_after_status2(self):
        self.drain_overviews()
        bus = self.window.buses[A]
        bus.calls.clear()
        self.window.status2_seen[A] = True
        self.window.status_queried[A] = 0.0
        bus.changed.emit()
        APP.processEvents()
        self.assertNotIn('GetOverview', bus.calls)


    def test_status_signal_storm_coalesces_overview_queries(self):
        from PySide6.QtTest import QTest
        slot = A
        bus = self.window.buses[slot]
        bus.waiting.clear()
        bus.calls.clear()
        window = self.window

        def call(method, args=(), callback=None):
            bus.calls.append(method)
            if callback:
                callback({'task_id': 'task-' + slot, 'connection': 'online', 'mounted': True},
                         None)

        bus.call = call
        baseline = window.overview_queries
        for _ in range(12):
            bus.changed.emit()
            QTest.qWait(80)
        QTest.qWait(700)
        self.assertLessEqual(window.overview_queries - baseline, 2,
                             'a signal burst must not issue more than one overview per second')


if __name__ == '__main__': unittest.main()
