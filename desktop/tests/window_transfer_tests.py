"""Overview transfer summary: synthetic GetTransfers replies, no NAS or mount."""
import os
import tempfile
import unittest
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal, QCoreApplication, QEvent
from PySide6.QtWidgets import QApplication
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


def entry(path='A.doc', direction='download', phase='transferring',
          done=10, total=100, elapsed=1000):
    return {
        'request': 1,
        'node': 'node',
        'path': path,
        'elapsed_ms': elapsed,
        'progress': {
            'epoch': 1,
            'direction': direction,
            'phase': phase,
            'bytes': done,
            'total': total,
        },
    }


class TransferSummaryTests(unittest.TestCase):
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

    def take(self, method='GetTransfers'):
        calls = [c for c in self.bus.waiting if c[0] == method]
        self.assertTrue(calls, self.bus.calls)
        self.bus.waiting.remove(calls[0])
        return calls[0][1]

    def reply(self, entries, task=TASK, error=None, schema=1):
        callback = self.take('GetTransfers')
        callback({'schema': schema, 'task_id': task, 'entries': entries}, error)
        app.processEvents()

    def test_empty_state_and_default_timer_refresh(self):
        self.assertIn('GetTransfers', self.bus.calls)
        self.reply([])
        self.assertEqual(self.window.transfer_summary.text(), '当前没有正在传输的文件')
        self.assertTrue(self.window.transfer_table.isHidden())
        self.assertTrue(self.window.transfer_more.isHidden())

    def test_single_upload_summary_and_row(self):
        self.reply([entry('A.doc', direction='upload')])
        self.assertEqual(self.window.transfer_summary.text(), '正在上传 1 项（A.doc）')
        self.assertTrue(self.window.transfer_table.isVisible())
        self.assertEqual(self.window.transfer_table.topLevelItemCount(), 1)
        item = self.window.transfer_table.topLevelItem(0)
        self.assertEqual(item.text(0), 'A.doc')
        self.assertEqual(item.text(1), '上传')
        self.assertEqual(item.text(2), '传输中')
        self.assertIn('10 B / 100 B', item.text(3))
        self.assertTrue(self.window.transfer_more.isHidden())

    def test_mixed_directions_summary(self):
        self.reply([entry('A.doc', direction='upload'),
                    entry('B.doc', direction='upload'),
                    entry('C.doc', direction='download')])
        self.assertEqual(self.window.transfer_summary.text(),
                         '正在传输 3 项（上传 2 项 · 下载 1 项）（A.doc、B.doc、C.doc）')
        self.assertEqual(self.window.transfer_table.topLevelItemCount(), 3)

    def test_many_transfers_are_truncated_with_see_all(self):
        self.reply([entry('file-%02d.doc' % index) for index in range(12)])
        self.assertEqual(self.window.transfer_table.topLevelItemCount(), 8)
        self.assertTrue(self.window.transfer_more.isVisible())
        self.assertEqual(self.window.transfer_more.text(), '查看全部（共 12 项）')

    def test_error_and_bad_schema_clear_the_summary(self):
        for error, schema in (('timeout', 1), (None, 2)):
            self.window.refresh_transfers()
            self.reply([], error=error, schema=schema)
            self.assertEqual(self.window.transfer_summary.text(), '当前传输状态暂不可用。')
            self.assertTrue(self.window.transfer_table.isHidden())

    def test_task_mismatch_is_not_rendered(self):
        self.reply([entry()], task='b' * 32)
        self.assertEqual(self.window.transfer_summary.text(), '任务身份不一致，等待刷新后重试。')
        self.assertTrue(self.window.transfer_table.isHidden())

    def test_late_reply_after_hide_is_discarded(self):
        callback = self.take('GetTransfers')
        self.window.hide()
        app.processEvents()
        callback({'schema': 1, 'task_id': TASK, 'entries': [entry()]}, None)
        app.processEvents()
        self.assertEqual(self.window.transfer_summary.text(), '当前没有正在传输的文件')
        self.assertFalse(self.window.transfer_timer.isActive())

    def test_hidden_page_does_not_query_transfers(self):
        self.window.hide()
        app.processEvents()
        before = len(self.bus.calls)
        self.window.refresh_transfers()
        self.assertEqual(len(self.bus.calls), before)

    def test_overview_refresh_cannot_clear_transfer_summary(self):
        self.reply([entry('A.doc', direction='upload')])
        self.window.received_overview({
            'connection': 'online',
            'mounted': True,
            'pending_operations': 0,
            'mount': '/tmp/synthetic',
            'scope_path': '/Lab',
            'initialization': {'active': True, 'phase': 'waiting'},
        }, None)
        self.assertEqual(self.window.transfer_summary.text(), '正在上传 1 项（A.doc）')
        self.assertEqual(self.window.transfer_table.topLevelItemCount(), 1)
        self.assertEqual(self.window.connection_value.text(), '已连接')

    def test_unreadable_progress_entry_is_not_rendered_as_bytes(self):
        broken = entry()
        broken['progress']['bytes'] = 'not-a-number'
        self.reply([broken])
        self.assertEqual(self.window.transfer_table.topLevelItemCount(), 0)
        self.assertTrue(self.window.transfer_table.isHidden())
        self.assertIn('正在传输 1 项', self.window.transfer_summary.text())

    def test_waiting_progress_is_visible_without_fake_bytes(self):
        waiting = entry()
        waiting.pop('progress')
        self.reply([waiting])
        item = self.window.transfer_table.topLevelItem(0)
        self.assertEqual(item.text(1), '—')
        self.assertEqual(item.text(2), '等待传输')
        self.assertEqual(item.text(3), '等待网络')


if __name__ == '__main__':
    unittest.main()
