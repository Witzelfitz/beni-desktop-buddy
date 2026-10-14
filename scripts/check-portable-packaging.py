#!/usr/bin/env python3
"""Offline installer tests; no builds, signing keys or installed applications."""
import importlib.util
import io
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


ROOT = Path(__file__).resolve().parents[1]
package = load('portable_package', ROOT / 'scripts/package-portable.py')
menu = load('linux_menu', ROOT / 'packaging/linux/install-desktop.py')
installer = load('portable_installer', ROOT / 'scripts/check-portable-installer.py')


class StartupDiagnosticsTests(unittest.TestCase):
    def test_failure_identifies_backend_and_exit_without_echoing_output(self):
        process = Mock(); process.poll.return_value = 1
        log = io.BytesIO(b'protocol-output\n' * 2000 + b'Error: Glutin: EGL Display failed. secret-token user-text\n')
        with self.assertRaises(installer.package.PackagingError) as failure:
            installer.wait_window(process, lambda: False, label='Linux/Wayland', log=log)
        message = str(failure.exception)
        self.assertIn('Linux/Wayland', message); self.assertIn('Prozess-Exitcode 1', message)
        self.assertIn('Glutin', message); self.assertIn('EGL', message)
        self.assertNotIn('secret-token', message); self.assertNotIn('user-text', message)

    def test_panic_location_excludes_paths_and_raw_message(self):
        process = Mock(); process.poll.return_value = 101
        log = io.BytesIO(b"thread 'main' panicked at /private/user/path/app.rs:10:7:\nsecret-message\n")
        message = installer.startup_summary(process, log, 'Linux/Wayland')
        self.assertIn('Panikstelle app.rs:10:7', message)
        self.assertNotIn('/private', message); self.assertNotIn('secret-message', message)

    def test_live_window_success_does_not_read_logs_and_timeout_stays_a_failure(self):
        process = Mock(); process.poll.return_value = None
        log = Mock()
        installer.wait_window(process, lambda: True, label='Linux/X11', log=log)
        log.read.assert_not_called()
        with patch.object(installer.time, 'monotonic', side_effect=[0, 26]):
            with self.assertRaises(installer.package.PackagingError) as failure:
                installer.wait_window(process, lambda: False, label='Linux/X11')
        self.assertIn('Prozess läuft', str(failure.exception))


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

    def test_output_copies_are_exclusive_and_rollback_incomplete_packages(self):
        output = self.folder/'image.AppImage'
        def installer(image, directory): image.write_bytes(b'installer fixture')
        with patch.object(package.platform,'system',return_value='Linux'), patch.object(package.platform,'machine',return_value='x86_64'), \
                patch.object(package,'check_versions',return_value=('0.1.0',1)), patch.object(package,'linux_installer',side_effect=installer):
            with patch.object(package.shutil,'copyfileobj',side_effect=OSError('fixture failed copy')):
                with self.assertRaises(OSError): package.build('linux',output)
            self.assertFalse(output.exists()); self.assertEqual(list(self.folder.iterdir()),[])
            package.build('linux',output)
            self.assertEqual(output.read_bytes(),b'installer fixture')
            with self.assertRaises(package.PackagingError): package.build('linux',output)

    def test_tool_tampering_refuses_execution(self):
        tools = self.folder / 'packaging'; tools.mkdir()
        (tools / 'tools.json').write_text('{"nsis":{"version":"test","sha256":"'+'0'*64+'","url":"https://invalid.example"}}')
        cache = self.folder / 'dist/packaging-tools'; cache.mkdir(parents=True)
        (cache / 'nsis-test.zip').write_bytes(b'changed')
        with patch.object(package, 'ROOT', self.folder):
            with self.assertRaises(package.PackagingError): package.tool('nsis')

    @unittest.skipIf(__import__('os').name == 'nt', '/usr-merge is a Linux filesystem convention')
    def test_library_ownership_uses_usr_merge_and_refuses_unowned_files(self):
        missing = subprocess.CompletedProcess([],1,b'',b'no matching path')
        found = subprocess.CompletedProcess([],0,b'libwayland-egl1:amd64: /usr/lib/x86_64-linux-gnu/libwayland-egl.so.1\n',b'')
        with patch.object(package.subprocess,'run',side_effect=[missing,found]) as query:
            self.assertEqual(package.library_owner(Path('/lib/x86_64-linux-gnu/libwayland-egl.so.1')),'libwayland-egl1')
            self.assertIn('/usr/lib/',query.call_args.args[0][-1])
        with patch.object(package.subprocess,'run',return_value=missing):
            with self.assertRaises(package.PackagingError): package.library_owner(Path('/lib/x86_64-linux-gnu/foreign.so'))


if __name__ == '__main__': unittest.main()
