import argparse, json, tempfile
from pathlib import Path
from gi.repository import Gio, GLib


def async_call(start, finish):
    loop = GLib.MainLoop()
    result = []

    def done(source, task, *_):
        try:
            result.append((True, finish(source, task)))
        except Exception as error:
            result.append((False, error))
        loop.quit()

    def timeout():
        result.append((False, TimeoutError("backend callback timed out")))
        loop.quit()
        return False

    timer = GLib.timeout_add_seconds(15, timeout)
    start(done)
    loop.run()
    if not isinstance(result[0][1], TimeoutError):
        GLib.source_remove(timer)
    if not result[0][0]:
        raise result[0][1]
    return result[0][1]


parser = argparse.ArgumentParser()
parser.add_argument("--report", type=Path)
options = parser.parse_args()
if options.report:
    options.report.write_text(json.dumps({"passed": False, "stage": "starting"}) + "\n")

with tempfile.TemporaryDirectory(prefix="drive-native-gvfs-backend-") as tmp:
    path = Path(tmp) / "body.txt"
    path.write_bytes(b"backend-v1")
    remote = Gio.File.new_for_uri("localtest://" + str(path))
    assert remote.__gtype__.name == "GDaemonFile"
    assert Gio.Vfs.get_default().__gtype__.name == "ProbeVfs"
    async_call(
        lambda cb: remote.mount_enclosing_volume(Gio.MountMountFlags.NONE, None, None, cb, None),
        lambda s, r: s.mount_enclosing_volume_finish(r),
    )
    mount = remote.find_enclosing_mount(None)
    try:
        assert remote.load_contents(None)[1] == b"backend-v1"
        remote.replace_contents(b"backend-v2", None, False, Gio.FileCreateFlags.NONE, None)
        assert path.read_bytes() == b"backend-v2"
        assert (
            remote.query_info("standard::size", Gio.FileQueryInfoFlags.NONE, None).get_size() == 10
        )
    finally:
        async_call(
            lambda cb: mount.unmount_with_operation(
                Gio.MountUnmountFlags.NONE, None, None, cb, None
            ),
            lambda s, r: s.unmount_with_operation_finish(r),
        )

report = {
    "passed": True,
    "actual_gdaemon_backend_io": True,
    "backend": "localtest",
    "unmount_completed": True,
    "network_transport_tested": False,
}
if options.report:
    options.report.write_text(json.dumps(report) + "\n")
print(json.dumps(report))
