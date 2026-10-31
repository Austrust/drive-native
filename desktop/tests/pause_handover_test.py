import json
import os
import tempfile
import unittest
import uuid
from pathlib import Path
from unittest.mock import patch
from drive_native_gui.pause_handover import preserve_pause


class PauseIntentTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.profile = {"task_id": str(uuid.uuid4())}
        self.state = Path(self.temp.name) / "drive-native" / self.profile["task_id"]
        self.state.mkdir(parents=True, mode=0o700)
        self.env = patch.dict(os.environ, XDG_DATA_HOME=self.temp.name)
        self.env.start()

    def tearDown(self):
        self.env.stop()
        self.temp.cleanup()

    def test_publish_is_private_and_task_bound(self):
        preserve_pause(self.profile)
        path = self.state / "startup-pause-v1.json"
        self.assertEqual(json.loads(path.read_text()), {"schema": 1, **self.profile})
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(list(self.state.iterdir()), [path])

    def test_overwide_directory_and_owner_mismatch_rejected(self):
        self.state.chmod(0o755)
        with self.assertRaises(ValueError):
            preserve_pause(self.profile)
        self.state.chmod(0o700)
        with patch("drive_native_gui.pause_handover.os.getuid", return_value=os.getuid()+1):
            with self.assertRaises(ValueError):
                preserve_pause(self.profile)
        self.assertEqual(list(self.state.iterdir()), [])

    def test_directory_symlink_rejected(self):
        other = Path(self.temp.name) / "other"
        self.state.rename(other)
        self.state.symlink_to(other, target_is_directory=True)
        with self.assertRaises(OSError):
            preserve_pause(self.profile)

    def test_fsync_failure_is_not_acknowledged(self):
        actual = os.fsync
        calls = []
        def sync(fd):
            calls.append(fd)
            if len(calls) == 2:
                raise OSError("synthetic directory sync failure")
            actual(fd)
        with patch("drive_native_gui.pause_handover.os.fsync", side_effect=sync):
            with self.assertRaises(OSError):
                preserve_pause(self.profile)
        self.assertEqual(len(calls), 2)

    def test_existing_unrelated_intent_is_not_overwritten(self):
        path = self.state / "startup-pause-v1.json"
        original = json.dumps({"schema": 1, "task_id": str(uuid.uuid4())})
        path.write_text(original)
        path.chmod(0o600)
        with self.assertRaises(ValueError):
            preserve_pause(self.profile)
        self.assertEqual(path.read_text(), original)


if __name__ == "__main__":
    unittest.main()
