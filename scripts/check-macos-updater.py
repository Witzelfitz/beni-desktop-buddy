#!/usr/bin/env python3
"""Probe the real Sparkle bridge in a disposable signed macOS bundle."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import importlib.util
from pathlib import Path
import plistlib
import re
import subprocess
import sys
import tempfile
import threading
import uuid

from release import ROOT

spec = importlib.util.spec_from_file_location("beni_updater_probe_package", ROOT / "scripts/package-macos.py")
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


def probe(identity):
    domain = f"io.witzelfitz.beni-buddy.update-probe-{uuid.uuid4().hex}"
    with tempfile.TemporaryDirectory(prefix="beni-updater-native-") as folder:
        root = Path(folder)
        binary = root / "beni-desktop-buddy"
        package.run(["xcrun", "clang", "-fobjc-arc", "-fblocks", "-mmacosx-version-min=11.0",
                     "-framework", "AppKit", "-framework", "Foundation",
                     str(ROOT / "packaging/macos/updater-smoke.m"), "-o", str(binary)], "Native Updater-Probe kompilieren")
        app = root / "Beni Update Probe.app"
        package.make_bundle(app, package.native_architecture(), identity, binaries=(binary,))
        plist = app / "Contents/Info.plist"
        info = plistlib.loads(plist.read_bytes())
        info["CFBundleIdentifier"] = domain
        # A suite must differ from the current app's own bundle identifier.
        info["SUDefaultsDomain"] = domain + ".sparkle"
        info["CFBundleDisplayName"] = "Beni Update Probe"
        info["BeniUpdateFeedReady"] = False
        # Local fixture server only. Production packaging still pins HTTPS.
        class Fixture(BaseHTTPRequestHandler):
            requests = 0
            def do_GET(self):
                Fixture.requests += 1
                content = b'<?xml version="1.0"?><rss version="2.0"><channel><title>Fixture</title></channel></rss>'
                self.send_response(200); self.send_header("Content-Type", "application/xml")
                self.send_header("Content-Length", str(len(content))); self.end_headers()
                self.wfile.write(content)
            def log_message(self, *args):
                pass
        server = ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
        thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
        info["SUFeedURL"] = f"http://127.0.0.1:{server.server_port}/appcast.xml"
        info["NSAppTransportSecurity"] = {"NSAllowsLocalNetworking": True}

        def save_info():
            plist.write_bytes(plistlib.dumps(info))
            package.sign(app, identity, executable=True)
            package.verify_signature(app, production=identity != "-")

        def run(mode):
            try:
                result = subprocess.run([str(app / "Contents/MacOS/beni-desktop-buddy"), mode], stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE, text=True, timeout=20)
            except subprocess.TimeoutExpired as error:
                raise package.PackagingError(f"Sparkle-Probe ({mode}): Zeitlimit erreicht.") from error
            if result.returncode:
                diagnostics = re.findall(r"FAIL: native updater (?:probe line|create status) [0-9]+", result.stderr)
                raise package.PackagingError(f"Sparkle-Probe ({mode}) fehlgeschlagen: {'; '.join(diagnostics) or 'kein gültiges Ergebnis'}.")
            print(f"PASS: native Sparkle-Brücke ({mode}).", flush=True)

        try:
            save_info()
            run("pending")
            info["BeniUpdateFeedReady"] = True
            save_info()
            for mode in ("enable", "persisted", "gate", "bad-feed"):
                run(mode)
            if Fixture.requests == 0:
                raise package.PackagingError("Die ungültige lokale Feed-Fixture wurde nicht angefordert.")
        finally:
            server.shutdown(); server.server_close(); thread.join(timeout=2)
            # Only this freshly generated test domain; never production preferences.
            for temporary_domain in (domain, domain + ".sparkle"):
                subprocess.run(["defaults", "delete", temporary_domain], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--developer-id", action="store_true", help="Use the existing local Developer ID; default is ad hoc.")
    args = parser.parse_args()
    try:
        identity = package.preflight("signed-test" if args.developer_id else "development", package.native_architecture(), None)
        probe(identity)
    except package.PackagingError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 2
    except Exception:
        print("FAIL: native Sparkle-Brücke; keine bestehende App oder Hooks verändert.", file=sys.stderr)
        return 2
    print("NOT_RUN: tatsächliches Update A → B, Downloadabbruch und Installationsort-Abnahme.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
