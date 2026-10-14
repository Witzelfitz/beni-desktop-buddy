#!/usr/bin/env python3
"""Compare the entire vendored eframe tree against its pinned published archive."""
from __future__ import annotations

import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import sys
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.36.2"
ARCHIVE_HASH = "55ad41fac6e3149abb4866e93df37dbd33baa95db525c3368c944584e95a5dc2"
COMMIT = "49682f8baa058bf49e011035cfbd6e825f88a5ef"
CHANGED = {"src/epi.rs", "src/native/file_storage.rs", "src/native/glow_integration.rs"}
LOCAL_DOCS = {"BENI_PATCH.md", "BENI_PATCH.json"}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unpack(data: bytes) -> dict[str, bytes]:
    if len(data) > 1024 * 1024 or digest(data) != ARCHIVE_HASH:
        raise ValueError("upstream archive checksum mismatch")
    files = {}
    total = 0
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        for member in archive:
            path = PurePosixPath(member.name)
            if path.parts[:1] != (f"eframe-{VERSION}",) or ".." in path.parts:
                raise ValueError("unexpected upstream archive path")
            if member.isdir():
                continue
            if not member.isfile() or member.size > 2 * 1024 * 1024:
                raise ValueError("unexpected upstream archive entry")
            total += member.size
            if total > 8 * 1024 * 1024:
                raise ValueError("upstream archive too large")
            name = path.relative_to(f"eframe-{VERSION}").as_posix()
            if name in files:
                raise ValueError("duplicate upstream archive entry")
            files[name] = archive.extractfile(member).read()
    return files


def verify(original: dict[str, bytes], vendor: Path, contract: dict) -> None:
    if (set(contract) != {"version", "upstream_version", "archive_sha256", "upstream_commit", "changed", "omitted", "added"}
        or contract["version"] != 1 or contract["upstream_version"] != VERSION
        or contract["archive_sha256"] != ARCHIVE_HASH or contract["upstream_commit"] != COMMIT
        or set(contract["changed"]) != CHANGED or contract["omitted"] != ["Cargo.lock"]
        or set(contract["added"]) != {"LICENSE-MIT", "LICENSE-APACHE"}):
        raise ValueError("patch contract differs from pinned scope")
    actual = {}
    for path in vendor.rglob("*"):
        if path.is_symlink():
            raise ValueError("vendor symlinks are not allowed")
        if path.is_file():
            if path.stat().st_size > 2 * 1024 * 1024:
                raise ValueError("vendor file too large")
            actual[path.relative_to(vendor).as_posix()] = path.read_bytes()
    expected = (set(original) - set(contract["omitted"])) | set(contract["added"]) | LOCAL_DOCS
    if set(actual) != expected:
        raise ValueError("unexpected added or missing vendor file")
    for name, data in original.items():
        if name in contract["omitted"]:
            continue
        if name in CHANGED:
            entry = contract["changed"][name]
            if set(entry) != {"original", "patched"} or digest(data) != entry["original"] or digest(actual[name]) != entry["patched"]:
                raise ValueError("declared patch checksum mismatch")
        elif actual[name] != data:
            raise ValueError("undeclared upstream modification")
    for name, checksum in contract["added"].items():
        if digest(actual[name]) != checksum:
            raise ValueError("retained license checksum mismatch")
    vcs = json.loads(actual[".cargo_vcs_info.json"])
    if vcs["git"]["sha1"] != COMMIT or vcs["path_in_vcs"] != "crates/eframe":
        raise ValueError("upstream provenance mismatch")


def archive_bytes() -> bytes:
    # No extraction, no Cargo workspace mutation. CI needs one public 140 KiB fetch
    # when the official archive is absent because Cargo uses our path patch.
    import os
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    for path in sorted((cargo_home / "registry/cache").glob(f"*/eframe-{VERSION}.crate")):
        if path.is_file() and not path.is_symlink() and path.stat().st_size <= 1024 * 1024:
            return path.read_bytes()
    url = f"https://static.crates.io/crates/eframe/eframe-{VERSION}.crate"
    with urllib.request.urlopen(url, timeout=15) as response:
        if response.geturl() != url:
            raise ValueError("unexpected archive redirect")
        return response.read(1024 * 1024 + 1)


def main() -> int:
    try:
        vendor = ROOT / "vendor/eframe"
        contract = json.loads((vendor / "BENI_PATCH.json").read_text())
        verify(unpack(archive_bytes()), vendor, contract)
    except (OSError, ValueError, KeyError, TypeError, tarfile.TarError):
        print("FAIL eframe archive, provenance or patch inventory", file=sys.stderr)
        return 1
    print("PASS eframe 0.36.2: exactly three declared source patches; remaining upstream files unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main())
