import hashlib
import importlib.util
from pathlib import Path
import struct
import unittest

import brotli


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


diagnostic = load("payload_diagnostic", "diagnose-native-payload.py")
pe_fixture = load("pe_fixture", "test_reproducible_pe.py")


def portable(name=b"rustdesk.exe", content=b"original bytes"):
    compressed = brotli.compress(content)
    return (b"MZ stub" + b"rustdesk" + struct.pack(">I", len(name)) + name
            + struct.pack(">I", len(compressed)) + compressed + hashlib.md5(content).hexdigest().encode()
            + b"rustdesk.\\rustdesk.exe\0")


class NativePayloadDiagnosticTests(unittest.TestCase):
    def test_extracts_original_bytes_without_modifying_input(self):
        raw = portable()
        digest = hashlib.sha256(raw).hexdigest()
        self.assertEqual(diagnostic.payload(raw), {"rustdesk.exe": b"original bytes"})
        self.assertEqual(hashlib.sha256(raw).hexdigest(), digest)

    def test_rejects_traversal_corruption_and_truncation(self):
        raw = portable()
        checksum = hashlib.md5(b"original bytes").hexdigest().encode()
        for invalid in [portable(b"../rustdesk.exe"), portable(b"/rustdesk.exe"),
                        raw.replace(checksum, b"0" * 32), raw[:-30]]:
            with self.subTest(size=len(invalid)), self.assertRaises(ValueError):
                diagnostic.payload(invalid)

    def test_reports_original_debug_identity_and_section_differences(self):
        first = pe_fixture.fixture()
        second = bytearray(first)
        second[580] ^= 1
        before = bytes(first), bytes(second)
        a, b = diagnostic.pe_metadata(first), diagnostic.pe_metadata(second)
        self.assertNotEqual(a["debug"][1]["guid"], b["debug"][1]["guid"])
        self.assertNotEqual(a["sections"], b["sections"])
        self.assertEqual(before, (bytes(first), bytes(second)))
        with self.assertRaises(ValueError):
            diagnostic.pe_metadata(b"not a PE")


if __name__ == "__main__":
    unittest.main()
