#!/usr/bin/env python3
"""Offline appcast regressions; optional native Sparkle compatibility probe."""
import argparse
import base64
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


sparkle = load("beni_sparkle_test", "sparkle.py")
fixtures = load("beni_sparkle_fixtures", "check-local-release.py")
NATIVE_TOOL = None


class AppcastTests(unittest.TestCase):
    def test_shared_python_node_contract_vectors(self):
        fixture = json.loads((ROOT / "release/examples/sparkle-contract.json").read_bytes())
        trusted = Ed25519PublicKey.from_public_bytes(base64.b64decode(fixture["keys"]["public_key_base64"]))
        for vector in fixture["vectors"]:
            data = base64.b64decode(vector["manifest_base64"])
            trusted.verify(base64.b64decode(vector["manifest_signature"]), data)
            manifest = sparkle.release.validate_manifest(json.loads(data))
            trusted.verify(base64.b64decode(vector["package_signature"]), base64.b64decode(vector["package_base64"]))
            feed = base64.b64decode(vector["feed_base64"])
            self.assertEqual(sparkle.verify_feed(feed, trusted), sparkle.feed_content(manifest, manifest["artifacts"][0], vector["package_signature"]))
            self.assertEqual(hashlib.sha256(feed).hexdigest(), manifest["macos_update_feeds"][0]["sha256"])

    def add_release_feeds(self):
        path = self.candidate / "release.json"
        manifest = json.loads(path.read_bytes())
        sparkle.release.write_candidate_feeds(self.candidate, manifest, self.key)
        path.write_text(json.dumps(manifest))
        sparkle.release.sign_file(path, self.key)
        return manifest

    def test_integrated_release_feeds_verify_and_export_without_private_provenance(self):
        before = {p.name: p.read_bytes() for p in self.candidate.iterdir()}
        manifest = self.add_release_feeds()
        self.assertEqual(len(manifest["macos_update_feeds"]), 1)
        self.assertEqual(sparkle.release.verify_candidate(self.candidate, self.trusted), manifest)
        output = self.root / "public"
        with patch.object(sparkle.release, "signing_key") as secret:
            sparkle.release.export_release(self.candidate, output, self.trusted)
            secret.assert_not_called()
        self.assertFalse((output / "candidate.json").exists())
        self.assertFalse((output / "candidate.json.sig").exists())
        self.assertEqual({p.name for p in output.iterdir()}, {p.name for p in self.candidate.iterdir()} - {"candidate.json", "candidate.json.sig"})
        for name, data in before.items():
            if name not in ("release.json", "release.json.sig", "candidate.json", "candidate.json.sig"):
                self.assertEqual((output / name).read_bytes(), data)
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.release.export_release(self.candidate, output, self.trusted)

    def test_missing_tampered_and_symlink_integrated_feeds_are_rejected(self):
        manifest = self.add_release_feeds()
        path = self.candidate / manifest["macos_update_feeds"][0]["url"].rsplit("/", 1)[1]
        original = path.read_bytes()
        path.unlink()
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.release.verify_candidate(self.candidate, self.trusted)
        target = self.root / "external.xml"; target.write_bytes(original)
        path.symlink_to(target)
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.release.verify_candidate(self.candidate, self.trusted)
        path.unlink(); path.write_bytes(original + b"tampered")
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.release.export_release(self.candidate, self.root / "rejected", self.trusted)
        self.assertFalse((self.root / "rejected").exists())

    def test_resigned_foreign_feed_cannot_announce_another_build(self):
        manifest = self.add_release_feeds()
        feed = manifest["macos_update_feeds"][0]
        path = self.candidate / feed["url"].rsplit("/", 1)[1]
        original = path.read_bytes()
        content = sparkle.verify_feed(original, self.key.public_key())
        for data in (content, sparkle.sign_feed(content.replace(b"<sparkle:version>1", b"<sparkle:version>99"), self.key),
                     sparkle.sign_feed(content, Ed25519PrivateKey.generate())):
            path.write_bytes(data)
            feed.update(size_bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
            metadata = self.candidate / "release.json"
            metadata.write_text(json.dumps(manifest)); sparkle.release.sign_file(metadata, self.key)
            with self.assertRaises(sparkle.release.ReleaseError):
                sparkle.release.verify_candidate(self.candidate, self.trusted)

    def test_export_verifies_copied_bytes_and_rejects_nested_destination(self):
        self.add_release_feeds()
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.release.export_release(self.candidate, self.candidate / "nested", self.trusted)
        copy = sparkle.release.shutil.copy2
        def corrupt_copy(source, destination):
            result = copy(source, destination)
            if source.suffix == ".xml":
                destination.write_bytes(destination.read_bytes() + b"tampered copy")
            return result
        with patch.object(sparkle.release.shutil, "copy2", side_effect=corrupt_copy), self.assertRaises(sparkle.release.ReleaseError):
            sparkle.release.export_release(self.candidate, self.root / "rejected", self.trusted)
        self.assertFalse((self.root / "rejected").exists())
        sparkle.release.verify_candidate(self.candidate, self.trusted)

    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        self.root = Path(folder.name)
        self.candidate = self.root / "candidate"
        self.candidate.mkdir()
        self.trusted, self.key, self.package = fixtures.candidate(self.candidate)
        self.output = self.root / "feeds"
        self.output.mkdir()
        self.generate()

    def generate(self):
        self.content = sparkle.candidate_feeds(self.candidate, self.trusted)
        for filename, content in self.content.items():
            (self.output / filename).write_bytes(sparkle.sign_feed(content, self.key))

    def rewrite_manifest(self, change):
        path = self.candidate / "release.json"
        manifest = json.loads(path.read_bytes())
        change(manifest)
        path.write_text(json.dumps(manifest))
        sparkle.release.sign_file(path, self.key)

    def test_release_fields_and_package_signature_are_preserved(self):
        sparkle.verify_feeds(self.candidate, self.output, self.trusted)
        content = next(iter(self.content.values()))
        item = ET.fromstring(content).find("channel/item")
        self.assertEqual(item.findtext(f"{{{sparkle.SPARKLE}}}version"), "1")
        self.assertEqual(item.findtext(f"{{{sparkle.SPARKLE}}}shortVersionString"), "0.1.0")
        self.assertEqual(item.findtext(f"{{{sparkle.SPARKLE}}}minimumSystemVersion"), "11.0")
        enclosure = item.find("enclosure")
        self.assertEqual(enclosure.attrib[f"{{{sparkle.SPARKLE}}}edSignature"],
                         Path(str(self.package) + ".sig").read_text().strip())
        self.assertEqual(int(enclosure.attrib["length"]), self.package.stat().st_size)
        self.assertFalse(item.findall(f"{{{sparkle.SPARKLE}}}releaseNotesLink"))

    def test_intel_and_arm_are_separate(self):
        manifest_path = self.candidate / "release.json"
        manifest = json.loads(manifest_path.read_bytes())
        intel = json.loads(json.dumps(manifest["artifacts"][0]))
        intel["architecture"] = "x86_64"
        for field in (intel, intel["signature"]):
            field["url"] = field["url"].replace("aarch64", "x86_64")
        image = self.candidate / intel["url"].rsplit("/", 1)[1]
        image.write_bytes(self.package.read_bytes())
        sparkle.release.sign_file(image, self.key)
        Path(str(image) + ".sha256").write_text(f"{intel['sha256']}  {image.name}\n")
        manifest["artifacts"].append(intel)
        manifest_path.write_text(json.dumps(manifest))
        sparkle.release.sign_file(manifest_path, self.key)
        provenance = self.candidate / "candidate.json"
        data = json.loads(provenance.read_bytes())
        data["native_builds"].append({"architecture": "x86_64"})
        provenance.write_text(json.dumps(data))
        sparkle.release.sign_file(provenance, self.key)
        feeds = sparkle.candidate_feeds(self.candidate, self.trusted)
        self.assertEqual(set(feeds), {"appcast-macos-aarch64-stable.xml", "appcast-macos-x86_64-stable.xml"})
        for filename, content in feeds.items():
            self.assertIn(filename.split("-")[2].encode() + b".dmg", content)

    def test_beta_feed_cannot_be_selected_as_stable(self):
        # Rename all version-bound inputs and sign a coherent beta fixture.
        for path in list(self.candidate.iterdir()):
            if "0.1.0" in path.name:
                path.rename(path.with_name(path.name.replace("0.1.0", "0.1.0-beta.1")))
        image = next(self.candidate.glob("*.dmg"))
        checksum = Path(str(image) + ".sha256")
        checksum.write_text(checksum.read_text().replace("0.1.0", "0.1.0-beta.1"))
        manifest_path = self.candidate / "release.json"
        manifest = json.loads(manifest_path.read_text().replace("0.1.0", "0.1.0-beta.1"))
        manifest["channel"] = "beta"
        manifest_path.write_text(json.dumps(manifest))
        sparkle.release.sign_file(manifest_path, self.key)
        feeds = sparkle.candidate_feeds(self.candidate, self.trusted)
        self.assertEqual(set(feeds), {"appcast-macos-aarch64-beta.xml"})
        self.assertIn(b"<sparkle:channel>beta</sparkle:channel>", next(iter(feeds.values())))

    def test_unsigned_or_tampered_feed_is_rejected(self):
        data = next(iter(self.content.values()))
        for broken in (data, sparkle.sign_feed(data, self.key).replace(b"Beni Buddy", b"Evil Buddy", 1)):
            with self.assertRaises((sparkle.release.ReleaseError, InvalidSignature)):
                sparkle.verify_feed(broken, self.key.public_key())

    def test_wrong_key_is_rejected(self):
        data = sparkle.sign_feed(next(iter(self.content.values())), Ed25519PrivateKey.generate())
        with self.assertRaises(InvalidSignature):
            sparkle.verify_feed(data, self.key.public_key())

    def test_length_trailing_data_and_duplicate_blocks_are_rejected(self):
        data = next(iter(self.content.values()))
        signed = sparkle.sign_feed(data, self.key)
        for broken in (signed.replace(f"length: {len(data)}".encode(), b"length: 1"), signed + b"extra", signed + signed[len(data):]):
            with self.assertRaises((sparkle.release.ReleaseError, InvalidSignature)):
                sparkle.verify_feed(broken, self.key.public_key())

    def test_oversized_feed_is_rejected(self):
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.verify_feed(b"x" * (sparkle.MAX_FEED_BYTES + 1), self.key.public_key())

    def test_signed_feed_with_wrong_release_is_rejected(self):
        for original, replacement in ((b"<sparkle:version>1", b"<sparkle:version>2"), (b"aarch64.dmg", b"x86_64.dmg")):
            filename, content = next(iter(self.content.items()))
            (self.output / filename).write_bytes(sparkle.sign_feed(content.replace(original, replacement), self.key))
            with self.assertRaises(sparkle.release.ReleaseError):
                sparkle.verify_feeds(self.candidate, self.output, self.trusted)

    def test_corrupt_manifest_package_and_provenance_are_rejected(self):
        for path in (self.candidate / "release.json", self.package, self.candidate / "candidate.json"):
            original = path.read_bytes()
            path.write_bytes(original + b"x")
            with self.assertRaises(sparkle.release.ReleaseError):
                sparkle.candidate_feeds(self.candidate, self.trusted)
            path.write_bytes(original)

    def test_missing_extra_and_symlink_feeds_are_rejected(self):
        path = self.output / next(iter(self.content))
        original = path.read_bytes()
        path.unlink()
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.verify_feeds(self.candidate, self.output, self.trusted)
        target = self.root / "external.xml"
        target.write_bytes(original)
        path.symlink_to(target)
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.verify_feeds(self.candidate, self.output, self.trusted)
        path.unlink(); path.write_bytes(original)
        (self.output / "extra").write_text("foreign")
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.verify_feeds(self.candidate, self.output, self.trusted)

    def test_invalid_inputs_do_not_access_private_key(self):
        args = argparse.Namespace(candidate=self.candidate, output=self.output, public_key=self.trusted)
        with patch.object(sparkle.release, "signing_key") as secret:
            with self.assertRaises(sparkle.release.ReleaseError):
                sparkle.prepare(args)
            args.output = self.candidate / "nested"
            with self.assertRaises(sparkle.release.ReleaseError):
                sparkle.prepare(args)
            args.output = self.root / "new-output"
            self.package.write_bytes(b"tampered")
            with self.assertRaises(sparkle.release.ReleaseError):
                sparkle.prepare(args)
            secret.assert_not_called()
            self.assertFalse(args.output.exists())

    def test_prepare_preserves_candidate_and_does_not_overwrite(self):
        before = {p.name: p.read_bytes() for p in self.candidate.iterdir()}
        args = argparse.Namespace(candidate=self.candidate, output=self.root / "prepared", public_key=self.trusted)
        with patch.object(sparkle, "ROOT", self.root), patch.object(sparkle.release, "signing_key", return_value=self.key) as secret:
            (self.root / "dist").mkdir()
            (self.root / "dist/release-signing.json").write_text(json.dumps({"key_id": "beni-update-v1", "item_id": "fixture"}))
            sparkle.prepare(args)
            sparkle.verify_feeds(self.candidate, args.output, self.trusted)
            with self.assertRaises(sparkle.release.ReleaseError):
                sparkle.prepare(args)
            secret.assert_called_once()
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.candidate.iterdir()})

    def test_cli_errors_hide_untrusted_data(self):
        (self.candidate / "release.json").write_text("private-token-must-not-leak")
        error = io.StringIO()
        with contextlib.redirect_stderr(error):
            result = sparkle.main(["verify", "--candidate", str(self.candidate), "--output", str(self.output), "--public-key", str(self.trusted)])
        self.assertEqual(result, 2)
        self.assertNotIn("private-token", error.getvalue())

    def test_manifest_cannot_introduce_a_new_signing_key(self):
        self.rewrite_manifest(lambda manifest: manifest["artifacts"][0]["signature"].update(key_id="beni-update-v2"))
        with self.assertRaises(sparkle.release.ReleaseError):
            sparkle.candidate_feeds(self.candidate, self.trusted)

    def test_presentation_is_escaped_and_invalid_urls_are_rejected(self):
        self.rewrite_manifest(lambda manifest: manifest["artifacts"][0]["requirements"].update(description="Text <Beispiel> & Details"))
        content = next(iter(sparkle.candidate_feeds(self.candidate, self.trusted).values()))
        item = ET.fromstring(content).find("channel/item")
        self.assertEqual(item.findtext("description"), "Text <Beispiel> & Details")
        self.assertEqual(len(item.findall("description")), 1)
        self.rewrite_manifest(lambda manifest: manifest["artifacts"][0].update(url="https://example.invalid/untrusted.dmg"))
        with self.assertRaises(ValueError):
            sparkle.candidate_feeds(self.candidate, self.trusted)


class NativeCompatibilityTests(unittest.TestCase):
    def test_native_signer_accepts_python_package_and_feed_and_rejects_tampering(self):
        # Disposable test key only; never read/export the production key.
        key = Ed25519PrivateKey.generate()
        secret = base64.b64encode(key.private_bytes_raw()) + b"\n"
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            fixture = root / "candidate"; fixture.mkdir()
            trusted, _, package = fixtures.candidate(fixture)
            content = next(iter(sparkle.candidate_feeds(fixture, trusted).values()))
            feed = root / "appcast.xml"
            feed.write_bytes(sparkle.sign_feed(content, key))
            archive = root / "fixture.dmg"; archive.write_bytes(package.read_bytes())
            signature = base64.b64encode(key.sign(archive.read_bytes())).decode()

            def native(*args):
                result = subprocess.run([str(NATIVE_TOOL), "--ed-key-file", "-", *args], input=secret,
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
                return result.returncode

            self.assertEqual(native("--verify", str(archive), signature), 0)
            self.assertEqual(native("--verify", str(feed)), 0)
            # Native Sparkle produces identical bytes when no warning is added.
            original = feed.read_bytes()
            feed.write_bytes(content)
            self.assertEqual(native("--disable-signing-warning", "-p", str(feed)), 0)
            self.assertEqual(feed.read_bytes(), original)
            archive.write_bytes(archive.read_bytes() + b"tampered")
            self.assertNotEqual(native("--verify", str(archive), signature), 0)
            feed.write_bytes(original.replace(b"Beni Buddy", b"Evil Buddy", 1))
            self.assertNotEqual(native("--verify", str(feed)), 0)


def main():
    global NATIVE_TOOL
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sparkle-archive", type=Path, help="Optional pinned Sparkle SDK tar.xz; macOS only, no network or production secrets.")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="beni-sparkle-probe-") as folder:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(AppcastTests)
        if args.sparkle_archive:
            pinned = json.loads((ROOT / "release/sparkle-sdk.json").read_bytes())
            archive = args.sparkle_archive
            if (platform.system() != "Darwin" or archive.is_symlink() or not archive.is_file()
                    or archive.stat().st_size != pinned["size_bytes"]
                    or hashlib.sha256(archive.read_bytes()).hexdigest() != pinned["sha256"]):
                print("FAIL: Nativer Versuch benötigt macOS und das unveränderte, gepinnte SDK.", file=sys.stderr)
                return 2
            with tarfile.open(archive) as sdk:
                sdk.extractall(folder, filter="data")
            NATIVE_TOOL = Path(folder) / "bin/sign_update"
            suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(NativeCompatibilityTests))
        result = unittest.TextTestRunner(verbosity=1).run(suite)
        if not args.sparkle_archive:
            print("NOT_RUN: native Sparkle-Kompatibilität (explizit --sparkle-archive auf macOS verwenden).")
        return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    sys.exit(main())
