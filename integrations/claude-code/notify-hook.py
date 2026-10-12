#!/usr/bin/env python3
"""Translate approved hook events into bounded, generic Beni messages. No transcript access."""
import json
import os
import subprocess
import sys


def main():
    raw = sys.stdin.buffer.read(8193)
    if len(raw) > 8192:
        return 0
    try:
        event = json.loads(raw)
    except (ValueError, UnicodeError):
        return 0
    if not isinstance(event, dict):
        return 0
    name = event.get("hook_event_name")
    if name == "Stop" and not event.get("stop_hook_active", False):
        text, state, priority = "Claude Code ist fertig.", "daumen", "normal"
    elif name == "Notification" and event.get("notification_type") in {
        "permission_prompt", "worker_permission_prompt", "agent_needs_input", "elicitation_dialog"
    }:
        text, state, priority = "Claude Code braucht dich.", "ueberrascht", "high"
    else:
        return 0
    try:
        result = subprocess.run(
            [os.environ.get("BENI_CLI", "beni-buddy-cli"), "notify", text,
             "--state", state, "--source", "claude-code", "--priority", priority],
            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=2, check=False,
        )
        if result.returncode:
            sys.stderr.write("Beni-Hook: Statusmeldung nicht zugestellt.\n")
    except (OSError, subprocess.TimeoutExpired):
        sys.stderr.write("Beni-Hook: CLI nicht verfügbar oder Timeout.\n")
    # Beni is optional and must never stop or extend the coding session.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
