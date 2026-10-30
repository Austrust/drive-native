"""Exercise public GIO operations through the opt-in module on disposable files."""

import json
import os
import tempfile
from pathlib import Path

import gi

gi.require_version("Gio", "2.0")
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
        result.append((False, TimeoutError("GIO callback did not complete")))
        loop.quit()
        return False

    timer = GLib.timeout_add_seconds(10, timeout)
    start(done)
    loop.run()
    if not isinstance(result[0][1], TimeoutError):
        GLib.source_remove(timer)
    assert len(result) == 1, result
    if not result[0][0]:
        raise result[0][1]
    return result[0][1]


def expect_error(code, call):
    try:
        call()
    except GLib.Error as error:
        assert error.matches(Gio.io_error_quark(), code), error
    else:
        raise AssertionError(f"expected GIO error {code}")


def write(stream, content):
    assert stream.write_all(content, None) == (True, len(content))
    assert stream.close(None)


with tempfile.TemporaryDirectory(prefix="drive-native-gio-write-") as tmp:
    base = Path(tmp)
    root = base / "mount"
    root.mkdir()
    outside = base / "outside"
    outside.mkdir()
    os.environ["DRIVE_NATIVE_METADATA_ROOT"] = str(root)
    folder = Gio.File.new_for_path(str(root))
    assert folder.__gtype__.name == "ProbeFile"
    file = folder.get_child("document.txt")
    write(file.create(Gio.FileCreateFlags.NONE, None), b"v1")
    write(file.append_to(Gio.FileCreateFlags.NONE, None), b"-append")
    assert (root / "document.txt").read_bytes() == b"v1-append"

    # The old complete body remains visible until replacement is closed.
    stream = file.replace(None, False, Gio.FileCreateFlags.NONE, None)
    stream.write_all(b"v2", None)
    assert (root / "document.txt").read_bytes() == b"v1-append"
    stream.close(None)
    assert (root / "document.txt").read_bytes() == b"v2"
    expect_error(
        Gio.IOErrorEnum.WRONG_ETAG,
        lambda: file.replace("deliberately-wrong-etag", False, Gio.FileCreateFlags.NONE, None),
    )
    assert (root / "document.txt").read_bytes() == b"v2"

    rw = file.open_readwrite(None)
    rw.seek(1, GLib.SeekType.SET, None)
    assert rw.get_output_stream().write_all(b"3", None) == (True, 1)
    rw.close(None)
    assert (root / "document.txt").read_bytes() == b"v3"
    renamed = file.set_display_name("renamed.txt", None)
    assert renamed.__gtype__.name == "ProbeFile"
    assert not (root / "document.txt").exists()

    async_call(
        lambda done: renamed.replace_contents_async(
            b"async-v4", None, False, Gio.FileCreateFlags.NONE, None, done, None
        ),
        lambda source, result: source.replace_contents_finish(result),
    )
    assert (root / "renamed.txt").read_bytes() == b"async-v4"

    target = Gio.File.new_for_path(str(outside / "copied.txt"))
    assert target.__gtype__.name != "ProbeFile"
    async_call(
        lambda done: renamed.copy_async(
            target, Gio.FileCopyFlags.NONE, GLib.PRIORITY_DEFAULT, None, None, None, done, None
        ),
        lambda source, result: source.copy_finish(result),
    )
    assert (outside / "copied.txt").read_bytes() == b"async-v4"
    back = folder.get_child("back.txt")
    assert target.copy(back, Gio.FileCopyFlags.NONE, None, None, None)
    assert (root / "back.txt").read_bytes() == b"async-v4"
    moved = Gio.File.new_for_path(str(outside / "moved.txt"))
    assert back.move(moved, Gio.FileCopyFlags.NONE, None, None, None)
    assert not (root / "back.txt").exists()
    assert (outside / "moved.txt").read_bytes() == b"async-v4"

    cancelled = Gio.Cancellable.new()
    cancelled.cancel()
    expect_error(
        Gio.IOErrorEnum.CANCELLED,
        lambda: renamed.replace(None, False, Gio.FileCreateFlags.NONE, cancelled),
    )
    assert (root / "renamed.txt").read_bytes() == b"async-v4"
    cancelled_file = folder.get_child("cancelled.txt")
    expect_error(
        Gio.IOErrorEnum.CANCELLED,
        lambda: async_call(
            lambda done: cancelled_file.create_async(
                Gio.FileCreateFlags.NONE, GLib.PRIORITY_DEFAULT, cancelled, done, None
            ),
            lambda source, result: source.create_finish(result),
        ),
    )
    assert not (root / "cancelled.txt").exists()

    directory = folder.get_child("child")
    assert directory.make_directory(None)
    link = directory.get_child("link")
    assert link.make_symbolic_link("../renamed.txt", None)
    assert (root / "child" / "link").read_bytes() == b"async-v4"
    assert link.delete(None)
    assert directory.delete(None)
    assert renamed.delete(None)
    assert list(root.iterdir()) == []
    print(
        json.dumps(
            {
                "passed": True,
                "scope": "isolated_local_filesystem",
                "create_append_random_write": True,
                "replacement_published_on_close": True,
                "wrong_etag_preserves_body": True,
                "rename_preserves_adapter": True,
                "async_replace_copy": True,
                "copy_in_and_move_out": True,
                "cancelled_replace_and_async_create_preserve_files": True,
                "directory_symlink_delete": True,
                "native_nas_writes_tested": False,
            }
        )
    )
