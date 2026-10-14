#!/usr/bin/env python3
"""Offline installer tests; no builds, signing keys or installed applications."""
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


ROOT = Path(__file__).resolve().parents[1]
package = load('portable_package', ROOT / 'scripts/package-portable.py')
menu = load('linux_menu', ROOT / 'packaging/linux/install-desktop.py')


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.folder = Path(self.temporary.name).resolve()

    def test_native_architecture_rejects_wrong_binary(self):
        binary = self.folder / 'app'
        elf = bytearray(64); elf[:6] = b'\x7fELF\x02\x01'; struct.pack_into('<H', elf, 18, 62)
        binary.write_bytes(elf); package.check_binary(binary, 'linux')
        struct.pack_into('<H', elf, 18, 183); binary.write_bytes(elf)
        with self.assertRaises(package.PackagingError): package.check_binary(binary, 'linux')
        pe = bytearray(70); pe[:2] = b'MZ'; struct.pack_into('<I', pe, 60, 64)
        pe[64:68] = b'PE\0\0'; struct.pack_into('<H', pe, 68, 0x8664)
        binary.write_bytes(pe); package.check_binary(binary, 'windows')
        struct.pack_into('<H', pe, 68, 0xaa64); binary.write_bytes(pe)
        with self.assertRaises(package.PackagingError): package.check_binary(binary, 'windows')

    def test_installer_deletes_only_payload_files(self):
        payload = self.folder / 'payload'; (payload / 'integrations').mkdir(parents=True)
        (payload / 'integrations/example$1.txt').write_text('template')
        (payload / 'app.exe').write_bytes(b'app')
        install, remove = package.nsis_lists(payload)
        self.assertIn('example$$1.txt', install)
        self.assertIn('Delete "$INSTDIR\\app.exe"', remove)
        self.assertNotIn('/r', remove.lower())
        self.assertNotIn('*', remove)
        with self.assertRaises(package.PackagingError): package.nsis_escape('unsafe\nfile')

    @unittest.skipIf(__import__('os').name == 'nt', 'Windows symlink privilege not assumed')
    def test_payload_symlink_rejected(self):
        (self.folder / 'foreign').write_bytes(b'keep')
        (self.folder / 'link').symlink_to('foreign')
        with self.assertRaises(package.PackagingError): package.nsis_lists(self.folder)

    def menu_fixture(self):
        source = self.folder / 'download.AppImage'; source.write_bytes(b'AppImage fixture')
        appdir = self.folder / 'appdir'
        icon = appdir / 'usr/share/icons/hicolor/256x256/apps/io.witzelfitz.beni-buddy.png'
        icon.parent.mkdir(parents=True); icon.write_bytes(b'original icon')
        home = self.folder / 'data'
        return source, appdir, home

    def test_explicit_install_remove_preserves_other_files(self):
        source, appdir, home = self.menu_fixture()
        menu.integrate('--install-desktop', appdir, home, source)
        installed = home / 'beni-buddy/Beni-Buddy.AppImage'
        self.assertEqual(installed.read_bytes(), source.read_bytes())
        foreign = home / 'beni-buddy/my-notes.txt'; foreign.write_text('keep')
        menu.integrate('--remove-desktop', appdir, home, None)
        self.assertFalse(installed.exists()); self.assertEqual(foreign.read_text(), 'keep')

    def test_foreign_menu_and_changed_image_preserved(self):
        source, appdir, home = self.menu_fixture()
        entry = home / 'applications/io.witzelfitz.beni-buddy.desktop'
        entry.parent.mkdir(parents=True); entry.write_text('unmanaged')
        with self.assertRaises(ValueError): menu.integrate('--install-desktop', appdir, home, source)
        self.assertEqual(entry.read_text(), 'unmanaged'); entry.unlink()
        menu.integrate('--install-desktop', appdir, home, source)
        image = home / 'beni-buddy/Beni-Buddy.AppImage'; image.write_bytes(b'changed')
        with self.assertRaises(ValueError): menu.integrate('--remove-desktop', appdir, home, None)
        self.assertEqual(image.read_bytes(), b'changed')

    def test_invalid_action_relative_target_and_absent_source_rejected(self):
        source, appdir, home = self.menu_fixture()
        for action, target, image in [('unknown', home, source), ('--install-desktop', Path('relative'), source),
                                       ('--install-desktop', home, None), ('--remove-desktop', home, None)]:
            with self.assertRaises(ValueError): menu.integrate(action, appdir, target, image)
        self.assertFalse(home.exists())

    @unittest.skipIf(__import__('os').name == 'nt', 'Windows symlink privilege not assumed')
    def test_menu_symlink_target_rejected(self):
        source, appdir, home = self.menu_fixture()
        foreign = self.folder / 'foreign'; foreign.mkdir(); home.symlink_to(foreign)
        with self.assertRaises(ValueError): menu.integrate('--install-desktop', appdir, home, source)
        self.assertEqual(list(foreign.iterdir()), [])

    def test_existing_sidecar_refuses_build(self):
        output = self.folder / 'image.AppImage'; sidecar = Path(str(output)+'.sha256'); sidecar.write_text('keep')
        with patch.object(package.platform, 'system', return_value='Linux'), patch.object(package.platform, 'machine', return_value='x86_64'):
            with self.assertRaises(package.PackagingError): package.build('linux', output)
        self.assertEqual(sidecar.read_text(), 'keep'); self.assertFalse(output.exists())

    def test_tool_tampering_refuses_execution(self):
        tools = self.folder / 'packaging'; tools.mkdir()
        (tools / 'tools.json').write_text('{"nsis":{"version":"test","sha256":"'+'0'*64+'","url":"https://invalid.example"}}')
        cache = self.folder / 'dist/packaging-tools'; cache.mkdir(parents=True)
        (cache / 'nsis-test.zip').write_bytes(b'changed')
        with patch.object(package, 'ROOT', self.folder):
            with self.assertRaises(package.PackagingError): package.tool('nsis')


if __name__ == '__main__': unittest.main()
