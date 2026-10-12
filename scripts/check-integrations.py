#!/usr/bin/env python3
"""Portable regression tests for hook filtering and real native CLI wrappers."""
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("beni_hook", ROOT / "integrations/claude-code/notify-hook.py")
hook = importlib.util.module_from_spec(spec)
spec.loader.exec_module(hook)
CLI = ROOT / "target/debug" / ("beni-buddy-cli.exe" if os.name == "nt" else "beni-buddy-cli")


class IntegrationTests(unittest.TestCase):
    def invoke_hook(self, data, runner):
        stdin = type("Input", (), {"buffer": io.BytesIO(data)})()
        with patch.object(hook.sys, "stdin", stdin), patch.object(hook.subprocess, "run", runner):
            self.assertEqual(hook.main(), 0)

    def test_hook_drops_unrelated_recursive_malformed_and_oversized_events(self):
        from unittest.mock import Mock
        runner = Mock()
        for data in [b"bad", b"[]", b"x" * 8193, b'{"hook_event_name":"Stop","stop_hook_active":true}',
                     b'{"hook_event_name":"Notification","notification_type":"auth_success"}',
                     b'{"hook_event_name":"Notification","notification_type":"idle_prompt"}']:
            self.invoke_hook(data, runner)
        runner.assert_not_called()

    def test_hook_never_executes_or_forwards_untrusted_message_or_transcript_fields(self):
        from unittest.mock import Mock
        runner = Mock(return_value=subprocess.CompletedProcess([], 0))
        event = {"hook_event_name": "Notification", "notification_type": "permission_prompt",
                 "message": "$(touch /tmp/should-never-exist)", "transcript_path": "/private/secrets"}
        self.invoke_hook(json.dumps(event).encode(), runner)
        args = runner.call_args.args[0]
        self.assertEqual(args[2], "Claude Code braucht dich.")
        self.assertNotIn(event["message"], args)
        self.assertNotIn(event["transcript_path"], args)
        self.assertEqual(args[args.index("--state")+1], "ueberrascht")
        self.assertEqual(args[args.index("--priority")+1], "high")
        self.assertNotIn("shell", runner.call_args.kwargs)

    def test_hook_unavailable_cli_never_blocks_session(self):
        from unittest.mock import Mock
        self.invoke_hook(b'{"hook_event_name":"Stop"}', Mock(side_effect=FileNotFoundError()))

    def test_hook_configuration_is_valid(self):
        config = json.loads((ROOT / "integrations/claude-code/settings.example.json").read_text())
        self.assertEqual(set(config["hooks"]), {"Stop", "Notification"})

    def test_native_wrapper_preserves_exit_and_argument_boundaries_without_beni(self):
        self.assertTrue(CLI.exists(), "Build the CLI first")
        with tempfile.TemporaryDirectory(prefix="beni-integration-") as directory:
            env = dict(os.environ, BENI_CLI=str(CLI), BENI_IPC_ENDPOINT=str(Path(directory) / "missing.sock"))
            if os.name == "nt":
                pwsh = shutil.which("pwsh")
                self.assertIsNotNone(pwsh, "PowerShell is required on the Windows CI runner")
                wrapper = [pwsh, "-NoProfile", "-File", str(ROOT / "integrations/beni-run.ps1")]
            else:
                wrapper = ["bash", str(ROOT / "integrations/beni-run.sh")]
            success = subprocess.run(wrapper + [str(CLI), "sample", "wave", "321"], env=env, capture_output=True, timeout=8)
            self.assertEqual(success.returncode, 0, success.stderr)
            self.assertEqual(json.loads(success.stdout)["frame"], "winken_o")
            failure = subprocess.run(wrapper + [str(CLI), "sample", "state with spaces; $(not-a-command)"], env=env, capture_output=True, timeout=8)
            self.assertEqual(failure.returncode, 2, failure.stderr)


if __name__ == "__main__":
    unittest.main()
