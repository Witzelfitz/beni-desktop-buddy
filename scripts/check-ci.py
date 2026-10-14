#!/usr/bin/env python3
"""Offline checks for CI scope, artifact validation and bounded infrastructure retries."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
def load(name, filename):
    s=importlib.util.spec_from_file_location(name, ROOT/'scripts'/filename); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); return m
ci=load('ci_scope','ci-native.py'); probe=load('ci_probe','check-input-linux.py')

class PolicyTests(unittest.TestCase):
    def test_scope_includes_native_dependencies_and_skips_headless_and_docs_changes(self):
        for name in ['apps/desktop/src/app.rs','assets/beni/atlas.png','vendor/eframe/src/epi.rs','Cargo.lock','scripts/check-input-linux.py','.github/workflows/ci.yml']:
            self.assertTrue(ci.required([name]),name)
        self.assertFalse(ci.required(['README.md','docs/CHAT.md','crates/buddy-chat/src/reply.rs']))
        self.assertTrue(ci.plan('workflow_dispatch',{},'not-needed'))
        self.assertTrue(ci.plan('push',{'before':'0'*40},'a'*40))
        with self.assertRaises(ValueError):ci.plan('pull_request',{},'a'*40)
        with patch.object(ci.subprocess,'check_output',return_value=b'crates/buddy-core/src/lib.rs\0'):
            self.assertFalse(ci.plan('push',{'before':'b'*40},'a'*40))

    def test_only_one_compositor_crash_can_retry_and_app_or_protocol_errors_cannot(self):
        self.assertTrue(probe.retryable(101,-11,False,0))
        for case in [(101,None,False,0),(101,-11,True,0),(101,-11,False,1),(0,-11,False,0),(1,-15,False,0)]:
            self.assertFalse(probe.retryable(*case),case)
        self.assertTrue(probe.protocol_failed(b'wl_display@1.error(5, 3, "invalid")'))
        self.assertFalse(probe.protocol_failed(b'Broken pipe'))

    def test_probe_rejects_wrong_commit_tampered_binary_and_foreign_files(self):
        with tempfile.TemporaryDirectory() as folder:
            directory=Path(folder); binary=directory/'beni-native-probe'; data=bytearray(64);data[:6]=b'\x7fELF\x02\x01';data[18:20]=(62).to_bytes(2,'little');binary.write_bytes(data)
            receipt=directory/'probe.json';receipt.write_text(json.dumps({'source_commit':'a'*40,'sha256':hashlib.sha256(data).hexdigest()}))
            self.assertEqual(ci.verify(directory,'a'*40),binary)
            with self.assertRaises(ValueError):ci.verify(directory,'b'*40)
            binary.write_bytes(b'tampered')
            with self.assertRaises(ValueError):ci.verify(directory,'a'*40)
            binary.write_bytes(data);(directory/'extra').write_text('unexpected')
            with self.assertRaises(ValueError):ci.verify(directory,'a'*40)

    def test_export_selector_rejects_no_or_foreign_test_executable(self):
        with self.assertRaises(ValueError):ci.executable([])
        record={'reason':'compiler-artifact','target':{'name':'beni-desktop-buddy'},'profile':{'test':True},'executable':'/foreign/program'}
        with self.assertRaises(ValueError):ci.executable([record])

if __name__=='__main__':unittest.main()
