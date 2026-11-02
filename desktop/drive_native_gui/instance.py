"""Session-scoped GUI activation, isolated by the configuration root."""
import hashlib
from PySide6.QtCore import QObject, Signal, Slot
from PySide6.QtDBus import QDBus, QDBusConnection, QDBusMessage
from .config import config_path

PATH = '/org/drivenative/Gui1'


def validate_activation(task, name):
    if not isinstance(task, str) or not isinstance(name, str):
        raise ValueError('无效的界面请求')
    if task: config_path('' if task == 'legacy' else task)
    if name and (not task or name in ('.', '..') or '/' in name or any(ord(c) < 32 for c in name) or len(name.encode()) > 255):
        raise ValueError('无效的挂载名称')


class GuiInstance(QObject):
    activated = Signal(str, str)

    def __init__(self, parent=None):
        super().__init__(parent)
        digest = hashlib.sha256(str(config_path('').parent.resolve()).encode()).hexdigest()[:32]
        self.name = 'org.drivenative.Gui1.c' + digest
        self.bus = QDBusConnection.sessionBus()
        self.primary = False
        self.handler = None
        self.pending_activation = None

    @Slot(str, str, result=bool)
    def Activate(self, task, name):
        try: validate_activation(task, name)
        except ValueError: return False
        if self.handler is None:
            # Startup may enter a nested Qt loop before the manager is ready.
            # Activation is last-wins, just as repeated clicks on the same UI.
            self.pending_activation = (task, name)
        else:
            self.handler(task, name)
        self.activated.emit(task, name)
        return True

    def bind(self, handler):
        self.handler = handler
        pending, self.pending_activation = self.pending_activation, None
        if pending is not None: handler(*pending)

    def claim_or_activate(self, task='', name=''):
        validate_activation(task, name)
        if not self.bus.isConnected():
            # No session bus: keep a visible local window rather than hiding it.
            self.primary = True
            return True
        if self.bus.registerService(self.name):
            if not self.bus.registerObject(PATH, self, QDBusConnection.ExportAllSlots):
                self.bus.unregisterService(self.name)
                raise RuntimeError('无法注册界面激活入口')
            self.primary = True
            return True
        message = QDBusMessage.createMethodCall(self.name, PATH, '', 'Activate')
        message.setArguments([task, name])
        reply = self.bus.call(message, QDBus.Block, 3000)
        if reply.type() == QDBusMessage.ErrorMessage or reply.arguments() != [True]:
            raise RuntimeError('已有界面未响应。后台同步保持运行，请稍后再试。')
        return False
