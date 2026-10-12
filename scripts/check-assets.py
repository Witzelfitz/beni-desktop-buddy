#!/usr/bin/env python3
"""Check unchanged asset imports. No image transformations or dependencies."""
import hashlib
import json
from pathlib import Path
root = Path(__file__).resolve().parents[1] / "assets" / "beni"
manifest = json.loads((root / "provenance.json").read_text())
for name, expected in manifest["files"].items():
    actual = hashlib.sha256((root / name).read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"Asset verändert: {name}; Provenienz bewusst aktualisieren.")
print(f"PASS: {len(manifest['files'])} importierte Asset-Dateien unverändert.")
