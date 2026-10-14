#!/usr/bin/env python3
"""Select native smoke checks and export one already-built Linux test executable."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PREFIXES = ('apps/desktop/', 'vendor/eframe/', 'assets/', 'packaging/linux/')
PROBES = ('native_linux_input_regions', 'native_x11_hotkey_registration_conflicts_and_release')
FILES = {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.github/workflows/ci.yml',
         'scripts/ci-native.py', 'scripts/check-ci.py', 'scripts/check-input-linux.sh', 'scripts/check-input-linux.py'}


def required(paths):
    return any(path in FILES or path.startswith(PREFIXES) for path in paths)


def plan(event_name, event, head):
    if event_name == 'workflow_dispatch': return True
    base = event.get('pull_request', {}).get('base', {}).get('sha') if event_name == 'pull_request' else event.get('before')
    if base == '0' * 40: return True
    if not all(isinstance(sha, str) and re.fullmatch('[0-9a-f]{40}', sha) for sha in (base, head)):
        raise ValueError('CI-Commit-Zuordnung fehlt.')
    data = subprocess.check_output(['git', 'diff', '--name-only', '-z', base, head, '--'], cwd=ROOT)
    return required(data.decode('utf-8', errors='surrogateescape').split('\0'))


def executable(records):
    candidates = {record['executable'] for record in records
                  if record.get('reason') == 'compiler-artifact'
                  and record.get('target', {}).get('name') == 'beni-desktop-buddy'
                  and record.get('profile', {}).get('test') is True and record.get('executable')}
    if len(candidates) != 1: raise ValueError('Genau ein Desktop-Testprogramm erwartet.')
    path = Path(candidates.pop()).resolve()
    if not path.is_relative_to((ROOT / 'apps/desktop/target').resolve()) or not path.is_file():
        raise ValueError('Testprogramm ausserhalb des eigenen Build-Verzeichnisses.')
    return path


def export(directory, head):
    if not re.fullmatch('[0-9a-f]{40}', head): raise ValueError('Build-Commit fehlt.')
    if directory.exists(): raise ValueError('Probe-Verzeichnis existiert bereits.')
    data = subprocess.check_output(['cargo', 'test', '--manifest-path', 'apps/desktop/Cargo.toml',
                                    '--locked', '--no-run', '--message-format=json'], cwd=ROOT)
    source = executable([json.loads(line) for line in data.splitlines()])
    directory.mkdir(parents=True)
    target = directory / 'beni-native-probe'
    shutil.copy2(source, target)
    subprocess.run(['strip', '--strip-debug', str(target)], check=True)
    listing = subprocess.check_output([str(target), '--list'], cwd=ROOT).decode()
    if not all(sum(line.endswith(name + ': test') for line in listing.splitlines()) == 1 for name in PROBES):
        raise ValueError('Erwartete native Tests fehlen im Programm.')
    digest = hashlib.sha256(target.read_bytes()).hexdigest()
    (directory / 'probe.json').write_text(json.dumps({'source_commit': head, 'sha256': digest}) + '\n')


def verify(directory, head):
    if {p.name for p in directory.iterdir()} != {'probe.json', 'beni-native-probe'}:
        raise ValueError('Unerwartete Probe-Dateien.')
    if any(p.is_symlink() or not p.is_file() for p in directory.iterdir()):
        raise ValueError('Probe benötigt reguläre Dateien.')
    metadata = directory / 'probe.json'
    if metadata.stat().st_size > 4096: raise ValueError('Probe-Metadaten zu gross.')
    receipt = json.loads(metadata.read_text())
    target = directory / 'beni-native-probe'
    if target.stat().st_size > 128 * 1024 * 1024: raise ValueError('Probe zu gross.')
    with target.open('rb') as stream: header = stream.read(20)
    if (len(header) < 20 or header[:6] != b'\x7fELF\x02\x01' or int.from_bytes(header[18:20], 'little') != 62
            or receipt != {'source_commit': head, 'sha256': hashlib.sha256(target.read_bytes()).hexdigest()}
            or not isinstance(head, str) or not re.fullmatch('[0-9a-f]{40}', head)):
        raise ValueError('Probe-Bytes, Architektur oder Commit stimmen nicht.')
    target.chmod(0o700)
    return target


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('command', choices=['plan', 'export'])
    parser.add_argument('--directory', type=Path)
    args = parser.parse_args()
    try:
        if args.command == 'export':
            if args.directory is None: raise ValueError('Probe-Verzeichnis fehlt.')
            export(args.directory, os.environ.get('GITHUB_SHA', ''))
        else:
            event = json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text())
            value = plan(os.environ['GITHUB_EVENT_NAME'], event, os.environ['GITHUB_SHA'])
            with Path(os.environ['GITHUB_OUTPUT']).open('a') as output: output.write(f'required={str(value).lower()}\n')
        return 0
    except (ValueError, OSError, KeyError, subprocess.SubprocessError):
        print('FAIL: native CI-Zuordnung oder Probe-Export ungültig.'); return 2


if __name__ == '__main__': raise SystemExit(main())
