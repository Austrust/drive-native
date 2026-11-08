"""Synthetic keyboard/navigation coverage for the unified window; no real display."""
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent, Qt
from PySide6.QtGui import QShortcut, QKeySequence
from PySide6.QtTest import QTest
from PySide6.QtWidgets import QApplication, QInputDialog
from drive_native_gui.manager import MountManager
from drive_native_gui.window import MainWindow

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


class Launcher(QObject):
    completed = Signal(object)
    def __init__(self, parent=None, *, managed=True, unit='', instance=None):
        super().__init__(parent)
        self.managed = managed; self.unit = unit; self.instance = instance; self.started = False
    def start(self, enable=True):
        self.started = True


class KeyboardTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name, 'DRIVE_NATIVE_INSTANCE': B})
        self.env.start()
        self.records = [(slot, {'schema': 1, 'task_id': 'task-' + slot, 'mount': self.tmp.name + '/' + slot,
                               'scope': {'path': '/'}, 'host': 'fixture', 'username': 'synthetic'}) for slot in [A, B]]
        self.patches = [patch('drive_native_gui.manager.profile_records', lambda: list(self.records)),
                        patch('drive_native_gui.manager.ClientBus', Bus),
                        patch('drive_native_gui.window.ClientBus', Bus),
                        patch('drive_native_gui.manager.ServiceLauncher', Launcher),
                        patch.object(MainWindow, 'start', lambda _: None)]
        for p in self.patches: p.start()
        self.window = MountManager(); self.window.show()
        APP.processEvents()

    def tearDown(self):
        self.window.tray.icon.hide(); self.window.hide(); self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        for p in reversed(self.patches): p.stop()
        self.env.stop(); self.tmp.cleanup()

    def shortcut(self, name):
        return self.window.findChild(QShortcut, name)

    def emit(self, name):
        shortcut = self.shortcut(name)
        self.assertIsNotNone(shortcut, name)
        shortcut.activated.emit()

    def complete_overview(self, slot, state='online'):
        method, callback = self.window.buses[slot].waiting.pop(0)
        self.assertEqual(method, 'GetOverview')
        callback({'task_id': 'task-' + slot, 'connection': state, 'mounted': True}, None)

    def test_shortcut_keys_and_object_names_follow_the_contract(self):
        expected = {'action_add_mount': 'Ctrl+Shift+A', 'action_show_history': 'Ctrl+H',
                    'action_hide_window': 'Ctrl+W', 'action_exit_menu': 'Ctrl+Q',
                    'action_focus_cycle': 'F6'}
        for name, keys in expected.items():
            shortcut = self.shortcut(name)
            self.assertIsNotNone(shortcut, name)
            self.assertEqual(shortcut.key(), QKeySequence(keys))
            self.assertEqual(shortcut.context(), Qt.WindowShortcut)
        keys = [shortcut.key().toString() for shortcut in self.window.findChildren(QShortcut)]
        for index in range(1, 6):
            self.assertIn(f'Ctrl+{index}', keys)
        self.assertIn('Ctrl+,', keys)
        self.assertEqual(self.shortcut('action_show_settings').key(), QKeySequence('Ctrl+,'))

    def test_ctrl_number_switches_tabs_of_the_current_mount(self):
        self.window.activate_task(A)
        page = self.window.pages[A]
        for index in range(2, 6):
            self.window.shortcuts[index - 1].activated.emit()
            self.assertEqual(page.tabs.currentIndex(), index - 1)
        self.assertEqual(self.window.activate_tab(1), 'mount')
        self.assertEqual(page.tabs.currentIndex(), 0)

    def test_ctrl_number_never_switches_hidden_views(self):
        self.window.activate_task(A)
        self.window.activate_tab(2)
        self.assertEqual(self.window.all_tabs.currentIndex(), 0)
        self.window.activate('')
        self.assertEqual(self.window.activate_tab(1), 'overview')
        self.assertEqual(self.window.all_tabs.currentIndex(), 0)
        self.assertEqual(self.window.activate_tab(3), 'history')
        self.assertEqual(self.window.all_tabs.currentIndex(), 1)
        self.assertEqual(self.window.activate_tab(5), 'global')
        self.assertEqual(self.window.all_tabs.currentIndex(), 1)

    def test_ctrl_h_and_ctrl_comma_open_history_and_global_settings(self):
        self.emit('action_show_history')
        self.assertIs(self.window.stack.currentWidget(), self.window.all_page)
        self.assertIs(self.window.all_tabs.currentWidget(), self.window.history_page)
        self.emit('action_show_settings')
        self.assertIs(self.window.stack.currentWidget(), self.window.settings_page)
        item = self.window.navigation.currentItem()
        self.assertEqual(item.data(0, Qt.UserRole), ('settings', ''))

    def test_ctrl_q_opens_the_two_choice_exit_menu(self):
        self.emit('action_exit_menu')
        menu = self.window.exit_menu
        APP.processEvents()
        self.assertTrue(menu.isVisible())
        self.assertEqual([action.text() for action in menu.actions()],
                         ['退出界面，继续同步', '停止全部同步并退出'])
        menu.close()

    def test_ctrl_shift_a_opens_add_mount_without_creating_a_mount(self):
        with patch.object(QInputDialog, 'getText', return_value=('', False)) as question:
            self.emit('action_add_mount')
            APP.processEvents()
        question.assert_called_once()
        self.assertEqual(len(self.window.records), 2)

    def test_ctrl_w_requires_a_usable_tray(self):
        with patch.object(self.window.tray, 'available', return_value=True):
            self.emit('action_hide_window')
        self.assertTrue(self.window.isHidden())
        self.window.show(); APP.processEvents()
        with patch.object(self.window.tray, 'available', return_value=False):
            self.emit('action_hide_window')
        self.assertTrue(self.window.isVisible())
        self.assertIn('托盘不可用', self.window.message.text())

    def test_f6_cycles_focus_between_navigation_and_current_page(self):
        self.window.activate_task(A)
        self.assertIs(self.window.page_focus_target(), self.window.pages[A].tabs)
        self.window.activate(''); self.assertIs(self.window.page_focus_target(), self.window.all_tabs)
        self.window.show_settings(); self.assertIs(self.window.page_focus_target(), self.window.problems_check)
        self.window.activateWindow(); self.window.navigation.setFocus(Qt.TabFocusReason); APP.processEvents()
        if QApplication.focusWidget() is not self.window.navigation:
            self.skipTest('offscreen platform did not grant widget focus')
        self.assertEqual(self.window.focus_cycle(), 'page')
        self.assertEqual(self.window.focus_cycle(), 'navigation')

    def test_enter_on_navigation_opens_the_mount(self):
        self.window.activate('')
        item = self.window.nav_items[('task', A)]
        self.window.navigation.setCurrentItem(item); APP.processEvents()
        self.window.navigation.setFocus(Qt.TabFocusReason)
        QTest.keyClick(self.window.navigation, Qt.Key_Return); APP.processEvents()
        self.assertIs(self.window.stack.currentWidget(), self.window.pages[A])

    def test_escape_closes_the_activity_panel(self):
        panel = self.window.tray.activity
        panel.show(); APP.processEvents()
        QTest.keyClick(panel, Qt.Key_Escape); APP.processEvents()
        self.assertFalse(panel.isVisible())

    def test_hidden_transfer_table_stays_out_of_the_focus_chain(self):
        self.assertFalse(self.window.transfer_table.isVisible())
        self.assertFalse(self.window.transfer_table.isVisibleTo(self.window))
        self.window.show_settings(); APP.processEvents()
        self.assertTrue(self.window.problems_check.isVisibleTo(self.window))
        # The transfer section lives on the overview page; settings never shows it.
        self.assertFalse(self.window.transfer_table.isVisibleTo(self.window))

    def test_accessible_names_cover_new_controls(self):
        self.assertEqual(self.window.navigation.accessibleName(), '挂载导航')
        self.assertEqual(self.window.table.accessibleName(), '所有挂载列表')
        self.assertEqual(self.window.transfer_table.accessibleName(), '所有挂载的当前传输')
        self.assertEqual(self.window.transfer_summary.accessibleName(), '所有挂载的当前传输摘要')
        self.assertEqual(self.window.settings_page.accessibleName(), '全局界面设置')
        self.assertEqual(self.window.problems_check.accessibleName(), '同步问题通知')
        self.assertEqual(self.window.success_check.accessibleName(), '同步成功通知')

    def test_history_pages_can_start_a_mount_for_retry(self):
        self.assertTrue(callable(self.window.start_slot))
        self.assertTrue(callable(self.window.start_mount))
        self.assertIs(self.window.history_page.mount_actions, self.window)
        self.assertIs(self.window.tray.activity.history.mount_actions, self.window)
        self.assertFalse(self.window.start_slot())
        self.assertIn('请先在挂载列表中选择一个挂载', self.window.message.text())
        self.window.table.setCurrentItem(self.window.items[A])
        self.assertTrue(self.window.start_slot(A))
        launcher = self.window.launchers[-1]
        self.assertEqual(launcher.instance, A); self.assertTrue(launcher.started)
        launcher.completed.emit(None); APP.processEvents()
        self.assertFalse(self.window.launchers)
        self.assertEqual(self.window.records[A]['mount'], self.tmp.name + '/' + A)
        self.assertFalse(self.window.start_slot('missing'))
        self.assertIn('无法启动', self.window.message.text())


if __name__ == '__main__':
    unittest.main()
