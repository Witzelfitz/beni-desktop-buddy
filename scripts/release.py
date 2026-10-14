#!/usr/bin/env python3
"""Offline release/version validation. Never downloads, signs or publishes files."""
import argparse
import json
from pathlib import Path
import plistlib
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
MAX_MANIFEST_BYTES = 256 * 1024
VERSION_PATTERN = r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-beta\.[1-9][0-9]*)?"
REPOSITORY = "https://github.com/Witzelfitz/beni-desktop-buddy"
FORMATS = {"macos": "dmg", "windows": "exe", "linux": "appimage"}


def feed_filename(architecture, channel):
    return f"appcast-macos-{architecture}-{channel}.xml"


def toml_file(path):
    return tomllib.loads(path.read_text(encoding="utf-8"))


def version_info(root=ROOT):
    cargo = toml_file(root / "Cargo.toml")
    version = cargo["workspace"]["package"]["version"]
    build = cargo["workspace"]["metadata"]["beni-release"]["build-number"]
    if not isinstance(version, str) or len(version) > 64 or not re.fullmatch(VERSION_PATTERN, version):
        raise ValueError("Workspace-Version muss X.Y.Z oder X.Y.Z-beta.N sein.")
    if type(build) is not int or not 1 <= build <= 9999:
        raise ValueError("Buildnummer muss zwischen 1 und 9999 liegen.")
    return version, build


def check_versions(root=ROOT, tag=None):
    version, build = version_info(root)
    for relative in ("apps/desktop/Cargo.toml", "crates/buddy-ipc/Cargo.toml"):
        if toml_file(root / relative)["package"]["version"] != version:
            raise ValueError(f"{relative}: Version stimmt nicht mit dem Workspace überein.")
    for relative in ("crates/buddy-core/Cargo.toml", "crates/buddy-cli/Cargo.toml"):
        if toml_file(root / relative)["package"]["version"] != {"workspace": True}:
            raise ValueError(f"{relative}: Version muss vom Workspace geerbt werden.")
    expected = {
        "Cargo.lock": {"buddy-core", "buddy-cli", "buddy-ipc"},
        "apps/desktop/Cargo.lock": {"beni-desktop-buddy", "buddy-core", "buddy-ipc"},
    }
    for relative, names in expected.items():
        found = {p["name"]: p["version"] for p in toml_file(root / relative)["package"] if p["name"] in names}
        if set(found) != names or any(v != version for v in found.values()):
            raise ValueError(f"{relative}: Beni-Versionen fehlen oder sind nicht synchron.")
    if tag is not None and tag != f"v{version}":
        raise ValueError("Release-Tag muss v plus Workspace-Version entsprechen.")
    return version, build


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("Doppelte JSON-Felder sind nicht erlaubt.")
        result[key] = value
    return result


def read_manifest(path):
    with path.open("rb") as stream:
        data = stream.read(MAX_MANIFEST_BYTES + 1)
    if len(data) > MAX_MANIFEST_BYTES:
        raise ValueError("Release-Metadaten überschreiten 256 KiB.")
    return json.loads(data, object_pairs_hook=unique_object)


def validate_manifest(manifest, expected_version=None, expected_build=None, channel=None):
    # Import only for manifest checks; version stamping needs Python's standard library alone.
    from jsonschema import Draft202012Validator

    schema_version = manifest.get("schema_version") if isinstance(manifest, dict) else None
    if type(schema_version) is not int or schema_version not in (1, 2):
        raise ValueError("Unbekannte Release-Schema-Version.")
    schema = json.loads((ROOT / f"release/schema-v{schema_version}.json").read_text(encoding="utf-8"))
    Draft202012Validator.check_schema(schema)
    errors = list(Draft202012Validator(schema).iter_errors(manifest))
    if errors:
        location = ".".join(map(str, errors[0].absolute_path)) or "manifest"
        # Do not echo arbitrary manifest values, URLs or query strings into CI logs.
        raise ValueError(f"Ungültiges Release-Feld: {location} ({errors[0].validator}).")
    version = manifest["version"]
    if not re.fullmatch(VERSION_PATTERN, version):
        raise ValueError("Release-Version muss X.Y.Z oder X.Y.Z-beta.N sein.")
    actual_channel = "beta" if "-beta." in version else "stable"
    if manifest["channel"] != actual_channel or (channel is not None and channel != actual_channel):
        raise ValueError("Version und Release-Kanal stimmen nicht überein.")
    if expected_version is not None and version != expected_version:
        raise ValueError("Release-Version stimmt nicht mit dem Build überein.")
    if expected_build is not None and manifest["build_number"] != expected_build:
        raise ValueError("Release-Buildnummer stimmt nicht mit dem Build überein.")
    if manifest["release_notes_url"] != f"{REPOSITORY}/releases/tag/v{version}":
        raise ValueError("Release-Notizen müssen zur gleichen Version gehören.")
    targets = set()
    for artifact in manifest["artifacts"]:
        target = (artifact["platform"], artifact["architecture"])
        if target in targets:
            raise ValueError("Jede Plattform/Architektur darf nur einmal vorkommen.")
        targets.add(target)
        extension = "AppImage" if artifact["format"] == "appimage" else artifact["format"]
        filename = f"beni-buddy-{version}-{target[0]}-{target[1]}.{extension}"
        url = f"{REPOSITORY}/releases/download/v{version}/{filename}"
        if artifact["url"] != url or artifact["signature"]["url"] != url + ".sig":
            raise ValueError("Paket und Signatur müssen zum versionierten Ziel gehören.")
        if not re.fullmatch(r"beni-update-v[1-9][0-9]*", artifact["signature"]["key_id"]):
            raise ValueError("Signierschlüssel-ID ist ungültig.")
        for text in artifact["requirements"].values():
            if not text.strip() or any(ord(c) < 32 or 127 <= ord(c) <= 159 for c in text):
                raise ValueError("Systemanforderungen dürfen weder leer sein noch Steuerzeichen enthalten.")
    if "macos_update_feeds" in manifest:
        macs = {a["architecture"]: a for a in manifest["artifacts"] if a["platform"] == "macos"}
        feeds = manifest["macos_update_feeds"]
        if len(feeds) != len(macs) or {f["architecture"] for f in feeds} != set(macs):
            raise ValueError("Appcasts müssen genau die vorhandenen Mac-Ziele abdecken.")
        for feed in feeds:
            name = feed_filename(feed["architecture"], actual_channel)
            if (feed["url"] != f"{REPOSITORY}/releases/download/v{version}/{name}"
                    or feed["key_id"] != macs[feed["architecture"]]["signature"]["key_id"]):
                raise ValueError("Appcast muss zur gleichen Version, Architektur, zum Kanal und Schlüssel gehören.")
    return manifest


def stamp_plist(path, root=ROOT):
    version, build = check_versions(root)
    with path.open("rb") as stream:
        info = plistlib.load(stream)
    info["CFBundleShortVersionString"] = version.split("-")[0]
    info["CFBundleVersion"] = str(build)
    with path.open("wb") as stream:
        plistlib.dump(info, stream, sort_keys=False)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("version")
    versions = commands.add_parser("check-versions")
    versions.add_argument("--tag")
    validate = commands.add_parser("validate")
    validate.add_argument("manifest", type=Path)
    validate.add_argument("--for-build", action="store_true", help="Prüft gegen aktuelle Cargo-Version/Buildnummer.")
    validate.add_argument("--channel", choices=("stable", "beta"))
    validate.add_argument("--tag")
    plist = commands.add_parser("stamp-plist")
    plist.add_argument("path", type=Path)
    args = parser.parse_args(argv)
    try:
        if args.command == "version":
            print(version_info()[0])
        elif args.command == "check-versions":
            version, build = check_versions(tag=args.tag)
            print(f"PASS: Version {version}, Build {build}.")
        elif args.command == "stamp-plist":
            stamp_plist(args.path)
        else:
            manifest = read_manifest(args.manifest)
            version, build = check_versions(tag=args.tag) if args.for_build else (None, None)
            validate_manifest(manifest, version, build, args.channel)
            if args.tag is not None and args.tag != f"v{manifest['version']}":
                raise ValueError("Release-Tag stimmt nicht mit den Metadaten überein.")
            print("PASS: Release-Metadaten und Versionsbezüge.")
    except (ValueError, OSError, KeyError, TypeError, RecursionError, plistlib.InvalidFileException) as error:
        # Filenames and JSON parser excerpts can contain sensitive input; only fixed validation errors are printed.
        detail = str(error) if type(error) is ValueError else "Datei oder Datenformat kann nicht gelesen werden."
        print(f"FAIL: {detail}", file=sys.stderr)
        return 2
    except ImportError:
        print("FAIL: Für Metadatenprüfung zuerst scripts/requirements-release.txt installieren.", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
