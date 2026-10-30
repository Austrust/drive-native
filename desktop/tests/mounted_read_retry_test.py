"""Explicit reads of one existing lab file; never writes NAS contents."""

import concurrent.futures
import json
import threading
import time
from pathlib import Path

import gi

gi.require_version("Gio", "2.0")
from gi.repository import Gio, GLib

proxy = Gio.DBusProxy.new_for_bus_sync(
    Gio.BusType.SESSION,
    Gio.DBusProxyFlags.DO_NOT_AUTO_START,
    None,
    "org.drivenative.Client1",
    "/org/drivenative/Client1",
    "org.drivenative.Client1",
    None,
)


def call(method, parameters=None):
    return json.loads(
        proxy.call_sync(method, parameters, Gio.DBusCallFlags.NO_AUTO_START, 5000, None).unpack()[0]
    )


before = call("GetOverview")
assert (
    before["scope_path"]
    == "/DriveNative-Lab/20261028T124720-1e31763a/service-134016b3-afcb-4d63-a130-db4023063ee8"
)
path = Path(before["mount"]) / "A.doc"
rounds = []
for readers in [1, 8]:
    call("ReleaseSpace", GLib.Variant("(s)", (str(path),)))
    start = call("GetOverview")
    barrier = threading.Barrier(readers)

    def read_once(_):
        barrier.wait()
        return path.read_bytes()

    begin = time.monotonic()
    with concurrent.futures.ThreadPoolExecutor(max_workers=readers) as pool:
        contents = list(pool.map(read_once, range(readers)))
    elapsed = time.monotonic() - begin
    assert all(value == b"offline-v3" for value in contents), "wrong existing fixture content"
    end = call("GetOverview")
    attempts = end["downloads_started"] - start["downloads_started"]
    failures = (
        end["download_diagnostics"]["failures_seen"]
        - start["download_diagnostics"]["failures_seen"]
    )
    assert 1 <= attempts <= 2, (readers, attempts)
    assert end["download_errors"] == 0
    if failures:
        assert end["download_diagnostics"]["last_failure"]["server_code"] == 32772
    rounds.append(
        {
            "readers": readers,
            "attempts": attempts,
            "server_32772_retries": failures,
            "single_open_per_reader": True,
            "elapsed_ms": round(elapsed * 1000),
        }
    )
call("ReleaseSpace", GLib.Variant("(s)", (str(path),)))
assert sum(r["server_32772_retries"] for r in rounds) > 0, "server rejection trigger not observed"
print(
    json.dumps(
        {
            "passed": True,
            "native_mounted_reads": True,
            "rounds": rounds,
            "content_verified": True,
            "cache_released_after_test": True,
        }
    )
)
