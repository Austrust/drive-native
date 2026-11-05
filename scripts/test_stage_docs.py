"""Local development packages must not invent published source URLs."""
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('stage_docs', Path(__file__).with_name('stage-docs.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class DocumentationTests(unittest.TestCase):
    def fixture(self, root):
        names = ['README.md', 'DELIVERY_GATES.md', 'docs/P0.md', 'docs/DURABILITY.md',
                 'docs/REFERENCES.md', 'docs/PERFORMANCE.md', 'docs/RELEASE_NOTES.md',
                 'docs/UBUNTU_INTEGRATION.md', 'docs/FIX_7_REPORT.md', 'docs/FIX_5_2_REPORT.md',
                 'docs/CONTRACTS.md', 'docs/UPGRADE_RECOVERY.md', 'docs/FILE_TIMESTAMPS.md',
                 'evidence/case.json', 'evidence/image.png']
        for name in names:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('fixture\n', encoding='utf-8')
        (root / 'README.md').write_text('[ACL](docs/FIX_7_REPORT.md)\n[Raw](evidence/case.json)\n![Plot](evidence/image.png)\n[Web](https://example.invalid/reference)\n', encoding='utf-8')
        (root / 'docs/FIX_7_REPORT.md').write_text('[Raw](../evidence/case.json#sample)\n', encoding='utf-8')

    def test_offline_bundle_has_local_source_references_and_readable_images(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'source'; root.mkdir()
            destination = Path(temporary) / 'staged'
            self.fixture(root)
            with patch.dict(os.environ, {'DRIVE_NATIVE_SOURCE_COMMIT': 'a' * 40, 'DRIVE_NATIVE_OFFLINE_DOCS': '1'}), patch.object(module.subprocess, 'check_output', return_value='test-version\n'):
                module.stage(root, destination)
            readme = (destination / 'README.md').read_text(encoding='utf-8')
            acl = (destination / 'docs/FIX_7_REPORT.md').read_text(encoding='utf-8')
            references = (destination / 'SOURCE_REFERENCE.md').read_text(encoding='utf-8')
            self.assertIn('[ACL](docs/FIX_7_REPORT.md)', readme)
            self.assertIn('[Raw](SOURCE_REFERENCE.md#source-', readme)
            self.assertIn('[Raw](../SOURCE_REFERENCE.md#source-', acl)
            self.assertIn('[源码图片：Plot](SOURCE_REFERENCE.md#source-', readme)
            self.assertNotIn('github.com', readme + acl)
            self.assertIn('[Web](https://example.invalid/reference)', readme)
            self.assertIn('evidence/case.json#sample', references)
            self.assertIn('drive-native-source.tar.gz', references)
            self.assertEqual((destination / 'SOURCE_COMMIT').read_text().strip(), 'a' * 40)

    def test_published_mode_preserves_exact_commit_source_links(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'source'; root.mkdir()
            destination = Path(temporary) / 'staged'
            self.fixture(root)
            with patch.dict(os.environ, {'DRIVE_NATIVE_SOURCE_COMMIT': 'b' * 40, 'DRIVE_NATIVE_OFFLINE_DOCS': '0'}), patch.object(module.subprocess, 'check_output', return_value='test-version\n'):
                module.stage(root, destination)
            self.assertIn('https://github.com/Austrust/drive-native/blob/' + 'b' * 40 + '/evidence/case.json',
                          (destination / 'README.md').read_text(encoding='utf-8'))
            self.assertFalse((destination / 'SOURCE_REFERENCE.md').exists())

if __name__ == '__main__':
    unittest.main()
