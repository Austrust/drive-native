"""Render only lifecycle phases actually supplied by the background service."""
import time

from PySide6.QtCore import QObject, QTimer

PHASES = {
    'requested': '已请求停止', 'stopping_workers': '正在结束网络传输',
    'detaching_mount': '正在卸载目录', 'draining_filesystem': '正在结束文件系统请求',
    'checkpointing_local': '正在保存本地状态', 'complete': '后台已停止',
    'failed': '停止未完成，需要检查',
}

# Optional blocker supplied by a newer background.  Unknown values are never
# echoed to the UI: the stage text above remains the only claim.
BLOCKING_REASONS = {
    'waiting_for_workers': '等待正在进行的上传/下载和后台任务结束',
    'unmounting_mount': '等待文件系统卸载完成',
    'draining_open_files': '等待应用关闭正在使用的文件',
    'persisting_local_changes': '等待本地状态保存完成',
    'mount_still_present': '挂载点仍在使用，目录尚未卸载',
    'unmount_failed': '卸载目录失败，需要检查占用',
    'checkpoint_failed': '本地状态保存失败，需要检查存储',
    'worker_join_failed': '后台任务未能正常结束',
}

# The background marks a single stage overdue after 30s; the GUI gives the
# whole cooperative stop a longer, monotonic deadline before it stops waiting.
OVERDUE_SECONDS = 30
STOP_DEADLINE_SECONDS = 90


def blocking_reason_text(value):
    """Localised blocker for the active stop stage, or '' when absent/unknown."""
    if not isinstance(value, dict):
        return ''
    reason = value.get('blocking_reason')
    return BLOCKING_REASONS.get(reason, '')


def shutdown_text(value, *, deadline_reached=False):
    if not isinstance(value, dict) or value.get('phase') not in PHASES: return ''
    text = PHASES[value['phase']]
    elapsed = value.get('stage_elapsed_ms')
    if type(elapsed) is int and elapsed >= 0:
        text += f' · 本阶段 {elapsed // 1000} 秒'
    if value.get('overdue') is True and value['phase'] != 'complete':
        text += '。耗时超过预期，请检查正在使用挂载的程序并保留诊断；尚未确认停止。'
    reason = blocking_reason_text(value)
    if reason and value['phase'] not in ('complete', 'failed'):
        text += '。阻塞原因：' + reason
    if deadline_reached and value['phase'] != 'complete':
        text += (f'。已等待超过 {STOP_DEADLINE_SECONDS} 秒，后台仍未确认停止；不会强制结束进程。'
                 '请复制诊断并检查正在使用该目录的程序。')
    return text


class ShutdownDeadline:
    """GUI-side monotonic deadline for a cooperative stop.

    The deadline only controls how long the interface keeps polling; it never
    kills the backend and never turns an unfinished stop into a success.  The
    first observed snapshot starts the clock, and the backend's own elapsed
    time seeds it when the window attaches mid-stop.
    """
    def __init__(self, limit=STOP_DEADLINE_SECONDS, clock=None):
        self.limit = limit
        self.clock = clock or time.monotonic
        self.started = None

    def reset(self):
        self.started = None

    def elapsed(self):
        return 0.0 if self.started is None else max(0.0, self.clock() - self.started)

    def update(self, value):
        """Return True once the whole stop has exceeded the UI deadline."""
        phase = value.get('phase') if isinstance(value, dict) else None
        if phase in (None, 'idle', 'complete', 'failed'):
            self.started = None
            return False
        if self.started is None:
            elapsed = value.get('elapsed_ms')
            offset = elapsed / 1000.0 if type(elapsed) is int and 0 <= elapsed < (1 << 62) else 0.0
            self.started = self.clock() - offset
        return self.elapsed() >= self.limit

    def render(self, value):
        """Stage text including the deadline notice when it has been reached."""
        return shutdown_text(value, deadline_reached=self.update(value))


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
