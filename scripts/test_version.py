"""Regression checks for version synchronization and rejected updates."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('version', Path(__file__).with_name('version.py'))
version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(version)


class VersionTests(unittest.TestCase):
    def setUp(self):
        self.original_root = version.ROOT
        self.temp = tempfile.TemporaryDirectory()
        version.ROOT = Path(self.temp.name)
        for path in ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json',
                     'src-tauri/Cargo.toml', 'src-tauri/Cargo.lock',
                     'test-api/Cargo.toml', 'test-api/Cargo.lock']:
            destination = version.ROOT / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(self.original_root / path, destination)

    def tearDown(self):
        version.ROOT = self.original_root
        self.temp.cleanup()

    def snapshot(self):
        return {str(p.relative_to(version.ROOT)): p.read_bytes()
                for p in version.ROOT.rglob('*') if p.is_file()}

    def test_desktop_bump_preserves_dependencies_and_api(self):
        old_desktop, api = version.check()
        major, minor, patch = version.version_tuple(old_desktop)
        new = f'{major}.{minor}.{patch + 1}'
        before = version.read_toml('src-tauri/Cargo.lock')['package']
        with contextlib.redirect_stdout(io.StringIO()):
            version.bump('desktop', new)
        self.assertEqual(version.check(), (new, api))
        after = version.read_toml('src-tauri/Cargo.lock')['package']
        self.assertEqual([p for p in before if p['name'] != 'mvsep-gui'],
                         [p for p in after if p['name'] != 'mvsep-gui'])

    def test_api_bump_is_independent(self):
        desktop, old_api = version.check()
        major, minor, patch = version.version_tuple(old_api)
        new = f'{major}.{minor}.{patch + 1}'
        before = self.snapshot()
        with contextlib.redirect_stdout(io.StringIO()):
            version.bump('api', new)
        self.assertEqual(version.check(), (desktop, new))
        after = self.snapshot()
        self.assertEqual({p for p in before if before[p] != after[p]},
                         {'test-api/Cargo.toml', 'test-api/Cargo.lock'})

    def test_invalid_or_nonincreasing_versions_do_not_write(self):
        desktop, _ = version.check()
        before = self.snapshot()
        for value in [desktop, '0.0.0', '01.2.3', '1.2', 'v1.2.3', '1.2.3-rc.1']:
            with self.subTest(value=value), self.assertRaises(ValueError):
                version.bump('desktop', value)
            self.assertEqual(self.snapshot(), before)

    def test_mismatch_is_rejected_before_bump(self):
        path = version.ROOT / 'src-tauri/tauri.conf.json'
        data = json.loads(path.read_text())
        data['version'] = '0.0.0'
        path.write_text(json.dumps(data))
        before = self.snapshot()
        with self.assertRaises(ValueError):
            version.bump('desktop', '99.0.0')
        self.assertEqual(self.snapshot(), before)

    def test_release_tag_matches_desktop(self):
        desktop, _ = version.check()
        version.check(f'v{desktop}')
        with self.assertRaises(ValueError):
            version.check('v0.0.0')


if __name__ == '__main__':
    unittest.main()
