#!/usr/bin/env python3
"""Build macOS bundles and DMGs; public packages require successful notarization."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import subprocess
import sys
import tempfile

from release import check_versions

ROOT = Path(__file__).resolve().parents[1]
ARCHITECTURES = {"aarch64": "arm64", "x86_64": "x86_64"}
MINIMUM_MACOS = "11.0"


class PackagingError(Exception):
    pass


def run(args, label, env=None, timeout=300):
    try:
        result = subprocess.run(args, cwd=ROOT, env=env, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True, errors="replace", timeout=timeout)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise PackagingError(f"{label}: Werkzeug nicht verfügbar oder Zeitlimit erreicht.") from error
    if result.returncode:
        # Signing/notary output may contain account details; never echo commands or raw output.
        if args[0] == "codesign":
            diagnostics = (("resource fork", "Nicht erlaubte Finder-/Resource-Fork-Metadaten."),
                           ("code object is not signed at all", "Codesign-Objekt ist noch nicht signiert."),
                           ("bundle format", "Codesign-Bundleformat wird nicht erkannt."),
                           ("invalid or unsupported format", "Codesign-Format wird nicht unterstützt."),
                           ("errSecInternalComponent", "Interner Fehler des Signierdienstes."))
            for fragment, detail in diagnostics:
                if fragment in result.stdout:
                    raise PackagingError(f"{label} fehlgeschlagen: {detail}")
        raise PackagingError(f"{label} fehlgeschlagen (Exitcode {result.returncode}).")
    return result.stdout


def native_architecture():
    machine = platform.machine()
    return "aarch64" if machine == "arm64" else machine


def signing_identity(requested=None):
    output = run(["security", "find-identity", "-v", "-p", "codesigning"], "Signieridentität prüfen")
    identities = re.findall(r'\b([0-9A-Fa-f]{40})\s+"(Developer ID Application:[^"\r\n]+)"', output)
    matches = [(digest, name) for digest, name in identities if requested is None or requested in (digest, name)]
    if len(matches) != 1:
        raise PackagingError("Genau eine gültige Developer-ID-Application-Identität auswählen.")
    return matches[0][0]


def preflight(mode, architecture, profile, identity=None):
    if platform.system() != "Darwin":
        raise PackagingError("Dieses Script benötigt macOS.")
    if architecture not in ARCHITECTURES:
        raise PackagingError("Nicht unterstützte Mac-Architektur.")
    if mode not in ("development", "signed-test", "release"):
        raise PackagingError("Unbekannter Paketmodus.")
    if mode == "release" and not profile:
        raise PackagingError("Für Release-Pakete fehlt BENI_NOTARY_PROFILE; zuerst den Apple-Zugang einrichten.")
    check_versions()
    return "-" if mode == "development" else signing_identity(identity)


def executable_paths(architecture):
    # Reuse native caches; cross-compilation requires an explicitly installed target.
    target = f"{architecture}-apple-darwin"
    cross = architecture != native_architecture()
    target_args = ["--target", target] if cross else []
    if cross and target not in run(["rustup", "target", "list", "--installed"], "Rust-Ziele prüfen").splitlines():
        raise PackagingError(f"Rust-Ziel fehlt: rustup target add {target}.")
    env = dict(os.environ, MACOSX_DEPLOYMENT_TARGET=MINIMUM_MACOS)
    print(f"Release-Binaries bauen: {architecture}.", flush=True)
    run(["cargo", "build", "--manifest-path", "apps/desktop/Cargo.toml", "--release", "--locked", *target_args],
        "Desktop-Build", env=env, timeout=1500)
    run(["cargo", "build", "--release", "--locked", "-p", "buddy-cli", *target_args], "CLI-Build", env=env, timeout=600)
    suffix = Path(target) / "release" if cross else Path("release")
    return (ROOT / "apps/desktop/target" / suffix / "beni-desktop-buddy",
            ROOT / "target" / suffix / "beni-buddy-cli")


def check_binary(path, architecture):
    actual = run(["lipo", "-archs", str(path)], "Binary-Architektur prüfen").split()
    if actual != [ARCHITECTURES[architecture]]:
        raise PackagingError("Binary-Architektur entspricht nicht dem angebotenen Paket.")
    load_commands = run(["otool", "-l", str(path)], "macOS-Mindestversion prüfen")
    versions = re.findall(r"^\s+minos ([0-9.]+)$", load_commands, re.MULTILINE)
    if not versions:
        blocks = load_commands.split("cmd LC_VERSION_MIN_MACOSX")[1:]
        versions = [re.search(r"\bversion ([0-9.]+)", block).group(1) for block in blocks
                    if re.search(r"\bversion ([0-9.]+)", block)]
    minimum = (tuple(map(int, MINIMUM_MACOS.split("."))) + (0, 0, 0))[:3]
    if not versions or any((tuple(map(int, v.split("."))) + (0, 0, 0))[:3] > minimum for v in versions):
        raise PackagingError("Binary-Mindestversion fehlt oder übersteigt die Paketangabe.")


def sign(path, identity, executable=False):
    options = ["--timestamp=none"] if identity == "-" else ["--timestamp"]
    if executable and identity != "-":
        options += ["--options", "runtime"]
    kind = "DMG" if path.suffix == ".dmg" else ("App-Bundle" if path.suffix == ".app" else "Executable")
    run(["codesign", "--force", "--sign", identity, *options, str(path)], f"Codesignierung ({kind})")


def verify_signature(path, production=False):
    run(["codesign", "--verify", "--deep", "--strict", str(path)], "Signaturprüfung")
    if production:
        details = run(["codesign", "-d", "--verbose=4", str(path)], "Developer-ID-Signatur prüfen")
        if "Authority=Developer ID Application:" not in details or "Timestamp=" not in details:
            raise PackagingError("Developer-ID-Signatur oder sicherer Zeitstempel fehlt.")
        if path.suffix != ".dmg" and not re.search(r"\bflags=0x[0-9a-fA-F]+\([^\n)]*\bruntime\b", details):
            raise PackagingError("Hardened Runtime fehlt.")


def make_bundle(destination, architecture, identity, binaries=None):
    version, build = check_versions()
    binaries = executable_paths(architecture) if binaries is None else binaries
    for binary in binaries:
        check_binary(binary, architecture)
    macos = destination / "Contents/MacOS"
    resources = destination / "Contents/Resources"
    macos.mkdir(parents=True)
    resources.mkdir()
    for source in binaries:
        shutil.copy2(source, macos / source.name)
    # Only committed templates: no local keys, caches or user-created hook settings in packages.
    tracked = run(["git", "ls-files", "-z", "--", "integrations"], "Integrationsvorlagen auflisten")
    for relative in filter(None, tracked.split("\0")):
        source = ROOT / relative
        if source.is_symlink() or not source.is_file():
            raise PackagingError("Integrationsvorlagen müssen reguläre Projektdateien sein.")
        target = resources / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    for source in (ROOT / "docs/INTEGRATIONS.md", ROOT / "assets/app-icon/Beni.icns",
                   ROOT / "LICENSE.md", ROOT / "LICENSE-MIT"):
        shutil.copy2(source, resources / source.name)
    info = {
        "CFBundleName": "Beni Buddy", "CFBundleDisplayName": "Beni Buddy",
        "CFBundleIdentifier": "io.witzelfitz.beni-buddy", "CFBundleExecutable": "beni-desktop-buddy",
        "CFBundleIconFile": "Beni.icns", "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version.split("-")[0], "CFBundleVersion": str(build),
        "LSMinimumSystemVersion": MINIMUM_MACOS, "LSUIElement": True, "NSHighResolutionCapable": True,
    }
    with (destination / "Contents/Info.plist").open("wb") as stream:
        plistlib.dump(info, stream, sort_keys=False)
    run(["plutil", "-lint", str(destination / "Contents/Info.plist")], "Bundle-Plist prüfen")
    # Signing the main executable can make codesign inspect the entire surrounding app.
    # Sign nested helpers first; x86_64 linkers do not provide arm64's implicit ad-hoc signature.
    for source in sorted(binaries, key=lambda path: path.name == "beni-desktop-buddy"):
        executable = macos / source.name
        sign(executable, identity, executable=True)
        verify_signature(executable, production=identity != "-")
    sign(destination, identity, executable=True)
    verify_signature(destination, production=identity != "-")


def notarize(path, profile):
    print(f"Apple-Notarisierung: {path.suffix}.", flush=True)
    keychain = os.environ.get("BENI_NOTARY_KEYCHAIN")
    keychain_args = ["--keychain", keychain] if keychain else []
    result = run(["xcrun", "notarytool", "submit", str(path), "--keychain-profile", profile, *keychain_args,
                  "--wait", "--timeout", "20m", "--output-format", "json"], "Apple-Notarisierung", timeout=1260)
    try:
        status = json.loads(result)["status"]
    except (ValueError, KeyError, TypeError) as error:
        raise PackagingError("Apple hat keinen gültigen Notarisierungsstatus geliefert.") from error
    if status != "Accepted":
        raise PackagingError("Apple hat das Paket nicht akzeptiert; kein Release-Paket wird übernommen.")


def staple_and_assess(path):
    run(["xcrun", "stapler", "staple", str(path)], "Notarisierungsticket anhängen")
    run(["xcrun", "stapler", "validate", str(path)], "Notarisierungsticket prüfen")
    verify_signature(path, production=True)
    kind = ["--type", "execute"] if path.suffix == ".app" else ["--type", "open", "--context", "context:primary-signature"]
    run(["spctl", "--assess", *kind, str(path)], "Gatekeeper-Abnahme")


def notarize_bundle(app, directory, profile):
    archive = directory / "notarization.zip"
    run(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(app), str(archive)], "Notarisierungsarchiv bauen")
    notarize(archive, profile)
    staple_and_assess(app)


def installer_filename(version, architecture, mode):
    suffix = {"development": "-development", "signed-test": "-unnotarized", "release": ""}[mode]
    return f"beni-buddy-{version}-macos-{architecture}{suffix}.dmg"


def make_dmg(app, directory, image, identity):
    payload = directory / "payload"
    payload.mkdir()
    run(["ditto", str(app), str(payload / "Beni Buddy.app")], "Installer-App kopieren")
    (payload / "Programme").symlink_to("/Applications", target_is_directory=True)
    shutil.copy2(ROOT / "packaging/macos/INSTALLATION.txt", payload / "Installation.txt")
    run(["hdiutil", "create", "-volname", "Beni Buddy", "-srcfolder", str(payload),
         "-format", "UDZO", "-ov", str(image)], "DMG erstellen")
    # Development containers are unsigned; ad-hoc signatures still protect the app and binaries.
    # Apple distribution containers always retain their Developer-ID signature.
    if identity != "-":
        sign(image, identity)
        verify_signature(image, production=True)
    run(["hdiutil", "verify", str(image)], "DMG-Integrität prüfen")


def install_bundle(args, identity):
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="beni-bundle-", dir=output.parent) as folder:
        staging = Path(folder)
        app = staging / "Beni Buddy.app"
        make_bundle(app, args.architecture, identity)
        if args.mode == "release":
            notarize_bundle(app, staging, args.notary_profile)
        previous = staging / "previous.app"
        if output.exists():
            output.rename(previous)
        try:
            app.rename(output)
        except OSError:
            if previous.exists():
                previous.rename(output)
            raise
    print(f"Bundle vorbereitet ({args.mode}): {output}.")


def install_dmg(args, identity):
    version, build = check_versions()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    filename = installer_filename(version, args.architecture, args.mode)
    output = output_dir / filename
    receipt = output_dir / (filename + ".build.json")
    checksum = output_dir / (filename + ".sha256")
    if any(p.exists() or p.is_symlink() for p in (output, receipt, checksum)):
        raise PackagingError("Paketadresse existiert bereits; unveröffentlichte Testdateien bewusst entfernen oder neue Version wählen.")
    with tempfile.TemporaryDirectory(prefix="beni-installer-", dir=output_dir) as folder:
        staging = Path(folder)
        app = staging / "Beni Buddy.app"
        image = staging / filename
        make_bundle(app, args.architecture, identity)
        if args.mode == "release":
            notarize_bundle(app, staging, args.notary_profile)
        make_dmg(app, staging, image, identity)
        if args.mode == "release":
            notarize(image, args.notary_profile)
            staple_and_assess(image)
            verify_signature(image, production=True)
        with image.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        metadata = {
            "type": "macos-build-receipt", "version": version, "build_number": build,
            "architecture": args.architecture, "minimum_os_version": MINIMUM_MACOS,
            "filename": filename, "size_bytes": image.stat().st_size, "sha256": digest,
            "signing": "ad_hoc" if args.mode == "development" else "developer_id",
            "notarized": args.mode == "release",
        }
        (staging / checksum.name).write_text(f"{digest}  {filename}\n", encoding="utf-8")
        (staging / receipt.name).write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
        # Only after all validation; link() refuses to overwrite concurrent output.
        published = []
        try:
            for source, destination in ((image, output), (staging / checksum.name, checksum), (staging / receipt.name, receipt)):
                os.link(source, destination)
                published.append(destination)
        except OSError:
            for path in published:
                path.unlink()
            raise
    print(f"Installer vorbereitet ({args.mode}): {output}.")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for command in ("bundle", "installer"):
        child = commands.add_parser(command)
        child.add_argument("--architecture", choices=tuple(ARCHITECTURES), default=native_architecture())
        child.add_argument("--mode", choices=("development", "signed-test", "release"),
                           default="development" if command == "bundle" else "release")
        child.add_argument("--identity", default=os.environ.get("BENI_SIGNING_IDENTITY"))
        child.add_argument("--notary-profile", default=os.environ.get("BENI_NOTARY_PROFILE"))
        if command == "bundle":
            child.add_argument("--output", type=Path, default=ROOT / "dist/Beni Buddy.app")
        else:
            child.add_argument("--output-dir", type=Path, default=ROOT / "dist/macos")
    args = parser.parse_args(argv)
    try:
        identity = preflight(args.mode, args.architecture, args.notary_profile, args.identity)
        if args.command == "bundle":
            install_bundle(args, identity)
        else:
            install_dmg(args, identity)
    except (PackagingError, ValueError, OSError, KeyError) as error:
        detail = str(error) if isinstance(error, PackagingError) else "Datei oder Versionsdaten konnten nicht verarbeitet werden."
        print(f"FAIL: {detail}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
