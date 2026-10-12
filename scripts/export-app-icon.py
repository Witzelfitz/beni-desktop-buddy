#!/usr/bin/env python3
"""Export the approved raster master to native icon formats on macOS."""
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import hashlib

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets/app-icon"


def main():
    if sys.platform != "darwin":
        raise SystemExit("Der Icon-Export benötigt macOS mit sips und iconutil.")
    master = ASSETS / "beni-master.png"
    with tempfile.TemporaryDirectory(prefix="beni-icon-") as directory:
        work = Path(directory)
        iconset = work / "Beni.iconset"
        iconset.mkdir()
        images = {}
        for size in [16, 24, 32, 48, 64, 128, 256, 512, 1024]:
            target = work / f"{size}.png"
            subprocess.run(["sips", "-z", str(size), str(size), str(master), "--out", str(target)],
                           check=True, stdout=subprocess.DEVNULL)
            images[size] = target
        for size in [16, 32, 128, 256, 512]:
            shutil.copyfile(images[size], iconset / f"icon_{size}x{size}.png")
            shutil.copyfile(images[size * 2], iconset / f"icon_{size}x{size}@2x.png")
        subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(ASSETS / "Beni.icns")],
                       check=True)
        # Vista and later support PNG payloads inside multi-resolution ICO files.
        sizes = [16, 24, 32, 48, 64, 128, 256]
        offset = 6 + 16 * len(sizes)
        entries, payload = [], bytearray()
        for size in sizes:
            data = images[size].read_bytes()
            edge = size if size < 256 else 0
            entries.append(struct.pack("<BBBBHHII", edge, edge, 0, 0, 1, 32, len(data), offset))
            payload.extend(data)
            offset += len(data)
        (ASSETS / "Beni.ico").write_bytes(struct.pack("<HHH", 0, 1, len(sizes)) + b"".join(entries) + payload)
        shutil.copyfile(images[1024], ASSETS / "beni.png")
        shutil.copyfile(images[256], ASSETS / "beni-256.png")
    provenance = ASSETS / "provenance.json"
    data = json.loads(provenance.read_text())
    data["files"] = {name: hashlib.sha256((ASSETS / name).read_bytes()).hexdigest()
                     for name in ["beni-master.png", "beni.png", "beni-256.png", "Beni.icns", "Beni.ico"]}
    provenance.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    print("PASS: PNG, ICNS und mehrstufiges ICO exportiert und mit SHA-256 erfasst.")


if __name__ == "__main__":
    main()
