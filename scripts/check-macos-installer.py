#!/usr/bin/env python3
"""Verify an owned DMG, copy/install it in a temporary folder and remove it again."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import plistlib
import subprocess
import sys
import tempfile
import time

spec = importlib.util.spec_from_file_location("beni_installer_package", Path(__file__).with_name("package-macos.py"))
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


def launch_on_runner(app, directory):
    if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("RUNNER_OS") != "macOS":
        raise package.PackagingError("Der Fenster-Starttest ist auf frische macOS-Actions-Runner begrenzt.")
    inspector = directory / "window-smoke"
    package.run(["xcrun", "swiftc", str(package.ROOT / "packaging/macos/window-smoke.swift"), "-o", str(inspector)], "Fensterprüfer bauen")
    process = subprocess.Popen([str(app / "Contents/MacOS/beni-desktop-buddy")], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline and process.poll() is None:
            check = subprocess.run([str(inspector), str(process.pid)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5)
            if check.returncode == 0:
                return
            time.sleep(0.25)
        raise package.PackagingError("Die installierte App hat kein sichtbares eigenes Fenster geöffnet.")
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    parser.add_argument("--architecture", choices=tuple(package.ARCHITECTURES), required=True)
    parser.add_argument("--mode", choices=("development", "signed-test", "release"), required=True)
    parser.add_argument("--launch", action="store_true", help="Eigenes Fenster nur auf frischem Actions-Runner prüfen.")
    args = parser.parse_args(argv)
    try:
        if platform.system() != "Darwin":
            raise package.PackagingError("DMG-Abnahme benötigt macOS.")
        if args.architecture != package.native_architecture():
            raise package.PackagingError("Native Abnahme muss auf derselben Architektur laufen.")
        image = args.image.resolve()
        with Path(str(image) + ".build.json").open("rb") as stream:
            data = stream.read(8193)
        if len(data) > 8192:
            raise package.PackagingError("Build-Beleg ist zu gross.")
        receipt = json.loads(data)
        if receipt["type"] != "macos-build-receipt" or receipt["architecture"] != args.architecture:
            raise package.PackagingError("Build-Beleg gehört nicht zum gewählten Ziel.")
        if receipt["minimum_os_version"] != package.MINIMUM_MACOS:
            raise package.PackagingError("Systemanforderung stimmt nicht mit der Paketbasis überein.")
        expected = package.installer_filename(receipt["version"], args.architecture, args.mode)
        if receipt["filename"] != image.name or image.name != expected:
            raise package.PackagingError("Paketname und Build-Beleg stimmen nicht überein.")
        if receipt["notarized"] is not (args.mode == "release") or receipt["signing"] != ("ad_hoc" if args.mode == "development" else "developer_id"):
            raise package.PackagingError("Signiermodus stimmt nicht mit dem Build-Beleg überein.")
        with image.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if digest != receipt["sha256"] or image.stat().st_size != receipt["size_bytes"]:
            raise package.PackagingError("Paketbytes stimmen nicht mit dem Build-Beleg überein.")
        production = args.mode != "development"
        package.verify_signature(image, production=production)
        with tempfile.TemporaryDirectory(prefix="beni-install-check-") as folder:
            directory = Path(folder)
            mount = directory / "volume"
            mount.mkdir()
            attached = False
            try:
                package.run(["hdiutil", "attach", "-readonly", "-nobrowse", "-plist", "-mountpoint", str(mount), str(image)], "DMG öffnen")
                attached = True
                app = mount / "Beni Buddy.app"
                if (mount / "Programme").readlink() != Path("/Applications") or not (mount / "Installation.txt").is_file():
                    raise package.PackagingError("Programme-Link oder Installationshilfe fehlt.")
                for name in ("LICENSE.md", "LICENSE-MIT", "Beni.icns"):
                    if not (app / "Contents/Resources" / name).is_file():
                        raise package.PackagingError("Icon oder Lizenztext fehlt.")
                installed = directory / "installed/Beni Buddy.app"
                package.run(["ditto", str(app), str(installed)], "App in Testordner installieren")
                package.verify_signature(installed, production=production)
                with (installed / "Contents/Info.plist").open("rb") as stream:
                    info = plistlib.load(stream)
                if (info["CFBundleIdentifier"] != "io.witzelfitz.beni-buddy"
                        or info["CFBundleShortVersionString"] != receipt["version"].split("-")[0]
                        or info["CFBundleVersion"] != str(receipt["build_number"])
                        or info["LSMinimumSystemVersion"] != receipt["minimum_os_version"]):
                    raise package.PackagingError("Installierte Bundle-Version entspricht nicht dem Build-Beleg.")
                for name in ("beni-desktop-buddy", "beni-buddy-cli"):
                    binary = installed / "Contents/MacOS" / name
                    package.check_binary(binary, args.architecture)
                    package.verify_signature(binary, production=production)
                package.run([str(installed / "Contents/MacOS/beni-buddy-cli"), "validate-assets"], "Installierte CLI starten")
                if args.mode == "release":
                    for target in (image, installed):
                        package.run(["xcrun", "stapler", "validate", str(target)], "Installiertes Notarisierungsticket prüfen")
                    package.run(["spctl", "--assess", "--type", "execute", str(installed)], "Installierte App mit Gatekeeper prüfen")
                if args.launch:
                    launch_on_runner(installed, directory)
            finally:
                if attached:
                    package.run(["hdiutil", "detach", str(mount)], "Testvolume auswerfen")
        print("PASS: DMG, Signaturen, temporäre Installation, CLI-Start und Entfernung.")
        if args.launch:
            print("PASS: eigenes sichtbares App-Fenster auf dem frischen Runner.")
    except (package.PackagingError, OSError, ValueError, KeyError, TypeError, subprocess.TimeoutExpired) as error:
        detail = str(error) if isinstance(error, package.PackagingError) else "Installer oder Build-Beleg kann nicht geprüft werden."
        print(f"FAIL: {detail}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
