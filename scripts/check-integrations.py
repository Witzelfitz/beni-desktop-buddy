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


def load_module(name, relative_path):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


codex_hook = load_module("beni_codex_hook", "integrations/codex/notify-hook.py")
codex_installer = load_module("beni_codex_install", "integrations/codex/install-hook.py")


class CodexIntegrationTests(unittest.TestCase):
    def invoke(self, event, data, runner):
        stdin = type("Input", (), {"buffer": io.BytesIO(data)})()
        with patch.object(codex_hook.sys, "stdin", stdin), patch.object(codex_hook.sys, "argv", ["notify-hook.py", event]), patch.object(codex_hook.subprocess, "run", runner):
            self.assertEqual(codex_hook.main(), 0)

    def test_supported_events_map_to_gestures_without_approval_decisions(self):
        from unittest.mock import Mock
        for event, expected in [("UserPromptSubmit", "nachdenken"), ("Stop", "daumen"), ("PermissionRequest", "ueberrascht"), ("Interrupt", "stopp")]:
            runner = Mock(return_value=subprocess.CompletedProcess([], 0))
            self.invoke(event, b'{"session_id":"session-1"}', runner)
            args = runner.call_args.args[0]
            self.assertEqual(args[args.index("--state")+1], expected)
            self.assertEqual(runner.call_args.kwargs["stdin"], subprocess.DEVNULL)
            self.assertNotIn("shell", runner.call_args.kwargs)
        runner = Mock()
        self.invoke("Stop", b'{"stop_hook_active":true}', runner)
        self.invoke("SubagentStop", b'{}', runner)
        runner.assert_not_called()

    def test_metadata_does_not_forward_prompt_paths_or_assistant_text_and_distinguishes_sessions(self):
        from unittest.mock import Mock
        runner = Mock(return_value=subprocess.CompletedProcess([], 0))
        sources = []
        for session in ["session-1", "session-2"]:
            data = {"session_id":session, "prompt":"$(touch /private/secret)", "transcript_path":"/private/secret", "last_assistant_message":"sensitive response", "hook_event_name":"PermissionRequest"}
            self.invoke("UserPromptSubmit", json.dumps(data).encode(), runner)
            args = runner.call_args.args[0]
            self.assertEqual(args[args.index("--state")+1], "nachdenken")
            for field in ["prompt", "transcript_path", "last_assistant_message", "session_id"]:
                self.assertNotIn(data[field], args)
            sources.append(args[args.index("--source")+1])
        self.assertNotEqual(*sources)
        self.assertTrue(all(s.startswith("codex-") and len(s) == 16 for s in sources))

    def test_large_or_malformed_metadata_still_delivers_generic_status(self):
        from unittest.mock import Mock
        for data in [b'not-json', b'[]', b'x'*100000, b'['*2000 + b']'*2000]:
            runner = Mock(return_value=subprocess.CompletedProcess([], 0))
            self.invoke("Stop", data, runner)
            self.assertEqual(runner.call_args.args[0][2], "Codex hat geantwortet.")

    def test_standalone_hook_outputs_empty_json_even_when_cli_is_unavailable(self):
        with tempfile.TemporaryDirectory() as directory:
            env = dict(os.environ, BENI_CLI=str(Path(directory)/"missing-cli"))
            for event in ["Stop", "PermissionRequest", "UserPromptSubmit", "Interrupt"]:
                result = subprocess.run([sys.executable, str(ROOT/"integrations/codex/notify-hook.py"), event], input=b'{}', env=env, capture_output=True, timeout=4)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(json.loads(result.stdout), {})
                self.assertEqual(result.stderr, b'')

    def test_installer_preserves_existing_hooks_and_does_not_duplicate_or_trust(self):
        with tempfile.TemporaryDirectory(prefix="beni config with spaces ") as directory:
            root = Path(directory)
            existing = {"description":"must stay", "hooks":{"Stop":[{"hooks":[{"type":"command","command":"echo existing"}]}]}}
            (root/"hooks.json").write_text(json.dumps(existing))
            (root/"trust.sentinel").write_text("unchanged")
            codex_installer.install(root, sys.executable)
            codex_installer.install(root, sys.executable)
            result = json.loads((root/"hooks.json").read_text())
            self.assertEqual(result["description"], "must stay")
            self.assertEqual(result["hooks"]["Stop"][0], existing["hooks"]["Stop"][0])
            self.assertEqual(len(result["hooks"]["Stop"]), 2)
            for event in ["PermissionRequest","Interrupt","UserPromptSubmit"]:
                self.assertEqual(len(result["hooks"][event]), 1)
            self.assertEqual((root/"trust.sentinel").read_text(), "unchanged")
            self.assertEqual(len(list(root.glob("hooks.json.beni-backup-*"))), 1)

    def test_installer_refuses_to_overwrite_a_foreign_script(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/"hooks").mkdir()
            (root/"hooks/beni-status.py").write_text("foreign content")
            with self.assertRaises(ValueError):
                codex_installer.install(root, sys.executable)
            self.assertEqual((root/"hooks/beni-status.py").read_text(), "foreign content")
            self.assertFalse((root/"hooks.json").exists())


if __name__ == "__main__":
    unittest.main()
