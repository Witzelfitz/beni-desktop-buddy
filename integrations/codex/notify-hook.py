#!/usr/bin/env python3
# Beni Desktop Buddy: Codex status bridge, installed by install-hook.py.
"""Forward lifecycle metadata only. Never return approval or continuation decisions."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

EVENTS = {
    "UserPromptSubmit": ("Codex arbeitet an deiner Anfrage.", "nachdenken", "normal", "15"),
    "Stop": ("Codex hat geantwortet.", "daumen", "normal", "30"),
    "PermissionRequest": ("Codex braucht deine Freigabe.", "ueberrascht", "high", "60"),
    "Interrupt": ("Codex wurde unterbrochen.", "stopp", "normal", "15"),
}
MAX_INPUT = 65536


def metadata():
    raw = sys.stdin.buffer.read(MAX_INPUT + 1)
    if len(raw) > MAX_INPUT:
        # Drain large payloads without retaining prompts or causing a broken stdin pipe.
        while sys.stdin.buffer.read(MAX_INPUT):
            pass
        return {}
    try:
        data = json.loads(raw)
        return data if isinstance(data, dict) else {}
    except (ValueError, UnicodeError, RecursionError):
        return {}


def cli_path():
    configured = os.environ.get("BENI_CLI")
    if configured:
        return configured
    found = shutil.which("beni-buddy-cli")
    if found:
        return found
    # GUI apps can have a smaller PATH than the terminal.
    fallback = Path.home() / ".cargo/bin" / ("beni-buddy-cli.exe" if os.name == "nt" else "beni-buddy-cli")
    return str(fallback) if fallback.is_file() else "beni-buddy-cli"


def main():
    data = metadata()
    # The trusted hook definition supplies the event; incoming text cannot change it.
    event = sys.argv[1] if len(sys.argv) == 2 else None
    if event not in EVENTS or (event == "Stop" and data.get("stop_hook_active", False)):
        return 0
    session = data.get("session_id")
    source = "codex"
    if isinstance(session, str) and 0 < len(session) <= 256:
        source += "-" + hashlib.sha256(session.encode("utf-8", errors="replace")).hexdigest()[:10]
    text, state, priority, ttl = EVENTS[event]
    try:
        subprocess.run(
            [cli_path(), "notify", text, "--state", state, "--source", source,
             "--priority", priority, "--ttl", ttl],
            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            timeout=2, check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        pass  # Beni is optional. It must never block a prompt, approval, interruption or Stop.
    return 0


if __name__ == "__main__":
    code = main()
    # Stop requires JSON. No decisions, context or original text are emitted for any event.
    print("{}")
    raise SystemExit(code)
