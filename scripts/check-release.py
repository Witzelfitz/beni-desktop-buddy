#!/usr/bin/env python3
"""Offline positive/negative tests for the release contract and packaging versions."""
import contextlib
import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("beni_release", ROOT / "scripts/release.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


def fixture(name="stable"):
    return release.read_manifest(ROOT / f"release/examples/{name}.json")


class ManifestTests(unittest.TestCase):
    def test_v2_feeds_require_exact_mac_coverage_version_channel_key_and_bounds(self):
        manifest = fixture()
        feed = {"architecture": "aarch64", "url": release.REPOSITORY + "/releases/download/v0.1.0/appcast-macos-aarch64-stable.xml",
                "size_bytes": 1024, "sha256": "a" * 64, "key_id": "beni-update-v1"}
        manifest["macos_update_feeds"] = [feed]
        with self.assertRaises(ValueError): release.validate_manifest(manifest)
        manifest["schema_version"] = 2
        release.validate_manifest(manifest)
        missing = copy.deepcopy(manifest); del missing["macos_update_feeds"]
        with self.assertRaises(ValueError): release.validate_manifest(missing)
        for field, value in (("architecture", "x86_64"), ("url", feed["url"].replace("stable", "beta")),
                             ("url", feed["url"].replace("0.1.0", "0.2.0")), ("key_id", "beni-update-v2"),
                             ("sha256", "0"), ("size_bytes", 262145), ("size_bytes", True), ("extra", True)):
            invalid = copy.deepcopy(manifest); invalid["macos_update_feeds"][0][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                release.validate_manifest(invalid)
        for feeds in ([], [feed, feed]):
            invalid = copy.deepcopy(manifest); invalid["macos_update_feeds"] = feeds
            with self.assertRaises(ValueError): release.validate_manifest(invalid)
        invalid = copy.deepcopy(manifest); invalid["artifacts"] = [a for a in invalid["artifacts"] if a["platform"] != "macos"]
        with self.assertRaises(ValueError): release.validate_manifest(invalid)

    def test_examples_and_single_platform_releases(self):
        for name in ("stable", "beta"):
            manifest = fixture(name)
            self.assertIs(release.validate_manifest(manifest), manifest)
            for artifact in manifest["artifacts"]:
                single = copy.deepcopy(manifest)
                single["artifacts"] = [artifact]
                release.validate_manifest(single)

    def test_release_can_list_only_built_architectures(self):
        manifest = fixture()
        self.assertEqual({a["platform"] for a in manifest["artifacts"]}, {"macos", "windows", "linux"})
        arm_mac = copy.deepcopy(manifest["artifacts"][0])
        self.assertEqual(arm_mac["architecture"], "aarch64")
        intel_mac = copy.deepcopy(arm_mac)
        intel_mac["architecture"] = "x86_64"
        intel_mac["url"] = intel_mac["url"].replace("aarch64", "x86_64")
        intel_mac["signature"]["url"] = intel_mac["url"] + ".sig"
        manifest["artifacts"].append(intel_mac)
        release.validate_manifest(manifest)

    def reject_change(self, path, value):
        manifest = fixture()
        target = manifest
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        with self.assertRaises(ValueError):
            release.validate_manifest(manifest)

    def test_unknown_fields_versions_products_and_empty_artifacts(self):
        for path, value in [
            (["schema_version"], 2), (["product"], "other-product"),
            (["extra"], "unsupported"), (["artifacts"], []),
            (["artifacts", 0, "extra"], "unsupported"),
            (["artifacts", 0, "signature", "extra"], "unsupported"),
            (["artifacts", 0, "requirements", "extra"], "unsupported"),
            (["artifacts"], fixture()["artifacts"] * 3),
        ]:
            with self.subTest(path=path):
                self.reject_change(path, value)

    def test_missing_required_fields(self):
        for path in ([], ["artifacts", 0], ["artifacts", 0, "signature"]):
            original = fixture()
            target = original
            for key in path:
                target = target[key]
            for key in list(target):
                manifest = copy.deepcopy(original)
                current = manifest
                for part in path:
                    current = current[part]
                del current[key]
                with self.subTest(path=path, key=key), self.assertRaises(ValueError):
                    release.validate_manifest(manifest)

    def test_semver_channels_and_prerelease_policy(self):
        for version in ("1.0", "v1.0.0", "01.0.0", "1.0.0+build", "1.0.0-rc.1", "1.0.0-beta.0", "1.0.0-beta.01", "0.1.0\n"):
            with self.subTest(version=version):
                self.reject_change(["version"], version)
        self.reject_change(["channel"], "beta")
        self.reject_change(["channel"], "nightly")
        with self.assertRaises(ValueError):
            release.validate_manifest(fixture("beta"), channel="stable")

    def test_build_size_and_hash_bounds(self):
        for key, values in [("build_number", [0, -1, True, "1", 10000]),
                            ("size_bytes", [0, -1, True, "123", 4294967297]),
                            ("sha256", ["a" * 63, "g" * 64, "A" * 64, "a" * 64 + "\n"])]:
            for value in values:
                path = [key] if key == "build_number" else ["artifacts", 0, key]
                with self.subTest(key=key, value=value):
                    self.reject_change(path, value)

    def test_platform_format_architecture_and_requirements(self):
        for key, value in [("platform", "android"), ("architecture", "universal"),
                           ("format", "exe"), ("requirements", {"description": "macOS"}),
                           ("requirements", {"description": "  ", "minimum_os_version": "11.0"}),
                           ("requirements", {"description": "a\nb", "minimum_os_version": "11.0"}),
                           ("requirements", {"description": "macOS", "minimum_os_version": "eleven"}),
                           ("requirements", {"description": "macOS", "minimum_os_version": "11.0", "minimum_glibc_version": "2.39"})]:
            with self.subTest(key=key):
                self.reject_change(["artifacts", 0, key], value)
        self.reject_change(["artifacts", 2, "requirements"], {"description": "Linux", "minimum_os_version": "6.0"})

    def test_duplicate_targets_with_different_hashes_are_rejected(self):
        manifest = fixture()
        duplicate = copy.deepcopy(manifest["artifacts"][0])
        duplicate["sha256"] = "f" * 64
        manifest["artifacts"].append(duplicate)
        with self.assertRaises(ValueError):
            release.validate_manifest(manifest)

    def test_urls_cannot_redirect_to_other_versions_hosts_or_files(self):
        original = fixture()["artifacts"][0]["url"]
        for url in (original.replace("https:", "http:"), original.replace("github.com", "evil.example"),
                    original.replace("0.1.0", "0.2.0"), original.replace("aarch64", "x86_64"),
                    original + "?token=secret", original + "#fragment", original + "/../other.dmg",
                    original.replace("github.com/", "github.com:443/"),
                    original.replace("github.com/", "user:pass@github.com/"),
                    original.replace("Witzelfitz", "AnotherOwner"), original.replace("dmg", "zip")):
            with self.subTest(url=url):
                self.reject_change(["artifacts", 0, "url"], url)
        self.reject_change(["release_notes_url"], release.REPOSITORY + "/releases/tag/v0.2.0")

    def test_signature_reference_is_mandatory_and_bound_to_artifact(self):
        for key, value in [("algorithm", "sha256"), ("key_id", "unknown-key"),
                           ("key_id", "beni-update-v1\n"),
                           ("url", fixture()["artifacts"][1]["signature"]["url"])]:
            self.reject_change(["artifacts", 0, "signature", key], value)

    def test_build_and_channel_matching_are_explicit(self):
        release.validate_manifest(fixture(), "0.1.0", 1, "stable")
        for kwargs in ({"expected_version": "0.2.0"}, {"expected_build": 2}, {"channel": "beta"}):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                release.validate_manifest(fixture(), **kwargs)

    def test_reader_rejects_duplicate_keys_bad_json_and_oversized_input(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "manifest.json"
            for data in (b'{"version":"0.1.0","version":"0.2.0"}', b'{"a":{"b":1,"b":2}}',
                         b"broken", b"x" * (release.MAX_MANIFEST_BYTES + 1), b"[" * 2000):
                path.write_bytes(data)
                with self.subTest(size=len(data)), self.assertRaises((ValueError, RecursionError)):
                    release.read_manifest(path)

    def test_cli_validates_fixtures_and_returns_failure_without_leaking_input(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "manifest.json"
            version, build = release.version_info()
            current = fixture()
            current["version"] = version
            current["build_number"] = build
            current["channel"] = "beta" if "-beta." in version else "stable"
            current["release_notes_url"] = current["release_notes_url"].replace("0.1.0", version)
            for artifact in current["artifacts"]:
                artifact["url"] = artifact["url"].replace("0.1.0", version)
                artifact["signature"]["url"] = artifact["url"] + ".sig"
            path.write_text(json.dumps(current), encoding="utf-8")
            for args in (["validate", str(path), "--for-build", "--tag", "v" + version],
                         ["validate", str(ROOT / "release/examples/beta.json"), "--channel", "beta"]):
                result = subprocess.run([sys.executable, str(ROOT / "scripts/release.py"), *args], capture_output=True, timeout=10,
                                        env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
                self.assertEqual(result.returncode, 0, result.stderr)
            bad = fixture()
            bad["artifacts"][0]["url"] += "?token=do-not-print-this"
            path.write_text(json.dumps(bad), encoding="utf-8")
            with contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(release.main(["validate", str(path)]), 2)
            self.assertNotIn("do-not-print-this", err.getvalue())
            self.assertNotIn("Traceback", err.getvalue())
            for data in (b"invalid", b'{"secret":"do-not-print-this"', b"[" * 2000):
                path.write_bytes(data)
                with contextlib.redirect_stderr(io.StringIO()) as err:
                    self.assertEqual(release.main(["validate", str(path)]), 2)
                self.assertNotIn("do-not-print-this", err.getvalue())


class VersionTests(unittest.TestCase):
    def copied_project(self, directory):
        root = Path(directory)
        version, build = release.version_info()
        for relative in ("Cargo.toml", "Cargo.lock", "apps/desktop/Cargo.toml", "apps/desktop/Cargo.lock",
                         "crates/buddy-ipc/Cargo.toml", "crates/buddy-core/Cargo.toml", "crates/buddy-cli/Cargo.toml"):
            destination = root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, destination)
            # Fixtures remain stable when the real product version/build later increases.
            text = destination.read_text(encoding="utf-8")
            text = text.replace(f'version = "{version}"', 'version = "0.1.0"')
            if relative == "Cargo.toml":
                text = text.replace(f"build-number = {build}", "build-number = 1")
            destination.write_text(text, encoding="utf-8")
        return root

    def test_current_versions_and_tags(self):
        version, build = release.check_versions()
        self.assertEqual(release.check_versions(tag="v" + version), (version, build))
        for tag in (version, "v9.9.9", "v" + version + "-beta.1"):
            with self.assertRaises(ValueError):
                release.check_versions(tag=tag)

    def test_mismatched_manifests_and_lockfiles_fail(self):
        for relative in ("apps/desktop/Cargo.toml", "crates/buddy-ipc/Cargo.toml", "Cargo.lock", "apps/desktop/Cargo.lock"):
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as directory:
                root = self.copied_project(directory)
                path = root / relative
                path.write_text(path.read_text(encoding="utf-8").replace('version = "0.1.0"', 'version = "0.2.0"'), encoding="utf-8")
                with self.assertRaises(ValueError):
                    release.check_versions(root)

    def test_invalid_build_numbers_fail(self):
        for replacement in ("0", "-1", "true", "10000"):
            with tempfile.TemporaryDirectory() as directory:
                root = self.copied_project(directory)
                path = root / "Cargo.toml"
                path.write_text(path.read_text(encoding="utf-8").replace("build-number = 1", "build-number = " + replacement), encoding="utf-8")
                with self.assertRaises(ValueError):
                    release.version_info(root)

    def test_workspace_rejects_unsupported_versions(self):
        for version in ("0.01.0", "0.1.0-rc.1", "0.1.0-beta.0", "0.1.0+metadata"):
            with self.subTest(version=version), tempfile.TemporaryDirectory() as directory:
                root = self.copied_project(directory)
                path = root / "Cargo.toml"
                path.write_text(path.read_text(encoding="utf-8").replace('version = "0.1.0"', f'version = "{version}"'), encoding="utf-8")
                with self.assertRaises(ValueError):
                    release.version_info(root)

    def test_core_and_cli_must_inherit_workspace_versions(self):
        for relative in ("crates/buddy-core/Cargo.toml", "crates/buddy-cli/Cargo.toml"):
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as directory:
                root = self.copied_project(directory)
                path = root / relative
                path.write_text(path.read_text(encoding="utf-8").replace("version.workspace = true", 'version = "0.1.0"'), encoding="utf-8")
                with self.assertRaises(ValueError):
                    release.check_versions(root)

    def test_plist_stamping_preserves_identity_and_maps_beta_to_numeric_versions(self):
        for version, build in (("0.1.0", 1), ("0.2.0-beta.2", 7)):
            with tempfile.TemporaryDirectory() as directory:
                root = self.copied_project(directory)
                for relative in ("Cargo.toml", "Cargo.lock", "apps/desktop/Cargo.toml", "apps/desktop/Cargo.lock", "crates/buddy-ipc/Cargo.toml"):
                    path = root / relative
                    path.write_text(path.read_text(encoding="utf-8").replace('version = "0.1.0"', f'version = "{version}"').replace("build-number = 1", f"build-number = {build}"), encoding="utf-8")
                path = root / "Info.plist"
                with path.open("wb") as stream:
                    plistlib.dump({"CFBundleIdentifier": "io.witzelfitz.beni-buddy", "LSUIElement": True,
                                   "CFBundleShortVersionString": "old", "CFBundleVersion": "old"}, stream)
                release.stamp_plist(path, root)
                with path.open("rb") as stream:
                    result = plistlib.load(stream)
                self.assertEqual(result["CFBundleShortVersionString"], version.split("-")[0])
                self.assertEqual(result["CFBundleVersion"], str(build))
                self.assertEqual(result["CFBundleIdentifier"], "io.witzelfitz.beni-buddy")
                self.assertTrue(result["LSUIElement"])


if __name__ == "__main__":
    unittest.main()
