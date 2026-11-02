"""Render only lifecycle phases actually supplied by the background service."""
PHASES = {
    'requested': '已请求停止', 'stopping_workers': '正在结束网络传输',
    'detaching_mount': '正在卸载目录', 'draining_filesystem': '正在结束文件系统请求',
    'checkpointing_local': '正在保存本地状态', 'complete': '后台已停止',
    'failed': '停止未完成，需要检查',
}


def shutdown_text(value):
    if not isinstance(value, dict) or value.get('phase') not in PHASES: return ''
    text = PHASES[value['phase']]
    elapsed = value.get('stage_elapsed_ms')
    if type(elapsed) is int and elapsed >= 0:
        text += f' · 本阶段 {elapsed // 1000} 秒'
    if value.get('overdue') is True:
        text += '。耗时超过预期，请检查正在使用挂载的程序并保留诊断；尚未确认停止。'
    return text


from PySide6.QtCore import QObject, QTimer


class ShutdownWatcher(QObject):
    """Independent of Store-backed overview requests; validates task identity."""
    def __init__(self, bus, expected_task, render, parent):
        super().__init__(parent)
        self.bus, self.expected_task, self.render = bus, expected_task, render
        self.pending = False
        self.again = False
        self.value = None
        self.mounted = None
        self.generation = 0
        if hasattr(bus, 'shutdown_changed'): bus.shutdown_changed.connect(self.refresh)
        bus.availability.connect(self.availability)

    def availability(self, available):
        self.value = None
        self.mounted = None
        self.generation += 1
        self.pending = False
        self.again = False
        self.render(None, None)
        if available: self.refresh()

    def refresh(self):
        expected = self.expected_task()
        if not expected: return
        if self.pending:
            self.again = True; return
        self.pending = True
        generation = self.generation
        def received(data, error):
            if generation != self.generation: return
            self.pending = False
            if self.again:
                self.again = False
                QTimer.singleShot(100, self, self.refresh)
            if error or not isinstance(data, dict) or data.get('task_id') != expected or expected != self.expected_task(): return
            value = data.get('shutdown')
            if not isinstance(value, dict): return
            self.value = value
            self.mounted = data.get("mounted")
            self.render(value, data.get('mounted'))
        self.bus.call('GetShutdown', callback=received)
