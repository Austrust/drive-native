import json
import os
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
HISTORY_METHODS = frozenset(("GetSyncHistory", "GetRecentActivity", "ResolveHistoryTarget",
                             "ClearSyncHistory", "GetSyncHistoryRetention",
                             "SetSyncHistoryRetention"))
STATUS_SCHEMA = 1
STATUS_KINDS = frozenset(("status", "listing", "subtree"))


class StatusEvent:
    """A normalised ``StatusChanged2`` notification (issue #31).

    Consumers keep the last ``(epoch, to_revision)`` they saw and fall back to a
    full GetOverview whenever the revision stream is discontinuous.
    """

    __slots__ = ("from_revision", "to_revision", "epoch", "reset", "overview_changed",
                 "changes", "truncated", "invalid")

    def __init__(self, from_revision, to_revision, epoch, reset, overview_changed,
                 changes=None, truncated=False, invalid=False):
        self.from_revision = int(from_revision)
        self.to_revision = int(to_revision)
        self.epoch = str(epoch)
        self.reset = bool(reset)
        self.overview_changed = bool(overview_changed)
        self.changes = changes or []
        self.truncated = bool(truncated)
        self.invalid = bool(invalid)

    def discontinuous(self, epoch, revision):
        """True when this event must be applied as a full refresh."""
        return (
            self.invalid
            or self.reset
            or self.truncated
            or epoch != self.epoch
            or revision is None
            or self.from_revision != revision
        )


def parse_status_event(from_revision, to_revision, epoch, reset, overview_changed, changes_json):
    """Validate and normalise the JSON change set carried by StatusChanged2.

    Any malformed payload is surfaced as ``invalid`` so consumers can take the
    full-refresh fallback instead of guessing at partial changes.
    """

    def rejected():
        return StatusEvent(from_revision, to_revision, epoch, reset, overview_changed,
                           changes=[], truncated=False, invalid=True)

    try:
        payload = json.loads(changes_json) if changes_json else None
        if not isinstance(payload, dict) or payload.get("schema") != STATUS_SCHEMA:
            return rejected()
        changes = []
        for item in payload.get("changes") or []:
            if not isinstance(item, dict):
                return rejected()
            path = item.get("path")
            kind = item.get("kind")
            if not isinstance(path, str) or not path.startswith("/"):
                return rejected()
            if kind not in STATUS_KINDS:
                return rejected()
            changes.append({
                "path": os.path.normpath(path),
                "directory": bool(item.get("directory")),
                "kind": kind,
            })
        return StatusEvent(from_revision, to_revision, epoch, reset, overview_changed,
                           changes=changes, truncated=bool(payload.get("truncated")))
    except (ValueError, TypeError):
        return rejected()


class ClientBus(QObject):
    changed = Signal()
    changed2 = Signal(object)
    progress_changed = Signal()
    shutdown_changed = Signal()
    availability = Signal(bool)

    def __init__(self, parent=None, *, instance=None):
        super().__init__(parent)
        self.service = bus_name(instance)
        self.bus = QDBusConnection.sessionBus()
        self.pending = set()
        self.generation = 0
        # Old daemons only emit StatusChanged; once a StatusChanged2 arrives the
        # legacy signal is a duplicate of the same change and is suppressed.
        self.status2_seen = False
        self.watcher = QDBusServiceWatcher(
            self.service, self.bus, QDBusServiceWatcher.WatchForOwnerChange, self
        )
        self.watcher.serviceOwnerChanged.connect(self._owner_changed)
        reply = self.bus.interface().serviceOwner(self.service)
        self.owner = reply.value() if reply.isValid() else ""
        self.bus.connect(
            self.service, PATH, NAME, "StatusChanged", self, SLOT("_status_changed(qulonglong)")
        )
        self.status2_connected = self.bus.connect(
            self.service, PATH, NAME, "StatusChanged2", self,
            SLOT("_status_changed2(qulonglong,qulonglong,QString,bool,bool,QString)")
        )
        self.bus.connect(
            self.service, PATH, NAME, "ProgressChanged", self, SLOT("_progress_changed(qulonglong)")
        )
        self.bus.connect(
            self.service, PATH, NAME, "ShutdownChanged", self, SLOT("_shutdown_changed(qulonglong)")
        )

    @Slot("qulonglong")
    def _shutdown_changed(self, _revision):
        self.shutdown_changed.emit()

    @Slot("qulonglong")
    def _progress_changed(self, _revision):
        self.progress_changed.emit()

    @Slot("qulonglong")
    def _status_changed(self, _revision):
        if not self.status2_seen:
            self.changed.emit()

    @Slot("qulonglong", "qulonglong", "QString", "bool", "bool", "QString")
    def _status_changed2(self, from_revision, to_revision, epoch, reset, overview_changed,
                         changes_json):
        self.status2_seen = True
        self.changed2.emit(parse_status_event(
            from_revision, to_revision, epoch, reset, overview_changed, changes_json))

    @Slot(str, str, str)
    def _owner_changed(self, _name, _old, new):
        self.generation += 1
        self.owner = new
        self.status2_seen = False
        self.availability.emit(bool(new))
        # A replacement owner may speak either protocol: consumers reset their
        # revision tracking and refresh everything once.
        self.changed2.emit(StatusEvent(0, 0, "", True, True, [], invalid=True))
        self.changed.emit()

    def call(self, method, args=(), callback=None):
        # Never redirect a credential to a replacement owner of the well-known name.
        destination = (
            self.owner
            if method
            in ("Unlock", "KeepBoth", "KeepDirectoryBoth", "AbandonUpload", "RecheckIssues", "SetCacheSettings",
                "SetTransferSettings", "SetTrustSettings", "Pause", "Resume", "StopMount",
                "ClearSyncHistory", "SetWritePolicy", "GetSyncHistoryRetention",
                "SetSyncHistoryRetention")
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
                        if not isinstance(data, dict) or (
                            data.get("schema", 1 if method in HISTORY_METHODS else None) != 1
                        ):
                            raise ValueError()
                        if method in HISTORY_METHODS and data.get("error"):
                            error = str(data["error"])
                    except (ValueError, TypeError):
                        data, error = None, "invalid_service_response"
            if callback:
                callback(data, error)
            watcher.deleteLater()

        watcher.finished.connect(finished)

    def get_sync_history(self, query, callback):
        self.call("GetSyncHistory", [json.dumps(query)], callback)

    def get_recent_activity(self, limit, callback):
        self.call("GetRecentActivity", [json.dumps({"limit": limit})], callback)

    def resolve_history_target(self, history_id, callback):
        self.call("ResolveHistoryTarget", [json.dumps({"id": history_id})], callback)

    def clear_sync_history(self, mode, days, callback):
        query = {"mode": mode}
        if mode == "older_than_days" and days is not None: query["days"] = days
        self.call("ClearSyncHistory", [json.dumps(query)], callback)
