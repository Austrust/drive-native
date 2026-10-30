"""Actual user-systemd controller acceptance, no NAS or credentials."""

import json, os, subprocess, tempfile, time, uuid
from pathlib import Path
from PySide6.QtCore import QCoreApplication
from drive_native_gui.lifecycle import ServiceLauncher

app = QCoreApplication([])
unit = "drive-native-launcher-test-" + uuid.uuid4().hex + ".service"


def ctl(*args):
    return subprocess.run(
        ["/usr/bin/systemctl", "--user", *args],
        check=True,
        capture_output=True,
        text=True,
        timeout=15,
    )


def start():
    launcher = ServiceLauncher(managed=True, unit=unit)
    results = []
    launcher.completed.connect(results.append)
    launcher.start()
    limit = time.monotonic() + 25
    while not results and time.monotonic() < limit:
        app.processEvents()
        time.sleep(0.01)
    assert len(results) == 1
    return results[0]


with tempfile.TemporaryDirectory(prefix="drive-native-launcher-test-") as directory:
    folder = Path(directory)
    condition = folder / "ready"
    service = folder / unit
    service.write_text(
        "[Unit]\nConditionPathExists="
        + str(condition)
        + "\n[Service]\nType=oneshot\nExecStart=/usr/bin/true\nRemainAfterExit=yes\n"
    )
    try:
        ctl("link", str(service))
        ctl("daemon-reload")
        assert start() is not None, "skipped condition must not report active"
        condition.touch()
        assert start() is None, "active service should be accepted"
    finally:
        ctl("stop", unit)
        ctl("disable", unit)
        ctl("daemon-reload")
print(
    json.dumps(
        {
            "passed": True,
            "real_user_systemd": True,
            "skipped_condition_reported_as_failure": True,
            "active_service_accepted": True,
        }
    )
)
