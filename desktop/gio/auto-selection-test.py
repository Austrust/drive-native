"""Selection checks in an isolated process; no NAS or installed desktop changes."""

import argparse
import json
import os
import tempfile
from pathlib import Path
from gi.repository import Gio, GObject

parser = argparse.ArgumentParser()
parser.add_argument("--expect", choices=["automatic", "local", "system"], required=True)
args = parser.parse_args()
vfs = Gio.Vfs.get_default()
actual = vfs.__gtype__.name
expected = {"automatic": "ProbeVfs", "local": "GLocalVfs", "system": "GDaemonVfs"}[args.expect]
assert actual == expected, (actual, expected)
assert (os.environ.get("GIO_USE_VFS") == "local") == (args.expect == "local")
with tempfile.TemporaryDirectory(prefix="drive-native-auto-select-") as directory:
    path = Path(directory) / "ordinary.txt"
    os.environ["DRIVE_NATIVE_METADATA_ROOT"] = directory
    path.write_bytes(b"ordinary-content")
    file = Gio.File.new_for_path(str(path))
    assert file.__gtype__.name != "ProbeFile"
    assert file.load_contents(None)[1] == b"ordinary-content"
    file.replace_contents(b"saved-content", None, False, Gio.FileCreateFlags.NONE, None)
    assert path.read_bytes() == b"saved-content"
    assert (
        file.query_info(
            "standard::content-type", Gio.FileQueryInfoFlags.NONE, None
        ).get_content_type()
        == "text/plain"
    )
print(
    json.dumps(
        {
            "passed": True,
            "mode": args.expect,
            "vfs": actual,
            "parent": GObject.type_parent(vfs.__gtype__).name,
            "ordinary_paths_unwrapped": True,
            "ordinary_read_save_type": True,
            "forced_vfs": os.environ.get("GIO_USE_VFS"),
            "nas_contacted": False,
        }
    )
)
