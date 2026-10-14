#!/usr/bin/env python3
"""Check interpreter selection, early failure and workspace isolation without builds."""
import importlib.util
from importlib.metadata import PackageNotFoundError
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("environment_check", Path(__file__).with_name("check-environment.py"))
preflight = importlib.util.module_from_spec(spec)
spec.loader.exec_module(preflight)


class EnvironmentTests(unittest.TestCase):
    def test_exact_dependencies_pass_and_wrong_missing_or_unpinned_fail(self):
        requirements = "# fixed dependencies\ncryptography==50.0.2\n"
        self.assertEqual(preflight.check(requirements, (3, 13), lambda _: "50.0.2"), [])
        self.assertTrue(preflight.check(requirements, (3, 10), lambda _: "50.0.2"))
        self.assertTrue(preflight.check(requirements, (3, 13), lambda _: "49.0.0"))
        def missing(_): raise PackageNotFoundError()
        self.assertTrue(preflight.check(requirements, (3, 13), missing))
        with self.assertRaises(ValueError):
            preflight.check("cryptography>=50.0.2", (3, 13))


class EntryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="beni-tooling-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "scripts").mkdir()
        shutil.copy2(Path(__file__).with_name("check.sh"), self.root / "scripts/check.sh")
        self.log = self.root / "calls"
        self.bin = self.root / "bin"
        self.bin.mkdir()
        for name in ["python3", "chosen-python", "cargo"]:
            self.stub(self.bin / name)
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
            BENI_TEST_LOG=str(self.log), BENI_TEST_PREFLIGHT="0", BENI_TEST_CARGO="0")
        self.env.pop("BENI_RELEASE_PYTHON", None)

    def stub(self, path):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('''#!/bin/bash
printf '%s:' "$0" >> "$BENI_TEST_LOG"
printf ' %s' "$@" >> "$BENI_TEST_LOG"
printf '\\n' >> "$BENI_TEST_LOG"
if [[ "${1:-}" == "scripts/check-environment.py" ]]; then exit "$BENI_TEST_PREFLIGHT"; fi
if [[ "$0" == */cargo ]]; then exit "$BENI_TEST_CARGO"; fi
''')
        path.chmod(0o700)

    def run_entry(self, *args):
        result = subprocess.run(["bash", str(self.root / "scripts/check.sh"), *args],
            env=self.env, capture_output=True, text=True, timeout=10)
        return result.returncode, self.log.read_text().splitlines() if self.log.exists() else []

    def test_help_and_bad_arguments_run_nothing(self):
        for args, expected in [(('--help',), 0), (('--desktpo',), 2), (('--desktop', '--unknown'), 2)]:
            code, calls = self.run_entry(*args)
            self.assertEqual(code, expected)
            self.assertEqual(calls, [])

    def test_interpreter_precedence_and_desktop_is_explicit(self):
        self.stub(self.root / ".venv-release/bin/python")
        code, calls = self.run_entry()
        self.assertEqual(code, 0)
        self.assertTrue(all('/.venv-release/bin/python:' in call for call in calls if '/cargo:' not in call))
        self.assertFalse(any('apps/desktop/Cargo.toml' in call for call in calls))
        self.log.unlink()
        self.env['BENI_RELEASE_PYTHON'] = str(self.bin / 'chosen-python')
        code, calls = self.run_entry('--desktop')
        self.assertEqual(code, 0)
        self.assertTrue(all('/chosen-python:' in call for call in calls if '/cargo:' not in call))
        self.assertTrue(any('apps/desktop/Cargo.toml' in call for call in calls))

    def test_missing_interpreter_preflight_and_cargo_errors_stop_immediately(self):
        self.env['BENI_RELEASE_PYTHON'] = str(self.root / 'missing-python')
        self.assertEqual(self.run_entry(), (2, []))
        self.env.pop('BENI_RELEASE_PYTHON')
        self.env['BENI_TEST_PREFLIGHT'] = '2'
        code, calls = self.run_entry()
        self.assertEqual(code, 2)
        self.assertEqual(len(calls), 1)
        self.log.unlink()
        self.env['BENI_TEST_PREFLIGHT'] = '0'
        self.env['BENI_TEST_CARGO'] = '42'
        code, calls = self.run_entry('--desktop')
        self.assertEqual(code, 42)
        self.assertTrue(calls[-1].endswith(' fmt --all --check'))
        self.assertFalse(any('apps/desktop/Cargo.toml' in call for call in calls))


if __name__ == "__main__":
    unittest.main()
