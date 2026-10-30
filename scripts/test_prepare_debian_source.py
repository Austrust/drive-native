"""Regression checks for complete, isolated Debian build input collection."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "prepare_source", Path(__file__).with_name("prepare-debian-source.py")
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class PrepareSourceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / "project"
        self.source.mkdir()
        for name in ("crates", "desktop", "scripts", "debian", "evidence"):
            (self.source / name).mkdir()
        for name in ("Cargo.toml", "Cargo.lock", "LICENSE", "README.md", "DELIVERY_GATES.md"):
            (self.source / name).write_text("fixture")

    def test_discovers_nested_compile_fixture_and_hashes_it(self):
        (self.source / "crates/lib.rs").write_text(
            'const X: &str = include_str!("../evidence/new-sample.json");'
        )
        (self.source / "evidence/new-sample.json").write_text("{}")
        target = self.root / "build"
        module.prepare(self.source, target)
        manifest = json.loads((target / "build-source-manifest.json").read_text())
        self.assertEqual(manifest["compile_time_fixtures"], ["evidence/new-sample.json"])
        self.assertEqual((target / "evidence/new-sample.json").read_text(), "{}")
        self.assertTrue(
            any(
                f["path"] == "evidence/new-sample.json" and len(f["sha256"]) == 64
                for f in manifest["files"]
            )
        )

    def test_rejects_destination_resolving_inside_source(self):
        alias = self.root / "alias"
        alias.symlink_to(self.source, target_is_directory=True)
        with self.assertRaises(ValueError):
            module.prepare(self.source, alias / "build")
        self.assertFalse((self.source / "build").exists())

    def test_missing_fixture_fails_before_destination_is_created(self):
        (self.source / "crates/lib.rs").write_text(
            'const X: &str = include_str!("../evidence/missing.json");'
        )
        target = self.root / "build"
        with self.assertRaises(FileNotFoundError):
            module.prepare(self.source, target)
        self.assertFalse(target.exists())

    def test_keeps_vendored_dependency_inputs(self):
        vendor = self.source / "vendor" / "fuser"
        vendor.mkdir(parents=True)
        (vendor / "Cargo.toml").write_text("locked vendored dependency")
        target = self.root / "build"
        module.prepare(self.source, target)
        self.assertEqual(
            (target / "vendor/fuser/Cargo.toml").read_text(), "locked vendored dependency"
        )

    def test_existing_target_is_not_overwritten(self):
        target = self.root / "build"
        target.mkdir()
        (target / "keep").write_text("preserve")
        with self.assertRaises(ValueError):
            module.prepare(self.source, target)
        self.assertEqual((target / "keep").read_text(), "preserve")


if __name__ == "__main__":
    unittest.main()
