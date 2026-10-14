#!/usr/bin/env python3
"""Build native Windows NSIS installers and Linux AppImages; never publish or read signing keys."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import shutil
import struct
import subprocess
import tempfile
import urllib.request
import zipfile

from release import ROOT, check_versions


class PackagingError(Exception):
    pass


def run(args, label, *, env=None, cwd=ROOT, timeout=300):
    try:
        result = subprocess.run(args, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise PackagingError(f"{label}: Werkzeug nicht verfügbar oder Zeitlimit erreicht.") from error
    if result.returncode:
        # These commands compile/package public inputs; this module never reads credentials.
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print(result.stdout.decode('utf-8', errors='replace')[-8000:])
        raise PackagingError(f"{label} fehlgeschlagen (Exitcode {result.returncode}).")
    return result.stdout.decode('utf-8', errors='replace')


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def tool(name):
    info = json.loads((ROOT / 'packaging/tools.json').read_text())[name]
    folder = ROOT / 'dist/packaging-tools'
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / (name + ('-' + info['version']) + ('.zip' if name == 'nsis' else ''))
    if path.is_symlink(): raise PackagingError('Verlinktes Packaging-Werkzeug wird nicht benutzt.')
    if not path.exists():
        with urllib.request.urlopen(info['url'], timeout=60) as response:
            data = response.read(64 * 1024 * 1024 + 1)
        if len(data) > 64 * 1024 * 1024 or hashlib.sha256(data).hexdigest() != info['sha256']:
            raise PackagingError('Packaging-Werkzeug hat eine falsche Prüfsumme oder Grösse.')
        with path.open('xb') as stream: stream.write(data)
    if digest(path) != info['sha256']: raise PackagingError('Gespeichertes Packaging-Werkzeug ist verändert.')
    path.chmod(0o755)
    return path


def check_binary(path, target):
    if path.is_symlink() or not path.is_file(): raise PackagingError('Executable muss eine reguläre Datei sein.')
    with path.open('rb') as stream:
        header = stream.read(64)
        if target == 'linux':
            if len(header) < 20 or header[:6] != b'\x7fELF\x02\x01' or struct.unpack_from('<H', header, 18)[0] != 62:
                raise PackagingError('Linux-Paket benötigt ein x86_64-ELF-Executable.')
        else:
            if len(header) < 64 or header[:2] != b'MZ': raise PackagingError('Windows-Executable ist kein PE-File.')
            stream.seek(struct.unpack_from('<I', header, 60)[0]); pe = stream.read(6)
            if len(pe) != 6 or pe[:4] != b'PE\0\0' or struct.unpack_from('<H', pe, 4)[0] != 0x8664:
                raise PackagingError('Windows-Paket benötigt x86_64-Executables.')


def payload(folder, target):
    suffix = '.exe' if target == 'windows' else ''
    for path in (ROOT / f'apps/desktop/target/release/beni-desktop-buddy{suffix}', ROOT / f'target/release/beni-buddy-cli{suffix}'):
        check_binary(path, target)
        shutil.copy2(path, folder / path.name)
    tracked = run(['git', 'ls-files', '-z', '--', 'integrations'], 'Integrationsvorlagen auflisten')
    for relative in filter(None, tracked.split('\0')):
        source = ROOT / relative
        if source.is_symlink() or not source.is_file(): raise PackagingError('Nur reguläre eingecheckte Vorlagen übernehmen.')
        destination = folder / relative
        destination.parent.mkdir(parents=True, exist_ok=True); shutil.copy2(source, destination)
    for relative in ('LICENSE.md', 'LICENSE-MIT', 'docs/INTEGRATIONS.md'):
        shutil.copy2(ROOT / relative, folder / Path(relative).name)


def nsis_escape(value):
    if any(ord(c) < 32 for c in value): raise PackagingError('NSIS-Pfad enthält Steuerzeichen.')
    return value.replace('$', '$$').replace('"', '$\\"')


def nsis_lists(folder):
    files = sorted(path for path in folder.rglob('*') if path.is_file())
    installs, removes, directories = [], [], set()
    for path in files:
        if path.is_symlink(): raise PackagingError('Installer darf keine verlinkten Dateien enthalten.')
        relative = path.relative_to(folder)
        directory = '\\'.join(relative.parent.parts)
        destination = '\\'.join(relative.parts)
        installs.extend([f'SetOutPath "$INSTDIR{chr(92) + nsis_escape(directory) if directory else ""}"',
                         f'File "{nsis_escape(str(path))}"'])
        removes.append(f'Delete "$INSTDIR\\{nsis_escape(destination)}"')
        for parent in relative.parents:
            if str(parent) != '.': directories.add('\\'.join(parent.parts))
    removes.extend(f'RMDir "$INSTDIR\\{nsis_escape(directory)}"' for directory in sorted(directories, key=lambda d: (-d.count('\\'), d)))
    return '\n'.join(installs) + '\n', '\n'.join(removes) + '\n'


def windows_installer(image, directory):
    archive_path = tool('nsis')
    compiler = directory / 'nsis'
    compiler.mkdir()
    with zipfile.ZipFile(archive_path) as archive:
        for member in archive.infolist():
            if member.filename.startswith('/') or '..' in Path(member.filename).parts:
                raise PackagingError('Unzulässiger Werkzeug-Archivpfad.')
        archive.extractall(compiler)
    compiler = compiler / 'nsis-3.12'
    stage = directory / 'payload'; stage.mkdir()
    payload(stage, 'windows')
    shutil.copy2(compiler / 'COPYING', stage / 'NSIS-COPYING.txt')
    install, remove = nsis_lists(stage)
    install_path, remove_path = directory / 'install-files.nsh', directory / 'remove-files.nsh'
    install_path.write_text(install, encoding='utf-8'); remove_path.write_text(remove, encoding='utf-8')
    version, _ = check_versions()
    arguments = [str(compiler / 'makensis.exe'), '/V2', '/WX', '/INPUTCHARSET', 'UTF8',
                 f'/DVERSION={version.split("-")[0]}', f'/DOUTPUT={image}', f'/DINSTALL_FILES={install_path}',
                 f'/DREMOVE_FILES={remove_path}', f'/DICON={ROOT / "assets/app-icon/Beni.ico"}', f'/DLICENSE={ROOT / "LICENSE-MIT"}',
                 str(ROOT / 'packaging/windows/installer.nsi')]
    run(arguments, 'NSIS-Installer erstellen')
    if not image.is_file(): raise PackagingError('NSIS hat keinen Installer unter dem erwarteten Dateinamen erstellt.')


SYSTEM_LIBRARIES = re.compile(r'^(?:ld-linux|libc\.|libm\.|libpthread\.|libdl\.|librt\.|libresolv\.|libGL\.|libEGL\.|libGLX|libGLdispatch|libdrm|libgbm)')


def library_owner(path):
    candidates = [path, path.resolve()]
    if str(path).startswith('/lib/'):
        candidates.append(Path('/usr')/path.relative_to('/'))
    if str(path).startswith('/usr/lib/'):
        candidates.append(Path('/')/path.relative_to('/usr'))
    for candidate in dict.fromkeys(candidates):
        result = subprocess.run(['dpkg-query','-S',str(candidate)], stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=15)
        if result.returncode: continue
        lines = result.stdout.decode('utf-8').splitlines()
        if len(lines) != 1 or ': ' not in lines[0]: continue
        owner = lines[0].split(': ',1)[0]
        if re.fullmatch(r'[a-z0-9][a-z0-9+.-]*(?::amd64)?',owner): return owner.split(':',1)[0]
    raise PackagingError('Bibliotheksherkunft kann auch nach Auflösung von /usr-merge nicht belegt werden.')


def linux_libraries(appdir):
    destination = appdir / 'usr/lib'; destination.mkdir(parents=True)
    initial = ['libxkbcommon.so.0', 'libxkbcommon-x11.so.0', 'libX11.so.6', 'libX11-xcb.so.1', 'libXcursor.so.1',
               'libXrandr.so.2', 'libXi.so.6', 'libXinerama.so.1', 'libXext.so.6', 'libXrender.so.1', 'libXfixes.so.3',
               'libwayland-client.so.0', 'libwayland-cursor.so.0', 'libwayland-egl.so.1']
    listing = run(['ldconfig', '-p'], 'Systembibliotheken auflisten')
    available = dict(re.findall(r'^\s*(\S+) .*x86-64.*=> (\S+)$', listing, re.MULTILINE))
    queue = [appdir / 'usr/bin/beni-desktop-buddy', appdir / 'usr/bin/beni-buddy-cli']
    for name in initial:
        if name not in available: raise PackagingError('Erforderliche Linux-Bibliothek fehlt.')
        queue.append(Path(available[name]))
    copied, records = set(), []
    copyrights = appdir / 'usr/share/beni-buddy/library-licenses'; copyrights.mkdir(parents=True)
    while queue:
        path = queue.pop()
        name = path.name
        if name in copied or SYSTEM_LIBRARIES.match(name): continue
        check_binary(path.resolve(), 'linux')
        symbols = run(['objdump', '-T', str(path)], 'GLIBC-Basis prüfen')
        versions = re.findall(r'\bGLIBC_([0-9]+)\.([0-9]+)', symbols)
        if any((int(a), int(b)) > (2, 35) for a, b in versions):
            raise PackagingError('Linux-Executable/Bibliothek verlangt GLIBC neuer als 2.35.')
        copied.add(name)
        if path.parent != appdir / 'usr/bin':
            resolved = path.resolve()
            if not str(resolved).startswith(('/usr/lib/', '/lib/')): raise PackagingError('Fremder Bibliothekspfad wird nicht gebündelt.')
            shutil.copy2(resolved, destination / name)
            owner = library_owner(path)
            license_path = Path('/usr/share/doc') / owner / 'copyright'
            if not license_path.is_file(): raise PackagingError('Bibliothekslizenz fehlt.')
            shutil.copy2(license_path, copyrights / (owner + '.copyright'))
            records.append({'file':name, 'sha256':digest(destination / name), 'ubuntu_package':owner})
        dependencies = run(['ldd', str(path)], 'Bibliotheksabhängigkeiten prüfen')
        if 'not found' in dependencies: raise PackagingError('Linux-Laufzeitabhängigkeit fehlt.')
        queue.extend(Path(value) for value in re.findall(r'=> (/[^\s]+)', dependencies))
    (copyrights / 'provenance.json').write_text(json.dumps(records, indent=2)+'\n')


def linux_installer(image, directory):
    appdir = directory / 'Beni-Buddy.AppDir'
    bin_dir = appdir / 'usr/bin'; bin_dir.mkdir(parents=True)
    resources = appdir / 'usr/share/beni-buddy'; resources.mkdir(parents=True)
    payload(resources, 'linux')
    shutil.copy2(ROOT / 'packaging/linux/AppImage-runtime-LICENSE.txt', resources / 'AppImage-runtime-LICENSE.txt')
    runtime_info = json.loads((ROOT / 'packaging/tools.json').read_text())['appimage-runtime']
    (resources / 'AppImage-runtime-source.txt').write_text(
        'AppImage type2-runtime '+runtime_info['version']+'\nBinary SHA-256: '+runtime_info['sha256']+
        '\nSource: https://github.com/AppImage/type2-runtime/tree/'+runtime_info['version']+
        '\nBuild instructions: https://github.com/AppImage/type2-runtime/blob/'+runtime_info['version']+'/BUILD.md\n', encoding='utf-8')
    for name in ('beni-desktop-buddy', 'beni-buddy-cli'):
        shutil.move(resources / name, bin_dir / name)
    shutil.copy2(ROOT / 'packaging/linux/AppRun', appdir / 'AppRun'); (appdir / 'AppRun').chmod(0o755)
    shutil.copy2(ROOT / 'packaging/linux/install-desktop.py', resources / 'install-desktop.py')
    icon_name = 'io.witzelfitz.beni-buddy.png'
    icon = appdir / 'usr/share/icons/hicolor/256x256/apps' / icon_name
    icon.parent.mkdir(parents=True); shutil.copy2(ROOT / 'assets/app-icon/beni-256.png', icon)
    shutil.copy2(icon, appdir / icon_name); (appdir / '.DirIcon').symlink_to(icon_name)
    entry = (ROOT / 'packaging/linux/io.witzelfitz.beni-buddy.desktop').read_text().replace('Exec=beni-desktop-buddy', 'Exec=AppRun')
    (appdir / 'io.witzelfitz.beni-buddy.desktop').write_text(entry)
    linux_libraries(appdir)
    runtime, builder = tool('appimage-runtime'), tool('appimagetool')
    env = dict(os.environ, ARCH='x86_64', APPIMAGE_EXTRACT_AND_RUN='1')
    run([str(builder), '--runtime-file', str(runtime), str(appdir), str(image)], 'AppImage erstellen', env=env)
    image.chmod(0o755)


def build(target, output):
    expected = 'Windows' if target == 'windows' else 'Linux'
    if platform.system() != expected or platform.machine().lower() not in ('amd64','x86_64'):
        raise PackagingError('Installer auf einem nativen x86_64-System des richtigen Betriebssystems bauen.')
    destinations = [output, Path(str(output)+'.sha256'), Path(str(output)+'.build.json')]
    if any(p.exists() or p.is_symlink() for p in destinations):
        raise PackagingError('Installer oder Beleg existiert bereits; nichts überschrieben.')
    version, build_number = check_versions()
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='beni-installer-', dir=output.parent) as temporary:
        directory = Path(temporary); image = directory / output.name
        if target == 'windows': windows_installer(image, directory)
        else: linux_installer(image, directory)
        checksum = digest(image)
        receipt = {'type':'portable-build-receipt','platform':target,'architecture':'x86_64','version':version,
                   'build_number':build_number,'filename':image.name,'sha256':checksum,'size_bytes':image.stat().st_size,
                   'publisher_signed':False,'minimum_runtime':'10.0.22000' if target=='windows' else '2.35'}
        checksum_path = directory / (output.name+'.sha256')
        receipt_path = directory / (output.name+'.build.json')
        checksum_path.write_text(f'{checksum}  {output.name}\n')
        receipt_path.write_text(json.dumps(receipt, indent=2)+'\n')
        adopted = []
        try:
            for source, destination in zip((image, checksum_path, receipt_path), destinations):
                # Exclusive copies work on Windows volumes without hard-link support.
                with destination.open('xb') as target_stream:
                    adopted.append(destination)
                    with source.open('rb') as source_stream: shutil.copyfileobj(source_stream, target_stream)
                if target=='linux' and destination==output: destination.chmod(0o755)
        except OSError:
            for path in adopted: path.unlink()
            raise


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--platform', required=True, choices=['windows','linux'])
    args = parser.parse_args(); version, _ = check_versions()
    extension = 'exe' if args.platform == 'windows' else 'AppImage'
    path = ROOT / f'dist/{args.platform}/beni-buddy-{version}-{args.platform}-x86_64.{extension}'
    try:
        build(args.platform, path)
        print(f'PASS: {args.platform}-Installer erstellt; kein öffentlicher Release.')
    except PackagingError as error:
        print(f'FAIL: {error}')
        return 2
    except OSError as error:
        # Numeric error codes are sufficient to diagnose runner filesystem/network failures.
        print(f'FAIL: Packaging-Systemfehler ({type(error).__name__}, errno={error.errno}, winerror={getattr(error,"winerror",None)}).')
        return 2
    except (ValueError, KeyError, zipfile.BadZipFile):
        print('FAIL: Installer-Build abgelehnt; Werkzeug, Architektur und Eingaben prüfen.')
        return 2
    return 0


if __name__ == '__main__': raise SystemExit(main())
