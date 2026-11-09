"""Freedesktop notification delivery, exercised without a real session bus.

The D-Bus channel has two halves: the freedesktop ``Notify`` call (sent with
jeepney because PySide6 cannot marshal the ``uint32`` replaced-id argument) and
the ``ActionInvoked`` signal (subscribed with QtDBus).  Both are injected here
through fakes so the tests never need a session bus, a notification daemon or a
running desktop environment.
"""
import unittest
from unittest import mock

from PySide6.QtCore import QObject, Signal
from PySide6.QtWidgets import QApplication
from drive_native_gui import dbus_notifications as dbus
from drive_native_gui.notifications import NotificationCenter

APP = QApplication.instance() or QApplication([])


class FakeReply:
    def __init__(self, *body):
        self.body = body


class FakeJeepneyConnection:
    """Records outgoing freedesktop calls; optionally fails like a broken bus."""

    def __init__(self, reply=None, replies=None, error=None):
        self.reply = reply if reply is not None else FakeReply(7)
        self.replies = list(replies) if replies is not None else None
        self.error = error
        self.sent = []
        self.timeouts = []
        self.closed = False

    def send_and_get_reply(self, message, *, timeout=None):
        self.sent.append(message)
        self.timeouts.append(timeout)
        if self.error is not None:
            raise self.error
        if self.replies is not None:
            return self.replies.pop(0) if self.replies else FakeReply()
        return self.reply

    def close(self):
        self.closed = True


def header_field(message, name):
    fields = message.header.fields
    if isinstance(fields, dict):
        # jeepney 0.9 keys the fields by HeaderFields enum members.
        for key, value in fields.items():
            if getattr(key, 'name', str(key)) == name:
                return str(value)
        return None
    value = getattr(fields, name, None)
    return None if value is None else str(value)


class FakeQtBusConnection:
    """Only the bit of QDBusConnection the action watcher touches."""

    def __init__(self, connected=True, subscribe=True):
        self.connected = connected
        self.subscribe_result = subscribe
        self.calls = []

    def isConnected(self):
        return self.connected

    def connect(self, service, path, interface, name, receiver, slot):
        self.calls.append((service, path, interface, name, slot))
        return self.subscribe_result


class FakeWatcher(QObject):
    # The real watcher forwards the daemon's notification id so the host can
    # resolve the mount that raised the notice.
    clicked = Signal(int)

    def __init__(self):
        super().__init__()
        self.subscribes = 0

    def subscribe(self):
        self.subscribes += 1
        return True


class FakeTray:
    def __init__(self, available=True, show=True, error=None):
        self.is_available = available
        self.show_result = show
        self.error = error
        self.messages = []
        self.available_calls = 0

    def available(self):
        self.available_calls += 1
        return self.is_available

    def show_message(self, title, body, *, warning=False):
        self.messages.append({"title": title, "body": body, "warning": warning})
        if self.error is not None:
            raise self.error
        return self.show_result


class FakeFreedesktop:
    def __init__(self, available=True, error=None):
        self.is_available = available
        self.error = error
        self.messages = []

    def available(self):
        return self.is_available

    def show_message(self, title, body, *, warning=False, target=None):
        self.messages.append({"title": title, "body": body,
                              "warning": warning, "target": target})
        if self.error is not None:
            raise self.error
        return True


def overview(**extra):
    value = dict(task_id='task', connection='online', mounted=True, pending_operations=0,
                 background_error=None, background_error_since_ms=None, error=None,
                 queue_health={'namespace_uncertain': 0, 'upload_uncertain': 0})
    value.update(extra)
    return value


def make_session_bus(connection):
    return dbus.SessionNotificationBus(connection_factory=lambda: connection)


class SessionNotificationBusTests(unittest.TestCase):
    def test_notify_sends_the_freedesktop_contract_with_a_uint32_replaced_id(self):
        connection = FakeJeepneyConnection()
        bus = make_session_bus(connection)
        self.assertTrue(bus.available())
        result = bus.notify('「Archive」需要登录', '打开 Drive Native 解锁后继续同步。')
        self.assertEqual(len(connection.sent), 1)
        message = connection.sent[0]
        self.assertEqual(header_field(message, 'destination'), dbus.SERVICE)
        self.assertEqual(header_field(message, 'path'), dbus.PATH)
        self.assertEqual(header_field(message, 'interface'), dbus.INTERFACE)
        self.assertEqual(header_field(message, 'member'), 'Notify')
        # The whole point of jeepney: the message declares ``u`` for replaced-id
        # instead of the int32 a PySide6 QVariant would produce.
        self.assertEqual(header_field(message, 'signature'), dbus.NOTIFY_SIGNATURE)
        self.assertEqual(message.body, (
            dbus.APPLICATION_NAME,
            0,
            dbus.ICON_INFO,
            '「Archive」需要登录',
            '打开 Drive Native 解锁后继续同步。',
            [dbus.DEFAULT_ACTION, dbus.DEFAULT_ACTION_LABEL],
            {},
            dbus.NOTIFY_TIMEOUT_MS,
        ))
        self.assertEqual(connection.timeouts, [dbus.NOTIFY_CALL_TIMEOUT_S])
        # The daemon-assigned id is what ActionInvoked returns later.
        self.assertEqual(result, 7)

    def test_warning_notifications_use_the_warning_icon(self):
        connection = FakeJeepneyConnection()
        bus = make_session_bus(connection)
        bus.notify('「Archive」需要处理', '打开 Drive Native 查看同步问题。', warning=True)
        self.assertEqual(connection.sent[0].body[2], dbus.ICON_WARNING)

    def test_notify_failure_is_reported_as_runtime_error(self):
        bus = make_session_bus(FakeJeepneyConnection(error=OSError('session bus is gone')))
        self.assertTrue(bus.available())
        with self.assertRaises(RuntimeError):
            bus.notify('title', 'body')

    def test_missing_session_bus_makes_the_backend_unavailable_without_retrying(self):
        calls = []

        def factory():
            calls.append(True)
            raise KeyError('DBUS_SESSION_BUS_ADDRESS')

        bus = dbus.SessionNotificationBus(connection_factory=factory)
        self.assertFalse(bus.available())
        self.assertFalse(bus.available())
        self.assertEqual(len(calls), 1)
        with self.assertRaises(RuntimeError):
            bus.notify('title', 'body')

    def test_missing_jeepney_dependency_degrades_to_unavailable(self):
        factory = mock.Mock(side_effect=AssertionError('must not open a bus'))
        with mock.patch.object(dbus, '_HAVE_JEEPNEY', False):
            bus = dbus.SessionNotificationBus(connection_factory=factory)
            self.assertFalse(bus.available())
            with self.assertRaises(RuntimeError):
                bus.notify('title', 'body')
        factory.assert_not_called()

    def test_close_releases_the_connection_and_allows_reopening(self):
        connections = []

        def factory():
            connections.append(FakeJeepneyConnection())
            return connections[-1]

        bus = dbus.SessionNotificationBus(connection_factory=factory)
        self.assertTrue(bus.available())
        bus.close()
        self.assertTrue(connections[0].closed)
        self.assertTrue(bus.available())
        self.assertEqual(len(connections), 2)


class ActionWatcherTests(unittest.TestCase):
    def test_subscribes_to_action_invoked_with_the_freedesktop_uint_signature(self):
        connection = FakeQtBusConnection()
        watcher = dbus.ActionWatcher(connection=connection)
        self.assertTrue(watcher.subscribe())
        self.assertEqual(len(connection.calls), 1)
        service, path, interface, name, slot = connection.calls[0]
        self.assertEqual((service, path, interface, name),
                         (dbus.SERVICE, dbus.PATH, dbus.INTERFACE, 'ActionInvoked'))
        self.assertIn('action_invoked(uint,QString)', slot)
        # Subscribing twice must not install a duplicate handler.
        self.assertTrue(watcher.subscribe())
        self.assertEqual(len(connection.calls), 1)

    def test_the_default_action_forwards_the_notification_id(self):
        watcher = dbus.ActionWatcher(connection=FakeQtBusConnection())
        watcher.subscribe()
        clicks = []
        watcher.clicked.connect(clicks.append)
        watcher.action_invoked(5, dbus.DEFAULT_ACTION)
        self.assertEqual(clicks, [5])
        # Notifications only carry the default action; never activate on others.
        watcher.action_invoked(6, 'other-action')
        self.assertEqual(clicks, [5])

    def test_missing_qt_session_bus_does_not_subscribe(self):
        connection = FakeQtBusConnection(connected=False)
        watcher = dbus.ActionWatcher(connection=connection)
        self.assertFalse(watcher.subscribe())
        self.assertEqual(connection.calls, [])

    def test_failed_subscription_keeps_the_notification_usable(self):
        connection = FakeQtBusConnection(subscribe=False)
        watcher = dbus.ActionWatcher(connection=connection)
        self.assertFalse(watcher.subscribe())


class FreedesktopNotificationsTests(unittest.TestCase):
    def make_host(self, replies=None):
        self.connection = FakeJeepneyConnection(replies=replies)
        self.watcher = FakeWatcher()
        bus = make_session_bus(self.connection)
        host = dbus.FreedesktopNotifications(bus=bus, watcher=self.watcher)
        return host

    def test_show_message_subscribes_once_and_delivers_through_the_bus(self):
        host = self.make_host()
        self.assertTrue(host.available())
        self.assertTrue(host.show_message('「Archive」需要登录', '打开 Drive Native 解锁后继续同步。'))
        self.assertEqual(self.watcher.subscribes, 1)
        self.assertEqual(len(self.connection.sent), 1)
        self.assertTrue(host.show_message('「Archive」需要登录', '打开 Drive Native 解锁后继续同步。'))
        self.assertEqual(self.watcher.subscribes, 1)
        self.assertEqual(len(self.connection.sent), 2)

    def test_show_message_propagates_bus_failures_for_policy_bookkeeping(self):
        bus = make_session_bus(FakeJeepneyConnection(error=OSError('gone')))
        host = dbus.FreedesktopNotifications(bus=bus, watcher=FakeWatcher())
        with self.assertRaises(RuntimeError):
            host.show_message('title', 'body')

    def test_unavailable_bus_makes_the_host_unavailable_and_unshowable(self):
        def factory():
            raise KeyError('DBUS_SESSION_BUS_ADDRESS')

        host = dbus.FreedesktopNotifications(
            bus=dbus.SessionNotificationBus(connection_factory=factory),
            watcher=FakeWatcher())
        self.assertFalse(host.available())
        with self.assertRaises(RuntimeError):
            host.show_message('title', 'body')

    def test_clicked_signal_bridges_to_policy_click_targeting(self):
        host = self.make_host()
        activated = []
        center = NotificationCenter(host, activated.append)
        host.clicked.connect(center.clicked)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertEqual(center.log[-1]['shown'], True)
        self.assertEqual(center.last_target, 'slot')
        self.watcher.clicked.emit(7)
        self.assertEqual(activated, ['slot'])

    def test_each_notification_remembers_its_mount_for_action_clicks(self):
        host = self.make_host(replies=[FakeReply(11), FakeReply(12)])
        activated = []
        center = NotificationCenter(host, activated.append)
        host.clicked.connect(center.clicked)
        center.observe_overview('A', 'Archive', overview(connection='locked'))
        center.observe_overview('B', 'Backup', overview(connection='locked'))
        # Two notices coexist; the old one must not jump to the newer mount.
        self.watcher.clicked.emit(11)
        self.assertEqual(activated, ['A'])
        self.watcher.clicked.emit(12)
        self.assertEqual(activated, ['A', 'B'])

    def test_unknown_notification_id_falls_back_to_the_latest_target(self):
        host = self.make_host()
        activated = []
        center = NotificationCenter(host, activated.append)
        host.clicked.connect(center.clicked)
        center.observe_overview('A', 'Archive', overview(connection='locked'))
        # A daemon may deliver a click for an id we never recorded (e.g. a
        # notice shown before the GUI restarted); stay useful instead of silent.
        self.watcher.clicked.emit(999)
        self.assertEqual(activated, ['A'])

    def test_reused_notification_id_routes_to_the_newest_target(self):
        host = self.make_host(replies=[FakeReply(7), FakeReply(7)])
        activated = []
        center = NotificationCenter(host, activated.append)
        host.clicked.connect(center.clicked)
        center.observe_overview('A', 'Archive', overview(connection='locked'))
        center.observe_overview('B', 'Backup', overview(connection='locked'))
        # The daemon owns the ids; if it recycles one, the newest notice wins
        # and the stale mapping must not leak to the older mount.
        self.watcher.clicked.emit(7)
        self.assertEqual(activated, ['B'])

    def test_close_closes_the_bus(self):
        host = self.make_host()
        self.assertTrue(host.available())
        host.close()
        self.assertTrue(self.connection.closed)


class NotificationDeliveryTests(unittest.TestCase):
    def test_tray_is_preferred_when_available(self):
        tray = FakeTray(available=True)
        freedesktop = FakeFreedesktop(available=True)
        delivery = dbus.NotificationDelivery(tray, freedesktop)
        self.assertTrue(delivery.available())
        self.assertTrue(delivery.show_message('title', 'body'))
        self.assertEqual(len(tray.messages), 1)
        self.assertEqual(freedesktop.messages, [])

    def test_freedesktop_is_used_when_the_tray_is_missing(self):
        tray = FakeTray(available=False)
        freedesktop = FakeFreedesktop(available=True)
        delivery = dbus.NotificationDelivery(tray, freedesktop)
        self.assertTrue(delivery.available())
        self.assertTrue(delivery.show_message('title', 'body', warning=True))
        self.assertEqual(tray.messages, [])
        self.assertEqual(freedesktop.messages, [{'title': 'title', 'body': 'body',
                                                 'warning': True, 'target': None}])

    def test_delivery_forwards_the_mount_target_to_the_freedesktop_channel(self):
        tray = FakeTray(available=False)
        freedesktop = FakeFreedesktop(available=True)
        delivery = dbus.NotificationDelivery(tray, freedesktop)
        self.assertTrue(delivery.show_message('title', 'body', target='A'))
        self.assertEqual(freedesktop.messages[-1]['target'], 'A')
        # The tray carries no per-notification id; its click still uses the
        # center's remembered latest target, so no target is forwarded there.
        tray2 = FakeTray(available=True)
        delivery2 = dbus.NotificationDelivery(tray2, FakeFreedesktop(available=True))
        self.assertTrue(delivery2.show_message('title', 'body', target='A'))
        self.assertEqual(tray2.messages, [{'title': 'title', 'body': 'body', 'warning': False}])

    def test_freedesktop_is_used_when_the_tray_fails_to_show(self):
        tray = FakeTray(available=True, show=False)
        freedesktop = FakeFreedesktop(available=True)
        delivery = dbus.NotificationDelivery(tray, freedesktop)
        self.assertTrue(delivery.show_message('title', 'body'))
        self.assertEqual(len(tray.messages), 1)
        self.assertEqual(len(freedesktop.messages), 1)

    def test_freedesktop_is_used_when_the_tray_raises(self):
        tray = FakeTray(available=True, error=RuntimeError('tray is gone'))
        freedesktop = FakeFreedesktop(available=True)
        delivery = dbus.NotificationDelivery(tray, freedesktop)
        self.assertTrue(delivery.show_message('title', 'body'))
        self.assertEqual(len(freedesktop.messages), 1)

    def test_delivery_is_unavailable_when_neither_channel_works(self):
        delivery = dbus.NotificationDelivery(FakeTray(available=False),
                                             FakeFreedesktop(available=False))
        self.assertFalse(delivery.available())
        activated = []
        clock = lambda: 1_900_000_000_000
        center = NotificationCenter(delivery, activated.append, now_ms=clock)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertFalse(center.log[-1]['shown'])
        self.assertEqual(activated, [])
        self.assertIsNone(center.clicked())

    def test_dbus_failure_keeps_the_policy_silent_and_logged(self):
        tray = FakeTray(available=False)
        freedesktop = FakeFreedesktop(available=True, error=RuntimeError('daemon rejected'))
        center = NotificationCenter(dbus.NotificationDelivery(tray, freedesktop), [].append)
        center.observe_overview('slot', 'Archive', overview(connection='locked'))
        self.assertFalse(center.log[-1]['shown'])
        self.assertIsNone(center.last_target)


if __name__ == '__main__':
    unittest.main()
