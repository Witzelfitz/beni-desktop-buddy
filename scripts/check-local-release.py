#!/usr/bin/env python3
"""Release-candidate trust, archive, signature and rollback regression tests."""
import argparse
import base64
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("beni_local_release_test", ROOT / "scripts/local-release.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


def fixture_run(architecture="x86_64"):
    run = {"status": "completed", "conclusion": "success", "path": ".github/workflows/macos-installers.yml",
           "event": "workflow_dispatch", "head_branch": "main", "head_sha": "a" * 40,
           "repository": {"full_name": release.REPO}, "head_repository": {"full_name": release.REPO}}
    runner = "macos-15" if architecture == "aarch64" else "macos-15-intel"
    jobs = [{"name": f"installer ({architecture}, {runner})", "conclusion": "success",
             "steps": [{"name": name, "conclusion": "success"} for name in
                       ("Build development installer", "Native installation and window smoke test")]}]
    return run, jobs


def native_archive():
    name = "beni-buddy-0.1.0-macos-x86_64-development.dmg"
    image = b"fixture native disk image"
    digest = hashlib.sha256(image).hexdigest()
    receipt = {"type": "macos-build-receipt", "version": "0.1.0", "build_number": 1, "architecture": "x86_64",
               "minimum_os_version": "11.0", "filename": name, "signing": "ad_hoc", "notarized": False,
               "sha256": digest, "size_bytes": len(image)}
    native = {"type": "macos-native-install-check", "source_commit": "a" * 40, "architecture": "x86_64", "version": "0.1.0",
              "build_number": 1, "tested_os_version": "15.7.9", "package_sha256": digest,
              "native_window_passed": True, "temporary_install_removed": True}
    files = {name: image, name + ".sha256": f"{digest}  {name}\n".encode(), name + ".build.json": json.dumps(receipt).encode(),
             name + ".native.json": json.dumps(native).encode()}
    return name, files


def archive_bytes(files, symlink=None):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as archive:
        for name, data in files.items():
            info = zipfile.ZipInfo(name)
            if name == symlink: info.external_attr = (stat.S_IFLNK | 0o777) << 16
            archive.writestr(info, data)
    return buffer.getvalue()


def artifact(data):
    return {"expired": False, "name": "macos-x86_64-development", "digest": "sha256:" + hashlib.sha256(data).hexdigest()}


def candidate(directory):
    key = Ed25519PrivateKey.generate()
    trusted = directory.parent / "trusted.json"
    trusted.write_text(json.dumps({"algorithm": "ed25519", "key_id": "beni-update-v1",
                                  "public_key_base64": base64.b64encode(key.public_key().public_bytes_raw()).decode()}))
    manifest = release.read_manifest(ROOT / "release/examples/stable.json")
    manifest["artifacts"] = [manifest["artifacts"][0]]
    data = manifest["artifacts"][0]
    filename = data["url"].rsplit("/", 1)[1]
    payload = directory / filename
    payload.write_bytes(b"fixture final notarized image")
    data["size_bytes"] = payload.stat().st_size
    data["sha256"] = hashlib.sha256(payload.read_bytes()).hexdigest()
    data["signature"]["key_id"] = "beni-update-v1"
    (directory / (filename + ".sha256")).write_text(f"{data['sha256']}  {filename}\n")
    (directory / "release.json").write_text(json.dumps(manifest))
    (directory / "candidate.json").write_text(json.dumps({"type": "unpublished-release-candidate", "source_commit": "a" * 40,
                                                         "product_inputs_sha256": "b" * 64, "native_builds": [{"architecture": "aarch64"}]}))
    for file in (payload, directory / "release.json", directory / "candidate.json"):
        release.sign_file(file, key)
    return trusted, key, payload


class LocalReleaseTests(unittest.TestCase):
    def test_only_successful_own_main_native_runs_are_accepted(self):
        run, jobs = fixture_run()
        with patch.object(release, "command"), patch.object(release, "build_fingerprint", return_value="same"):
            self.assertEqual(release.reviewed_run(run, "x86_64", jobs, "b" * 40), "a" * 40)
            for field, value in (("status", "in_progress"), ("conclusion", "failure"), ("event", "pull_request"),
                                 ("head_branch", "feature"), ("path", "other.yml"), ("head_sha", "--malicious"),
                                 ("head_repository", {"full_name": "other/repo"}), ("repository", {"full_name": "other/repo"})):
                bad = dict(run, **{field: value})
                with self.subTest(field=field), self.assertRaises(release.ReleaseError):
                    release.reviewed_run(bad, "x86_64", jobs, "b" * 40)

    def test_missing_native_gui_step_and_different_product_inputs_fail(self):
        run, jobs = fixture_run()
        with patch.object(release, "command"), patch.object(release, "build_fingerprint", return_value="same"):
            for invalid in ([], [dict(jobs[0], conclusion="failure")], [dict(jobs[0], steps=[{"name": "Native installation and window smoke test", "conclusion": "failure"}])], jobs * 2):
                with self.assertRaises(release.ReleaseError):
                    release.reviewed_run(run, "x86_64", invalid, "b" * 40)
        with patch.object(release, "command"), patch.object(release, "build_fingerprint", side_effect=["old", "new"]), self.assertRaises(release.ReleaseError):
            release.reviewed_run(run, "x86_64", jobs, "b" * 40)

    def test_valid_native_archive_and_false_receipts(self):
        name, files = native_archive()
        raw = archive_bytes(files)
        with tempfile.TemporaryDirectory() as folder:
            image = release.extract_artifact(raw, artifact(raw), "x86_64", Path(folder), "a" * 40)
            self.assertEqual(image.name, name)
        for field, value in (("notarized", True), ("build_number", True), ("architecture", "aarch64"),
                             ("sha256", "0" * 64), ("size_bytes", 1), ("version", "0.2.0"), ("signing", "developer_id")):
            changed = dict(files)
            receipt = json.loads(changed[name + ".build.json"])
            receipt[field] = value
            changed[name + ".build.json"] = json.dumps(receipt).encode()
            raw = archive_bytes(changed)
            with self.subTest(field=field), tempfile.TemporaryDirectory() as folder, self.assertRaises(release.ReleaseError):
                release.extract_artifact(raw, artifact(raw), "x86_64", Path(folder), "a" * 40)

    def test_archive_hash_expiry_foreign_target_path_traversal_and_links_rejected(self):
        name, files = native_archive()
        raw = archive_bytes(files)
        for changed in (dict(artifact(raw), digest="sha256:" + "0" * 64), dict(artifact(raw), expired=True),
                        dict(artifact(raw), name="foreign")):
            with tempfile.TemporaryDirectory() as folder, self.assertRaises(release.ReleaseError):
                release.extract_artifact(raw, changed, "x86_64", Path(folder), "a" * 40)
        for changed, symlink in ((dict(files, **{"../escape": b"bad"}), None), (files, name)):
            raw = archive_bytes(changed, symlink)
            with tempfile.TemporaryDirectory() as folder, self.assertRaises(release.ReleaseError):
                release.extract_artifact(raw, artifact(raw), "x86_64", Path(folder), "a" * 40)

    def test_duplicate_archive_member_rejected(self):
        name, files = native_archive()
        raw = io.BytesIO(archive_bytes(files))
        import warnings
        with warnings.catch_warnings(), zipfile.ZipFile(raw, "a") as archive:
            warnings.simplefilter("ignore")
            archive.writestr(name, b"duplicate")
        raw = raw.getvalue()
        with tempfile.TemporaryDirectory() as folder, self.assertRaises(release.ReleaseError):
            release.extract_artifact(raw, artifact(raw), "x86_64", Path(folder), "a" * 40)

    def test_native_attestation_rejects_wrong_commit_os_hash_or_incomplete_cleanup(self):
        name, files = native_archive()
        for field, value in (("source_commit", "b" * 40), ("native_window_passed", False), ("temporary_install_removed", False),
                             ("tested_os_version", "11.0"), ("package_sha256", "0" * 64), ("architecture", "aarch64")):
            changed = dict(files)
            receipt = json.loads(changed[name + ".native.json"])
            receipt[field] = value
            changed[name + ".native.json"] = json.dumps(receipt).encode()
            raw = archive_bytes(changed)
            with self.subTest(field=field), tempfile.TemporaryDirectory() as folder, self.assertRaises(release.ReleaseError):
                release.extract_artifact(raw, artifact(raw), "x86_64", Path(folder), "a" * 40)

    def test_real_signatures_and_manifest_authentication(self):
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder) / "candidate"
            directory.mkdir()
            trusted, key, payload = candidate(directory)
            self.assertEqual(release.verify_candidate(directory, trusted)["version"], "0.1.0")
            manifest = directory / "release.json"
            manifest.write_bytes(b"unauthenticated-invalid-json")
            with patch.object(release, "read_manifest") as reader, self.assertRaises(release.ReleaseError):
                release.verify_candidate(directory, trusted)
            reader.assert_not_called()

    def test_tampered_package_signature_manifest_receipt_and_extra_files_rejected(self):
        for target in ("package", "signature", "manifest", "receipt", "extra", "checksum", "key"):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as folder:
                directory = Path(folder) / "candidate"
                directory.mkdir()
                trusted, key, payload = candidate(directory)
                if target == "package": payload.write_bytes(b"modified")
                elif target == "signature": Path(str(payload) + ".sig").write_text(base64.b64encode(b"0" * 64).decode())
                elif target == "manifest": (directory / "release.json").write_text('{}')
                elif target == "receipt": (directory / "candidate.json").write_text('{}')
                elif target == "extra": (directory / "unexpected").write_text('bad')
                elif target == "checksum": Path(str(payload) + ".sha256").write_text('bad')
                elif target == "key":
                    data = json.loads(trusted.read_text())
                    data["public_key_base64"] = base64.b64encode(Ed25519PrivateKey.generate().public_key().public_bytes_raw()).decode()
                    trusted.write_text(json.dumps(data))
                with self.assertRaises(release.ReleaseError): release.verify_candidate(directory, trusted)

    def test_signed_false_hash_size_or_key_id_rejected(self):
        for field, value in (("sha256", "0" * 64), ("size_bytes", 1), ("key_id", "beni-update-v2")):
            with tempfile.TemporaryDirectory() as folder:
                directory = Path(folder) / "candidate"
                directory.mkdir()
                trusted, key, _ = candidate(directory)
                manifest = directory / "release.json"
                data = json.loads(manifest.read_text())
                target = data["artifacts"][0]["signature"] if field == "key_id" else data["artifacts"][0]
                target[field] = value
                manifest.write_text(json.dumps(data))
                release.sign_file(manifest, key)
                with self.assertRaises(release.ReleaseError): release.verify_candidate(directory, trusted)

    def test_private_key_must_match_pinned_key_and_never_goes_in_arguments(self):
        key = Ed25519PrivateKey.generate()
        raw = base64.b64encode(key.private_bytes_raw())
        with patch.object(release, "command", return_value=raw) as run:
            self.assertEqual(release.signing_key("a" * 26, key.public_key()).public_key().public_bytes_raw(), key.public_key().public_bytes_raw())
            self.assertNotIn(raw.decode(), repr(run.call_args))
        with patch.object(release, "command", return_value=raw), self.assertRaises(release.ReleaseError):
            release.signing_key("a" * 26, Ed25519PrivateKey.generate().public_key())

    def test_private_key_creation_uses_stdin_and_only_public_data_in_files(self):
        with tempfile.TemporaryDirectory() as folder, patch.object(release, "ROOT", Path(folder)):
            def command(args, payload=None, **kwargs):
                self.assertIsNotNone(payload)
                template = json.loads(payload)
                self.assertEqual(template["category"], "PASSWORD")
                self.private = template["fields"][0]["value"]
                self.assertNotIn(self.private, repr(args))
                return b'{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaa"}'
            output = Path(folder) / "public.json"
            with patch.object(release, "command", side_effect=command), patch.object(release, "signing_key"), contextlib.redirect_stdout(io.StringIO()) as captured:
                release.initialize_key("beni-update-v1", output)
            for path in Path(folder).rglob('*'):
                if path.is_file(): self.assertNotIn(self.private, path.read_text())
            self.assertNotIn(self.private, captured.getvalue())
            with self.assertRaises(release.ReleaseError): release.initialize_key("beni-update-v1", output)

    def test_atomic_adoption_refuses_existing_directory_and_rolls_back_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "staging"
            source.mkdir()
            (source / "one").write_bytes(b"one")
            (source / "two").write_bytes(b"two")
            output = root / "candidate"
            real_link = os.link
            count = 0
            def fail(source, target):
                nonlocal count
                count += 1
                if count == 2: raise OSError("fixture")
                return real_link(source, target)
            with patch.object(release.os, "link", side_effect=fail), self.assertRaises(OSError):
                release.install_candidate(source, output)
            self.assertFalse(output.exists())
            release.install_candidate(source, output)
            with self.assertRaises(FileExistsError): release.install_candidate(source, output)
            self.assertEqual((output / "one").read_bytes(), b"one")

    def test_version_reuse_or_incomplete_history_fails(self):
        key = Ed25519PrivateKey.generate().public_key()
        for history in ([{"tag_name": "v0.1.0"}], [{"tag_name": "v0.0.9", "assets": []}], [{}] * 100):
            with patch.object(release, "api", return_value=history), self.assertRaises(release.ReleaseError):
                release.check_catalog("0.1.0", 1, key)
        with patch.object(release, "api", return_value=[]): release.check_catalog("0.1.0", 1, key)

    def test_historical_authenticated_build_numbers_are_globally_monotonic(self):
        key = Ed25519PrivateKey.generate()
        manifest = release.read_manifest(ROOT / "release/examples/stable.json")
        data = json.dumps(manifest).encode()
        signature = base64.b64encode(key.sign(data))
        history = [{"tag_name": "v0.1.0", "assets": [{"name": "release.json", "id": 1}, {"name": "release.json.sig", "id": 2}]}]
        for build in (1, 2):
            with patch.object(release, "api", return_value=history), patch.object(release, "command", side_effect=[data, signature]):
                if build == 1:
                    with self.assertRaises(release.ReleaseError): release.check_catalog("0.1.1", build, key.public_key())
                else: release.check_catalog("0.1.1", build, key.public_key())

    def test_prepare_rejects_bad_ids_existing_output_or_old_os_before_signing(self):
        with tempfile.TemporaryDirectory() as folder, patch.object(release.platform, "system", return_value="Darwin"), patch.object(release, "signing_key") as signer:
            output = Path(folder) / "candidate"
            output.mkdir()
            args = argparse.Namespace(architecture=["aarch64", "x86_64"], run=["123"], minimum_os="15.0", output=output)
            for change in ({}, {"run": ["../bad"]}, {"run": ["123", "123"]}, {"minimum_os": "11.0"}, {"architecture": ["aarch64", "aarch64"]}):
                altered = argparse.Namespace(**dict(vars(args), **change))
                with self.assertRaises(release.ReleaseError): release.prepare(altered)
            signer.assert_not_called()

    def test_complete_preparation_uses_all_native_inputs_and_real_signatures(self):
        for build_failure in (False, True):
            with tempfile.TemporaryDirectory() as folder, contextlib.ExitStack() as stack:
                root = Path(folder)
                fixture_dir = root / "fixture"
                fixture_dir.mkdir()
                trusted, key, _ = candidate(fixture_dir)
                (root / "dist").mkdir()
                (root / "dist/release-signing.json").write_text(json.dumps({"key_id": "beni-update-v1", "item_id": "a" * 26}))
                args = argparse.Namespace(architecture=["aarch64", "x86_64"], run=["123"], minimum_os="15.0", output=root / "final",
                                          public_key=trusted, notary_profile="fixture")
                def make_image(image, architecture, output, identity, profile):
                    if build_failure and architecture == "x86_64": raise release.ReleaseError("fixture")
                    self.assertEqual(downloader.call_count, 2)
                    path = output / f"beni-buddy-0.1.0-macos-{architecture}.dmg"
                    path.write_bytes(b"fixture completed release image")
                    return path
                stack.enter_context(patch.object(release, "ROOT", root))
                stack.enter_context(patch.object(release.platform, "system", return_value="Darwin"))
                stack.enter_context(patch.object(release, "command", side_effect=[b"", b"a" * 40]))
                stack.enter_context(patch.object(release, "check_catalog"))
                stack.enter_context(patch.object(release, "api", return_value={"artifacts": [{"name": "macos-aarch64-development"}, {"name": "macos-x86_64-development"}]}))
                downloader = stack.enter_context(patch.object(release, "download_installer", return_value=(root / "native.dmg", "a" * 40)))
                stack.enter_context(patch.object(release, "signing_key", return_value=key))
                stack.enter_context(patch.object(release.package, "preflight", return_value="identity"))
                stack.enter_context(patch.object(release, "build_from_installer", side_effect=make_image))
                stack.enter_context(patch.object(release, "build_fingerprint", return_value="b" * 64))
                stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
                if build_failure:
                    with self.assertRaises(release.ReleaseError): release.prepare(args)
                    self.assertFalse(args.output.exists())
                else:
                    release.prepare(args)
                    manifest = release.verify_candidate(args.output, trusted)
                    self.assertEqual(len(manifest["artifacts"]), 2)

    def test_missing_target_fails_before_private_key_or_apple_requests(self):
        with tempfile.TemporaryDirectory() as folder, contextlib.ExitStack() as stack:
            root = Path(folder)
            directory = root / "fixture"
            directory.mkdir()
            trusted, _, _ = candidate(directory)
            (root / "dist").mkdir()
            (root / "dist/release-signing.json").write_text(json.dumps({"key_id": "beni-update-v1", "item_id": "a" * 26}))
            stack.enter_context(patch.object(release, "ROOT", root))
            stack.enter_context(patch.object(release.platform, "system", return_value="Darwin"))
            stack.enter_context(patch.object(release, "command", side_effect=[b"", b"a" * 40]))
            stack.enter_context(patch.object(release, "check_catalog"))
            stack.enter_context(patch.object(release, "api", return_value={"artifacts": []}))
            signer = stack.enter_context(patch.object(release, "signing_key"))
            args = argparse.Namespace(architecture=["aarch64", "x86_64"], run=["123"], minimum_os="15.0", output=root / "candidate",
                                      public_key=trusted, notary_profile="fixture")
            with self.assertRaises(release.ReleaseError): release.prepare(args)
            signer.assert_not_called()
            self.assertFalse(args.output.exists())

    def test_tool_errors_do_not_echo_secrets(self):
        import subprocess
        with patch.object(release.subprocess, "run", return_value=subprocess.CompletedProcess([], 1, b"private-key", b"token")), self.assertRaises(release.ReleaseError) as raised:
            release.command(["op", "item", "get", "fixture"])
        self.assertNotIn("private-key", str(raised.exception))
        self.assertNotIn("token", str(raised.exception))
        with patch.object(release, "verify_candidate", side_effect=ValueError("secret")), contextlib.redirect_stderr(io.StringIO()) as stream:
            self.assertEqual(release.main(["verify", "fixture"]), 2)
        self.assertNotIn("secret", stream.getvalue())


if __name__ == "__main__":
    unittest.main()
