"""Private D-Bus/FUSE synthetic GUI-controller handover; no NAS or keyring."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
from PySide6.QtCore import QCoreApplication
from drive_native_gui.bus import ClientBus
from drive_native_gui.lifecycle import ServiceLauncher
from drive_native_gui.upgrade import BackendUpgrade

parser = argparse.ArgumentParser()
parser.add_argument("root", type=Path)
parser.add_argument("fixture", type=Path)
parser.add_argument("old_binaries", type=Path)
parser.add_argument("new_binaries", type=Path)
args = parser.parse_args()
assert os.getuid() != 0 and not args.root.exists()
os.umask(0o077)
args.root.mkdir()
subprocess.run([str(args.fixture), str(args.root / "fixture")], check=True, stdout=subprocess.DEVNULL)
fixture = args.root / "fixture"
config = fixture / "config/drive-native/profile.json"
profile = json.loads(config.read_text())
assert profile["username"] == "fixture" and profile["host"] == "example.invalid"
profile.update(host="127.0.0.1", tls_name="localhost", writable=True)
config.write_text(json.dumps(profile))
installed = args.root / "installed"
installed.mkdir()

def replace(source):
    for name in ("drive-native-fsd", "drive-native-worker"):
        temp = installed / (name + ".new")
        shutil.copy2(source / name, temp)
        temp.replace(installed / name)

replace(args.old_binaries)
os.environ.update(XDG_CONFIG_HOME=str(fixture / "config"), XDG_DATA_HOME=str(fixture / "data"), DRIVE_NATIVE_BIN_DIR=str(installed))
app = QCoreApplication([])
bus = ClientBus()
launcher = ServiceLauncher(managed=False)
upgrade = BackendUpgrade(bus, launcher)

def wait(check, seconds=20):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        app.processEvents()
        if check():
            return
        time.sleep(.01)
    raise AssertionError("bounded GUI condition did not complete")

def call(method):
    reply = []
    bus.call(method, callback=lambda data, error: reply.append((data, error)))
    wait(lambda: reply)
    assert not reply[0][1], reply[0][1]
    return reply[0][0]

old = subprocess.Popen([str(installed / "drive-native-fsd"), "--config", str(config)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
try:
    wait(lambda: bus.owner)
    wait(lambda: Path(profile["mount"]).is_mount())
    file = Path(profile["mount"]) / "A.txt"
    with file.open("r+b") as stream:
        stream.write(b"GUI pending handover")
        stream.truncate()
        stream.flush()
        os.fsync(stream.fileno())
    replace(args.new_binaries)
    upgrade.check()
    wait(lambda: upgrade.available)
    outcomes = []
    upgrade.finished.connect(outcomes.append)
    previous_owner = bus.owner
    assert upgrade.begin(profile)
    wait(lambda: outcomes, 40)
    assert outcomes == [None], outcomes
    wait(lambda: bus.owner and bus.owner != previous_owner)
    wait(lambda: Path(profile["mount"]).is_mount())
    overview = call("GetOverview")
    assert overview["connection"] == "paused"
    assert overview["pending_operations"] == 1
    assert file.read_bytes() == b"GUI pending handover"
    old.wait(timeout=5)
    assert old.returncode == 0
    result = dict(owner_detection=True, cooperative_handover=True, new_backend_paused=True,
                  pending_edit_preserved=True, old_exit=0, nas_accessed=False)
    (args.root / "result.json").write_text(json.dumps(result, indent=2))
    print(json.dumps(result))
finally:
    if bus.owner:
        bus.call("StopMount")
        wait(lambda: not bus.owner, 30)
    if old.poll() is None:
        old.terminate()
        old.wait(timeout=30)
