"""Explicit operations on a test FUSE mount; invoked by native_gio_roundtrip."""

import argparse
import json
import os
from pathlib import Path

assert not os.environ.get("GIO_USE_VFS"), "automatic selection must not be forced"

from gi.repository import Gio, GLib

parser = argparse.ArgumentParser()
parser.add_argument("mount")
parser.add_argument("action", choices=["read", "save", "rename", "offline", "copy", "delete"])
options = parser.parse_args()
mount = Path(options.mount).resolve(strict=True)
assert mount.is_mount(), "expected a mounted test filesystem"
assert not os.environ.get("DRIVE_NATIVE_METADATA_ROOT"), "registry test must not use test scope"
root = Gio.File.new_for_path(str(mount))
assert root.__gtype__.name == "ProbeFile", "adapter must be active"


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
        result.append((False, TimeoutError("GIO operation timed out")))
        loop.quit()
        return False

    timer = GLib.timeout_add_seconds(45, timeout)
    start(done)
    loop.run()
    if not isinstance(result[0][1], TimeoutError):
        GLib.source_remove(timer)
    assert len(result) == 1
    if not result[0][0]:
        raise result[0][1]
    return result[0][1]


def replace(file, content):
    async_call(
        lambda done: file.replace_contents_async(
            content, None, False, Gio.FileCreateFlags.NONE, None, done, None
        ),
        lambda source, task: source.replace_contents_finish(task),
    )
    assert file.load_contents(None)[1] == content


if options.action == "read":
    assert root.get_child("A.doc").load_contents(None)[1] == b"initial-v1"
elif options.action == "save":
    file = root.get_child("A.doc")
    replace(file, b"online-v2")
    cancelled = Gio.Cancellable.new()
    cancelled.cancel()
    try:
        file.replace(None, False, Gio.FileCreateFlags.NONE, cancelled)
    except GLib.Error as error:
        assert error.matches(Gio.io_error_quark(), Gio.IOErrorEnum.CANCELLED)
    else:
        raise AssertionError("cancelled replace succeeded")
    assert file.load_contents(None)[1] == b"online-v2"
elif options.action == "rename":
    file = root.get_child("A.doc").set_display_name("Renamed.doc", None)
    assert file.__gtype__.name == "ProbeFile"
    assert file.load_contents(None)[1] == b"online-v2"
    assert not (mount / "A.doc").exists()
elif options.action == "offline":
    replace(root.get_child("Renamed.doc"), b"offline-v3")
    folder = root.get_child("Offline")
    assert folder.make_directory(None)
    file = folder.get_child("Created.txt")
    stream = async_call(
        lambda done: file.create_async(
            Gio.FileCreateFlags.NONE, GLib.PRIORITY_DEFAULT, None, done, None
        ),
        lambda source, task: source.create_finish(task),
    )
    assert stream.write_all(b"offline-created", None) == (True, 15)
    assert stream.close(None)
    assert file.load_contents(None)[1] == b"offline-created"
elif options.action == "copy":
    source = root.get_child("Renamed.doc")
    target = root.get_child("Copied.doc")
    async_call(
        lambda done: source.copy_async(
            target,
            Gio.FileCopyFlags.NONE,
            GLib.PRIORITY_DEFAULT,
            None,
            None,
            None,
            done,
            None,
        ),
        lambda source, task: source.copy_finish(task),
    )
    assert target.load_contents(None)[1] == b"offline-v3"
elif options.action == "delete":
    assert root.get_child("Copied.doc").delete(None)
    assert root.get_child("Offline").get_child("Created.txt").delete(None)
    assert root.get_child("Offline").delete(None)
    assert not (mount / "Copied.doc").exists()
    assert not (mount / "Offline").exists()

print(json.dumps({"action": options.action, "passed": True, "actual_gio_adapter": True}))
