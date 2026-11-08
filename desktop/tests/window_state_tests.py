"""Window state cards and file columns stay on independent axes."""
import os
import tempfile
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, Qt, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication, QMainWindow
from drive_native_gui.window import MainWindow

app = QApplication.instance() or QApplication([])
TASK = 'a' * 32


class Bus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)
    owner = ':1.synthetic'

    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.calls = []
        self.waiting = []

    def call(self, method, args=(), callback=None):
        self.calls.append(method)
        if callback:
            self.waiting.append((method, callback))


def overview(**changes):
    data = {
        'connection': 'online',
        'mounted': True,
        'pending_operations': 0,
        'uncertain_operations': 0,
        'mount': '/tmp/synthetic',
        'scope_path': '/Lab',
    }
    data.update(changes)
    return data


class WindowStateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name})
        self.env.start()
        self.patches = [
            patch('drive_native_gui.window.ClientBus', Bus),
            patch.object(MainWindow, 'start', lambda _: None),
        ]
        for p in self.patches:
            p.start()
        self.window = MainWindow(instance='')
        self.window.profile = {'task_id': TASK, 'mount': '/tmp/synthetic'}
        self.window.show()
        app.processEvents()
        self.window.refresh_timer.stop()
        self.window.progress_timer.stop()
        self.bus = self.window.bus

    def tearDown(self):
        self.window.transfer_timer.stop()
        self.window.hide()
        self.window.close()
        self.window.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        for p in reversed(self.patches):
            p.stop()
        self.env.stop()
        self.tmp.cleanup()

    def test_three_cards_are_independent_axes(self):
        self.window.received_overview(overview(
            initialization={'active': True, 'phase': 'catching_up'},
            cache_availability={'state': 'partial', 'total': 10, 'offline': 4, 'pinned': 1},
        ), None)
        self.assertEqual(self.window.connection_value.text(), '已连接')
        self.assertEqual(self.window.sync_value.text(), '正在检查变更')
        self.assertEqual(self.window.availability_value.text(), '部分本地可用')
        self.assertIn('可离线打开 4/10', self.window.availability_value.toolTip())
        # A progress update must never rewrite the connection card.
        self.window.render_progress({'active': True, 'phase': 'enumerating', 'units': 3})
        self.assertEqual(self.window.connection_value.text(), '已连接')
        self.assertIn('枚举', self.window.progress_label.text())

    def test_waiting_initialization_means_idle_but_pending_wins(self):
        self.window.received_overview(overview(
            initialization={'active': True, 'phase': 'waiting'},
        ), None)
        self.assertEqual(self.window.sync_value.text(), '暂无待处理变更')
        self.window.received_overview(overview(
            initialization={'active': True, 'phase': 'waiting'},
            pending_operations=27,
        ), None)
        self.assertEqual(self.window.sync_value.text(), '27 项等待同步')

    def test_unknown_initialization_never_claims_idle(self):
        self.window.received_overview(overview(), None)
        self.assertEqual(self.window.sync_value.text(), '尚未核对')
        self.assertIn('按旧后台推导', self.window.sync_value.toolTip())

    def test_backend_sync_state_is_used_when_present(self):
        self.window.received_overview(overview(
            sync_state='transferring', active_transfers=3), None)
        self.assertEqual(self.window.sync_value.text(), '正在传输 3 项')
        self.window.received_overview(overview(
            sync_state='pending', pending_operations=4, uncertain_operations=1), None)
        self.assertEqual(self.window.sync_value.text(), '5 项等待同步')
        self.window.received_overview(overview(sync_state='needs_attention'), None)
        self.assertEqual(self.window.sync_value.text(), '需要处理')

    def test_upload_block_and_paused_are_conservative(self):
        self.window.received_overview(overview(upload_block={'reason': 'quota'}), None)
        self.assertEqual(self.window.sync_value.text(), '上传暂缓')
        self.window.received_overview(overview(connection='paused'), None)
        self.assertEqual(self.window.connection_value.text(), '未连接（同步已暂停）')
        self.assertEqual(self.window.sync_value.text(), '已暂停同步')

    def test_overview_error_keeps_last_confirmed_cards(self):
        self.window.received_overview(overview(), None)
        self.window.received_overview(
            None, 'org.freedesktop.DBus.Error.NoReply')
        self.assertEqual(self.window.connection_value.text(), '已连接')
        self.window.received_overview(None, 'org.freedesktop.DBus.Error.NoReply')
        self.assertEqual(self.window.connection_value.toolTip(), '状态可能过期（上次确认）')

    def test_file_columns_split_sync_and_availability(self):
        self.window.rows = [
            {'id': 'mixed', 'path': 'Mixed', 'directory': True,
             'status': 'mixed', 'offline_available': True},
            {'id': 'online', 'path': 'Online.doc', 'directory': False,
             'status': 'online_only', 'offline_available': False},
            {'id': 'conflict', 'path': 'Conflict.doc', 'directory': False,
             'status': 'conflict', 'offline_available': False},
            {'id': 'init', 'path': 'Init.doc', 'directory': False,
             'status': 'initializing', 'offline_available': False},
            {'id': 'broken', 'path': 'Broken.doc', 'directory': False,
             'status': 'online_only', 'error': 'boom'},
        ]
        self.window.render_rows()
        items = [self.window.table.topLevelItem(index) for index in range(
            self.window.table.topLevelItemCount())]
        by_name = {item.text(0): item for item in items}
        self.assertEqual(by_name['Mixed'].text(1), '暂无待处理变更')
        self.assertEqual(by_name['Mixed'].text(2), '部分本地可用')
        self.assertEqual(by_name['Online.doc'].text(1), '暂无待处理变更')
        self.assertEqual(by_name['Online.doc'].text(2), '仅在线')
        self.assertEqual(by_name['Conflict.doc'].text(1), '需要处理冲突')
        self.assertEqual(by_name['Init.doc'].text(2), '尚未可用')
        self.assertEqual(by_name['Broken.doc'].text(1), '状态读取失败')

    def test_directory_fallback_only_when_backend_has_no_cache_aggregate(self):
        data = {
            'preview_only': False,
            'offset': 0,
            'total': 1,
            'preview_generation': None,
            'entries': [{'id': 'file', 'path': 'A.doc', 'directory': False,
                         'status': 'local_complete', 'offline_available': True}],
        }
        # A directory reply may arrive before the first overview, before any
        # page accumulator exists: it must not crash and should expose the
        # empty fallback until an overview confirms the availability card.
        self.window.files_button.setChecked(True)
        app.processEvents()
        generation = self.window.view_generation
        empty = dict(data, total=0, entries=[])
        self.window.received_directory('', generation, empty, None)
        self.assertEqual(self.window.table.topLevelItemCount(), 0)
        self.assertIn('0 / 0 项', self.window.detail.text())
        self.assertEqual(self.window.availability_value.text(), '0 / 0')
        self.window.received_overview(overview(
            cache_availability={'state': 'partial', 'total': 10, 'offline': 4, 'pinned': 1},
        ), None)
        generation = self.window.view_generation
        self.window.received_directory('', generation, data, None)
        self.assertEqual(self.window.availability_value.text(), '部分本地可用')
        self.window.cache_from_backend = False
        self.window.received_directory('', generation, data, None)
        self.assertEqual(self.window.availability_value.text(), '1 / 1')


class MaximizedOwnerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.env = patch.dict(os.environ, {'XDG_CONFIG_HOME': self.tmp.name})
        self.env.start()
        self.patches = [
            patch('drive_native_gui.window.ClientBus', Bus),
            patch.object(MainWindow, 'start', lambda _: None),
        ]
        for p in self.patches:
            p.start()
        self.host = QMainWindow()
        self.window = MainWindow(instance='')
        self.host.setCentralWidget(self.window)
        self.host.resize(800, 600)
        self.host.show()
        app.processEvents()

    def tearDown(self):
        self.window.transfer_timer.stop()
        self.host.hide()
        self.host.close()
        self.host.deleteLater()
        QCoreApplication.sendPostedEvents(None, QEvent.DeferredDelete)
        for p in reversed(self.patches):
            p.stop()
        self.env.stop()
        self.tmp.cleanup()

    def test_owner_maximized_state_is_restored_after_hide_show(self):
        self.host.showMaximized()
        app.processEvents()
        if not self.host.isMaximized():
            self.host.setWindowState(self.host.windowState() | Qt.WindowMaximized)
            app.processEvents()
        self.assertTrue(self.host.isMaximized())
        self.window.hide()
        app.processEvents()
        self.assertTrue(self.window.restore_maximized)
        self.host.showNormal()
        app.processEvents()
        self.assertFalse(self.host.isMaximized())
        self.window.show()
        for _ in range(4):
            app.processEvents()
        self.assertFalse(self.window.restore_maximized)
        self.assertTrue(self.host.isMaximized())


if __name__ == '__main__':
    unittest.main()
