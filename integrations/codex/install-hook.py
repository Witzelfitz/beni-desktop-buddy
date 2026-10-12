#!/usr/bin/env python3
"""Register Beni user hooks, preserving existing definitions and Codex's trust review."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shlex
import shutil
import sys
import tempfile

EVENTS = ("UserPromptSubmit", "Stop", "PermissionRequest", "Interrupt")
MARKER = "# Beni Desktop Buddy: Codex status bridge, installed by install-hook.py."


def install(codex_dir, python_executable):
    source = Path(__file__).with_name("notify-hook.py")
    script = codex_dir / "hooks/beni-status.py"
    config = codex_dir / "hooks.json"
    content = source.read_text(encoding="utf-8")
    data = json.loads(config.read_text(encoding="utf-8")) if config.exists() else {"hooks": {}}
    if not isinstance(data, dict) or not isinstance(data.get("hooks", {}), dict):
        raise ValueError("Bestehende hooks.json hat ein ungültiges Format; nichts ersetzt.")
    hooks = data.setdefault("hooks", {})
    if script.is_symlink() or (script.exists() and MARKER not in script.read_text(encoding="utf-8")):
        raise ValueError("Die Zieldatei gehört nicht zur Beni-Integration; nichts ersetzt.")
    # Validate all affected groups before writing any file.
    for event in EVENTS:
        if not isinstance(hooks.get(event, []), list):
            raise ValueError(f"Ungültige Hook-Gruppen für {event}; nichts ersetzt.")
    if os.name == "nt":
        prefix = subprocess_quote([python_executable, str(script)])
    else:
        prefix = shlex.join([python_executable, str(script)])
    for event in EVENTS:
        command = prefix + " " + event
        groups = hooks.setdefault(event, [])
        # Re-running the installer doesn't create duplicate status messages.
        exists = any(isinstance(group, dict) and any(
            isinstance(handler, dict) and handler.get("command") == command
            for handler in group.get("hooks", [])
        ) for group in groups)
        if not exists:
            groups.append({"hooks": [{"type": "command", "command": command, "timeout": 3}]})
    codex_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
    script.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    if config.exists() and json.loads(config.read_text(encoding="utf-8")) != data:
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        backup = config.with_name(f"hooks.json.beni-backup-{timestamp}")
        shutil.copy2(config, backup)
        if os.name != "nt":
            backup.chmod(0o600)
    descriptor = os.open(script, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as file:
        file.write(content)
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=codex_dir,
                                     prefix="hooks.json.beni-new-", delete=False) as file:
        file.write(json.dumps(data, ensure_ascii=False, indent=2) + "\n")
        staging = Path(file.name)
    if os.name != "nt":
        script.chmod(0o600)
        staging.chmod(0o600)
    staging.replace(config)
    return script, config


def subprocess_quote(args):
    import subprocess
    return subprocess.list2cmdline(args)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--codex-dir", type=Path, default=Path(os.environ.get("CODEX_HOME", str(Path.home()/".codex"))))
    parser.add_argument("--python", default=sys.executable)
    args = parser.parse_args()
    script, config = install(args.codex_dir.resolve(), args.python)
    print(f"Beni-Hook vorbereitet: {script}\nKonfiguration: {config}")
    print("In Codex /hooks öffnen, die vier Beni-Definitionen prüfen und vertrauen. Danach Sitzung neu starten.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
