"""Preserve real GVfs metadata and trash while wrapping only a test root."""

import json
import os
import tempfile
import sys
import time
import uuid
from pathlib import Path

from gi.repository import Gio, GLib, GObject

with tempfile.TemporaryDirectory(prefix="drive-native-gvfs-contract-") as tmp:
    base = Path(tmp)
    root = base / "mount"
    root.mkdir()
    os.environ["DRIVE_NATIVE_METADATA_ROOT"] = str(root)
    vfs = Gio.Vfs.get_default()
    fallback = "--expect-system-fallback" in sys.argv
    ordinary = "--ordinary-only" in sys.argv
    if ordinary:
        assert not os.environ.get("GIO_USE_VFS")
    assert vfs.__gtype__.name == ("GDaemonVfs" if fallback else "ProbeVfs"), vfs.__gtype__.name
    if not fallback:
        assert GObject.type_parent(vfs.__gtype__).name == "GDaemonVfs"
    assert Gio.File.new_for_uri("trash:///").__gtype__.name == "GDaemonFile"
    # Object construction only: no network request to this reserved domain.
    assert Gio.File.new_for_uri("smb://example.invalid/share").__gtype__.name == "GDaemonFile"
    for where in (base, root):
        path = where / "metadata.txt"
        path.write_bytes(b"metadata-body")
        file = Gio.File.new_for_path(str(path))
        assert (file.__gtype__.name == "ProbeFile") == (
            where == root and not fallback and not ordinary
        )
        assert file.set_attribute_string(
            "metadata::drive-native-test", "retained", Gio.FileQueryInfoFlags.NONE, None
        )
        info = file.query_info("metadata::drive-native-test", Gio.FileQueryInfoFlags.NONE, None)
        assert info.get_attribute_string("metadata::drive-native-test") == "retained"
        renamed = file.set_display_name("renamed.txt", None)
        info = renamed.query_info("metadata::drive-native-test", Gio.FileQueryInfoFlags.NONE, None)
        assert info.get_attribute_string("metadata::drive-native-test") == "retained"
        assert renamed.delete(None)
    trash = Gio.File.new_for_uri("trash:///")
    trash_source = root / f"trash-contract-{uuid.uuid4()}.txt"
    trash_source.write_bytes(b"trash-body")
    assert Gio.File.new_for_path(str(trash_source)).trash(None)
    assert not trash_source.exists()
    deadline = time.monotonic() + 5
    owned = []
    while not owned and time.monotonic() < deadline:
        entries = trash.enumerate_children(
            "standard::name,trash::orig-path", Gio.FileQueryInfoFlags.NONE, None
        )
        while True:
            info = entries.next_file(None)
            if info is None:
                break
            if info.get_attribute_byte_string("trash::orig-path") == str(trash_source):
                owned.append(entries.get_child(info))
        entries.close(None)
        if not owned:
            time.sleep(0.05)
    assert len(owned) == 1, "new trash entry absent"
    assert owned[0].load_contents(None)[1] == b"trash-body"
    assert owned[0].delete(None)  # Only the exact entry created above.
    print(
        json.dumps(
            {
                "passed": True,
                "vfs_parent": GObject.type_parent(vfs.__gtype__).name,
                "outside_file_unwrapped": True,
                "inside_file_wrapped": not fallback and not ordinary,
                "automatic_selection": ordinary and not fallback,
                "module_cache_missing_fallback": fallback,
                "metadata_set_query_and_rename_inside_outside": True,
                "actual_trash_roundtrip": True,
                "remote_uri_dispatch_preserved": True,
                "remote_network_transfer_tested": False,
            }
        )
    )
