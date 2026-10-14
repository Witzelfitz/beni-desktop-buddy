#!/usr/bin/env python3
"""Run the actual vendored persistence tests without adding GUI to the core workspace."""
import json
from pathlib import Path
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def main():
    approved_lock = (ROOT / "apps/desktop/Cargo.lock").read_text()
    packages = tomllib.loads(approved_lock)["package"]
    versions = {p["name"]: p["version"] for p in packages}
    with tempfile.TemporaryDirectory(prefix="beni-storage-sync-tests-") as folder:
        root = Path(folder)
        (root / "src").mkdir()
        manifest = '[package]\nname="beni-storage-sync-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\nresolver="3"\n[dependencies]\n'
        manifest += 'eframe={path=' + json.dumps(str(ROOT / "vendor/eframe")) + ',default-features=false,features=["accesskit","default_fonts","glow","persistence","x11","wayland"]}\n'
        for name in ("directories", "log", "ron", "serde", "egui"):
            manifest += f'{name}={{version="={versions[name]}",default-features=false}}\n' if name == "egui" else f'{name}="={versions[name]}"\n'
        manifest += f'profiling={{version="={versions["profiling"]}",default-features=false}}\n'
        manifest += '\n[target.\'cfg(windows)\'.dependencies]\nwindows-sys={version="=0.61.2",features=["Win32_System_Com","Win32_System_Memory","Win32_UI_Shell"]}\n'
        (root / "Cargo.toml").write_text(manifest)
        source = ROOT / "vendor/eframe/src/native/file_storage.rs"
        # Compile the production source itself with the real eframe Storage API,
        # not a duplicate writer. Extra features only reuse the existing cache.
        (root / "src/lib.rs").write_text('#![allow(dead_code)]\npub use eframe::Storage;\n#[path=' + json.dumps(str(source)) + ']\nmod file_storage;\n')
        (root / "Cargo.lock").write_text(approved_lock)
        subprocess.run(["cargo", "generate-lockfile", "--manifest-path", str(root / "Cargo.toml"), "--offline"], check=True, cwd=ROOT)
        approved = {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in packages}
        resolved = tomllib.loads((root / "Cargo.lock").read_text())["package"]
        if any((p["name"], p["version"], p.get("source"), p.get("checksum")) not in approved
               for p in resolved if p["name"] != "beni-storage-sync-probe"):
            raise ValueError("Probe würde vom freigegebenen Desktop-Lockfile abweichende Bibliotheken verwenden.")
        subprocess.run(["cargo", "test", "--manifest-path", str(root / "Cargo.toml"), "--target-dir", str(ROOT / "apps/desktop/target"),
                        "--offline", "--locked", "synchronous_flush"], check=True, cwd=ROOT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
