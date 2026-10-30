"""Explicit real-user-systemd + Secret Service + NAS read-only lifecycle test."""

import getpass, json, os, resource, secrets, subprocess, sys, tempfile, time, uuid
from pathlib import Path

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
assert os.getuid() != 0
assert os.environ["DBUS_SESSION_BUS_ADDRESS"] == f"unix:path=/run/user/{os.getuid()}/bus"
from PySide6.QtCore import QCoreApplication
from drive_native_gui.bus import ClientBus
from drive_native_gui.lifecycle import ServiceLauncher
import secretstorage

project = Path(__file__).resolve().parents[2]
bins = Path(os.environ["DRIVE_NATIVE_BIN_DIR"]).resolve()
frozen_gui = os.environ.get("DRIVE_NATIVE_TEST_FROZEN_GUI")
if frozen_gui:
    assert Path(frozen_gui).is_absolute() and Path(frozen_gui).is_file()
scope = "/DriveNative-Lab/20261028T124720-1e31763a/service-134016b3-afcb-4d63-a130-db4023063ee8"
output = Path(sys.argv[1])
output.write_text('{"passed":false,"stage":"starting"}\n')
app = QCoreApplication([])


def command(args, **kw):
    return subprocess.run(args, check=True, text=True, capture_output=True, timeout=25, **kw)


def ctl(*args):
    return command(["/usr/bin/systemctl", "--user", *args])


def wait(predicate, label, timeout=80):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        app.processEvents()
        value = predicate()
        if value:
            return value
        time.sleep(0.05)
    raise AssertionError(label)


def name_owned(name):
    return (
        command(
            [
                "gdbus",
                "call",
                "--session",
                "--dest",
                "org.freedesktop.DBus",
                "--object-path",
                "/org/freedesktop/DBus",
                "--method",
                "org.freedesktop.DBus.NameHasOwner",
                name,
            ]
        ).stdout.strip()
        == "(true,)"
    )


assert not name_owned("org.drivenative.Client1"), "existing sync service must not be replaced"
assert not name_owned("org.freedesktop.secrets"), "existing keyring service must not be replaced"
password = getpass.getpass("Test account password (private isolated keyring only): ")
base = Path(tempfile.mkdtemp(prefix="drive-native-managed-test-"))
config = base / "config"
data = base / "data"
mount = base / "mount"
for path in [config / "drive-native", data, mount, base / "keyring"]:
    path.mkdir(parents=True, mode=0o700)
task = str(uuid.uuid4())
device = str(uuid.uuid4())
unit = f"drive-native-lifecycle-{uuid.uuid4().hex}.service"
unlock = unit[:-8] + "-unlock.service"
helper = None
keyring = None
connection = None
collection = None
item = None
linked = []
bus = ClientBus()


def call(method, args=()):
    results = []
    bus.call(method, args, lambda value, error: results.append((value, error)))
    wait(lambda: bool(results), "D-Bus response " + method, 20)
    if results[0][1]:
        raise RuntimeError("D-Bus operation failed: " + method)
    return results[0][0]


def overview():
    try:
        return call("GetOverview")
    except RuntimeError:
        return {}


def online():
    value = overview()
    return value if value.get("connection") == "online" and value.get("mounted") else None


def setup_request(payload):
    helper.stdin.write(json.dumps(payload) + "\n")
    helper.stdin.flush()
    response = json.loads(helper.stdout.readline())
    assert response["ok"], response.get("error")
    return response["data"]


def env_line(key, value):
    return (
        'Environment="'
        + key
        + "="
        + str(value).replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%")
        + '"\n'
    )


try:
    helper = subprocess.Popen(
        [str(bins / "drive-native-setup"), "--private-stdin"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    login = setup_request(
        dict(
            command="login",
            host="192.0.2.10",
            tls_name="nas.example.invalid",
            username="test-user",
            password=password,
            device_id=device,
        )
    )
    folders = login["folders"]
    selected = None
    for index, segment in enumerate(scope.strip("/").split("/")):
        selected = next(x for x in folders if x["name"] == segment)
        if index < 2:
            folders = setup_request(dict(command="list", path=selected["path"]))["folders"]
    helper.stdin.write('{"command":"quit"}\n')
    helper.stdin.flush()
    helper.wait(timeout=10)
    profile = dict(
        schema=1,
        task_id=task,
        host="192.0.2.10",
        tls_name="nas.example.invalid",
        username="test-user",
        device_id=device,
        scope=selected["scope"],
        mount=str(mount),
        writable=False,
    )
    profile_path = config / "drive-native/profile.json"
    profile_path.write_text(json.dumps(profile))
    profile_path.chmod(0o600)
    secret_env = dict(
        os.environ,
        XDG_CONFIG_HOME=str(config),
        XDG_DATA_HOME=str(data),
        GNOME_KEYRING_CONTROL=str(base / "keyring"),
    )
    master = secrets.token_urlsafe(32)
    keyring = subprocess.Popen(
        [
            "gnome-keyring-daemon",
            "--foreground",
            "--unlock",
            "--components=secrets",
            "--control-directory",
            str(base / "keyring"),
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=secret_env,
        text=True,
    )
    keyring.stdin.write(master + "\n")
    keyring.stdin.close()
    wait(lambda: name_owned("org.freedesktop.secrets"), "isolated keyring ready", 10)
    connection = secretstorage.dbus_init()
    collection = secretstorage.get_default_collection(connection)
    assert not collection.is_locked()
    item = collection.create_item(
        "Drive Native lifecycle test",
        {"application": "drive-native", "task": task},
        password.encode(),
        replace=False,
    )

    common = "".join(
        env_line(k, v)
        for k, v in {
            "XDG_CONFIG_HOME": config,
            "XDG_DATA_HOME": data,
            "PYTHONPATH": project / "desktop",
        }.items()
    )
    original = (project / "desktop/drive-native.service").read_text()
    service = original.replace("%E/drive-native/profile.json", str(profile_path)).replace(
        "drive-native-unlock.service", unlock
    )
    service = service.replace(
        "/usr/lib/drive-native/drive-native-fsd", str(bins / "drive-native-fsd")
    )
    service = service.replace("[Service]\n", "[Service]\n" + common)
    one = (
        (project / "desktop/drive-native-unlock.service")
        .read_text()
        .replace("drive-native.service", unit)
    )
    one = one.replace("%E/drive-native/profile.json", str(profile_path)).replace(
        "/usr/lib/drive-native/gui/drive-native-gui --unlock-service",
        (
            (frozen_gui + " --unlock-service")
            if frozen_gui
            else (sys.executable + " -m drive_native_gui.startup")
        ),
    )
    one = one.replace("[Service]\n", "[Service]\n" + common)
    for name, body in [(unit, service), (unlock, one)]:
        file = base / name
        file.write_text(body)
        ctl("link", str(file))
        linked.append(name)
    ctl("daemon-reload")
    launcher = ServiceLauncher(managed=True, unit=unit)
    result = []
    launcher.completed.connect(lambda error: result.append(error))
    launcher.start(enable=True)
    wait(lambda: bool(result), "managed first start", 25)
    assert result[0] is None, result[0]
    first = wait(online, "keyring automatically unlocks native service")
    first_pid = ctl("show", unit, "--property=MainPID", "--value").stdout.strip()
    assert ctl("is-enabled", unit).stdout.strip() == "enabled"
    call("SetPinned", ["A.doc", True])

    def pinned():
        states = call("GetStates", [["A.doc"]])["entries"]
        return states and states[0].get("status") == "pinned_complete"

    wait(pinned, "native pinned content")
    assert (mount / "A.doc").read_bytes() == b"offline-v3"
    print("PASS user service enable/start and saved keyring native login without GUI", flush=True)

    collection.lock()
    assert collection.is_locked()
    ctl("restart", unit)

    def cached_locked():
        value = overview()
        return value if value.get("mounted") and value.get("connection") == "locked" else None

    wait(cached_locked, "restart mounts complete cache while keyring locked")
    assert (mount / "A.doc").read_bytes() == b"offline-v3"
    assert collection.is_locked()
    second_pid = ctl("show", unit, "--property=MainPID", "--value").stdout.strip()
    assert first_pid != second_pid
    from keyring_control import unlock_test_keyring

    unlock_test_keyring(base / "keyring", master + "\n")
    wait(lambda: not collection.is_locked(), "keyring unlock", 10)
    wait(online, "background helper resumes native connection after keyring unlock")
    assert (mount / "A.doc").read_bytes() == b"offline-v3"
    print(
        "PASS restart with locked keyring preserves offline cache; unlock resumes without GUI",
        flush=True,
    )
    call("StopMount")
    wait(lambda: not mount.is_mount(), "explicit stop unmounts", 15)
    wait(
        lambda: ctl("show", unit, "--property=ActiveState", "--value").stdout.strip() == "inactive",
        "normal stop stays stopped",
        15,
    )
    journal = command(
        ["journalctl", "--user", "-u", unit, "-u", unlock, "--no-pager", "-o", "cat"]
    ).stdout
    assert password not in journal and master not in journal
    assert password not in profile_path.read_text()
    for file in (data / "keyrings").glob("*"):
        if file.is_file():
            assert password.encode() not in file.read_bytes()
    output.write_text(
        json.dumps(
            {
                "passed": True,
                "real_user_systemd": True,
                "isolated_user_units": True,
                "native_server_read_only_fixture": scope,
                "first_setup_enable_and_start": True,
                "keyring_auto_unlock_without_gui": True,
                "keyring_locked_restart_cache_available": True,
                "later_keyring_unlock_reconnects": True,
                "explicit_stop_not_restarted": True,
                "credentials_absent_from_profile_journal_and_keyring_bytes": True,
                "full_os_logout_login_tested": False,
                "frozen_helper_tested": bool(frozen_gui),
            },
            indent=2,
        )
        + "\n"
    )
finally:
    for name in reversed(linked):
        subprocess.run(
            ["/usr/bin/systemctl", "--user", "stop", name], capture_output=True, timeout=30
        )
        subprocess.run(
            ["/usr/bin/systemctl", "--user", "disable", name], capture_output=True, timeout=15
        )
    if linked:
        ctl("daemon-reload")
    if item:
        try:
            item.delete()
        except Exception:
            pass
    if connection:
        connection.close()
    if helper and helper.poll() is None:
        helper.kill()
        helper.wait(timeout=5)
    if keyring and keyring.poll() is None:
        keyring.terminate()
        keyring.wait(timeout=10)
    print("test_state=" + str(base), flush=True)
