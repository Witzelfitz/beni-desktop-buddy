#!/usr/bin/env python3
"""Fail before Cargo work when the selected release interpreter is unsuitable."""
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
import re
import sys


def check(requirements: str, python_version: tuple, installed=version) -> list[str]:
    failures = []
    if python_version < (3, 11):
        failures.append("Python >= 3.11 erforderlich")
    for line in requirements.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        match = re.fullmatch(r"([a-zA-Z0-9-]+)==([0-9.]+)", line)
        if not match:
            raise ValueError("Abhängigkeit ist nicht exakt fixiert")
        name, required = match.groups()
        try:
            actual = installed(name)
        except PackageNotFoundError:
            actual = None
        if actual != required:
            failures.append(f"{name}=={required} erforderlich")
    return failures


def main() -> int:
    try:
        requirements = Path(__file__).with_name("requirements-release.txt").read_text()
        failures = check(requirements, sys.version_info[:2])
    except (OSError, ValueError):
        print("FAIL Abhängigkeitsvertrag für die Prüfungen", file=sys.stderr)
        return 2
    if failures:
        print("FAIL " + "; ".join(failures), file=sys.stderr)
        print("Einrichtung: docs/TOOLING.md; danach scripts/check.sh erneut ausführen.", file=sys.stderr)
        return 2
    print("PASS Python und festgelegte Release-Abhängigkeiten")
    return 0


if __name__ == "__main__":
    sys.exit(main())
