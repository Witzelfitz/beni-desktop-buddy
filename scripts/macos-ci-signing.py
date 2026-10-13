#!/usr/bin/env python3
"""Use configured release secrets in an ephemeral GitHub Actions keychain."""
import argparse
import base64
import os
from pathlib import Path
import re
import secrets
import shlex
import sys
import tempfile

import importlib.util

spec = importlib.util.spec_from_file_location("beni_macos_package", Path(__file__).with_name("package-macos.py"))
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)

SECRET_NAMES = ("BENI_MACOS_CERTIFICATE_P12", "BENI_MACOS_CERTIFICATE_PASSWORD",
                "BENI_APPLE_API_KEY", "BENI_APPLE_API_KEY_ID", "BENI_APPLE_API_ISSUER")


def credentials(env):
    if any(not env.get(name) for name in SECRET_NAMES):
        raise package.PackagingError("macOS-Release-Secrets sind noch nicht vollständig eingerichtet.")
    if not re.fullmatch(r"[A-Za-z0-9]{10,32}", env["BENI_APPLE_API_KEY_ID"]):
        raise package.PackagingError("Apple-API-Key-ID ist ungültig.")
    if not re.fullmatch(r"[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}", env["BENI_APPLE_API_ISSUER"]):
        raise package.PackagingError("Apple-API-Issuer ist ungültig.")
    try:
        certificate = base64.b64decode(env["BENI_MACOS_CERTIFICATE_P12"], validate=True)
    except ValueError as error:
        raise package.PackagingError("Signierzertifikat ist kein gültiges Base64.") from error
    key = env["BENI_APPLE_API_KEY"].encode("utf-8")
    if not 1 <= len(certificate) <= 1024 * 1024 or not 1 <= len(key) <= 16384 or b"PRIVATE KEY" not in key:
        raise package.PackagingError("Signierzertifikat oder Apple-API-Key hat ein ungültiges Format.")
    return certificate, key


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--architecture", choices=tuple(package.ARCHITECTURES), required=True)
    args = parser.parse_args(argv)
    keychain = None
    previous = None
    try:
        if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("RUNNER_OS") != "macOS":
            raise package.PackagingError("Dieser Helfer läuft ausschliesslich auf einem macOS-Actions-Runner.")
        certificate, key = credentials(os.environ)
        previous = shlex.split(package.run(["security", "list-keychains", "-d", "user"], "Runner-Schlüsselbünde lesen"))
        with tempfile.TemporaryDirectory(prefix="beni-signing-", dir=os.environ.get("RUNNER_TEMP")) as folder:
            staging = Path(folder)
            p12 = staging / "certificate.p12"
            api = staging / "AuthKey.p8"
            for path, data in ((p12, certificate), (api, key)):
                descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
                with os.fdopen(descriptor, "wb") as stream:
                    stream.write(data)
            keychain = staging / "signing.keychain-db"
            password = secrets.token_urlsafe(32)
            try:
                package.run(["security", "create-keychain", "-p", password, str(keychain)], "Runner-Schlüsselbund erstellen")
                package.run(["security", "set-keychain-settings", "-lut", "7200", str(keychain)], "Schlüsselbund-Frist setzen")
                package.run(["security", "unlock-keychain", "-p", password, str(keychain)], "Runner-Schlüsselbund entsperren")
                package.run(["security", "import", str(p12), "-k", str(keychain), "-P", os.environ["BENI_MACOS_CERTIFICATE_PASSWORD"],
                             "-T", "/usr/bin/codesign", "-T", "/usr/bin/security"], "Developer-ID importieren")
                package.run(["security", "set-key-partition-list", "-S", "apple-tool:,apple:,codesign:",
                             "-s", "-k", password, str(keychain)], "Signierzugriff setzen")
                package.run(["security", "list-keychains", "-d", "user", "-s", str(keychain), *previous], "Runner-Suchliste setzen")
                package.run(["xcrun", "notarytool", "store-credentials", "beni-ci", "--key", str(api),
                             "--key-id", os.environ["BENI_APPLE_API_KEY_ID"], "--issuer", os.environ["BENI_APPLE_API_ISSUER"],
                             "--keychain", str(keychain)], "Apple-Zugang prüfen und speichern")
                os.environ["BENI_NOTARY_KEYCHAIN"] = str(keychain)
                return package.main(["installer", "--mode", "release", "--architecture", args.architecture,
                                     "--notary-profile", "beni-ci"])
            finally:
                os.environ.pop("BENI_NOTARY_KEYCHAIN", None)
                try:
                    package.run(["security", "list-keychains", "-d", "user", "-s", *previous], "Runner-Suchliste wiederherstellen")
                finally:
                    package.run(["security", "delete-keychain", str(keychain)], "Runner-Schlüsselbund entfernen")
    except (package.PackagingError, OSError) as error:
        detail = str(error) if isinstance(error, package.PackagingError) else "Runner-Signierung konnte nicht eingerichtet werden."
        print(f"FAIL: {detail}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
