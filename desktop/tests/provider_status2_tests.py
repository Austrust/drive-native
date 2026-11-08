"""Pure-function tests for the #31 StatusChanged2 consumer in the provider.

Run with a Python that has PyGObject and the Gio typelib (the GUI venv may not):

    /usr/bin/python3 desktop/tests/provider_status2_tests.py

The full Nemo/Nautilus provider tests need the file manager typelib, which is
not installed in the build container; these tests cover the payload parsing and
tracked-path selection that the extension uses before it schedules GetStates.
"""

import importlib.util
import os
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROVIDER = os.path.join(ROOT, "filemanager", "drive_native_provider.py")


def load_provider():
    spec = importlib.util.spec_from_file_location(
        "drive_native_provider_under_test", PROVIDER)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


try:
    provider = load_provider()
except (ImportError, ValueError):  # pragma: no cover - host typelib dependent
    provider = None


@unittest.skipIf(provider is None, "PyGObject/Gio typelib not available")
class StatusChangesParsingTests(unittest.TestCase):
    def test_valid_payload_is_normalized(self):
        changes, truncated = provider.parse_status_changes(
            '{"schema":1,"changes":[{"path":"/mnt/d/Lab/../Lab/a.txt",'
            '"directory":false,"kind":"status"},{"path":"/mnt/d/Lab/",'
            '"directory":true,"kind":"listing"}],"truncated":true}')
        self.assertEqual([
            {"path": "/mnt/d/Lab/a.txt", "directory": False, "kind": "status"},
            {"path": "/mnt/d/Lab", "directory": True, "kind": "listing"},
        ], changes)
        self.assertTrue(truncated)

    def test_empty_change_set_is_valid(self):
        self.assertEqual(([], False), provider.parse_status_changes(
            '{"schema":1,"changes":[]}'))

    def test_missing_change_list_is_valid(self):
        self.assertEqual(([], False), provider.parse_status_changes(
            '{"schema":1}'))

    def test_invalid_payloads_fall_back_to_full_refresh(self):
        for payload in (
                "",
                None,
                "not json",
                "[]",
                '{"schema":2,"changes":[]}',
                '{"schema":1,"changes":"nope"}',
                '{"schema":1,"changes":[{"path":"relative","kind":"status"}]}',
                '{"schema":1,"changes":[{"path":"/mnt/d/Lab","kind":"moved"}]}',
                '{"schema":1,"changes":[{"path":"/mnt/d/Lab"}]}',
                '{"schema":1,"changes":[42]}',
        ):
            self.assertEqual((None, False),
                             provider.parse_status_changes(payload), payload)


@unittest.skipIf(provider is None, "PyGObject/Gio typelib not available")
class AffectedTrackedTests(unittest.TestCase):
    def test_status_change_only_marks_the_exact_path(self):
        tracked = {"/mnt/d/Lab/a.txt", "/mnt/d/Lab/b.txt", "/mnt/d/Lab/sub/c.txt"}
        changes = [{"path": "/mnt/d/Lab/a.txt", "directory": False,
                    "kind": "status"}]
        self.assertEqual({"/mnt/d/Lab/a.txt"},
                         provider.affected_tracked(changes, tracked))

    def test_listing_change_marks_descendants_but_not_siblings(self):
        tracked = {"/mnt/d/Lab/a.txt", "/mnt/d/Lab/sub/c.txt",
                   "/mnt/d/Other/d.txt"}
        changes = [{"path": "/mnt/d/Lab", "directory": True, "kind": "listing"}]
        self.assertEqual({"/mnt/d/Lab/a.txt", "/mnt/d/Lab/sub/c.txt"},
                         provider.affected_tracked(changes, tracked))

    def test_subtree_change_marks_nested_tracked_paths(self):
        tracked = {"/mnt/d/Lab/sub/c.txt", "/mnt/d/Labc/x.txt"}
        changes = [{"path": "/mnt/d/Lab", "directory": True, "kind": "subtree"}]
        self.assertEqual({"/mnt/d/Lab/sub/c.txt"},
                         provider.affected_tracked(changes, tracked))

    def test_child_change_does_not_refresh_its_parent(self):
        tracked = {"/mnt/d/Lab"}
        changes = [{"path": "/mnt/d/Lab/a.txt", "directory": False,
                    "kind": "status"}]
        self.assertEqual(set(), provider.affected_tracked(changes, tracked))

    def test_root_change_marks_every_tracked_path(self):
        tracked = {"/mnt/d/a", "/mnt/d/b"}
        changes = [{"path": "/", "directory": True, "kind": "subtree"}]
        self.assertEqual(tracked, provider.affected_tracked(changes, tracked))


if __name__ == "__main__":
    unittest.main()
