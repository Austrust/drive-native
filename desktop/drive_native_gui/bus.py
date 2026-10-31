import json
from .config import bus_name
from PySide6.QtCore import QObject, Signal, Slot, SLOT
from PySide6.QtDBus import (
    QDBusConnection,
    QDBusMessage,
    QDBusPendingCallWatcher,
    QDBusArgument,
    QDBusServiceWatcher,
)

NAME = "org.drivenative.Client1"
PATH = "/org/drivenative/Client1"


class ClientBus(QObject):
    changed = Signal()
    progress_changed = Signal()
    availability = Signal(bool)

    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.service = bus_name(instance)
        self.bus = QDBusConnection.sessionBus()
        self.pending = set()
        self.generation = 0
        self.watcher = QDBusServiceWatcher(
            self.service, self.bus, QDBusServiceWatcher.WatchForOwnerChange, self
        )
        self.watcher.serviceOwnerChanged.connect(self._owner_changed)
        reply = self.bus.interface().serviceOwner(self.service)
        self.owner = reply.value() if reply.isValid() else ""
        self.bus.connect(
            self.service, PATH, NAME, "StatusChanged", self, SLOT("_status_changed(qulonglong)")
        )
        self.bus.connect(
            self.service, PATH, NAME, "ProgressChanged", self, SLOT("_progress_changed(qulonglong)")
        )

    @Slot("qulonglong")
    def _progress_changed(self, _revision):
        self.progress_changed.emit()

    @Slot("qulonglong")
    def _status_changed(self, _revision):
        self.changed.emit()

    @Slot(str, str, str)
    def _owner_changed(self, _name, _old, new):
        self.generation += 1
        self.owner = new
        self.availability.emit(bool(new))
        self.changed.emit()

    def call(self, method, args=(), callback=None):
        # Never redirect a credential to a replacement owner of the well-known name.
        destination = (
            self.owner
            if method
            in ("Unlock", "KeepBoth", "SetCacheSettings", "SetTransferSettings", "SetTrustSettings", "Pause", "Resume", "StopMount")
            else self.service
        )
        if not destination:
            if callback:
                callback(None, "service_unavailable")
            return
        message = QDBusMessage.createMethodCall(destination, PATH, NAME, method)
        message.setAutoStartService(False)
        if method == "GetStates":
            array = QDBusArgument()
            # PySide maps the sequence overload to QStringList (signature as).
            # Feeding a bare str inside beginArray selects another QStringList
            # overload and produces an invalid nested array in libdbus.
            array << list(args[0])
            args = [array]
        message.setArguments(list(args))
        generation = self.generation
        watcher = QDBusPendingCallWatcher(self.bus.asyncCall(message, 15000), self)
        self.pending.add(watcher)

        def finished(*_):
            self.pending.discard(watcher)
            if generation != self.generation:
                if callback:
                    callback(None, "service_changed")
                watcher.deleteLater()
                return
            if watcher.isError():
                error = watcher.error().name()
                data = None
            else:
                error = None
                values = watcher.reply().arguments()
                data = values[0] if values else None
                if isinstance(data, str):
                    try:
                        data = json.loads(data)
                        if not isinstance(data, dict) or data.get("schema") != 1:
                            raise ValueError()
                    except (ValueError, TypeError):
                        data, error = None, "invalid_service_response"
            if callback:
                callback(data, error)
            watcher.deleteLater()

        watcher.finished.connect(finished)
