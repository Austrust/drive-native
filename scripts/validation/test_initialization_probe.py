"""Unit tests and a real isolated session-bus fixture; no NAS or credentials.

Run under dbus-run-session with distribution Python (python3-gi installed).
"""
import argparse
import asyncio
import copy
import hashlib
import json
import pathlib
import subprocess
import sys
import threading
import time
import unittest
import uuid

import initialization_probe as probe


def state():
    value = {key: 0 for key in probe.COUNTERS}
    value.update(active=True, phase='enumerating', recovery_reason='initial_index',
                 since_progress_ms=0, recent_stages=[], phase_totals_ms=[])
    return {'schema': 1, 'initialization': value}


class PureTests(unittest.TestCase):
    def test_allowlist_drops_untrusted_text_at_every_level(self):
        document = state()
        document['secret'] = 'CANARY'
        document['initialization']['secret'] = 'CANARY'
        document['initialization']['recent_stages'] = [dict(
            phase='seeding', started_unix_ms=0, elapsed_ms=2, units=1, items=3, secret='CANARY')]
        self.assertNotIn('CANARY', json.dumps(probe.sanitize_progress(document)))

    def test_invalid_or_unbounded_progress_is_refused(self):
        for key, value in [('phase', 'CANARY'), ('attempt', -1), ('items', True),
                           ('recent_stages', [{}] * 17), ('phase_totals_ms', [[]] * 13)]:
            document = state()
            document['initialization'][key] = value
            with self.assertRaises(ValueError):
                probe.sanitize_progress(document)

    def test_no_progress_requires_contiguous_confirmed_active_samples(self):
        tracker = probe.ProgressTracker()
        value = probe.sanitize_progress(state())
        self.assertEqual(tracker.observe(value, 1), 0)
        self.assertEqual(tracker.observe(value, 5), 4)
        value['items'] += 1
        self.assertEqual(tracker.observe(value, 6), 0)
        self.assertEqual(tracker.observe(value, 8), 2)
        tracker.reset()  # Query gap / owner replacement.
        self.assertEqual(tracker.observe(value, 200), 0)
        value['phase'] = 'waiting'
        self.assertEqual(tracker.observe(value, 900), 0)
        value.update(phase='enumerating', active=False)
        self.assertEqual(tracker.observe(value, 2000), 0)

    def test_partial_stdout_is_joined_and_oversized_output_rejected(self):
        async def run():
            text = await probe.command([sys.executable, '-c',
                'import sys,time;print("one",flush=True);time.sleep(.05);print("two",flush=True)'], 2)
            self.assertEqual(text, 'one\ntwo\n')
            with self.assertRaises(ValueError):
                await probe.command([sys.executable, '-c', 'print("x"*10000)'], 2, limit=100)
        asyncio.run(run())


def integration(output):
    from gi.repository import Gio, GLib
    output.mkdir(mode=0o700, exist_ok=False)
    (output / 'manifest.json').write_text(json.dumps({
        'scope': 'synthetic isolated D-Bus only',
        'probe_sha256': hashlib.sha256(pathlib.Path(probe.__file__).read_bytes()).hexdigest(),
        'test_sha256': hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
        'nas_accessed': False, 'started': time.time()}, indent=2) + '\n')
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    busname = 'org.drivenative.InitializationTest.i' + uuid.uuid4().hex
    xml = '''<node><interface name="org.drivenative.Client1">
    <method name="GetInitialization"><arg type="s" direction="out"/></method>
    <method name="GetOverview"><arg type="s" direction="out"/></method>
    <signal name="ProgressChanged"><arg type="t"/></signal>
    <signal name="Ignored"><arg type="s"/></signal>
    </interface></node>'''
    document = state()
    document['initialization']['secret'] = 'DO_NOT_LOG_SYNTHETIC_CANARY'
    calls = {'GetOverview': 0, 'GetInitialization': 0}
    held = []
    lock = threading.Lock()

    def method(connection, sender, path, interface, name, parameters, invocation):
        with lock:
            calls[name] += 1
            if name == 'GetOverview':
                held.append(invocation)  # Intentionally never answer this lane.
                return
            reply = copy.deepcopy(document)
        invocation.return_value(GLib.Variant('(s)', (json.dumps(reply),)))

    node = Gio.DBusNodeInfo.new_for_xml(xml)
    registration = bus.register_object(probe.PATH, node.interfaces[0], method, None, None)
    owner = Gio.bus_own_name_on_connection(bus, busname, Gio.BusNameOwnerFlags.NONE, None, None)
    loop = GLib.MainLoop()
    thread = threading.Thread(target=loop.run)
    thread.start()
    process = None
    try:
        process = subprocess.Popen([sys.executable, str(pathlib.Path(probe.__file__).resolve()),
            '--bus', busname, '--output', str(output / 'probe'), '--seconds', '6',
            '--interval', '.2', '--overview-interval', '.2', '--timeout', '.3',
            '--stall-seconds', '.5'], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            with lock:
                if calls['GetInitialization'] >= 2 and calls['GetOverview'] >= 1:
                    break
            time.sleep(.05)
        else:
            raise AssertionError('mock service was not queried')
        # A burst must be coalesced, not spawn one query per signal.
        for index in range(1000):
            bus.emit_signal(None, probe.PATH, probe.IFACE, 'ProgressChanged', GLib.Variant('(t)', (index,)))
        bus.emit_signal(None, probe.PATH, probe.IFACE, 'Ignored', GLib.Variant('(s)', ('DO_NOT_LOG_SYNTHETIC_CANARY',)))
        time.sleep(1)
        with lock:
            document['initialization'].update(phase='importing', items=200, units=20)
        bus.emit_signal(None, probe.PATH, probe.IFACE, 'ProgressChanged', GLib.Variant('(t)', (1001,)))
        stdout, stderr = process.communicate(timeout=10)
        assert process.returncode == 0, (process.returncode, stderr.decode(errors='replace'))
        raw = (output / 'probe/observations.jsonl').read_text()
        assert 'DO_NOT_LOG_SYNTHETIC_CANARY' not in raw
        rows = [json.loads(line) for line in raw.splitlines()]
        progress = [r for r in rows if r['kind'] == 'progress']
        assert len(progress) >= 8, len(progress)
        assert len(progress) < 60, len(progress)
        assert any(r['progress']['phase'] == 'importing' for r in progress)
        assert any(r['no_counter_progress'] for r in progress)
        overview_failures = [r for r in rows if r['kind'] == 'query_failed' and r['lane'] == 'overview']
        assert len(overview_failures) >= 3
        assert not any(r['kind'] == 'query_failed' and r['lane'] == 'progress' for r in rows)
        completed = json.loads((output / 'probe/completed.json').read_text())
        assert completed['reason'] == 'window_complete'
        result = {'passed': True, 'scope': 'synthetic session bus only', 'progress_samples': len(progress),
                  'overview_timeouts': len(overview_failures), 'calls': calls,
                  'signal_burst': 1000, 'nas_accessed': False, 'fsd_validation': False}
        (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
    finally:
        if process and process.poll() is None:
            process.terminate()
            process.wait(timeout=5)
        for invocation in held:
            invocation.return_dbus_error('org.drivenative.Test.Ended', 'synthetic fixture ended')
        Gio.bus_unown_name(owner)
        bus.unregister_object(registration)
        loop.quit()
        thread.join(timeout=5)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=pathlib.Path)
    options = parser.parse_args()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(PureTests))
    if not result.wasSuccessful():
        raise SystemExit(1)
    integration(options.output)
