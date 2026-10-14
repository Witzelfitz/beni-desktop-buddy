#!/usr/bin/env python3
"""Derive signed, unpublished Sparkle feeds from authenticated release candidates."""
import argparse
import base64
import importlib.util
from pathlib import Path
import re
import sys
import tempfile
import xml.etree.ElementTree as ET

from release import ROOT

spec = importlib.util.spec_from_file_location("beni_sparkle_release", ROOT / "scripts/local-release.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)

SPARKLE = "http://www.andymatuschak.org/xml-namespaces/sparkle"
MAX_FEED_BYTES = 256 * 1024
ET.register_namespace("sparkle", SPARKLE)
# Sparkle 2.10.0 common_cli/Signing.swift. Sign the exact XML bytes, then append
# this block; it is not part of the signed content. Never reserialize signed XML.
SIGNING_BLOCK = re.compile(
    rb"<!-- sparkle-signatures:\nedSignature: ([A-Za-z0-9+/]{86}==)\nlength: ([1-9][0-9]{0,5})\n-->\n\Z"
)


def feed_content(manifest, artifact, package_signature):
    rss = ET.Element("rss", {"version": "2.0"})
    channel = ET.SubElement(rss, "channel")
    ET.SubElement(channel, "title").text = "Beni Buddy"
    ET.SubElement(channel, "link").text = manifest["release_notes_url"]
    ET.SubElement(channel, "description").text = "Updates für Beni Buddy."
    item = ET.SubElement(channel, "item")
    ET.SubElement(item, "title").text = f"Beni Buddy {manifest['version']}"
    ET.SubElement(item, f"{{{SPARKLE}}}version").text = str(manifest["build_number"])
    ET.SubElement(item, f"{{{SPARKLE}}}shortVersionString").text = manifest["version"]
    ET.SubElement(item, f"{{{SPARKLE}}}minimumSystemVersion").text = artifact["requirements"]["minimum_os_version"]
    if manifest["channel"] == "beta":
        ET.SubElement(item, f"{{{SPARKLE}}}channel").text = "beta"
    # No unsigned external HTML release notes. All presentation text is signed.
    ET.SubElement(item, "description").text = artifact["requirements"]["description"]
    ET.SubElement(item, "enclosure", {
        "url": artifact["url"], "length": str(artifact["size_bytes"]),
        "type": "application/octet-stream", f"{{{SPARKLE}}}edSignature": package_signature,
    })
    ET.indent(rss, space="  ")
    return ET.tostring(rss, encoding="utf-8", xml_declaration=True) + b"\n"


def candidate_feeds(candidate, trusted_path):
    # Authenticate the manifest BEFORE interpreting URLs/versions, then verify
    # every package and the signed provenance using the existing release policy.
    manifest = release.verify_candidate(candidate, trusted_path)
    _, trusted = release.public_key(trusted_path)
    feeds = {}
    for artifact in manifest["artifacts"]:
        if artifact["platform"] != "macos":
            continue
        package = candidate / artifact["url"].rsplit("/", 1)[1]
        signature = Path(str(package) + ".sig").read_bytes().strip()
        decoded = base64.b64decode(signature, validate=True)
        # Verify the exact signature used in the enclosure, rather than trust a
        # sidecar read after an earlier check. No new signing key is introduced.
        trusted.verify(decoded, package.read_bytes())
        filename = f"appcast-macos-{artifact['architecture']}-{manifest['channel']}.xml"
        feeds[filename] = feed_content(manifest, artifact, signature.decode("ascii"))
    if not feeds:
        raise release.ReleaseError("Kandidat enthält kein macOS-Ziel für Sparkle.")
    return feeds


def sign_feed(content, key):
    if not 0 < len(content) < MAX_FEED_BYTES - 256:
        raise release.ReleaseError("Appcast überschreitet die Grössengrenze.")
    signature = base64.b64encode(key.sign(content))
    return content + b"<!-- sparkle-signatures:\nedSignature: " + signature + b"\nlength: " + str(len(content)).encode() + b"\n-->\n"


def verify_feed(data, trusted):
    if not 0 < len(data) <= MAX_FEED_BYTES:
        raise release.ReleaseError("Appcast überschreitet die Grössengrenze.")
    block = SIGNING_BLOCK.search(data)
    if block is None:
        raise release.ReleaseError("Appcast hat keinen gültigen Sparkle-Signaturblock.")
    content = data[:block.start()]
    if len(content) != int(block[2]):
        raise release.ReleaseError("Signierte Appcast-Länge stimmt nicht.")
    trusted.verify(base64.b64decode(block[1], validate=True), content)
    # No XML parsing before successful authentication. Canonical content is
    # compared separately against the already authenticated release candidate.
    return content


def verify_feeds(candidate, output, trusted_path):
    expected = candidate_feeds(candidate, trusted_path)
    _, trusted = release.public_key(trusted_path)
    if {path.name for path in output.iterdir()} != set(expected):
        raise release.ReleaseError("Appcast-Ziele fehlen oder enthalten zusätzliche Dateien.")
    for filename, content in expected.items():
        path = output / filename
        if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_FEED_BYTES:
            raise release.ReleaseError("Appcast muss eine begrenzte reguläre Datei sein.")
        if verify_feed(path.read_bytes(), trusted) != content:
            raise release.ReleaseError("Appcast stimmt nicht mit dem authentifizierten Kandidaten überein.")


def prepare(args):
    candidate = args.candidate.resolve()
    if args.output.exists() or args.output.is_symlink():
        raise release.ReleaseError("Appcast-Ziel existiert bereits; nichts überschrieben.")
    output = args.output.absolute()
    if output.resolve().is_relative_to(candidate):
        raise release.ReleaseError("Appcasts müssen ausserhalb des unveränderten Kandidaten liegen.")
    feeds = candidate_feeds(candidate, args.public_key)
    key_id, trusted = release.public_key(args.public_key)
    config = release.json_bytes((ROOT / "dist/release-signing.json").read_bytes(), 4096)
    if config.get("key_id") != key_id:
        raise release.ReleaseError("Lokale Schlüsselzuordnung stimmt nicht mit dem Vertrauensanker überein.")
    # All input and destination checks happen before accessing 1Password.
    key = release.signing_key(config["item_id"], trusted)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="beni-appcasts-", dir=output.parent) as folder:
        staging = Path(folder)
        for filename, content in feeds.items():
            (staging / filename).write_bytes(sign_feed(content, key))
        verify_feeds(candidate, staging, args.public_key)
        release.install_candidate(staging, output)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("prepare", "verify"):
        command = commands.add_parser(name)
        command.add_argument("--candidate", type=Path, required=True)
        command.add_argument("--output", type=Path, required=True)
        command.add_argument("--public-key", type=Path, default=ROOT / "release/keys/beni-update-v1.json")
    args = parser.parse_args(argv)
    try:
        if args.command == "prepare":
            prepare(args)
        else:
            verify_feeds(args.candidate, args.output, args.public_key)
    except Exception:
        # Includes crypto/parser/tool errors; do not print untrusted input or
        # private key material returned by a signing tool.
        print("FAIL: Appcast-Eingaben, Signaturen oder Ausgabe ungültig; nichts veröffentlicht.", file=sys.stderr)
        return 2
    print("PASS: Sparkle-Appcasts geprüft; keine Veröffentlichung und kein installierter Updater.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
