#!/usr/bin/env python3
"""Explicit per-user AppImage menu integration; preserve settings and unrelated files."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile

MARKER = "X-Beni-Managed=true"


def regular_target(path):
    for parent in (path, *path.parents):
        if parent.is_symlink():
            raise ValueError("Verlinkte Installationspfade werden nicht verändert.")


def desktop_text(image):
    if any(ord(c) < 32 for c in str(image)):
        raise ValueError("Installationspfad enthält Steuerzeichen.")
    quoted = str(image).replace('\\', '\\\\').replace('"', '\\"').replace('$', '\\$').replace('`', '\\`').replace('%', '%%')
    return ("[Desktop Entry]\nType=Application\nName=Beni Buddy\nComment=Beni als Begleiter auf deinem Desktop\n"
            f'Exec="{quoted}"\nIcon=io.witzelfitz.beni-buddy\nTerminal=false\nCategories=Utility;\n{MARKER}\n')


def integrate(action, appdir, data_home, source):
    if action not in ('--install-desktop', '--remove-desktop') or not data_home.is_absolute():
        raise ValueError('Ungültige Aktion oder relativer Datenpfad.')
    home = data_home
    folder = home / "beni-buddy"
    image = folder / "Beni-Buddy.AppImage"
    entry = home / "applications/io.witzelfitz.beni-buddy.desktop"
    icon = home / "icons/hicolor/256x256/apps/io.witzelfitz.beni-buddy.png"
    receipt = folder / "installation.json"
    for path in (image, entry, icon, receipt): regular_target(path)
    if entry.exists() and MARKER not in entry.read_text(encoding='utf-8'):
        raise ValueError("Vorhandener fremder Menüeintrag bleibt erhalten.")
    if action == "--remove-desktop":
        if not receipt.is_file(): raise ValueError("Kein eigener Installationsbeleg vorhanden.")
        recorded = json.loads(receipt.read_text(encoding='utf-8'))
        if entry.exists() and entry.read_text(encoding='utf-8') != desktop_text(image):
            raise ValueError("Veränderter Menüeintrag bleibt erhalten.")
        if icon.exists() and hashlib.sha256(icon.read_bytes()).hexdigest() != recorded['icon_sha256']:
            raise ValueError("Verändertes Icon bleibt erhalten.")
        if image.exists() and hashlib.sha256(image.read_bytes()).hexdigest() != recorded['image_sha256']:
            raise ValueError('Verändertes AppImage bleibt erhalten.')
        for path in (entry, icon, image, receipt): path.unlink(missing_ok=True)
        try: folder.rmdir()
        except OSError: pass
        return
    if source is None or not source.is_file(): raise ValueError("Installation muss aus einem echten AppImage gestartet werden.")
    if (image.exists() or icon.exists()) and not receipt.is_file():
        raise ValueError("Vorhandene Dateien ohne eigenen Beleg bleiben erhalten.")
    if receipt.exists():
        recorded = json.loads(receipt.read_text(encoding='utf-8'))
        for path, field in ((image,'image_sha256'),(icon,'icon_sha256')):
            if path.exists() and hashlib.sha256(path.read_bytes()).hexdigest() != recorded[field]:
                raise ValueError('Veränderte Installationsdateien bleiben erhalten.')
        if entry.exists() and entry.read_text(encoding='utf-8') != desktop_text(image):
            raise ValueError('Veränderter Menüeintrag bleibt erhalten.')
    pixels = appdir / "usr/share/icons/hicolor/256x256/apps/io.witzelfitz.beni-buddy.png"
    icon_bytes = pixels.read_bytes()
    for path in (image, entry, icon, receipt): path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".install-", dir=folder) as temporary:
        staged = Path(temporary) / image.name
        shutil.copyfile(source, staged); staged.chmod(0o755)
        os.replace(staged, image)
    icon.write_bytes(icon_bytes)
    entry.write_text(desktop_text(image), encoding='utf-8')
    receipt.write_text(json.dumps({'icon_sha256':hashlib.sha256(icon_bytes).hexdigest(),
                                 'image_sha256':hashlib.sha256(image.read_bytes()).hexdigest()})+'\n')


if __name__ == '__main__':
    # Flags are positional actions, deliberately not general-purpose filesystem arguments.
    import sys
    try:
        action, appdir = sys.argv[1:]
        integrate(action, Path(appdir), Path(os.environ.get('XDG_DATA_HOME', str(Path.home()/'.local/share'))),
                  Path(os.environ['APPIMAGE']) if os.environ.get('APPIMAGE') else None)
        print('PASS: Beni-Menüeinbindung aktualisiert; bisherige Einstellungen bleiben erhalten.')
    except (ValueError, OSError, KeyError):
        print('FAIL: Menüeinbindung abgelehnt; Quelle und eigene Zielpfade prüfen.', file=sys.stderr)
        raise SystemExit(2)
