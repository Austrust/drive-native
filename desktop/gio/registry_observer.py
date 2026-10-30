"""Persistent GIO observer used by the explicit kernel registry test."""

import json, os, sys, time
from pathlib import Path

assert not os.environ.get("GIO_USE_VFS"), "automatic selection must not be forced"

from gi.repository import Gio

assert not os.environ.get("DRIVE_NATIVE_METADATA_ROOT")
first, second = map(Path, sys.argv[1:3])
old = Gio.File.new_for_path(str(first / "opaque"))
assert old.__gtype__.name == "ProbeFile"

for line in sys.stdin:
    command = line.strip()
    if command == "quit":
        break
    if command == "scope-environment":
        ordinary = first.parent / "ordinary-not-a-mount"
        os.environ["DRIVE_NATIVE_METADATA_ROOT"] = str(first.parent)
        result = {
            "ordinary_wrapped": Gio.File.new_for_path(str(ordinary)).__gtype__.name == "ProbeFile"
        }
        del os.environ["DRIVE_NATIVE_METADATA_ROOT"]
    elif command == "benchmark":
        started = time.monotonic()
        for _ in range(1000):
            file = Gio.File.new_for_path(str(second / "opaque"))
            assert file.__gtype__.name == "ProbeFile"
            file.query_filesystem_info("filesystem::use-preview", None)
        elapsed = round((time.monotonic() - started) * 1000)
        started = time.monotonic()
        for _ in range(1000):
            Gio.Vfs.get_local().get_file_for_path(str(second / "opaque")).query_filesystem_info(
                "filesystem::use-preview", None
            )
        result = {
            "queries": 1000,
            "elapsed_ms": elapsed,
            "underlying_local_query_ms": round((time.monotonic() - started) * 1000),
        }
    else:
        assert command == "inspect"
        a = Gio.File.new_for_path(str(first / "opaque"))
        b = Gio.File.new_for_path(str(second / "opaque"))
        info = old.query_filesystem_info("filesystem::use-preview", None)
        result = {
            "first": a.__gtype__.name == "ProbeFile",
            "second": b.__gtype__.name == "ProbeFile",
            "old_preview_disabled": info.get_attribute_uint32("filesystem::use-preview") == 2,
        }
    print(json.dumps(result), flush=True)
