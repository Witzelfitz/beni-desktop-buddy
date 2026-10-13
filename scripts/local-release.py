#!/usr/bin/env python3
"""Prepare locally notarized macOS release candidates; never publish a release."""
import argparse
import base64
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import platform
import re
import stat
import subprocess
import sys
import tempfile
import zipfile

from release import ROOT, REPOSITORY, check_versions, read_manifest, unique_object, validate_manifest

spec = importlib.util.spec_from_file_location("beni_local_package", ROOT / "scripts/package-macos.py")
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)
REPO = "Witzelfitz/beni-desktop-buddy"
MAX_PACKAGE_BYTES = 128 * 1024 * 1024
BUILD_INPUTS = ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "apps", "crates", "vendor", "assets",
                "integrations", "LICENSE.md", "LICENSE-MIT", "docs/INTEGRATIONS.md")


class ReleaseError(Exception):
    pass


def command(args, *, payload=None, timeout=120):
    try:
        result = subprocess.run(args, cwd=ROOT, input=payload, stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ReleaseError("Release-Werkzeug nicht verfügbar oder Zeitlimit erreicht.") from error
    if result.returncode:
        raise ReleaseError("Release-Werkzeug fehlgeschlagen; keine Rohdaten oder Zugangsdaten ausgegeben.")
    return result.stdout


def json_bytes(data, maximum=256 * 1024):
    if len(data) > maximum:
        raise ReleaseError("Release-Eingabe überschreitet die Grössengrenze.")
    return json.loads(data, object_pairs_hook=unique_object)


def api(endpoint):
    return json_bytes(command(["gh", "api", f"repos/{REPO}/{endpoint}"], timeout=60), maximum=1024 * 1024)


def check_catalog(version, build, trusted):
    releases = api("releases?per_page=100")
    if not isinstance(releases, list) or len(releases) >= 100:
        raise ReleaseError("Release-Katalog kann nicht vollständig geprüft werden.")
    for item in releases:
        if item.get("tag_name") == f"v{version}":
            raise ReleaseError("Release-Version ist bereits veröffentlicht oder reserviert.")
        assets = {a.get("name"): a.get("id") for a in item.get("assets", [])}
        if not all(type(assets.get(name)) is int for name in ("release.json", "release.json.sig")):
            raise ReleaseError("Vorhandener Release hat keine prüfbaren authentifizierten Build-Metadaten.")
        def asset(name):
            return command(["gh", "api", f"repos/{REPO}/releases/assets/{assets[name]}", "-H", "Accept: application/octet-stream"], timeout=60)
        data, signature = asset("release.json"), asset("release.json.sig")
        if len(data) > 256 * 1024 or len(signature) > 128:
            raise ReleaseError("Historischer Release überschreitet die Grössengrenze.")
        from cryptography.exceptions import InvalidSignature
        try:
            trusted.verify(base64.b64decode(signature.strip(), validate=True), data)
        except InvalidSignature as error:
            raise ReleaseError("Historische Release-Signatur ist ungültig; Schlüsselwechsel zuerst ausdrücklich vorbereiten.") from error
        manifest = json_bytes(data)
        validate_manifest(manifest)
        if item.get("tag_name") != f"v{manifest['version']}" or build <= manifest["build_number"]:
            raise ReleaseError("Release-Tag oder global steigende Buildnummer stimmt nicht.")


def build_fingerprint(commit):
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ReleaseError("Build-Commit ist ungültig.")
    records = command(["git", "ls-tree", "-rz", commit, "--", *BUILD_INPUTS])
    if not records:
        raise ReleaseError("Produkteingaben fehlen.")
    return hashlib.sha256(records).hexdigest()


def reviewed_run(run, architecture, jobs, current_commit):
    if (run.get("status") != "completed" or run.get("conclusion") != "success"
            or run.get("path") != ".github/workflows/macos-installers.yml"
            or run.get("event") != "workflow_dispatch" or run.get("head_branch") != "main"
            or run.get("repository", {}).get("full_name") != REPO
            or run.get("head_repository", {}).get("full_name") != REPO):
        raise ReleaseError("Nur erfolgreiche Installer-Läufe des eigenen main-Workflows übernehmen.")
    sha = run.get("head_sha", "")
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ReleaseError("Installer-Commit ist ungültig.")
    command(["git", "merge-base", "--is-ancestor", sha, "origin/main"])
    if build_fingerprint(sha) != build_fingerprint(current_commit):
        raise ReleaseError("Native Installer und lokaler Stand haben unterschiedliche Produkteingaben.")
    runner = "macos-15" if architecture == "aarch64" else "macos-15-intel"
    selected = [j for j in jobs if j.get("name") == f"installer ({architecture}, {runner})"]
    if len(selected) != 1 or selected[0].get("conclusion") != "success":
        raise ReleaseError("Die gewählte Architektur hat keine erfolgreiche native Abnahme.")
    steps = selected[0].get("steps", [])
    for name in ("Build development installer", "Native installation and window smoke test"):
        if steps and not any(s.get("name") == name and s.get("conclusion") == "success" for s in steps):
            raise ReleaseError("Nativer Build oder Fenster-Starttest ist nicht bestanden.")
    return sha


def extract_artifact(data, artifact, architecture, directory, source_commit):
    if artifact.get("expired") is not False or artifact.get("name") != f"macos-{architecture}-development":
        raise ReleaseError("Installer-Artefakt fehlt, ist verfallen oder hat das falsche Ziel.")
    digest = "sha256:" + hashlib.sha256(data).hexdigest()
    if artifact.get("digest") != digest or len(data) > MAX_PACKAGE_BYTES:
        raise ReleaseError("GitHub-Artefaktprüfsumme oder Archivgrösse stimmt nicht.")
    version, build = check_versions()
    name = package.installer_filename(version, architecture, "development")
    expected = {name, name + ".sha256", name + ".build.json", name + ".native.json"}
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        members = archive.infolist()
        if len(members) != 4 or {m.filename for m in members} != expected:
            raise ReleaseError("Installer-Archiv muss genau Paket, Prüfsumme, Build- und native Abnahmebelege enthalten.")
        for member in members:
            mode = member.external_attr >> 16
            if (member.is_dir() or stat.S_IFMT(mode) not in (0, stat.S_IFREG) or member.flag_bits & 1
                    or member.file_size > (MAX_PACKAGE_BYTES if member.filename == name else 8192)):
                raise ReleaseError("Unzulässiger Archivinhalt.")
        for member in members:
            destination = directory / member.filename
            with destination.open("xb") as stream:
                stream.write(archive.read(member))
    image = directory / name
    receipt = json_bytes(Path(str(image) + ".build.json").read_bytes(), 8192)
    required = {"type": "macos-build-receipt", "version": version, "build_number": build,
                "architecture": architecture, "minimum_os_version": package.MINIMUM_MACOS,
                "filename": name, "signing": "ad_hoc", "notarized": False}
    if any(type(receipt.get(k)) is not type(v) or receipt.get(k) != v for k, v in required.items()):
        raise ReleaseError("Native Build-Belege stimmen nicht mit dem Produkt und Ziel überein.")
    digest = hashlib.sha256(image.read_bytes()).hexdigest()
    if (receipt.get("sha256") != digest or type(receipt.get("size_bytes")) is not int
            or image.stat().st_size != receipt["size_bytes"] or image.stat().st_size == 0
            or Path(str(image) + ".sha256").read_text() != f"{digest}  {name}\n"):
        raise ReleaseError("Native Installerbytes stimmen nicht mit dem Build-Beleg überein.")
    native = json_bytes(Path(str(image) + ".native.json").read_bytes(), 8192)
    required = {"type": "macos-native-install-check", "source_commit": source_commit,
                "architecture": architecture, "version": version, "build_number": build,
                "package_sha256": digest, "native_window_passed": True, "temporary_install_removed": True}
    if (any(type(native.get(k)) is not type(v) or native.get(k) != v for k, v in required.items())
            or not re.fullmatch(r"15\.[0-9]+(?:\.[0-9]+)?", native.get("tested_os_version", ""))):
        raise ReleaseError("Native Fenster-/Installationsabnahme fehlt oder passt nicht zu Commit und Paket.")
    return image


def download_installer(run_id, architecture, directory, current_commit):
    if not re.fullmatch(r"[1-9][0-9]{0,19}", str(run_id)):
        raise ReleaseError("Workflow-Laufnummer ist ungültig.")
    run = api(f"actions/runs/{run_id}")
    jobs = api(f"actions/runs/{run_id}/jobs?per_page=100")
    sha = reviewed_run(run, architecture, jobs["jobs"], current_commit)
    listing = api(f"actions/runs/{run_id}/artifacts?per_page=100")
    matches = [a for a in listing["artifacts"] if a.get("name") == f"macos-{architecture}-development"]
    if len(matches) != 1:
        raise ReleaseError("Genau ein natives Entwicklungsartefakt für das gewünschte Ziel erforderlich.")
    artifact = matches[0]
    if type(artifact.get("id")) is not int or not 0 < artifact.get("size_in_bytes", 0) <= MAX_PACKAGE_BYTES:
        raise ReleaseError("Artefaktkennung oder Grössenangabe ist ungültig.")
    if artifact.get("workflow_run", {}).get("head_sha") != sha:
        raise ReleaseError("Artefakt gehört nicht zum geprüften Commit.")
    data = command(["gh", "api", f"repos/{REPO}/actions/artifacts/{artifact['id']}/zip"], timeout=180)
    return extract_artifact(data, artifact, architecture, directory, sha), sha


def build_from_installer(image, architecture, output, identity, profile):
    version, _ = check_versions()
    with tempfile.TemporaryDirectory(prefix="beni-native-resign-") as folder:
        directory = Path(folder)
        mount = directory / "volume"
        mount.mkdir()
        attached = False
        try:
            package.run(["hdiutil", "verify", str(image)], "Nativen Installer erneut prüfen")
            package.run(["hdiutil", "attach", "-readonly", "-nobrowse", "-plist", "-mountpoint", str(mount), str(image)], "Nativen Installer öffnen")
            attached = True
            source = mount / "Beni Buddy.app"
            if any(path.is_symlink() for path in (source, source / "Contents", source / "Contents/MacOS", source / "Contents/Info.plist")):
                raise ReleaseError("Native Bundle-Pfade dürfen keine Symlinks sein.")
            package.verify_signature(source)
            info = plistlib_read(source / "Contents/Info.plist")
            _, build = check_versions()
            if (info.get("CFBundleIdentifier") != "io.witzelfitz.beni-buddy"
                    or info.get("CFBundleShortVersionString") != version.split("-")[0]
                    or info.get("CFBundleVersion") != str(build)):
                raise ReleaseError("Native Bundle-Version stimmt nicht mit dem Release überein.")
            binaries = tuple(source / "Contents/MacOS" / name for name in ("beni-desktop-buddy", "beni-buddy-cli"))
            for binary in binaries:
                if binary.is_symlink() or not binary.is_file():
                    raise ReleaseError("Native Executables müssen reguläre Dateien sein.")
                package.verify_signature(binary)
                package.check_binary(binary, architecture)
            app = directory / "Beni Buddy.app"
            package.make_bundle(app, architecture, identity, binaries=binaries)
            package.notarize_bundle(app, directory, profile)
            final = output / package.installer_filename(version, architecture, "release")
            package.make_dmg(app, directory, final, identity)
            package.notarize(final, profile)
            package.staple_and_assess(final)
            return final
        finally:
            if attached:
                package.run(["hdiutil", "detach", str(mount)], "Natives Testvolume auswerfen")


def plistlib_read(path):
    import plistlib
    with path.open("rb") as stream:
        return plistlib.load(stream)


def public_key(path):
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    data = json_bytes(path.read_bytes(), 4096)
    if (set(data) != {"algorithm", "key_id", "public_key_base64"} or data["algorithm"] != "ed25519"
            or not re.fullmatch(r"beni-update-v[1-9][0-9]*", data["key_id"])):
        raise ReleaseError("Öffentlicher Release-Schlüssel ist ungültig.")
    raw = base64.b64decode(data["public_key_base64"], validate=True)
    return data["key_id"], Ed25519PublicKey.from_public_bytes(raw)


def signing_key(item, trusted):
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    if not re.fullmatch(r"[a-z0-9]{26}", item):
        raise ReleaseError("1Password-Eintrags-ID ist ungültig.")
    raw = command(["op", "item", "get", item, "--fields", "password", "--reveal"], timeout=60).strip()
    key = Ed25519PrivateKey.from_private_bytes(base64.b64decode(raw, validate=True))
    if key.public_key().public_bytes_raw() != trusted.public_bytes_raw():
        raise ReleaseError("Privater Schlüssel passt nicht zum bereits vertrauenswürdigen öffentlichen Schlüssel.")
    return key


def initialize_key(key_id, destination):
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    if not re.fullmatch(r"beni-update-v[1-9][0-9]*", key_id) or destination.exists() or destination.is_symlink():
        raise ReleaseError("Neue gültige Schlüsselkennung und noch nicht vorhandenen öffentlichen Zielpfad wählen.")
    config = ROOT / "dist/release-signing.json"
    if config.exists() or config.is_symlink():
        raise ReleaseError("Lokale Release-Schlüsselzuordnung existiert bereits; nichts überschrieben.")
    key = Ed25519PrivateKey.generate()
    private = base64.b64encode(key.private_bytes_raw()).decode("ascii")
    template = {"category": "PASSWORD", "title": f"Beni Buddy Release Signing ({key_id})",
                "fields": [{"id": "password", "type": "CONCEALED", "purpose": "PASSWORD", "label": "password", "value": private}]}
    created = json_bytes(command(["op", "item", "create", "-", "--format", "json"], payload=json.dumps(template).encode(), timeout=90))
    item = created.get("id", "")
    signing_key(item, key.public_key())
    data = {"algorithm": "ed25519", "key_id": key_id, "public_key_base64": base64.b64encode(key.public_key().public_bytes_raw()).decode("ascii")}
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("x") as stream:
        json.dump(data, stream, indent=2)
        stream.write("\n")
    config.parent.mkdir(exist_ok=True)
    with config.open("x") as stream:
        json.dump({"key_id": key_id, "item_id": item}, stream, indent=2)
        stream.write("\n")
    print(f"PASS: {key_id}; privater Schlüssel in 1Password, nur öffentlicher Schlüssel im Repo.")


def sign_file(path, key):
    if path.stat().st_size > MAX_PACKAGE_BYTES:
        raise ReleaseError("Paket überschreitet das lokale Signierlimit von 128 MiB.")
    signature = base64.b64encode(key.sign(path.read_bytes())).decode("ascii") + "\n"
    Path(str(path) + ".sig").write_text(signature, encoding="ascii")


def verify_file(path, key, limit=MAX_PACKAGE_BYTES):
    from cryptography.exceptions import InvalidSignature
    signature_path = Path(str(path) + ".sig")
    if path.stat().st_size > limit or signature_path.stat().st_size > 128:
        raise ReleaseError("Signierte Eingabe überschreitet die Grössengrenze.")
    signature = base64.b64decode(signature_path.read_bytes().strip(), validate=True)
    try:
        key.verify(signature, path.read_bytes())
    except InvalidSignature as error:
        raise ReleaseError("Ed25519-Signatur ist ungültig.") from error


def verify_candidate(directory, trusted_path):
    key_id, key = public_key(trusted_path)
    manifest_path = directory / "release.json"
    if any(p.is_symlink() or not p.is_file() for p in directory.iterdir()):
        raise ReleaseError("Release-Kandidat darf nur reguläre Dateien enthalten.")
    # Authenticate exact bytes before interpreting version, URLs or artifact choices.
    verify_file(manifest_path, key, limit=256 * 1024)
    manifest = read_manifest(manifest_path)
    validate_manifest(manifest)
    candidate = directory / "candidate.json"
    verify_file(candidate, key, limit=16384)
    provenance = json_bytes(candidate.read_bytes(), 16384)
    if (provenance.get("type") != "unpublished-release-candidate"
            or not re.fullmatch(r"[0-9a-f]{40}", provenance.get("source_commit", ""))
            or not re.fullmatch(r"[0-9a-f]{64}", provenance.get("product_inputs_sha256", ""))
            or {p.get("architecture") for p in provenance.get("native_builds", [])} != {a["architecture"] for a in manifest["artifacts"]}):
        raise ReleaseError("Signierter Kandidatenbeleg stimmt nicht mit den Zielpaketen überein.")
    expected = {"release.json", "release.json.sig", "candidate.json", "candidate.json.sig"}
    for artifact in manifest["artifacts"]:
        if artifact["signature"]["key_id"] != key_id:
            raise ReleaseError("Manifest verwendet keinen bereits vertrauenswürdigen Schlüssel.")
        filename = artifact["url"].rsplit("/", 1)[1]
        path = directory / filename
        if path.is_symlink() or not path.is_file():
            raise ReleaseError("Release-Paket fehlt oder ist kein reguläres File.")
        verify_file(path, key)
        if path.stat().st_size != artifact["size_bytes"] or hashlib.sha256(path.read_bytes()).hexdigest() != artifact["sha256"]:
            raise ReleaseError("Finale Paketbytes entsprechen nicht dem authentifizierten Manifest.")
        if (directory / (filename + ".sha256")).read_text() != f"{artifact['sha256']}  {filename}\n":
            raise ReleaseError("Release-Prüfsummendatei stimmt nicht überein.")
        expected.update((filename, filename + ".sig", filename + ".sha256"))
    if {p.name for p in directory.iterdir()} != expected or any(p.is_symlink() or not p.is_file() for p in directory.iterdir()):
        raise ReleaseError("Kandidat enthält fehlende, zusätzliche oder verlinkte Dateien.")
    return manifest


def install_candidate(staging, output):
    # Exclusive directory creation and no-overwrite links protect concurrent preparation.
    output.mkdir()
    copied = []
    try:
        for source in sorted(staging.iterdir()):
            target = output / source.name
            os.link(source, target)
            copied.append(target)
    except OSError:
        for path in copied:
            path.unlink()
        output.rmdir()
        raise


def prepare(args):
    if platform.system() != "Darwin":
        raise ReleaseError("Lokale Release-Signierung benötigt macOS.")
    if len(set(args.architecture)) != len(args.architecture) or len(set(args.run)) != len(args.run):
        raise ReleaseError("Ziele und Workflow-Laufnummern dürfen nicht doppelt vorkommen.")
    if any(not re.fullmatch(r"[1-9][0-9]{0,19}", run_id) for run_id in args.run):
        raise ReleaseError("Workflow-Laufnummer ist ungültig.")
    if not re.fullmatch(r"[0-9]+\.[0-9]+(?:\.[0-9]+)?", args.minimum_os):
        raise ReleaseError("Freigegebene macOS-Mindestversion ist ungültig.")
    minimum = tuple(int(n) for n in args.minimum_os.split(".")) + (0,)
    if minimum[:2] < (15, 0):
        raise ReleaseError("Erster Release-Kandidat benötigt macOS 15.0 oder neuer; ältere Systeme sind nicht abgenommen.")
    version, build = check_versions()
    output = args.output.resolve()
    if output.exists() or output.is_symlink():
        raise ReleaseError("Kandidatenverzeichnis existiert bereits; nichts überschrieben.")
    if command(["git", "status", "--porcelain", "--", *BUILD_INPUTS]).strip():
        raise ReleaseError("Produkteingaben haben nicht eingecheckte Änderungen.")
    commit = command(["git", "rev-parse", "HEAD"]).decode().strip()
    key_id, trusted = public_key(args.public_key)
    check_catalog(version, build, trusted)
    config = json_bytes((ROOT / "dist/release-signing.json").read_bytes(), 4096)
    if config.get("key_id") != key_id:
        raise ReleaseError("Lokale Schlüsselzuordnung passt nicht zum öffentlichen Schlüssel.")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="beni-release-candidate-", dir=output.parent) as folder:
        staging = Path(folder)
        final = staging / "candidate"
        final.mkdir()
        artifacts, provenance, inputs = [], [], []
        for architecture in args.architecture:
            picked = []
            for run_id in args.run:
                listing = api(f"actions/runs/{run_id}/artifacts?per_page=100")
                if any(a.get("name") == f"macos-{architecture}-development" for a in listing["artifacts"]):
                    picked.append(run_id)
            if len(picked) != 1:
                raise ReleaseError("Jedes gewünschte Ziel benötigt genau einen vollständig geprüften nativen Workflow-Lauf.")
            native = staging / architecture
            native.mkdir()
            image, sha = download_installer(picked[0], architecture, native, commit)
            inputs.append((architecture, picked[0], image, sha))
        # Validate all native inputs before touching signing keys or submitting anything to Apple.
        key = signing_key(config["item_id"], trusted)
        identity = package.preflight("release", args.architecture[0], args.notary_profile)
        for architecture, run_id, image, sha in inputs:
            print(f"Nativ geprüftes Paket lokal signieren: {architecture}.", flush=True)
            path = build_from_installer(image, architecture, final, identity, args.notary_profile)
            sign_file(path, key)
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            (final / (path.name + ".sha256")).write_text(f"{digest}  {path.name}\n", encoding="ascii")
            url = f"{REPOSITORY}/releases/download/v{version}/{path.name}"
            artifacts.append({"platform": "macos", "architecture": architecture, "format": "dmg",
                              "requirements": {"description": f"macOS {args.minimum_os} oder neuer; {'Apple Silicon' if architecture == 'aarch64' else 'Intel'}.", "minimum_os_version": args.minimum_os},
                              "url": url, "size_bytes": path.stat().st_size, "sha256": digest,
                              "signature": {"algorithm": "ed25519", "key_id": key_id, "url": url + ".sig"}})
            provenance.append({"architecture": architecture, "workflow_run": int(run_id), "source_commit": sha})
        manifest = {"schema_version": 1, "product": "io.witzelfitz.beni-buddy", "version": version,
                    "build_number": build, "channel": "beta" if "-beta." in version else "stable",
                    "release_notes_url": f"{REPOSITORY}/releases/tag/v{version}", "artifacts": artifacts}
        validate_manifest(manifest, version, build)
        metadata = final / "release.json"
        metadata.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        sign_file(metadata, key)
        (final / "candidate.json").write_text(json.dumps({"type": "unpublished-release-candidate", "source_commit": commit,
                                                         "product_inputs_sha256": build_fingerprint(commit), "native_builds": provenance}, indent=2) + "\n")
        sign_file(final / "candidate.json", key)
        verify_candidate(final, args.public_key)
        install_candidate(final, output)
    print(f"PASS: lokaler Release-Kandidat {output}; keine Veröffentlichung.")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    init = commands.add_parser("init-key")
    init.add_argument("--key-id", default="beni-update-v1")
    init.add_argument("--public-key", type=Path, default=ROOT / "release/keys/beni-update-v1.json")
    prepare_parser = commands.add_parser("prepare")
    prepare_parser.add_argument("--run", action="append", required=True)
    prepare_parser.add_argument("--architecture", nargs="+", choices=tuple(package.ARCHITECTURES), default=list(package.ARCHITECTURES))
    prepare_parser.add_argument("--minimum-os", required=True)
    prepare_parser.add_argument("--notary-profile", default=os.environ.get("BENI_NOTARY_PROFILE", "beni-notary"))
    prepare_parser.add_argument("--public-key", type=Path, default=ROOT / "release/keys/beni-update-v1.json")
    prepare_parser.add_argument("--output", type=Path, default=ROOT / "dist/release-candidate")
    verify = commands.add_parser("verify")
    verify.add_argument("directory", type=Path)
    verify.add_argument("--public-key", type=Path, default=ROOT / "release/keys/beni-update-v1.json")
    args = parser.parse_args(argv)
    try:
        if args.command == "init-key":
            initialize_key(args.key_id, args.public_key)
        elif args.command == "prepare":
            prepare(args)
        else:
            verify_candidate(args.directory, args.public_key)
            print("PASS: Manifest und sämtliche Paketbytes kryptografisch geprüft.")
    except ImportError:
        print("FAIL: Zuerst scripts/requirements-release.txt in einer virtuellen Umgebung installieren.", file=sys.stderr)
        return 2
    except (ReleaseError, package.PackagingError, ValueError, OSError, KeyError, TypeError, RecursionError, zipfile.BadZipFile):
        print("FAIL: Release-Vorbereitung abgelehnt; Quelle, Schlüssel, Paket und Abnahmen prüfen. Keine Veröffentlichung.", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
