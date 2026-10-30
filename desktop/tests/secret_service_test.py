"""Exercise actual GNOME Secret Service under isolated HOME and session bus."""

import os, sys, tempfile, subprocess, secrets, json, time, resource
from pathlib import Path

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
if "--inside" not in sys.argv:
    with tempfile.TemporaryDirectory(prefix="drive-native-keyring-test-") as d:
        env = dict(
            os.environ,
            HOME=d,
            XDG_DATA_HOME=d + "/data",
            XDG_CONFIG_HOME=d + "/config",
            XDG_RUNTIME_DIR=d + "/runtime",
        )
        Path(env["XDG_RUNTIME_DIR"]).mkdir(mode=0o700)
        r = subprocess.run(
            ["dbus-run-session", "--", sys.executable, __file__, "--inside"],
            env=env,
            capture_output=True,
            text=True,
            timeout=90,
        )
        print(r.stdout, end="")
        print(r.stderr, end="", file=sys.stderr)
        raise SystemExit(r.returncode)
from PySide6.QtCore import QCoreApplication, QThreadPool
from drive_native_gui.credentials import secret_job

app = QCoreApplication([])
master = secrets.token_urlsafe(32)
control = str(Path(os.environ["XDG_RUNTIME_DIR"]) / "owned-keyring")
Path(control).mkdir(mode=0o700)
os.environ["GNOME_KEYRING_CONTROL"] = control
daemon = subprocess.run(
    ["gnome-keyring-daemon", "--unlock", "--components=secrets", "--control-directory", control],
    input=master + "\n",
    capture_output=True,
    text=True,
    timeout=15,
)
assert daemon.returncode == 0
assert (Path(control) / "control").is_socket()
import secretstorage

connection = secretstorage.dbus_init()
collection = secretstorage.get_default_collection(connection)
assert not collection.is_locked()
task = secrets.token_hex(16)
first = secrets.token_urlsafe(48)
second = secrets.token_urlsafe(48)


def job(password=None):
    out = []
    handle = secret_job(task, lambda value, error: out.append((value, error)), password)
    deadline = time.monotonic() + 15
    while not out:
        app.processEvents()
        if time.monotonic() > deadline:
            raise AssertionError("Secret Service asynchronous operation timed out")
        time.sleep(0.01)
    assert out[0][1] is None, out[0][1]
    return out[0][0]


assert job() is None
assert job(first) is True and job() == first
assert job(second) is True and job() == second
items = list(collection.search_items({"application": "drive-native", "task": task}))
assert len(items) == 1 and items[0].get_secret().decode() == second
files = list((Path(os.environ["XDG_DATA_HOME"]) / "keyrings").glob("*"))
assert files
for path in files:
    if path.is_file():
        content = path.read_bytes()
        assert first.encode() not in content and second.encode() not in content
for item in items:
    item.delete()
assert job() is None
collection.lock()
assert collection.is_locked()
background = []
handle = secret_job(
    task, lambda value, error: background.append((value, error)), allow_unlock=False
)
deadline = time.monotonic() + 10
while not background and time.monotonic() < deadline:
    app.processEvents()
    time.sleep(0.01)
assert background and background[0][0] is None and background[0][1]
assert collection.is_locked(), "background access must not prompt or unlock the keyring"
from keyring_control import unlock_test_keyring

unlock_test_keyring(control, master + "\n")
deadline = time.monotonic() + 5
while collection.is_locked() and time.monotonic() < deadline:
    time.sleep(0.05)
assert not collection.is_locked(), "existing test login keyring did not unlock"
connection.close()
QThreadPool.globalInstance().waitForDone(5000)
print(
    json.dumps(
        {
            "passed": True,
            "provider": "gnome-keyring Secret Service",
            "isolated_session": True,
            "async_write_read_replace": True,
            "single_item_per_task": True,
            "missing_item_returns_none": True,
            "canary_absent_from_keyring_files": True,
            "uses_test_canary_not_nas_password": True,
            "background_locked_keyring_no_prompt": True,
            "existing_keyring_control_unlock": True,
        }
    )
)
