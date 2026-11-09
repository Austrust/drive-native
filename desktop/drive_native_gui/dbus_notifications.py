"""Freedesktop desktop notifications for systems without a usable tray.

Two transports share the work because no single available library can do both
halves:

* ``Notify`` needs a ``uint32`` replaced-id.  PySide6 maps every Python integer
  to a signed 32-bit ``int``, does not expose ``QVariant`` and cannot choose the
  C++ ``uint`` overload, so a strict notification daemon answers ``InvalidArgs``
  for the whole call.  jeepney serialises from an explicit signature string and
  therefore sends the exact ``susssasa{sv}i`` contract.
* ``ActionInvoked`` arrives as a D-Bus signal with ``uint`` arguments, which
  QtDBus does handle (the same pattern ``bus.py`` uses for ``StatusChanged2``).

jeepney is already part of the pinned GUI dependency set (SecretStorage depends
on it and the PyInstaller entry point imports SecretStorage), but the import is
kept optional: without the package the backend reports ``available()`` false and
callers keep using the tray or the in-window banner.
"""
from collections import OrderedDict

from PySide6.QtCore import QObject, Signal, Slot, SLOT
from PySide6.QtDBus import QDBusConnection

try:
    from jeepney import DBusAddress, new_method_call
    from jeepney.io.blocking import open_dbus_connection
    _HAVE_JEEPNEY = True
except Exception:  # pragma: no cover - exercised only in degraded installs
    DBusAddress = None
    new_method_call = None
    open_dbus_connection = None
    _HAVE_JEEPNEY = False


SERVICE = "org.freedesktop.Notifications"
PATH = "/org/freedesktop/Notifications"
INTERFACE = "org.freedesktop.Notifications"
APPLICATION_NAME = "Drive Native"
DEFAULT_ACTION = "default"
DEFAULT_ACTION_LABEL = "打开"
# freedesktop spec: app_name, replaces_id(uint32), app_icon, summary, body,
# actions(as), hints(a{sv}), expire_timeout(int32).
NOTIFY_SIGNATURE = "susssasa{sv}i"
NOTIFY_TIMEOUT_MS = 8000
NOTIFY_CALL_TIMEOUT_S = 1.0
ICON_INFO = "dialog-information"
ICON_WARNING = "dialog-warning"
# Bound on the notification-id -> mount-slot map.  The daemon has no obligation
# to close ids in order, so stale entries are dropped oldest-first; a click on an
# evicted id falls back to the policy's latest target instead of opening a wrong
# mount.
MAX_TRACKED_NOTIFICATIONS = 32


def _notification_id(reply):
    """Return the daemon-assigned ``uint32`` id from a Notify reply, if any."""
    body = getattr(reply, 'body', None)
    if isinstance(body, (list, tuple)) and body:
        value = body[0]
        if isinstance(value, int) and not isinstance(value, bool):
            return value
    return None


class SessionNotificationBus:
    """The freedesktop ``Notify`` channel, isolated behind an injectable bus.

    ``connection_factory`` is a zero-argument callable returning a jeepney
    connection (``send_and_get_reply(message, timeout=...)`` + ``close()``).
    Tests inject a recording fake; production opens the session bus lazily on
    first use so that a headless session never blocks GUI start-up.
    """

    def __init__(self, *, connection_factory=None, call_timeout=NOTIFY_CALL_TIMEOUT_S):
        self._connection_factory = connection_factory or self._open_session_bus
        self._call_timeout = call_timeout
        self._connection = None
        self._failed = False

    @staticmethod
    def _open_session_bus():
        if not _HAVE_JEEPNEY:
            raise RuntimeError('jeepney is not available')
        return open_dbus_connection(bus='SESSION', auth_timeout=NOTIFY_CALL_TIMEOUT_S)

    def _current_connection(self):
        if not _HAVE_JEEPNEY:
            self._failed = True
            return None
        if self._connection is not None:
            return self._connection
        if self._failed:
            return None
        try:
            self._connection = self._connection_factory()
        except Exception:
            # No session bus (headless session, minimal container, tests):
            # remember the failure so status polling does not retry a blocking
            # connect on every overview refresh.
            self._failed = True
            return None
        return self._connection

    def available(self):
        return self._current_connection() is not None

    def notify(self, title, body, *, warning=False):
        """Show one notification; return its daemon id or raise ``RuntimeError``."""
        connection = self._current_connection()
        if connection is None:
            raise RuntimeError('freedesktop notifications are unavailable')
        message = new_method_call(
            DBusAddress(PATH, bus_name=SERVICE, interface=INTERFACE),
            'Notify',
            NOTIFY_SIGNATURE,
            (
                APPLICATION_NAME,
                0,
                ICON_WARNING if warning else ICON_INFO,
                title,
                body,
                [DEFAULT_ACTION, DEFAULT_ACTION_LABEL],
                {},
                NOTIFY_TIMEOUT_MS,
            ),
        )
        try:
            reply = connection.send_and_get_reply(message, timeout=self._call_timeout)
        except Exception as error:
            raise RuntimeError('freedesktop notification failed: %s' % (error,)) from error
        return _notification_id(reply)

    def close(self):
        connection, self._connection = self._connection, None
        self._failed = False
        if connection is None:
            return
        try:
            connection.close()
        except Exception:
            pass


class ActionWatcher(QObject):
    """Turn freedesktop ``ActionInvoked`` into a tray-style click signal.

    ``clicked`` carries the daemon's notification id so
    :class:`FreedesktopNotifications` can resolve which mount raised the notice;
    the tray signal has no payload, hence the different contract.
    """

    clicked = Signal(int)

    def __init__(self, *, connection=None, parent=None):
        super().__init__(parent)
        self._connection = connection
        self._subscribed = False

    def _qt_bus(self):
        return self._connection if self._connection is not None else QDBusConnection.sessionBus()

    def subscribe(self):
        """Subscribe once; return False when QtDBus has no session bus."""
        if self._subscribed:
            return True
        try:
            connection = self._qt_bus()
            if not connection.isConnected():
                return False
            subscribed = bool(connection.connect(
                SERVICE, PATH, INTERFACE, 'ActionInvoked', self,
                SLOT('action_invoked(uint,QString)')))
        except Exception:
            # QtDBus raises on weird sessions; notifications must keep working
            # even when the click callback cannot be installed.
            return False
        self._subscribed = subscribed
        return subscribed

    @Slot('uint', 'QString')
    def action_invoked(self, notification_id, action_key):
        """Click on the notification body runs its default action."""
        if action_key == DEFAULT_ACTION:
            self.clicked.emit(int(notification_id))


class FreedesktopNotifications(QObject):
    """Notification host backed by the freedesktop D-Bus service.

    Implements the same host contract as ``MountTray``: ``available()`` and
    ``show_message(title, body, *, warning=False, target=None)``.  Each delivered
    notice records the mount slot that raised it under the daemon-assigned id, so
    ``clicked`` can carry the right target back to
    :meth:`NotificationCenter.clicked` even when several mounts have notices on
    screen at once.  Unknown ids emit ``None``, which makes the policy fall back
    to its latest-target click behaviour.
    """

    clicked = Signal(object)

    def __init__(self, *, bus=None, watcher=None, parent=None):
        super().__init__(parent)
        self.bus = bus if bus is not None else SessionNotificationBus()
        self.watcher = watcher if watcher is not None else ActionWatcher(parent=self)
        self.watcher.clicked.connect(self._on_action_invoked)
        self._targets = OrderedDict()
        self._subscribed = False

    def available(self):
        try:
            return bool(self.bus.available())
        except Exception:
            return False

    def show_message(self, title, body, *, warning=False, target=None):
        """Deliver through D-Bus; raises ``RuntimeError`` when that fails."""
        if not self._subscribed:
            # A failed subscription (no QtDBus session bus) is retried once per
            # notification; it must not block delivery of the notice itself.
            self._subscribed = bool(self.watcher.subscribe())
        notification_id = self.bus.notify(title, body, warning=warning)
        if target is not None and notification_id is not None:
            self._remember_target(notification_id, target)
        return True

    def _remember_target(self, notification_id, target):
        self._targets[notification_id] = target
        while len(self._targets) > MAX_TRACKED_NOTIFICATIONS:
            self._targets.popitem(last=False)

    def _on_action_invoked(self, notification_id):
        target = self._targets.pop(notification_id, None)
        self.clicked.emit(target)

    def close(self):
        self.bus.close()


class NotificationDelivery:
    """Composite host: the tray when it can show, freedesktop D-Bus otherwise."""

    def __init__(self, tray, freedesktop):
        self.tray = tray
        self.freedesktop = freedesktop

    def _tray_available(self):
        if self.tray is None:
            return False
        try:
            return bool(self.tray.available())
        except (AttributeError, RuntimeError):
            return False

    def available(self):
        if self._tray_available():
            return True
        if self.freedesktop is None:
            return False
        try:
            return bool(self.freedesktop.available())
        except (AttributeError, RuntimeError):
            return False

    def show_message(self, title, body, *, warning=False, target=None):
        if self._tray_available():
            try:
                if self.tray.show_message(title, body, warning=warning):
                    return True
            except (AttributeError, RuntimeError):
                pass
        if self.freedesktop is None:
            return False
        # Policy bookkeeping treats a raised RuntimeError as "not shown", which
        # is exactly what a dead notification daemon is.  The freedesktop
        # channel keeps the per-notification target so its click can find the
        # mount; the tray keeps using the center's latest target.
        return bool(self.freedesktop.show_message(title, body, warning=warning, target=target))
