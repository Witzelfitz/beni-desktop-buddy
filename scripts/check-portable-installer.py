#!/usr/bin/env python3
"""Install, launch and remove owned packages only on fresh GitHub Actions runners."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile
import time
import urllib.request

spec = importlib.util.spec_from_file_location('portable', Path(__file__).with_name('package-portable.py'))
package = importlib.util.module_from_spec(spec); spec.loader.exec_module(package)


def stop(process):
    if process.poll() is None:
        process.terminate()
        try: process.wait(timeout=5)
        except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=5)


def startup_summary(process, log, label):
    """Only fixed categories and numeric exits; never echo application output."""
    status = process.poll()
    details = ['Prozess läuft' if status is None else f'Prozess-Exitcode {status}']
    if log is not None:
        log.seek(0)
        output = log.read(16 * 1024).decode(errors='replace').lower()
        for marker, category in (
            ('panicked at', 'Rust-Panik'), ('error while loading shared libraries', 'Fehlende Laufzeitbibliothek'),
            ('glibc_', 'GLIBC-Konflikt'), ('winit', 'Winit'), ('glutin', 'Glutin'),
            ('wayland', 'Wayland'), ('x11', 'X11'), ('opengl', 'OpenGL'), ('egl', 'EGL'),
            ('dbus', 'D-Bus'), ('d-bus', 'D-Bus'), ('display', 'Display'),
        ):
            if marker in output and category not in details: details.append(category)
    return f'{label}: ' + '; '.join(details) + '.'


def wait_window(process, check, *, label='Fensterstart', log=None):
    deadline = time.monotonic()+25
    while time.monotonic() < deadline and process.poll() is None:
        if check(): return
        time.sleep(0.5)
    raise package.PackagingError('Die installierte App hat kein eigenes sichtbares Fenster geöffnet. ' + startup_summary(process, log, label))


def windows(image, folder):
    installed = folder / 'installed Beni'
    # NSIS /D must be the final, unquoted argument, including paths with spaces.
    install_command = f'"{image}" /S /D={installed}'
    package.run(install_command, 'Windows per Benutzer installieren')
    executable = installed / 'beni-desktop-buddy.exe'
    package.check_binary(executable, 'windows')
    package.run([str(installed / 'beni-buddy-cli.exe'), 'validate-assets'], 'Installierte Windows-CLI prüfen')
    # Fresh hosted runners lack a hardware OpenGL driver. This test-only DLL is
    # never included in the installer and never touches Windows system folders.
    url = 'https://github.com/mmozeiko/build-mesa/releases/download/26.2.4/mesa-llvmpipe-x64-26.2.4.7z'
    with urllib.request.urlopen(url,timeout=60) as response: driver = response.read(32*1024*1024+1)
    if hashlib.sha256(driver).hexdigest() != '6eff97dc5b33c8017a67eb617a1b010341b1569863caa8638b508983645daeec':
        raise package.PackagingError('Test-Grafiktreiber hat eine falsche Prüfsumme.')
    archive = folder/'mesa.7z'; archive.write_bytes(driver)
    graphics = folder/'mesa'
    package.run(['7z','x','-y',f'-o{graphics}',str(archive)],'Test-Grafiktreiber extrahieren')
    drivers = list(graphics.rglob('opengl32.dll'))
    if len(drivers)!=1: raise package.PackagingError('Test-Grafiktreiber hat einen unerwarteten Inhalt.')
    package.check_binary(drivers[0],'windows'); shutil.copy2(drivers[0],installed/'opengl32.dll')
    log = tempfile.TemporaryFile()
    process = subprocess.Popen([str(executable)], env=dict(os.environ,GALLIUM_DRIVER='llvmpipe'),stdout=log,stderr=log)
    inspector = package.ROOT / 'packaging/windows/window-smoke.ps1'
    try:
        wait_window(process, lambda: subprocess.run(['pwsh','-NoProfile','-File',str(inspector),'-ProcessId',str(process.pid)],
                                                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=15).returncode == 0)
        before = package.digest(executable)
        for command in (install_command, f'"{installed / "Uninstall.exe"}" /S _?={installed}'):
            result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=45)
            if result.returncode != 2 or package.digest(executable) != before or process.poll() is not None:
                raise package.PackagingError('Laufende Windows-App wird nicht sicher geschützt.')
    except package.PackagingError:
        log.seek(0); print(log.read(4000).decode(errors='replace'))
        raise
    finally: stop(process); log.close()
    foreign = installed / 'my-notes.txt'; foreign.write_text('keep')
    package.run(f'"{installed / "Uninstall.exe"}" /S _?={installed}', 'Windows deinstallieren')
    if executable.exists() or foreign.read_text() != 'keep':
        raise package.PackagingError('Windows-Deinstallation entfernt eigene Dateien nicht korrekt.')
    import winreg
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r'Software\Microsoft\Windows\CurrentVersion\Uninstall\io.witzelfitz.beni-buddy'):
            raise package.PackagingError('Deinstallations-Eintrag bleibt unerwartet erhalten.')
    except FileNotFoundError: pass
    return {'window_visible':True, 'running_app_protected':True, 'installed_then_removed':True,'graphics_backend':'ci_mesa_llvmpipe_26.2.4'}


def linux(image, folder):
    env = dict(os.environ, APPIMAGE_EXTRACT_AND_RUN='1', LIBGL_ALWAYS_SOFTWARE='1', XDG_DATA_HOME=str(folder/'data'), XDG_CONFIG_HOME=str(folder/'config'))
    package.run([str(image),'--cli','validate-assets'], 'AppImage-CLI ohne FUSE prüfen', env=env)
    package.run([str(image),'--appimage-extract'], 'Eigenes AppImage extrahieren', cwd=folder)
    appdir = folder/'squashfs-root'
    with tempfile.TemporaryFile() as log:
        process = subprocess.Popen([str(appdir/'AppRun')], env=dict(env, WINIT_UNIX_BACKEND='x11'), stdout=log, stderr=log)
        try:
            wait_window(process, lambda: subprocess.run(['xdotool','search','--onlyvisible','--pid',str(process.pid)],
                                                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5).returncode == 0,
                        label='Linux/X11', log=log)
        finally: stop(process)
    runtime = folder/'wayland-runtime'; runtime.mkdir(mode=0o700)
    wayland_env = dict(env, XDG_RUNTIME_DIR=str(runtime), WAYLAND_DISPLAY='beni-test', WINIT_UNIX_BACKEND='wayland', WAYLAND_DEBUG='1')
    wayland_env.pop('DISPLAY', None)
    with tempfile.TemporaryFile() as weston_log, tempfile.TemporaryFile() as client_log:
        compositor = subprocess.Popen(['weston','--backend=headless-backend.so','--use-pixman','--no-config','--socket=beni-test','--idle-time=0'],
                                     env=wayland_env, stdout=weston_log, stderr=weston_log)
        try:
            deadline = time.monotonic()+15
            while not (runtime/'beni-test').exists() and compositor.poll() is None and time.monotonic() < deadline: time.sleep(0.2)
            if not (runtime/'beni-test').exists(): raise package.PackagingError('Wayland-Testcompositor startet nicht.')
            process = subprocess.Popen([str(appdir/'AppRun')], env=wayland_env, stdout=client_log, stderr=client_log)
            def configured():
                client_log.seek(0); protocol = client_log.read().decode(errors='replace')
                return 'xdg_toplevel' in protocol and 'set_title(' in protocol and 'ack_configure(' in protocol and '.commit(' in protocol
            try: wait_window(process, configured, label='Linux/Wayland', log=client_log)
            finally: stop(process)
        finally: stop(compositor)
    package.run([str(image),'--install-desktop'], 'Linux-Menü explizit installieren', env=env)
    installed = folder/'data/beni-buddy/Beni-Buddy.AppImage'
    if package.digest(installed) != package.digest(image): raise package.PackagingError('Installierte AppImage-Bytes stimmen nicht überein.')
    package.run([str(installed),'--cli','validate-assets'], 'Installiertes AppImage prüfen', env=env)
    foreign = installed.parent/'my-notes.txt'; foreign.write_text('keep')
    package.run([str(installed),'--remove-desktop'], 'Linux-Menü entfernen', env=env)
    if installed.exists() or foreign.read_text() != 'keep': raise package.PackagingError('Linux-Deinstallation beschädigt fremde Dateien.')
    return {'window_visible':True, 'x11_window':True, 'wayland_surface':True, 'installed_then_removed':True}


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('image',type=Path); parser.add_argument('--platform',choices=['windows','linux'],required=True)
    args = parser.parse_args()
    try:
        expected = 'Windows' if args.platform == 'windows' else 'Linux'
        commit = os.environ.get('GITHUB_SHA','')
        if (os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_OS') != expected
                or platform.system() != expected or not re.fullmatch('[0-9a-f]{40}',commit)):
            raise package.PackagingError('Installationstest benötigt einen frischen nativen Actions-Runner.')
        image = args.image.resolve()
        receipt = json.loads(Path(str(image)+'.build.json').read_text())
        version, build = package.check_versions()
        if (receipt.get('type') != 'portable-build-receipt' or receipt.get('platform') != args.platform
                or receipt.get('architecture') != 'x86_64' or receipt.get('version') != version or receipt.get('build_number') != build
                or receipt.get('sha256') != package.digest(image) or receipt.get('size_bytes') != image.stat().st_size):
            raise package.PackagingError('Build-Beleg stimmt nicht mit nativen Paketbytes überein.')
        with tempfile.TemporaryDirectory(prefix='beni-native-installer-') as temporary:
            result = windows(image, Path(temporary)) if args.platform=='windows' else linux(image,Path(temporary))
        result.update({'type':'portable-native-acceptance','platform':args.platform,'architecture':'x86_64','version':version,
                       'build_number':build,'source_commit':commit,'sha256':package.digest(image),'os_version':platform.release(),
                       'publisher_signed':False})
        Path(str(image)+'.native.json').write_text(json.dumps(result,indent=2)+'\n')
        print('PASS: natives Paket installiert, eigene Oberfläche gestartet und wieder deinstalliert.')
        return 0
    except package.PackagingError as error:
        print(f'FAIL: {error}')
        return 2
    except (OSError,ValueError,KeyError,subprocess.TimeoutExpired):
        print('FAIL: native Installer-Abnahme fehlgeschlagen; Plattform, Paket und Fenster prüfen.')
        return 2


if __name__=='__main__': raise SystemExit(main())
