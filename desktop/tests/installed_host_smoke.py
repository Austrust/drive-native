"""Installed Nemo/Nautilus host + real local FSD, using multi_mount_fixture only.

Run as an ordinary user inside a fresh dbus-run-session and Xvfb session.
Arguments: nemo|nautilus, fixture base, new output directory. No NAS login.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

import gi
gi.require_version("Atspi", "2.0")
from gi.repository import Gio, GLib
import pyatspi

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("host", choices=("nemo", "nautilus"))
parser.add_argument("fixture", type=Path)
parser.add_argument("output", type=Path)
args = parser.parse_args()
base = args.fixture.resolve()
args.output.mkdir(parents=True, exist_ok=False)
profile_path = base / "config/drive-native/profile.json"
profile = json.loads(profile_path.read_text())
assert profile["host"] == profile["tls_name"] == "example.invalid"
assert profile["username"] == "fixture" and not profile["writable"]
assert Path(profile["mount"]).is_relative_to(base)
assert not any(k in os.environ for k in ("GIO_USE_VFS", "GIO_EXTRA_MODULES", "PYTHONPATH", "DRIVE_NATIVE_BIN_DIR"))
env = dict(os.environ, XDG_DATA_HOME=str(base / "data"), XDG_CONFIG_HOME=str(base / "config"),
           GSETTINGS_BACKEND="keyfile", GTK_MODULES="gail:atk-bridge", GTK_A11Y="atspi",
           NEMO_PYTHON_DEBUG="all", NAUTILUS_PYTHON_DEBUG="all")
bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)


def call(method, parameters=None):
    reply = bus.call_sync("org.drivenative.Client1", "/org/drivenative/Client1",
                          "org.drivenative.Client1", method, parameters, None,
                          Gio.DBusCallFlags.NO_AUTO_START, 5000, None).unpack()
    return json.loads(reply[0]) if reply else None


def wait(predicate, label, timeout=30):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            value = predicate()
            if value:
                return value
        except GLib.Error:
            pass
        time.sleep(0.15)
    raise AssertionError("timed out: " + label)


def nodes(root=None):
    queue = [root] if root is not None else list(pyatspi.Registry.getDesktop(0))
    visited = 0
    while queue and visited < 4000:
        node = queue.pop(0)
        visited += 1
        try:
            yield node
            queue.extend(node)
        except (RuntimeError, AttributeError):
            pass


def labels(root=None):
    result = []
    for node in nodes(root):
        try:
            if node.name:
                result.append(node.name)
            try:
                text = node.queryText().getText(0, -1)
                if text:
                    result.append(text)
            except (NotImplementedError, RuntimeError, AttributeError):
                pass
        except (RuntimeError, AttributeError):
            pass
    return list(dict.fromkeys(result))


def nemo_menu(label):
    table = next(node for node in nodes() if node.getRoleName() in ('table', 'tree table')
                 and 'A.txt' in labels(node))
    query = table.queryTable()
    row = next(i for i in range(query.nRows) if 'A.txt' in labels(query.getAccessibleAt(i, 0)))
    query.removeRowSelection(row)
    assert query.addRowSelection(row)
    assert table.queryComponent().grabFocus()
    def action(name):
        for node in nodes():
            try:
                if node.name == name and node.getState().contains(pyatspi.STATE_SHOWING) and node.queryAction().nActions:
                    return node
            except (NotImplementedError, RuntimeError, AttributeError):
                pass
        return None
    for name in ('File', 'Drive Native', label):
        assert wait(lambda: action(name), 'Nemo menu ' + name, 10).queryAction().doAction(0)


result = {"passed": False, "host": args.host, "installed_package": True,
          "nas_contacted": False, "commands_via_dbus": True,
          "context_menu_tested": False, "host_extension_menu_tested": False,
          "image_thumbnail_tested": False}
daemon = host = None
fslog = (args.output / "fsd.log").open("w")
hostlog = (args.output / "host.log").open("w")
try:
    daemon = subprocess.Popen(["/usr/lib/drive-native/drive-native-fsd", "--config", str(profile_path)],
                              env=env, stdout=fslog, stderr=fslog)
    before = wait(lambda: (d if (d := call("GetOverview"))["mounted"] and d["connection"] == "locked" else None), "locked cached mount")
    schema = "org.nemo" if args.host == "nemo" else "org.gnome.nautilus"
    for suffix, key, value in [
        ("preferences", "default-folder-viewer", "list-view"),
        ("list-view", "default-visible-columns", "['name','DriveNative::status','size']"),
        ("list-view", "default-column-order", "['name','DriveNative::status','size']"),
    ]:
        subprocess.run(["gsettings", "set", schema + "." + suffix, key, value], env=env, check=True, capture_output=True)
    command = [args.host] + (["--no-desktop"] if args.host == "nemo" else ["--new-window"]) + [profile["mount"]]
    host = subprocess.Popen(command, env=env, stdout=hostlog, stderr=hostlog)
    wait(lambda: "A.txt" in labels(), "fixture visible")
    wait(lambda: any("本地可用" in v for v in labels()), "cached status column")
    result["initial_labels"] = labels()
    assert call("GetOverview")["downloads_started"] == before["downloads_started"] == 0
    path = str(Path(profile["mount"]) / "A.txt")
    transitions = []
    for pinned, text in [(True, "已固定"), (False, "本地可用")]:
        call("SetPinned", GLib.Variant("(sb)", (path, pinned)))
        wait(lambda: any(text in v for v in labels()), "same-window " + text)
        transitions.append(text)
    result["same_window_transitions"] = transitions
    if args.host == 'nemo':
        for label, wanted in [('固定到本地', '已固定'), ('取消固定', '本地可用')]:
            nemo_menu(label)
            wait(lambda: any(wanted in value for value in labels()), 'actual menu ' + label)
        result['host_extension_menu_tested'] = True
        result['menu_access'] = 'Nemo File menu extension actions; not right-click popup'
    result["downloads_started"] = call("GetOverview")["downloads_started"]
    assert result["downloads_started"] == 0
    quit_request = subprocess.run([args.host, "--quit"], env=env, capture_output=True, timeout=10)
    # Noble Nautilus 46.4 returns 255 for the remote quit command even without
    # this extension. The actual host must still exit successfully; do not hide
    # host crashes or accept arbitrary controller failures.
    assert quit_request.returncode in ((0, 255) if args.host == 'nautilus' else (0,))
    result['quit_request_exit'] = quit_request.returncode
    host.wait(timeout=15)
    assert host.returncode == 0
    result['host_exit'] = host.returncode
    result["normal_host_exit"] = True
    assert Path(path).read_text() == "legacy"
    result["cached_body_unchanged"] = True
    result["passed"] = True
except Exception as error:
    result["error"] = str(error)
    result["failure_labels"] = labels()
    raise
finally:
    if host and host.poll() is None:
        subprocess.run([args.host, "--quit"], env=env, capture_output=True, timeout=10)
        host.wait(timeout=15)
    if daemon and daemon.poll() is None:
        call("StopMount")
        daemon.wait(timeout=20)
        result["fsd_exit"] = daemon.returncode
    fslog.close()
    hostlog.close()
    (args.output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2))
    print(json.dumps(result, ensure_ascii=False))
