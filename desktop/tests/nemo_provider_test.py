"""Run against an already mounted isolated service, using distro Python/GI."""

import importlib.util, json, os, sys, time, resource
from pathlib import Path

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def stage(name):
    print("Nemo test: " + name, file=sys.stderr, flush=True)


import gi

gi.require_version("Nemo", "3.0")
from gi.repository import Gio, GLib

spec = importlib.util.spec_from_file_location("drive_native_nemo", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
context = GLib.MainContext.default()


def wait(predicate, label):
    deadline = time.monotonic() + 75
    while not predicate():
        while context.pending():
            context.iteration(False)
        if time.monotonic() > deadline:
            print(
                json.dumps(
                    {
                        "timeout": label,
                        "overview": overview(),
                        "cached_states": provider.cache,
                        "pending_paths": list(provider.pending),
                        "busy": provider.busy,
                    },
                    ensure_ascii=False,
                ),
                file=sys.stderr,
                flush=True,
            )
            raise AssertionError(label)
        time.sleep(0.02)


probe = Gio.DBusProxy.new_for_bus_sync(
    Gio.BusType.SESSION,
    Gio.DBusProxyFlags.DO_NOT_AUTO_START | Gio.DBusProxyFlags.DO_NOT_LOAD_PROPERTIES,
    None,
    module.NAME,
    module.PATH,
    module.NAME,
    None,
)


def overview():
    return json.loads(
        probe.call_sync("GetOverview", None, Gio.DBusCallFlags.NO_AUTO_START, 5000, None).unpack()[
            0
        ]
    )


before = overview()
assert before["scope_path"].startswith("/DriveNative-Lab/")
provider = module.DriveNativeExtension()
stage("provider created")


class FileInfo:
    def __init__(self, path):
        self.path = path
        self.attributes = {}
        self.emblems = []
        self.invalidations = 0

    def get_uri_scheme(self):
        return "file"

    def get_uri(self):
        return Path(self.path).as_uri()

    def add_string_attribute(self, key, value):
        self.attributes[key] = value

    def add_emblem(self, name):
        self.emblems.append(name)

    def invalidate_extension_info(self):
        self.invalidations += 1
        self.attributes.clear()
        self.emblems.clear()
        provider.update_file_info(self)


folder = FileInfo(str(Path(before["mount"]) / "Pinned"))
file = FileInfo(str(Path(before["mount"]) / "A.doc"))
foreign = FileInfo("/tmp/not-drive-native")
for item in (folder, file, foreign):
    provider.update_file_info(item)
wait(lambda: bool(folder.emblems) and bool(file.emblems), "batched state/emblems")
stage("state and emblems received")
assert overview()["downloads_started"] == before["downloads_started"]
assert not foreign.attributes and not foreign.emblems
# Column ownership is tested in the real Nemo host. Nemo <= 6.6 has an
# upstream finalize bug if an isolated caller immediately drops a column;
# the actual host retains columns in permanent native GLists (upstream
# ff1af2c5d1d9f243c6bb8bb4af6489d354fe75f1). Do not emulate that with a leak.
menus = provider.get_file_items(None, [folder])
assert len(menus) == 1
items = menus[0].props.menu.get_items()
assert len(items) == 4
stage("menus created")
items[0].emit("activate")
wait(lambda: "drive-native-pinned" in folder.emblems, "pin menu and live fixed emblem")
stage("pin complete")
items[1].emit("activate")
wait(lambda: "drive-native-local" in folder.emblems, "unpin menu and local emblem")
stage("unpin complete")
items[2].emit("activate")
wait(lambda: "drive-native-online" in folder.emblems, "release menu and online emblem")
stage("release complete")
before_retry = overview()["downloads_started"]
retry_completed = []
# Observe the actual menu D-Bus completion without requiring icon churn.
for task in provider.tasks.values():
    original_call = task._call
    def observe(method, arguments, callback, original=original_call):
        def done(data, failed):
            if method == "RetryDownloads":
                retry_completed.append(not failed)
            callback(data, failed)
        original(method, arguments, done)
    task._call = observe
items[3].emit("activate")
wait(lambda: bool(retry_completed), "retry menu local response")
assert all(retry_completed)
assert overview()["downloads_started"] == before_retry
assert folder.invalidations >= 3
assert overview()["pending_operations"] == before["pending_operations"]
print(
    json.dumps(
        {
            "passed": True,
            "real_gi_provider_loaded": True,
            "local_batch_zero_downloads": True,
            "native_pin_unpin_release_actions": True,
            "retry_menu_without_failed_jobs_zero_downloads": True,
            "live_emblem_refresh": True,
            "column_host_test_required": True,
            "outside_mount_ignored": True,
            "initial_async_connection_refreshes_visible_files": True,
            "nemo_window_verified": False,
        }
    ),
    flush=True,
)
