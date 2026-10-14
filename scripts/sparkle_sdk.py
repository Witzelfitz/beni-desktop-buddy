"""Pinned Sparkle SDK acquisition and safe extraction for macOS packaging."""
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import tarfile
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def verify_archive(path, pinned):
    if (path.is_symlink() or not path.is_file() or path.stat().st_size != pinned["size_bytes"]
            or hashlib.sha256(path.read_bytes()).hexdigest() != pinned["sha256"]):
        raise ValueError("Sparkle-SDK ist nicht das unveränderte gepinnte Archiv.")


@contextmanager
def sdk():
    pinned = json.loads((ROOT / "release/sparkle-sdk.json").read_bytes())
    override = os.environ.get("BENI_SPARKLE_ARCHIVE")
    archive = Path(override) if override else ROOT / "dist/tools" / f"Sparkle-{pinned['version']}.tar.xz"
    if not archive.exists() and not archive.is_symlink() and not override:
        archive.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="beni-sdk-download-", dir=archive.parent) as folder:
            temporary = Path(folder) / "sdk.tar.xz"
            with urllib.request.urlopen(pinned["url"], timeout=30) as response, temporary.open("xb") as stream:
                remaining = pinned["size_bytes"] + 1
                while remaining:
                    chunk = response.read(min(remaining, 1024 * 1024))
                    if not chunk:
                        break
                    stream.write(chunk)
                    remaining -= len(chunk)
            verify_archive(temporary, pinned)
            try:
                os.link(temporary, archive)
            except FileExistsError:
                pass # Concurrent downloads must still pass the pinned check below.
    verify_archive(archive, pinned)
    with tempfile.TemporaryDirectory(prefix="beni-sparkle-sdk-") as folder:
        with tarfile.open(archive) as source:
            source.extractall(folder, filter="data")
        yield Path(folder)
