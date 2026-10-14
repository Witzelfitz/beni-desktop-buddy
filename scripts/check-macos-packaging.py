#!/usr/bin/env python3
"""Portable tests for release gates, signing policy and atomic installer output."""
import argparse
import base64
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


package = load("macos_package_test", "package-macos.py")
ci = load("macos_ci_test", "macos-ci-signing.py")
installer = load("macos_installer_test", "check-macos-installer.py")


def options(directory, mode="development"):
    return argparse.Namespace(mode=mode, architecture="aarch64", notary_profile="fixture-profile",
                              output_dir=Path(directory), output=Path(directory) / "Beni Buddy.app")


def fake_bundle(destination, architecture, identity):
    destination.mkdir()
    (destination / "fixture.txt").write_text("new app", encoding="utf-8")


def fake_dmg(app, directory, image, identity):
    image.write_bytes(b"fixture disk image")


class PackagingTests(unittest.TestCase):
    def test_sparkle_symlinks_licenses_and_inside_out_signing(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "sdk"; source.mkdir()
            framework = source / "Sparkle.framework"
            version = framework / "Versions/B"; version.mkdir(parents=True)
            (version / "Sparkle").write_text("fixture library")
            (framework / "Versions/Current").symlink_to("B", target_is_directory=True)
            (framework / "Sparkle").symlink_to("Versions/Current/Sparkle")
            for name in ("XPCServices/Downloader.xpc", "XPCServices/Installer.xpc", "Updater.app"):
                (version / name).mkdir(parents=True)
            (version / "Autoupdate").write_text("fixture helper")
            (source / "LICENSE").write_text("original SDK licence")
            app = root / "Buddy.app"
            resources = app / "Contents/Resources"; resources.mkdir(parents=True)
            with patch.object(package, "sdk", return_value=contextlib.nullcontext(source)), patch.object(package, "sign") as signer, patch.object(package, "verify_signature"):
                package.embed_sparkle(app, resources, "identity")
                self.assertEqual([call.args[0].name for call in signer.call_args_list],
                                 ["Downloader.xpc", "Installer.xpc", "Updater.app", "Autoupdate", "Sparkle.framework"])
                for call in signer.call_args_list:
                    self.assertEqual(call.kwargs, {"executable": True, "preserve_entitlements": True})
            self.assertTrue((app / "Contents/Frameworks/Sparkle.framework/Sparkle").is_symlink())
            self.assertEqual((resources / "Sparkle-LICENSE.txt").read_text(), "original SDK licence")

    def test_update_configuration_pins_key_channel_and_security_defaults(self):
        for architecture in package.ARCHITECTURES:
            for version, channel in (("0.1.2", "stable"), ("0.1.2-beta.1", "beta")):
                info = package.update_configuration(architecture, version)
                self.assertEqual(info["SUFeedURL"], f"https://mini.witzelfitz.io/updates/macos/{architecture}/{channel}.xml")
                self.assertEqual(info["BeniUpdateChannel"], channel)
                self.assertEqual(len(base64.b64decode(info["SUPublicEDKey"])), 32)
                for setting in ("SUEnableAutomaticChecks", "SUAutomaticallyUpdate", "SUAllowsAutomaticUpdates", "SUEnableSystemProfiling", "SUSendProfileInfo", "SUEnableJavaScript"):
                    self.assertIs(info[setting], False)
                self.assertIs(info["SURequireSignedFeed"], True)
                self.assertIs(info["SUVerifyUpdateBeforeExtraction"], True)
                self.assertEqual(info["SUSignedFeedFailureExpirationInterval"], 0)

    def test_sparkle_archive_hash_size_and_symlinks_are_checked(self):
        from sparkle_sdk import verify_archive
        with tempfile.TemporaryDirectory() as folder:
            archive = Path(folder) / "sdk.tar.xz"
            archive.write_bytes(b"fixture pinned SDK")
            pinned = {"size_bytes": archive.stat().st_size, "sha256": hashlib.sha256(archive.read_bytes()).hexdigest()}
            verify_archive(archive, pinned)
            alias = archive.with_name("alias.tar.xz"); alias.symlink_to(archive)
            for path, data in ((alias, pinned), (archive, {**pinned, "size_bytes": 1}), (archive, {**pinned, "sha256": "0" * 64})):
                with self.assertRaises(ValueError):
                    verify_archive(path, data)

    def test_unsigned_nested_helper_is_signed_before_main_executable_and_bundle(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            binaries = tuple(root / name for name in ("beni-desktop-buddy", "beni-buddy-cli"))
            for binary in binaries:
                binary.write_bytes(b"fixture unsigned Intel executable")
            signed = []
            def sign(path, identity, executable=False):
                if path.name == "beni-desktop-buddy" and "beni-buddy-cli" not in signed:
                    raise package.PackagingError("Unsigned nested helper blocks signing the surrounding app.")
                signed.append(path.name)
            with patch.object(package, "check_binary"), patch.object(package, "embed_sparkle"), patch.object(package, "run", return_value=""), patch.object(package, "sign", side_effect=sign), patch.object(package, "verify_signature"):
                package.make_bundle(root / "Buddy.app", "x86_64", "-", binaries=binaries)
            self.assertEqual(signed, ["beni-buddy-cli", "beni-desktop-buddy", "Buddy.app"])

    def test_development_container_unsigned_but_distribution_container_signed(self):
        for identity in ("-", "identity"):
            with tempfile.TemporaryDirectory() as folder, patch.object(package, "run"), patch.object(package, "sign") as signer, patch.object(package, "verify_signature") as verify:
                root = Path(folder)
                package.make_dmg(root / "source.app", root, root / "test.dmg", identity)
                if identity == "-":
                    signer.assert_not_called()
                    verify.assert_not_called()
                else:
                    signer.assert_called_once_with(root / "test.dmg", identity)
                    verify.assert_called_once_with(root / "test.dmg", production=True)

    def test_signing_diagnostics_are_fixed_and_do_not_echo_account_data(self):
        result = subprocess.CompletedProcess([], 1, "secret-token: resource fork, Finder information", "")
        with patch.object(package.subprocess, "run", return_value=result), self.assertRaises(package.PackagingError) as raised:
            package.run(["codesign", "fixture"], "Codesignierung (App-Bundle)")
        self.assertIn("Resource-Fork", str(raised.exception))
        self.assertNotIn("secret-token", str(raised.exception))

    def test_ci_check_only_does_not_create_keychain_or_build(self):
        with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "RUNNER_OS": "macOS"}), patch.object(ci, "credentials", return_value=(b"certificate", b"key")), patch.object(ci.package, "run") as runner, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(ci.main(["--architecture", "aarch64", "--check-only"]), 0)
            runner.assert_not_called()

    def test_native_attestation_cannot_be_created_without_runner_and_launch(self):
        with patch.object(installer.platform, "system", return_value="Darwin"), patch.dict(os.environ, {}, clear=True), contextlib.redirect_stderr(io.StringIO()), tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "false.native.json"
            self.assertEqual(installer.main(["missing.dmg", "--architecture", "aarch64", "--mode", "development", "--attestation", str(target)]), 2)
            self.assertFalse(target.exists())

    def test_release_requires_profile_before_any_build_or_signing(self):
        with patch.object(package.platform, "system", return_value="Darwin"), patch.object(package, "signing_identity") as signer:
            with self.assertRaises(package.PackagingError):
                package.preflight("release", "aarch64", None)
            signer.assert_not_called()

    def test_preflight_rejects_wrong_os_architecture_or_mode(self):
        with patch.object(package.platform, "system", return_value="Linux"), self.assertRaises(package.PackagingError):
            package.preflight("development", "aarch64", None)
        with patch.object(package.platform, "system", return_value="Darwin"):
            for mode, architecture in (("development", "universal"), ("unknown", "aarch64")):
                with self.assertRaises(package.PackagingError):
                    package.preflight(mode, architecture, None)

    def test_development_and_signed_modes_never_fall_back_silently(self):
        with patch.object(package.platform, "system", return_value="Darwin"), patch.object(package, "check_versions", return_value=("0.1.0", 1)), patch.object(package, "signing_identity", return_value="identity") as signer:
            self.assertEqual(package.preflight("development", "aarch64", None), "-")
            signer.assert_not_called()
            self.assertEqual(package.preflight("signed-test", "x86_64", None), "identity")
            signer.side_effect = package.PackagingError("missing identity")
            with self.assertRaises(package.PackagingError):
                package.preflight("release", "aarch64", "fixture")

    def test_identity_must_be_unique_and_developer_id(self):
        a, b = "A" * 40, "B" * 40
        first = f'1) {a} "Developer ID Application: Fixture (TEST)"'
        second = f'2) {b} "Developer ID Application: Other (TEST)"'
        with patch.object(package, "run", return_value=first):
            self.assertEqual(package.signing_identity(), a)
        with patch.object(package, "run", return_value=first + "\n" + second):
            with self.assertRaises(package.PackagingError):
                package.signing_identity()
            self.assertEqual(package.signing_identity(b), b)
        for output in ("", f'1) {a} "Apple Development: Fixture"'):
            with patch.object(package, "run", return_value=output), self.assertRaises(package.PackagingError):
                package.signing_identity()

    def test_production_code_uses_runtime_timestamp_without_extra_entitlements(self):
        with patch.object(package, "run") as runner:
            package.sign(Path("buddy"), "identity", executable=True)
            args = runner.call_args.args[0]
            self.assertIn("--timestamp", args)
            self.assertEqual(args[args.index("--options") + 1], "runtime")
            self.assertNotIn("--entitlements", args)
            self.assertNotIn("--deep", args)
            package.sign(Path("test.dmg"), "-", executable=False)
            self.assertIn("--timestamp=none", runner.call_args.args[0])
            self.assertNotIn("--options", runner.call_args.args[0])

    def test_verification_requires_actual_runtime_flag_and_secure_timestamp(self):
        good = "Authority=Developer ID Application: Fixture\nTimestamp=fixture\nCodeDirectory v=20500 flags=0x10000(runtime)\n"
        for path in (Path("Beni Buddy.app"), Path("beni-buddy-cli")):
            for details in (good.replace("Timestamp=fixture", ""), good.replace("runtime", "none"),
                            good.replace("CodeDirectory v=20500 flags=0x10000(runtime)", "Executable=/runtime/Beni.app")):
                with patch.object(package, "run", return_value=details), self.assertRaises(package.PackagingError):
                    package.verify_signature(path, production=True)
            with patch.object(package, "run", return_value=good):
                package.verify_signature(path, production=True)

    def test_binary_checks_reject_wrong_architecture_or_minimum_system(self):
        with patch.object(package, "run", side_effect=["arm64", " minos 11.0\n"]):
            package.check_binary(Path("buddy"), "aarch64")
        with patch.object(package, "run", side_effect=["x86_64", "cmd LC_VERSION_MIN_MACOSX\nversion 10.13\n"]):
            package.check_binary(Path("buddy"), "x86_64")
        for outputs in (["x86_64"], ["arm64 x86_64"], ["arm64", " minos 12.0\n"], ["arm64", "unknown"]):
            with patch.object(package, "run", side_effect=outputs), self.assertRaises(package.PackagingError):
                package.check_binary(Path("buddy"), "aarch64")

    def test_cross_build_requires_existing_target(self):
        with patch.object(package, "native_architecture", return_value="aarch64"), patch.object(package, "run", return_value="aarch64-apple-darwin") as runner:
            with self.assertRaises(package.PackagingError):
                package.executable_paths("x86_64")
            self.assertEqual(runner.call_count, 1)

    def test_notarization_accepts_only_accepted_and_keeps_auth_in_keychain(self):
        with patch.object(package, "run", return_value='{"status":"Accepted"}') as runner, patch.dict(os.environ, {}, clear=True):
            package.notarize(Path("test.dmg"), "fixture")
            args = runner.call_args.args[0]
            self.assertIn("--keychain-profile", args)
            self.assertNotIn("--password", args)
            self.assertNotIn("--force", args)
        for output in ('{"status":"Invalid"}', '{"status":"In Progress"}', '{}', '[]', 'not-json'):
            with patch.object(package, "run", return_value=output), self.assertRaises(package.PackagingError):
                package.notarize(Path("test.dmg"), "fixture")

    def test_stapling_validation_and_gatekeeper_are_required(self):
        for path in (Path("Beni Buddy.app"), Path("test.dmg")):
            with patch.object(package, "run") as runner, patch.object(package, "verify_signature") as verify:
                package.staple_and_assess(path)
                commands = [call.args[0] for call in runner.call_args_list]
                self.assertEqual([c[:3] for c in commands[:2]], [["xcrun", "stapler", "staple"], ["xcrun", "stapler", "validate"]])
                self.assertEqual(commands[2][0], "spctl")
                verify.assert_called_once_with(path, production=True)

    def test_test_modes_have_distinct_non_release_filenames(self):
        self.assertEqual(package.installer_filename("0.1.0", "aarch64", "release"), "beni-buddy-0.1.0-macos-aarch64.dmg")
        for mode in ("development", "signed-test"):
            self.assertNotEqual(package.installer_filename("0.1.0", "aarch64", mode), package.installer_filename("0.1.0", "aarch64", "release"))

    def test_receipts_have_real_size_hash_and_truthful_signing_state(self):
        for mode, identity in (("development", "-"), ("signed-test", "identity")):
            with tempfile.TemporaryDirectory() as directory, patch.object(package, "check_versions", return_value=("0.1.0", 1)), patch.object(package, "make_bundle", side_effect=fake_bundle), patch.object(package, "make_dmg", side_effect=fake_dmg), patch.object(package, "notarize") as notary, contextlib.redirect_stdout(io.StringIO()):
                package.install_dmg(options(directory, mode), identity)
                image = next(Path(directory).glob("*.dmg"))
                receipt = json.loads(Path(str(image) + ".build.json").read_text(encoding="utf-8"))
                self.assertEqual(receipt["type"], "macos-build-receipt")
                self.assertEqual(receipt["size_bytes"], len(image.read_bytes()))
                self.assertEqual(receipt["sha256"], hashlib.sha256(image.read_bytes()).hexdigest())
                self.assertFalse(receipt["notarized"])
                self.assertEqual(receipt["signing"], "ad_hoc" if mode == "development" else "developer_id")
                self.assertIn(receipt["sha256"], Path(str(image) + ".sha256").read_text())
                notary.assert_not_called()

    def test_release_failure_never_leaves_a_release_package(self):
        for failing in ("notarize_bundle", "make_dmg", "notarize", "staple_and_assess", "verify_signature"):
            with self.subTest(stage=failing), tempfile.TemporaryDirectory() as directory, contextlib.ExitStack() as stack:
                stack.enter_context(patch.object(package, "check_versions", return_value=("0.1.0", 1)))
                stack.enter_context(patch.object(package, "make_bundle", side_effect=fake_bundle))
                for name in ("notarize_bundle", "make_dmg", "notarize", "staple_and_assess", "verify_signature"):
                    effect = package.PackagingError("fixture failure") if name == failing else (fake_dmg if name == "make_dmg" else None)
                    stack.enter_context(patch.object(package, name, side_effect=effect))
                with self.assertRaises(package.PackagingError):
                    package.install_dmg(options(directory, "release"), "identity")
                self.assertEqual(list(Path(directory).iterdir()), [])

    def test_approved_release_has_canonical_name_and_notarized_receipt(self):
        with tempfile.TemporaryDirectory() as directory, contextlib.ExitStack() as stack:
            stack.enter_context(patch.object(package, "check_versions", return_value=("0.1.0", 1)))
            stack.enter_context(patch.object(package, "make_bundle", side_effect=fake_bundle))
            stack.enter_context(patch.object(package, "make_dmg", side_effect=fake_dmg))
            for name in ("notarize_bundle", "notarize", "staple_and_assess", "verify_signature"):
                stack.enter_context(patch.object(package, name))
            stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
            package.install_dmg(options(directory, "release"), "identity")
            image = Path(directory) / "beni-buddy-0.1.0-macos-aarch64.dmg"
            self.assertTrue(image.is_file())
            receipt = json.loads(Path(str(image) + ".build.json").read_text(encoding="utf-8"))
            self.assertIs(receipt["notarized"], True)
            self.assertEqual(receipt["signing"], "developer_id")

    def test_existing_installer_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(package, "check_versions", return_value=("0.1.0", 1)), patch.object(package, "make_bundle") as build:
            output = Path(directory) / package.installer_filename("0.1.0", "aarch64", "release")
            output.write_bytes(b"keep this")
            with self.assertRaises(package.PackagingError):
                package.install_dmg(options(directory, "release"), "identity")
            self.assertEqual(output.read_bytes(), b"keep this")
            build.assert_not_called()

    def test_partial_atomic_publication_is_rolled_back(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(package, "check_versions", return_value=("0.1.0", 1)), patch.object(package, "make_bundle", side_effect=fake_bundle), patch.object(package, "make_dmg", side_effect=fake_dmg):
            link = os.link
            def fail_sidecar(source, destination):
                if str(destination).endswith(".sha256"):
                    raise OSError("fixture")
                link(source, destination)
            with patch.object(package.os, "link", side_effect=fail_sidecar), self.assertRaises(OSError):
                package.install_dmg(options(directory), "-")
            self.assertEqual(list(Path(directory).iterdir()), [])

    def test_failed_bundle_build_preserves_existing_app(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(package, "make_bundle", side_effect=package.PackagingError("fixture")):
            args = options(directory)
            args.output.mkdir()
            marker = args.output / "keep.txt"
            marker.write_text("original")
            with self.assertRaises(package.PackagingError):
                package.install_bundle(args, "-")
            self.assertEqual(marker.read_text(), "original")

    def test_tool_failures_redact_raw_output_and_account_values(self):
        result = subprocess.CompletedProcess([], 1, "private-account-value")
        with patch.object(package.subprocess, "run", return_value=result), self.assertRaises(package.PackagingError) as caught:
            package.run(["tool", "private-account-value"], "Fixture")
        self.assertNotIn("private-account-value", str(caught.exception))


class CiCredentialTests(unittest.TestCase):
    def fixture(self):
        return {"GITHUB_ACTIONS": "true", "RUNNER_OS": "macOS",
                "BENI_MACOS_CERTIFICATE_P12": base64.b64encode(b"synthetic fixture").decode(),
                "BENI_MACOS_CERTIFICATE_PASSWORD": "synthetic fixture",
                "BENI_APPLE_API_KEY": "PRIVATE KEY synthetic fixture",
                "BENI_APPLE_API_KEY_ID": "TESTKEY1234",
                "BENI_APPLE_API_ISSUER": "00000000-0000-0000-0000-000000000000"}

    def test_missing_or_invalid_secrets_fail_before_keychain_changes(self):
        env = self.fixture()
        for name in ci.SECRET_NAMES:
            with self.subTest(name=name), patch.dict(os.environ, {**env, name: ""}, clear=True), patch.object(ci.package, "run") as runner, contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(ci.main(["--architecture", "aarch64"]), 2)
                runner.assert_not_called()
        for name, value in (("BENI_MACOS_CERTIFICATE_P12", "not base64!"), ("BENI_APPLE_API_ISSUER", "-" * 36),
                            ("BENI_APPLE_API_KEY_ID", "bad"), ("BENI_APPLE_API_KEY", "bad")):
            with self.assertRaises(ci.package.PackagingError):
                ci.credentials({**env, name: value})

    def test_helper_refuses_to_touch_a_local_user_keychain(self):
        with patch.dict(os.environ, {}, clear=True), patch.object(ci.package, "run") as runner, contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(ci.main(["--architecture", "aarch64"]), 2)
            runner.assert_not_called()

    def test_ci_uses_private_files_and_always_restores_keychain_search_list(self):
        for failure in (False, True):
            commands = []
            def runner(args, label):
                commands.append(args)
                if args[:2] == ["security", "import"] and os.name != "nt":
                    self.assertEqual(Path(args[2]).stat().st_mode & 0o777, 0o600)
                if args[:3] == ["xcrun", "notarytool", "store-credentials"]:
                    path = Path(args[args.index("--key") + 1])
                    if os.name != "nt":
                        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
                return '"/fixture/login.keychain-db"' if args == ["security", "list-keychains", "-d", "user"] else ""
            with patch.dict(os.environ, self.fixture(), clear=True), patch.object(ci.package, "run", side_effect=runner), patch.object(ci.package, "main", side_effect=ci.package.PackagingError("fixture") if failure else None, return_value=0), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(ci.main(["--architecture", "aarch64"]), 2 if failure else 0)
                self.assertNotIn("BENI_NOTARY_KEYCHAIN", os.environ)
            self.assertEqual(commands[-2], ["security", "list-keychains", "-d", "user", "-s", "/fixture/login.keychain-db"])
            self.assertEqual(commands[-1][:2], ["security", "delete-keychain"])
            self.assertFalse(Path(commands[-1][-1]).parent.exists())


class InstallerReceiptTests(unittest.TestCase):
    def test_modified_bytes_or_wrong_receipt_are_rejected_before_mount(self):
        for field, value in (("sha256", "0" * 64), ("architecture", "x86_64"), ("notarized", True),
                             ("size_bytes", 1), ("minimum_os_version", "10.0")):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                image = Path(directory) / "beni-buddy-0.1.0-macos-aarch64-unnotarized.dmg"
                image.write_bytes(b"synthetic disk")
                receipt = {"type": "macos-build-receipt", "architecture": "aarch64", "version": "0.1.0",
                           "build_number": 1, "minimum_os_version": installer.package.BUNDLE_MINIMUM_MACOS, "filename": image.name,
                           "sha256": hashlib.sha256(image.read_bytes()).hexdigest(), "size_bytes": image.stat().st_size,
                           "signing": "developer_id", "notarized": False}
                receipt[field] = value
                Path(str(image) + ".build.json").write_text(json.dumps(receipt), encoding="utf-8")
                with patch.object(installer.platform, "system", return_value="Darwin"), patch.object(installer.package, "native_architecture", return_value="aarch64"), patch.object(installer.package, "run") as runner, contextlib.redirect_stderr(io.StringIO()):
                    self.assertEqual(installer.main([str(image), "--architecture", "aarch64", "--mode", "signed-test"]), 2)
                    runner.assert_not_called()

    def test_window_smoke_never_launches_another_buddy_on_the_local_mac(self):
        with patch.dict(os.environ, {}, clear=True), patch.object(installer.subprocess, "Popen") as process:
            with self.assertRaises(installer.package.PackagingError):
                installer.launch_on_runner(Path("Beni Buddy.app"), Path("fixture"))
            process.assert_not_called()


if __name__ == "__main__":
    unittest.main()
