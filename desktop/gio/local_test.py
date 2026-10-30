import gi, os, json

gi.require_version("Gio", "2.0")
from gi.repository import Gio, GLib

root = os.environ["DRIVE_NATIVE_METADATA_ROOT"]
f = Gio.File.new_for_path(root + "/A.doc")
assert f.get_path() == root + "/A.doc"
assert f.get_uri().startswith("file:///") and f.is_native()
info = f.query_info("*", Gio.FileQueryInfoFlags.NONE, None)
assert info.get_size() == 19 and info.get_content_type()
parent = Gio.File.new_for_path(root)
e = parent.enumerate_children("standard::*", Gio.FileQueryInfoFlags.NONE, None)
names = []
while (n := e.next_file(None)) is not None:
    names.append(n.get_name())
e.close(None)
assert names == ["A.doc"]
assert parent.get_child("A.doc").equal(f)
loop = GLib.MainLoop()
errors = []


def done(file, result):
    try:
        n = file.query_info_finish(result)
        assert n.get_size() == 19
    except Exception as error:
        errors.append(str(error))
    loop.quit()


f.query_info_async("standard::*", Gio.FileQueryInfoFlags.NONE, 0, None, done)
loop.run()
assert not errors, errors
assert (
    f.query_filesystem_info("filesystem::use-preview", None).get_attribute_uint32(
        "filesystem::use-preview"
    )
    == 2
)
print(
    json.dumps(
        {
            "passed": True,
            "type": f.__gtype__.name,
            "metadata_only": True,
            "async_metadata": True,
            "uri_and_path_preserved": True,
        }
    )
)
