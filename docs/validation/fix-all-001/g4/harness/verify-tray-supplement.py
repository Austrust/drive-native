"""G4-only synthetic tray integration checks; no product files are modified.

Use PYTHONPATH=FROZEN_SOURCE/desktop and an isolated offscreen/private D-Bus.
Geometry is injected; this is not evidence of a real desktop panel position.
"""
import importlib.util
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

from PySide6.QtCore import QPoint, QRect
from PySide6.QtWidgets import QApplication, QSystemTrayIcon, QTabWidget
from drive_native_gui import tray as product_tray

source = Path(sys.argv.pop(1)).resolve()
spec = importlib.util.spec_from_file_location('frozen_unified_fixtures', source / 'desktop/tests/unified_manager_tests.py')
fixture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)


class TraySupplement(unittest.TestCase):
    def setUp(self):
        self.fixture = fixture.UnifiedTests()
        self.fixture.setUp()
        self.window = self.fixture.window

    def tearDown(self):
        self.fixture.tearDown()

    def test_left_click_toggles_activity_and_double_click_activates_manager(self):
        tray = self.window.tray
        self.assertFalse(tray.activity.isVisible())
        tray.activated(QSystemTrayIcon.Trigger)
        fixture.APP.processEvents()
        self.assertTrue(tray.activity.isVisible())
        tray.activated(QSystemTrayIcon.Trigger)
        self.assertFalse(tray.activity.isVisible())
        self.assertFalse(tray.activity.timer.isActive())
        with patch.object(self.window, 'activate') as activate:
            tray.activated(QSystemTrayIcon.DoubleClick)
            activate.assert_called_once_with()

    def test_injected_top_bottom_left_right_panel_geometry_and_cursor_fallback(self):
        tray = self.window.tray
        bounds = QRect(-1200, 40, 2400, 1400)
        class Screen:
            def availableGeometry(self):
                return bounds
        anchors = (QPoint(-1190, 50), QPoint(1190, 50), QPoint(-1190, 1430), QPoint(1190, 1430))
        for anchor in anchors:
            for use_icon in (True, False):
                with self.subTest(anchor=(anchor.x(), anchor.y()), use_icon=use_icon):
                    icon = QRect(anchor.x()-5, anchor.y()-5, 11, 11) if use_icon else QRect()
                    with patch.object(tray.icon, 'geometry', return_value=icon), \
                            patch.object(product_tray.QCursor, 'pos', return_value=anchor), \
                            patch.object(QApplication, 'screenAt', return_value=Screen()):
                        tray.show_activity()
                    position = tray.activity.pos()
                    self.assertGreaterEqual(position.x(), bounds.left())
                    self.assertGreaterEqual(position.y(), bounds.top())
                    self.assertLessEqual(position.x()+tray.activity.width(), bounds.right())
                    self.assertLessEqual(position.y()+tray.activity.height(), bounds.bottom())
                    tray.activity.hide()

    def test_recent_history_details_and_buttons_route_to_selected_record(self):
        tray = self.window.tray
        tray.show_activity()
        tabs = tray.activity.findChild(QTabWidget)
        tabs.setCurrentIndex(1)
        fixture.APP.processEvents()
        page = tray.activity.history
        self.assertTrue(page.recent)
        row = dict(id=7, seq=8, completed_at=1793750400, name='file.txt',
                   path_at_event='folder/file.txt', action='upload', result='failed', attempts=3)
        for slot in (fixture.A, fixture.B):
            replies = [callback for method, callback in self.window.buses[slot].waiting if method == 'GetRecentActivity']
            self.assertTrue(replies)
            replies[-1]({'records': [row] if slot == fixture.A else [], 'next_cursor': None}, None)
        self.assertEqual(page.model.rowCount(), 1)
        page.table.selectRow(0)
        fixture.APP.processEvents()
        for text in ('上传', '失败', '尝试 3 次', 'folder/file.txt'):
            self.assertIn(text, page.detail.text())
        with patch.object(page.actions, 'open') as open_record:
            page.open_button.click()
            opened_source, opened_row = open_record.call_args.args
            self.assertEqual(opened_source.slot, fixture.A)
            self.assertEqual(opened_row['id'], 7)
            self.assertEqual(open_record.call_args.kwargs, {'locate': False})
            page.locate_button.click()
            self.assertEqual(open_record.call_args.kwargs, {'locate': True})
        # No resolver completion or external application is invoked by this check.

    def test_tooltip_counts_and_text_distinguish_idle_pending_paused_and_attention(self):
        tray = self.window.tray
        cases = (
            ({'connection': 'online'}, '查看当前活动'),
            ({'connection': 'online', 'pending_operations': 3}, '有待处理操作'),
            ({'connection': 'paused'}, '全部已暂停'),
            ({'connection': 'needs_attention'}, '有任务需要检查'),
        )
        for state, expected in cases:
            self.window.overviews = {fixture.A: dict(state), fixture.B: dict(state)}
            tray.refresh()
            self.assertIn(expected, tray.icon.toolTip())
            self.assertIn('2 个挂载', tray.icon.toolTip())
            self.assertFalse(tray.icon.icon().isNull())
        self.window.overviews = {
            fixture.A: {'connection': 'online', 'uploading': 2, 'download_errors': 1},
            fixture.B: {'connection': 'paused', 'uploading': 1, 'download_errors': 2},
        }
        tray.refresh()
        self.assertIn('3 项上传/待同步', tray.icon.toolTip())
        self.assertIn('3 个问题', tray.icon.toolTip())


if __name__ == '__main__':
    unittest.main()
