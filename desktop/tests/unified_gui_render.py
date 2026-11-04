"""Render synthetic multi-mount UI, without profiles, services, or a NAS."""
import argparse
import json
from pathlib import Path
from unittest.mock import patch
from PySide6.QtCore import QObject, Signal
from PySide6.QtWidgets import QApplication
from drive_native_gui.manager import MountManager
from drive_native_gui.window import MainWindow
from drive_native_gui.upgrade import BackendUpgrade
from drive_native_gui.theme import install_theme, palette_for

A, B = 'a' * 32, 'b' * 32
RECORDS = [(slot, {'schema': 1, 'task_id': 'synthetic-' + slot, 'mount': '/home/demo/DriveMounts/' + name,
                 'host': 'demo.invalid', 'username': 'demo', 'scope': {'path': '/Demo/' + name}})
           for slot, name in [(A, '工作文档'), (B, '项目资料')]]


class Bus(QObject):
    changed = Signal(); progress_changed = Signal(); availability = Signal(bool)
    owner = ':1.synthetic'
    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent); self.instance = instance or A
    def call(self, method, args=(), callback=None):
        profile = dict(RECORDS)[self.instance]
        if method == 'GetOverview':
            callback(dict(profile, connection='online' if self.instance == A else 'paused', mounted=True,
                          scope_path=profile['scope']['path'], pending_operations=1 if self.instance == A else 0,
                          initialization={'active': False}), None)
        elif method == 'GetInitialization':
            callback({'task_id': profile['task_id'], 'initialization': {'active': False}}, None)
        elif method == 'GetTransfers':
            entries = [{'path': '报告/阶段总结.pdf', 'progress': {'direction': 'upload', 'phase': 'transferring',
                        'bytes': 1048576, 'total': 2097152}}] if self.instance == A else []
            callback({'schema': 1, 'task_id': profile['task_id'], 'entries': entries}, None)
        elif method in ('GetSyncHistory', 'GetRecentActivity'):
            callback({'records': [{'id': 1, 'seq': 1, 'mount': 'synthetic-root', 'node': 'synthetic-file',
                'name': '阶段总结.pdf', 'path_at_event': '报告/阶段总结.pdf', 'action': 'upload', 'result': 'ok',
                'attempts': 2, 'completed_at': 1793750400, 'error_summary': None}], 'next_cursor': None}, None)


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('output')
    output = Path(parser.parse_args().output)
    output.mkdir(parents=True, exist_ok=False)
    app = QApplication([]); install_theme(app)
    with patch('drive_native_gui.manager.profile_records', return_value=RECORDS), patch('drive_native_gui.manager.ClientBus', Bus), patch('drive_native_gui.window.ClientBus', Bus), patch.object(MainWindow, 'start', lambda _: None), patch.object(BackendUpgrade, 'check', lambda _: None):
        manager = MountManager(); manager.show(); app.processEvents()
        manager.grab().save(str(output / 'all-mounts.png'))
        manager.activate_task(A)
        page = manager.pages[A]; page.profile = dict(RECORDS)[A]
        page.bus.call('GetOverview', callback=page.received_overview)
        app.processEvents(); manager.grab().save(str(output / 'mount-details.png'))
        manager.show_history(A); app.processEvents(); manager.grab().save(str(output / 'mount-history.png'))
        manager.show_history(); app.processEvents(); manager.grab().save(str(output / 'all-history.png'))
        manager.tray.show_activity(); app.processEvents()
        manager.tray.activity.grab().save(str(output / 'current-activity.png'))
        manager.tray.activity.hide()
        app.setPalette(palette_for(True)); manager.apply_theme(); manager.activate(); app.processEvents()
        manager.grab().save(str(output / 'all-mounts-dark.png'))
        manager.activate_task(A); app.processEvents(); manager.grab().save(str(output / 'mount-details-dark.png'))
        manager.tray.icon.hide(); manager.hide()
    print(json.dumps({'synthetic_only': True, 'nas_access': False, 'images': 7}))


if __name__ == '__main__': main()
