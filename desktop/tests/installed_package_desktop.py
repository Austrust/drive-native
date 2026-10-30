"""Installed desktop host, cached fixture only; no NAS authentication or writes."""

import gc, json, os, resource, shutil, subprocess, time
from pathlib import Path

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
import gi

gi.require_version("Gdk", "3.0")
from gi.repository import Gio, GLib
from gi.repository import Gdk
import pyatspi

base = Path("/home/test-user/package-test-fixed/run")
out = base.parent / "desktop"
out.mkdir(mode=0o700, exist_ok=True)
env = dict(
    os.environ,
    XDG_DATA_HOME=str(base / "data"),
    XDG_CONFIG_HOME=str(out / "config"),
    GSETTINGS_BACKEND="keyfile",
    GTK_MODULES="gail:atk-bridge",
    NEMO_PYTHON_DEBUG="all",
)
assert not any(
    k in env for k in ("GIO_USE_VFS", "GIO_EXTRA_MODULES", "PYTHONPATH", "DRIVE_NATIVE_BIN_DIR")
)
profile_dir = out / "config" / "drive-native"
profile_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
shutil.copyfile(base / "profile.json", profile_dir / "profile.json")
(profile_dir / "profile.json").chmod(0o600)
profile = json.loads((base / "profile.json").read_text())
assert profile["scope"]["path"].startswith(
    "/DriveNative-Lab/20261028T124720-1e31763a/package-upgrade-"
)
bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)


def call(method, args=None):
    v = bus.call_sync(
        "org.drivenative.Client1",
        "/org/drivenative/Client1",
        "org.drivenative.Client1",
        method,
        args,
        None,
        Gio.DBusCallFlags.NO_AUTO_START,
        5000,
        None,
    ).unpack()
    return json.loads(v[0]) if v else None


def wait(test, label, timeout=40):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        try:
            result = test()
            if result:
                return result
        except GLib.Error:
            pass
        time.sleep(0.1)
    raise AssertionError(label)


def nodes(root):
    todo = [root]
    count = 0
    while todo and count < 3000:
        node = todo.pop(0)
        count += 1
        yield node
        try:
            todo.extend(node)
        except (RuntimeError, AttributeError):
            pass


def applications():
    return list(pyatspi.Registry.getDesktop(0))


def find_table():
    for app in applications():
        if app.name != "nemo":
            continue
        for node in nodes(app):
            try:
                if node.getRoleName() in ("table", "tree table"):
                    q = node.queryTable()
                    if "Drive 状态" in [q.getColumnHeader(i).name for i in range(q.nColumns)]:
                        return node
            except (NotImplementedError, RuntimeError, AttributeError):
                pass


def labels(cell):
    result = []
    for node in nodes(cell):
        if node.name:
            result.append(node.name)
        try:
            result.append(node.queryText().getText(0, -1))
        except (NotImplementedError, RuntimeError, AttributeError):
            pass
    return list(dict.fromkeys(filter(None, result)))


def rows(table):
    q = table.queryTable()
    return [[labels(q.getAccessibleAt(r, c)) for c in range(q.nColumns)] for r in range(q.nRows)]


def state(table, filename, wanted):
    expected = wanted + " · 未连接同步"
    return any(
        any(filename in cell for cell in row) and any(expected in cell for cell in row)
        for row in rows(table)
    )


def action_named(name):
    for app in applications():
        if app.name != "nemo":
            continue
        for node in nodes(app):
            if node.name == name and node.getState().contains(pyatspi.STATE_SHOWING):
                try:
                    action = node.queryAction()
                    if action.nActions:
                        return node
                except NotImplementedError:
                    pass


def menu_command(table, filename, label):
    q = table.queryTable()
    selected = next(r for r in range(q.nRows) if filename in labels(q.getAccessibleAt(r, 0)))
    q.removeRowSelection(selected)
    assert q.addRowSelection(selected)
    assert table.queryComponent().grabFocus()
    file_menu = wait(lambda: action_named("File"), "Nemo File menu", 10)
    assert file_menu.queryAction().doAction(0)
    root = wait(lambda: action_named("Drive Native"), "actual Drive Native File menu", 10)
    assert root.queryAction().doAction(0)
    item = wait(lambda: action_named(label), f"actual menu {label}", 10)
    assert item.queryAction().doAction(0)


result = {
    "passed": False,
    "installed_package": True,
    "display": "Xvfb",
    "nas_contacted": False,
    "upstream_column_teardown_defect_separate": True,
}
daemon = host = gui = None
fslog = (out / "fsd.log").open("w")
nlog = (out / "nemo.log").open("w")
glog = (out / "gui.log").open("w")
try:
    daemon = subprocess.Popen(
        ["/usr/lib/drive-native/drive-native-fsd", "--config", str(base / "profile.json")],
        env=env,
        stdout=fslog,
        stderr=fslog,
    )
    overview = wait(
        lambda: (
            v if (v := call("GetOverview"))["mounted"] and v["connection"] == "locked" else None
        ),
        "cached locked mount",
    )
    for schema, key, value in [
        ("org.nemo.preferences", "default-folder-viewer", "list-view"),
        ("org.nemo.list-view", "default-visible-columns", "['name','DriveNative::status','size']"),
        ("org.nemo.list-view", "default-column-order", "['name','DriveNative::status','size']"),
    ]:
        subprocess.run(["gsettings", "set", schema, key, value], env=env, check=True)
    host = subprocess.Popen(
        ["nemo", "--no-desktop", profile["mount"]], env=env, stdout=nlog, stderr=nlog
    )
    table = wait(find_table, "installed Nemo status column")
    for name, wanted in [
        ("A.doc", "本地可用"),
        ("Created.doc", "等待同步"),
        ("Pinned.doc", "已固定"),
    ]:
        wait(lambda: state(table, name, wanted), f"{name} {wanted}")
    result["initial_rows"] = rows(table)
    assert call("GetOverview")["downloads_started"] == overview["downloads_started"] == 0
    # Changes use the same local D-Bus commands as menus; actual menu interaction tested separately.
    call("SetPinned", GLib.Variant("(sb)", ("Pinned.doc", False)))
    wait(lambda: state(table, "Pinned.doc", "本地可用"), "same-window unpin")
    call("SetPinned", GLib.Variant("(sb)", ("Pinned.doc", True)))
    wait(lambda: state(table, "Pinned.doc", "已固定"), "same-window repin")
    result["same_window_state_refresh"] = True
    menu_command(table, "Pinned.doc", "取消固定")
    wait(lambda: state(table, "Pinned.doc", "本地可用"), "context menu unpin")
    menu_command(table, "Pinned.doc", "固定到本地")
    wait(lambda: state(table, "Pinned.doc", "已固定"), "context menu pin")
    result["actual_host_file_menu_pin_unpin"] = True
    result["right_click_popup_tested"] = False
    result["browse_downloads"] = call("GetOverview")["downloads_started"]
    # Snapshot host actions for menu interaction follow-up.
    q = table.queryTable()
    result["cell_actions"] = []
    for i in range(q.nRows):
        cell = q.getAccessibleAt(i, 0)
        try:
            a = cell.queryAction()
            result["cell_actions"].append(
                {"labels": labels(cell), "actions": [a.getName(j) for j in range(a.nActions)]}
            )
        except NotImplementedError:
            pass
    gui = subprocess.Popen(
        ["/usr/bin/drive-native"], env=dict(env, QT_QPA_PLATFORM="xcb"), stdout=glog, stderr=glog
    )
    time.sleep(3)
    assert gui.poll() is None, "installed frozen GUI exited"
    windows = subprocess.run(
        ["xwininfo", "-root", "-tree"], env=env, capture_output=True, text=True, check=True
    ).stdout
    (out / "windows.txt").write_text(windows)
    assert "Drive Native" in windows
    screen = Gdk.get_default_root_window()
    pixels = Gdk.pixbuf_get_from_window(screen, 0, 0, screen.get_width(), screen.get_height())
    assert pixels.savev(str(out / "installed-desktop.png"), "png", [], [])
    result["installed_frozen_gui_window"] = True
    # Graceful Nemo quit must release its own resources without aborting.
    subprocess.run(["nemo", "--quit"], env=env, capture_output=True, timeout=10, check=True)
    host.wait(timeout=15)
    assert host.returncode == 0, host.returncode
    result["normal_nemo_host_exit"] = True
    assert (
        Path(profile["mount"]) / "Created.doc"
    ).read_bytes() == b"unsent draft retained across package removal and reinstall"
    result["draft_retained"] = True
    result["passed"] = True
except Exception as e:
    result["error"] = str(e)
    result["accessible_snapshot"] = [
        {
            "name": n.name,
            "role": n.getRoleName(),
            "showing": n.getState().contains(pyatspi.STATE_SHOWING),
        }
        for a in applications()
        if a.name == "nemo"
        for n in nodes(a)
    ]
    if "table" in locals():
        result["failure_rows"] = rows(table)
    if daemon and daemon.poll() is None:
        result["failure_overview"] = call("GetOverview")
        result["failure_states"] = call(
            "GetStates", GLib.Variant("(as)", (["A.doc", "Created.doc", "Pinned.doc"],))
        )
    raise
finally:
    if gui and gui.poll() is None:
        gui.terminate()
        gui.wait(timeout=10)
    if host and host.poll() is None:
        subprocess.run(["nemo", "--quit"], env=env, capture_output=True, timeout=10)
        host.wait(timeout=10)
    if daemon and daemon.poll() is None:
        call("StopMount")
        daemon.wait(timeout=15)
        result["fsd_exit"] = daemon.returncode
    for f in (fslog, nlog, glog):
        f.close()
    (out / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2))
    print(json.dumps(result, ensure_ascii=False))
