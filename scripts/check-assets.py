#!/usr/bin/env python3
"""Check unchanged asset imports. No image transformations or dependencies."""
import hashlib
import json
import struct
from pathlib import Path
root = Path(__file__).resolve().parents[1] / "assets" / "beni"
manifest = json.loads((root / "provenance.json").read_text())
for name, expected in manifest["files"].items():
    actual = hashlib.sha256((root / name).read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"Asset verändert: {name}; Provenienz bewusst aktualisieren.")
print(f"PASS: {len(manifest['files'])} importierte Asset-Dateien unverändert.")
icons = root.parent / "app-icon"
manifest = json.loads((icons / "provenance.json").read_text())
for name, expected in manifest["files"].items():
    if hashlib.sha256((icons / name).read_bytes()).hexdigest() != expected:
        raise SystemExit(f"Icon verändert: {name}; Export und Provenienz aktualisieren.")
for name, expected_size in [("beni.png", 1024), ("beni-256.png", 256)]:
    data = (icons / name).read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n" or struct.unpack(">II", data[16:24]) != (expected_size, expected_size):
        raise SystemExit(f"Ungültiges App-Icon: {name}")
    if data[25] != 6:
        raise SystemExit(f"App-Icon ohne RGBA: {name}")
icns = (icons / "Beni.icns").read_bytes()
if icns[:4] != b"icns" or struct.unpack(">I", icns[4:8])[0] != len(icns):
    raise SystemExit("Ungültiges ICNS.")
ico = (icons / "Beni.ico").read_bytes()
reserved, kind, count = struct.unpack("<HHH", ico[:6])
if (reserved, kind, count) != (0, 1, 7):
    raise SystemExit("Ungültiges ICO.")
for i in range(count):
    width, height, _, _, planes, bits, size, offset = struct.unpack("<BBBBHHII", ico[6+i*16:22+i*16])
    payload = ico[offset:offset+size]
    edge = width or 256
    if width != height or planes != 1 or bits != 32 or len(payload) != size or payload[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit("Ungültige ICO-Auflösung.")
    if struct.unpack(">II", payload[16:24]) != (edge, edge):
        raise SystemExit("ICO-Geometrie stimmt nicht mit dem PNG überein.")
print(f"PASS: {len(manifest['files'])} Icon-Dateien mit Provenienz; PNG/ICNS/ICO gültig.")
# Public UI previews show the real renderer; keep exports and provenance together.
previews = root.parents[1] / "docs/images/compact-ui"
manifest = json.loads((previews / "provenance.json").read_text())
for name, expected in manifest["files"].items():
    if Path(name).name != name or not name.endswith(".png"):
        raise SystemExit("Ungültiger UI-Vorschauname.")
    data = (previews / name).read_bytes()
    if hashlib.sha256(data).hexdigest() != expected or data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"UI-Vorschau verändert: {name}; Export und Provenienz aktualisieren.")
print(f"PASS: {len(manifest['files'])} echte UI-Vorschauen mit SHA-256-Provenienz.")
