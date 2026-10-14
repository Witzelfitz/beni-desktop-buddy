#!/usr/bin/env python3
"""Owned Linux probes; retry only a crashed compositor once, never client/protocol failures."""
import argparse
import importlib.util
import os
from pathlib import Path
import platform
import re
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_ci', ROOT / 'scripts/ci-native.py')
ci = importlib.util.module_from_spec(spec); spec.loader.exec_module(ci)


def retryable(client_code, compositor_code, protocol_error, attempt):
    return attempt == 0 and client_code != 0 and compositor_code == -11 and not protocol_error


def protocol_failed(output):
    text = output.decode(errors='replace').lower()
    return bool(re.search(r'wl_display@[0-9]+\.error\(', text)) or any(marker in text for marker in ('protocolerror', 'protocol error'))


def stop(process):
    if process.poll() is None:
        process.terminate()
        try: process.wait(timeout=5)
        except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=5)


def run(binary, name, env):
    command = ([str(binary)] if binary else ['cargo', 'test', '--manifest-path', 'apps/desktop/Cargo.toml', '--locked'])
    command += [name]
    if binary is None: command += ['--']
    command += ['--ignored', '--test-threads=1']
    return subprocess.run(command, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=90)


def wayland(binary, env):
    for attempt in range(2):
        with tempfile.TemporaryDirectory(prefix='beni-wayland-smoke-') as folder:
            runtime = Path(folder); runtime.chmod(0o700)
            child_env = dict(env, XDG_RUNTIME_DIR=str(runtime), WAYLAND_DISPLAY='beni-native',
                             BENI_INPUT_BACKEND='wayland', LIBGL_ALWAYS_SOFTWARE='1', WAYLAND_DEBUG='1')
            child_env.pop('DISPLAY', None)
            with tempfile.TemporaryFile() as log:
                compositor = subprocess.Popen(['weston', '--backend=headless-backend.so', '--use-gl', '--no-config',
                                                '--socket=beni-native', '--idle-time=0'], env=child_env, stdout=log, stderr=log)
                try:
                    deadline = time.monotonic() + 15
                    while not (runtime / 'beni-native').exists() and compositor.poll() is None and time.monotonic() < deadline:
                        time.sleep(0.1)
                    if not (runtime / 'beni-native').exists():
                        code, output = 2, b''
                    else:
                        result = run(binary, 'native_linux_input_regions', child_env)
                        code, output = result.returncode, result.stdout
                    status = compositor.poll()
                    if code == 0 and status is None:
                        print('PASS: eigener Wayland-Eingaberegionentest; Compositor lebt.'); return
                    log.seek(0, os.SEEK_END); log.seek(max(0, log.tell() - 64 * 1024))
                    protocol_error = protocol_failed(output[-64 * 1024:]) or protocol_failed(log.read(64 * 1024))
                    if not retryable(code, status, protocol_error, attempt):
                        raise RuntimeError('Wayland-Probe fehlgeschlagen.')
                    print('RETRY: Testcompositor SIGSEGV, keine erkannte Protokollverletzung; ein frischer Versuch.')
                finally: stop(compositor)
    raise RuntimeError('Wayland-Probe fehlgeschlagen.')


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--probe-directory', type=Path); args = parser.parse_args()
    if platform.system() != 'Linux': print('FAIL: native Linux-Probe benötigt Linux.'); return 2
    try:
        binary = ci.verify(args.probe_directory, os.environ.get('GITHUB_SHA')) if args.probe_directory else None
        env = dict(os.environ, LIBGL_ALWAYS_SOFTWARE='1')
        with tempfile.TemporaryDirectory(prefix='beni-x11-smoke-') as folder:
            script = Path(folder) / 'probe.sh'
            if binary:
                # Pass the executable as a positional argument, never shell-interpolate paths.
                script.write_text('#!/bin/sh\nset -eu\nBENI_INPUT_BACKEND=x11 "$1" native_linux_input_regions --ignored --test-threads=1\n"$1" native_x11_hotkey_registration_conflicts_and_release --ignored --test-threads=1\n')
                command = ['xvfb-run', '-a', 'sh', str(script), str(binary)]
            else:
                script.write_text('#!/bin/sh\nset -eu\nBENI_INPUT_BACKEND=x11 cargo test --manifest-path apps/desktop/Cargo.toml --locked native_linux_input_regions -- --ignored --test-threads=1\ncargo test --manifest-path apps/desktop/Cargo.toml --locked native_x11_hotkey_registration_conflicts_and_release -- --ignored --test-threads=1\n')
                command = ['xvfb-run', '-a', 'sh', str(script)]
            subprocess.run(command, env=env, cwd=ROOT, check=True, timeout=180, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print('PASS: eigene X11-Eingaberegionen und Hotkey-Konfliktprüfung.')
        wayland(binary, env)
        return 0
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
        print('FAIL: native Eingabe-/Hotkey-Abnahme; kein allgemeiner App-Retry.'); return 2


if __name__ == '__main__': raise SystemExit(main())
