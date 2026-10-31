"""Exercise wrapped enumerator ownership with G_DEBUG=fatal-criticals.

Use a --test-scopes module in an isolated GIO_MODULE_DIR. All data is synthetic.
Adapted from the MIT-licensed d5c548a review snapshot; tests also cover explicit
close, EOF and cancellation rather than only abandoning a partial enumeration.
"""
import gc
import json
import os
from pathlib import Path
import resource
import tempfile

from gi.repository import Gio, GLib


def cancelled(call):
    try:
        call()
    except GLib.Error as error:
        assert error.matches(Gio.io_error_quark(), Gio.IOErrorEnum.CANCELLED), error
    else:
        raise AssertionError("cancelled operation unexpectedly succeeded")


def exercise(folder, mode):
    enumerator = Gio.File.new_for_path(folder).enumerate_children(
        "standard::*", Gio.FileQueryInfoFlags.NONE, None
    )
    assert enumerator.__gtype__.name == "ProbeEnumerator"
    if mode == "partial":
        assert enumerator.next_file(None).get_name() in {"a.txt", "b.txt"}
    elif mode == "explicit-close":
        assert enumerator.close(None)
        assert enumerator.is_closed()
        assert enumerator.close(None)  # Closing twice is safe.
    elif mode == "eof":
        names = []
        while (item := enumerator.next_file(None)) is not None:
            names.append(item.get_name())
        assert sorted(names) == ["a.txt", "b.txt"]
    elif mode in {"cancel-read", "cancel-close"}:
        token = Gio.Cancellable()
        token.cancel()
        if mode == "cancel-read":
            cancelled(lambda: enumerator.next_file(token))
        else:
            # A local backend may finish closing before observing cancellation.
            try:
                assert enumerator.close(token)
            except GLib.Error as error:
                assert error.matches(Gio.io_error_quark(), Gio.IOErrorEnum.CANCELLED)
            # Finalization must be safe whichever close outcome the backend uses.
    elif mode != "unread":
        raise AssertionError(mode)
    # Dropping the final reference must close any still-open underlying object.
    del enumerator
    gc.collect()


resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
assert "fatal-criticals" in os.environ.get("G_DEBUG", "")
modes = ("unread", "partial", "explicit-close", "eof", "cancel-read", "cancel-close")
with tempfile.TemporaryDirectory(prefix="drive-native-enumerator-") as folder:
    os.environ["DRIVE_NATIVE_METADATA_ROOT"] = folder
    for name in ("a.txt", "b.txt"):
        Path(folder, name).write_text("synthetic local fixture")
    # Cancellation initializes GLib's process-wide wakeup descriptor too.
    # Warm every path before measuring sustained descriptor growth.
    for mode in modes:
        exercise(folder, mode)
    before = len(os.listdir("/proc/self/fd"))
    for mode in modes:
        for _ in range(20):
            exercise(folder, mode)
    after = len(os.listdir("/proc/self/fd"))
    assert after == before, (before, after)
print(json.dumps({"passed": True, "cases": len(modes) * 20,
                  "modes": modes, "fd_before": before, "fd_after": after}))
