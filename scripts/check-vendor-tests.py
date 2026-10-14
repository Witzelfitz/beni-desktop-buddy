#!/usr/bin/env python3
"""Regression checks for the vendor inventory guard; no network or Cargo."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("vendor_check", Path(__file__).with_name("check-vendor.py"))
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


class InventoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="beni-vendor-test-")
        self.addCleanup(self.temp.cleanup)
        self.vendor = Path(self.temp.name)
        self.original = {name: b"original" for name in guard.CHANGED}
        self.original.update({"Cargo.lock": b"lock", "Cargo.toml": b"manifest",
            ".cargo_vcs_info.json": json.dumps({"git": {"sha1": guard.COMMIT}, "path_in_vcs": "crates/eframe"}).encode()})
        self.contract = {"version": 1, "upstream_version": guard.VERSION,
            "archive_sha256": guard.ARCHIVE_HASH, "upstream_commit": guard.COMMIT,
            "changed": {name: {"original": guard.digest(b"original"), "patched": guard.digest(b"patched")} for name in guard.CHANGED},
            "omitted": ["Cargo.lock"], "added": {name: guard.digest(b"license") for name in ["LICENSE-MIT", "LICENSE-APACHE"]}}
        for name, data in self.original.items():
            if name != "Cargo.lock":
                self.write(name, b"patched" if name in guard.CHANGED else data)
        for name in self.contract["added"]:
            self.write(name, b"license")
        for name in guard.LOCAL_DOCS:
            self.write(name, b"local documentation")

    def write(self, name, data):
        path = self.vendor / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def verify(self):
        guard.verify(self.original, self.vendor, self.contract)

    def test_exact_contract_passes_and_archive_corruption_fails(self):
        self.verify()
        with self.assertRaises(ValueError):
            guard.unpack(b"corrupted archive")

    def test_undeclared_changes_additions_deletions_and_links_fail(self):
        for name in ["Cargo.toml", "src/epi.rs", "LICENSE-MIT"]:
            with self.subTest(name=name):
                path = self.vendor / name
                data = path.read_bytes()
                self.write(name, b"unexpected change")
                with self.assertRaises(ValueError): self.verify()
                self.write(name, data)
        self.write("src/extra.rs", b"unexpected")
        with self.assertRaises(ValueError): self.verify()
        (self.vendor / "src/extra.rs").unlink()
        path = self.vendor / "Cargo.toml"
        path.unlink()
        with self.assertRaises(ValueError): self.verify()
        path.symlink_to(self.vendor / "LICENSE-MIT")
        with self.assertRaises(ValueError): self.verify()

    def test_scope_version_and_baseline_cannot_silently_drift(self):
        baseline = copy.deepcopy(self.contract)
        for key, value in [("upstream_version", "9.0.0"), ("archive_sha256", "0" * 64),
            ("upstream_commit", "0" * 40), ("omitted", ["Cargo.lock", "Cargo.toml"]), ("changed", {})]:
            with self.subTest(key=key):
                self.contract = copy.deepcopy(baseline)
                self.contract[key] = value
                with self.assertRaises(ValueError): self.verify()
        self.contract = baseline
        self.original["src/epi.rs"] = b"wrong upstream baseline"
        with self.assertRaises(ValueError): self.verify()


if __name__ == "__main__":
    unittest.main()
