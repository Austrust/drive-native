"""Real Nemo host/AT-SPI test; only isolated test XDG paths may be modified."""

import os, sys, json, time, subprocess, shutil, resource, signal
from pathlib import Path

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
import gi

gi.require_version("Nemo", "3.0")
from gi.repository import Gio, GLib
import pyatspi

data = Path(os.environ["XDG_DATA_HOME"])
config = Path(os.environ["XDG_CONFIG_HOME"])
assert str(data).startswith("/tmp/drive-native-gui-") and str(config).startswith(
    "/tmp/drive-native-gui-"
)
module = Path(sys.argv[1])
extensions = data / "nemo-python/extensions"
extensions.mkdir(parents=True, exist_ok=True)
shutil.copyfile(module, extensions / module.name)
icons = data / "icons/hicolor/scalable/emblems"
icons.mkdir(parents=True, exist_ok=True)
for icon in (module.parent / "icons").glob("*.svg"):
    shutil.copyfile(icon, icons / icon.name)
env = dict(
    os.environ, GSETTINGS_BACKEND="keyfile", GTK_MODULES="gail:atk-bridge", NEMO_PYTHON_DEBUG="all"
)


def setting(schema, key, value):
    subprocess.run(
        ["gsettings", "set", schema, key, value], env=env, check=True, capture_output=True
    )


setting("org.nemo.preferences", "default-folder-viewer", "list-view")
setting("org.nemo.list-view", "default-visible-columns", "['name','DriveNative::status','size']")
setting("org.nemo.list-view", "default-column-order", "['name','DriveNative::status','size']")
proxy = Gio.DBusProxy.new_for_bus_sync(
    Gio.BusType.SESSION,
    Gio.DBusProxyFlags.DO_NOT_AUTO_START,
    None,
    "org.drivenative.Client1",
    "/org/drivenative/Client1",
    "org.drivenative.Client1",
    None,
)


def overview():
    return json.loads(
        proxy.call_sync("GetOverview", None, Gio.DBusCallFlags.NO_AUTO_START, 5000, None).unpack()[
            0
        ]
    )


before = overview()
probe_modules = os.environ.get("DRIVE_NATIVE_TEST_GIO_MODULES")
if probe_modules:
    module_dir = Path(probe_modules)
    assert module_dir.is_absolute() and (module_dir / "libdrive-native-probe.so").is_file()
    assert (
        "libdrive-native-probe.so: gio-vfs"
        in (module_dir / "giomodule.cache").read_text().splitlines()
    )
    env.pop("DRIVE_NATIVE_METADATA_ROOT", None)
    assert (module_dir / "scope-mode.txt").read_text().strip() == "registry"
    env.pop("GIO_USE_VFS", None)
    env.update(GIO_EXTRA_MODULES=str(module_dir))

assert before["scope_path"].startswith("/DriveNative-Lab/")
browse_downloads = None
transitions = []

def cell_labels(cell):
    # GTK renderer cells may expose text through Text or a child accessible,
    # while the composite table cell itself has an empty accessible name.
    labels = []
    queue = [cell]
    count = 0
    while queue and count < 32:
        node = queue.pop(0)
        count += 1
        if node.name:
            labels.append(node.name)
        try:
            value = node.queryText().getText(0, -1)
            if value:
                labels.append(value)
        except (NotImplementedError, RuntimeError, AttributeError):
            pass
        queue.extend(node)
    return list(dict.fromkeys(labels))

def table_rows(table):
    query = table.queryTable()
    return [[cell_labels(query.getAccessibleAt(row, col)) for col in range(query.nColumns)]
            for row in range(query.nRows)]

def row_state(table, filename):
    query = table.queryTable()
    headers = [query.getColumnHeader(i).name for i in range(query.nColumns)]
    state_column = headers.index("Drive 状态")
    for cells in table_rows(table):
        if any(filename in labels for labels in cells):
            return cells[state_column]
    return []

def wait_row(table, filename, state):
    deadline = time.monotonic() + 50
    last = None
    while time.monotonic() < deadline:
        last = row_state(table, filename)
        if state in last:
            transitions.append({"file": filename, "state": state})
            return
        if host.poll() is not None:
            raise AssertionError("Nemo exited during state transition")
        time.sleep(0.1)
    raise AssertionError(f"Nemo row {filename}: expected {state}, saw {last}; rows={table_rows(table)}")

log = config.parent / "nemo-host.log"
with log.open("w") as stream:
    trace = config.parent / "nemo-file-open.trace"
    host = subprocess.Popen(
        [
            "strace",
            "-f",
            "-e",
            "trace=openat",
            "-o",
            str(trace),
            "nemo",
            "--no-desktop",
            before["mount"],
        ],
        env=env,
        stdout=stream,
        stderr=stream,
        start_new_session=True,
    )
    try:
        deadline = time.monotonic() + 35
        seen = []
        table = None
        application = None
        while time.monotonic() < deadline:
            desktop = pyatspi.Registry.getDesktop(0)
            for candidate in desktop:
                if candidate.name == "nemo":
                    application = candidate
            if application:
                queue = [application]
                seen = []
                count = 0
                while queue and count < 2000:
                    node = queue.pop(0)
                    count += 1
                    try:
                        if node.name:
                            seen.append(node.name)
                        if node.getRoleName() in ("table", "tree table"):
                            query = node.queryTable()
                            columns = [query.getColumnHeader(i).name for i in range(query.nColumns)]
                            if "Drive 状态" in columns:
                                table = node
                        queue.extend(node)
                    except (NotImplementedError, RuntimeError, AttributeError):
                        pass
            if table and any("仅在线" in name for name in seen):
                break
            if host.poll() is not None:
                raise AssertionError("Nemo exited early")
            time.sleep(0.2)
        if table is None or not any("仅在线" in name for name in seen):
            print(
                json.dumps({"seen": seen, "host_log": log.read_text()}, ensure_ascii=False),
                file=sys.stderr,
            )
            raise AssertionError("actual Nemo status column not visible")
        reads = [line for line in trace.read_text().splitlines() if before["mount"] in line]
        if overview()["downloads_started"] != before["downloads_started"]:
            print(
                json.dumps(
                    {
                        "mount_opens": reads,
                        "before": before["downloads_started"],
                        "after": overview()["downloads_started"],
                    },
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
        assert (
            overview()["downloads_started"] == before["downloads_started"]
        ), "Nemo browsing triggered body download"
        browse_downloads = overview()["downloads_started"]
        # Observe the same live host/window/row through service state signals.
        # This is not a right-click interaction test: commands use local D-Bus.
        wait_row(table, "A.doc", "仅在线")
        path = str(Path(before["mount"]) / "A.doc")
        proxy.call_sync("SetPinned", GLib.Variant("(sb)", (path, True)),
                        Gio.DBusCallFlags.NO_AUTO_START, 5000, None)
        wait_row(table, "A.doc", "已固定")
        proxy.call_sync("SetPinned", GLib.Variant("(sb)", (path, False)),
                        Gio.DBusCallFlags.NO_AUTO_START, 5000, None)
        wait_row(table, "A.doc", "本地可用")
        released = json.loads(proxy.call_sync("ReleaseSpace", GLib.Variant("(s)", (path,)),
                        Gio.DBusCallFlags.NO_AUTO_START, 5000, None).unpack()[0])
        assert not released.get("protected_or_failed"), released
        wait_row(table, "A.doc", "仅在线")
        # Native host owns returned Column objects for its process lifetime.
        subprocess.run(["nemo", "--quit"], env=env, capture_output=True, timeout=10)
        host.wait(timeout=10)
        assert host.returncode == 0, host.returncode
        print(
            json.dumps(
                {
                    "passed": True,
                    "actual_nemo_host": True,
                    "automatic_gio_selection": bool(probe_modules) and "GIO_USE_VFS" not in env,
                    "experimental_gio_metadata": bool(probe_modules),
                    "status_column_visible": True,
                    "online_state_cells_visible": True,
                    "fixture_browse_zero_downloads": True,
                    "normal_host_exit": True,
                    "image_thumbnail_tested": False,
                    "same_window_state_transitions": transitions,
                    "actions_via_dbus_not_host_context_menu": True,
                }
            )
        )
    finally:
        observation = {
            "schema": 1,
            "experimental_gio_metadata": bool(probe_modules),
            "status_column_visible": table is not None,
            "visible_state_cells": [
                name
                for name in seen
                if any(s in name for s in ["仅在线", "本地可用", "正在读取", "已固定"])
            ],
            "downloads_before": before["downloads_started"],
            "downloads_after": browse_downloads if browse_downloads is not None else overview()["downloads_started"],
            "downloads_after_explicit_actions": overview()["downloads_started"],
            "same_window_state_transitions": transitions,
            "mount_opens": [
                line for line in trace.read_text().splitlines() if before["mount"] in line
            ],
        }
        Path(sys.argv[2]).write_text(
            json.dumps(observation, ensure_ascii=False, indent=2) + "\n", encoding="utf8"
        )
        if host.poll() is None:
            subprocess.run(["nemo", "--quit"], env=env, capture_output=True, timeout=10)
            try:
                host.wait(timeout=3)
            except subprocess.TimeoutExpired:
                if os.getpgid(host.pid) == host.pid:
                    os.killpg(host.pid, signal.SIGTERM)
                host.wait(timeout=5)
