"""A monitor's files must retain metadata-only behavior after external rename."""

import json
import os
import tempfile
from pathlib import Path

from gi.repository import Gio, GLib

with tempfile.TemporaryDirectory(prefix="drive-native-gio-monitor-") as tmp:
    root = Path(tmp)
    os.environ["DRIVE_NATIVE_METADATA_ROOT"] = str(root)
    # This is the only body creation. Rename below never opens the body.
    body = root / "original.doc"
    body.write_bytes(b"opaque body")
    folder = Gio.File.new_for_path(tmp)
    monitor = folder.monitor_directory(Gio.FileMonitorFlags.WATCH_MOVES, None)
    events = []
    errors = []
    loop = GLib.MainLoop()

    def changed(_monitor, file, other, event):
        try:
            for item in (file, other):
                if item is not None:
                    assert item.__gtype__.name == "ProbeFile"
                    if Path(item.get_path()).exists():
                        info = item.query_info("*", Gio.FileQueryInfoFlags.NONE, None)
                        assert info.get_size() == 11
            events.append(int(event))
        except Exception as error:
            errors.append(str(error))
        loop.quit()

    monitor.connect("changed", changed)
    body.rename(root / "renamed.doc")

    def timeout():
        errors.append("monitor event did not arrive")
        loop.quit()
        return False

    timer = GLib.timeout_add_seconds(5, timeout)
    loop.run()
    if events:
        GLib.source_remove(timer)
    assert not errors, errors
    assert events, "no rename event"
    assert monitor.cancel()
    assert monitor.is_cancelled()
    before = len(events)
    (root / "renamed.doc").rename(root / "after-cancel.doc")
    quiet = GLib.MainLoop()
    GLib.timeout_add(300, lambda: (quiet.quit(), False)[1])
    quiet.run()
    assert len(events) == before
    print(
        json.dumps(
            {
                "passed": True,
                "event_files_wrapped": True,
                "cancel_stops_events": True,
                "events": events,
                "scope": "isolated_local_filesystem",
            }
        )
    )
