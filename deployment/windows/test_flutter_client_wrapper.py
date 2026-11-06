import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).parent
spec = importlib.util.spec_from_file_location("wrapper_patch", ROOT / "patch-flutter-client-wrapper.py")
patcher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(patcher)
ORIGINAL = ROOT / "fixtures/flutter-sdk-3.24.5/core_implementations.cc"


class WrapperPatchTests(unittest.TestCase):
    def sdk(self, directory):
        sdk = Path(directory) / "sdk"
        version = sdk / "bin/internal/engine.version"
        version.parent.mkdir(parents=True)
        version.write_text(patcher.ENGINE)
        source = sdk / "bin/cache/artifacts/engine/common/cpp_client_wrapper/core_implementations.cc"
        source.parent.mkdir(parents=True)
        source.write_bytes(ORIGINAL.read_bytes())
        return sdk, source, Path(directory) / "receipt.json"

    def test_reviewed_upstream_and_both_source_line_endings(self):
        data = ORIGINAL.read_bytes().replace(b"\r\n", b"\n")
        self.assertEqual(hashlib.sha256(data).hexdigest(), patcher.ORIGINAL_SHA256)
        patched = patcher.transform(data)
        self.assertEqual(hashlib.sha256(patched).hexdigest(), patcher.PATCHED_SHA256)
        self.assertEqual(patched, patcher.transform(data.replace(b"\n", b"\r\n")))
        self.assertTrue(patched.startswith(b"// Copyright 2013 The Flutter Authors."))
        self.assertIn(b"BSD-style license", patched)

    def test_unknown_source_cannot_be_patched(self):
        with self.assertRaises(ValueError):
            patcher.transform(ORIGINAL.read_bytes() + b"\n// changed upstream\n")

    def test_unknown_engine_does_not_change_source(self):
        with tempfile.TemporaryDirectory() as directory:
            sdk, source, receipt = self.sdk(directory)
            (sdk / "bin/internal/engine.version").write_text("wrong-engine")
            with self.assertRaises(ValueError):
                patcher.patch(sdk, receipt)
            self.assertEqual(source.read_bytes(), ORIGINAL.read_bytes())
            self.assertFalse(receipt.exists())

    def test_all_preimages_validated_before_any_write(self):
        with tempfile.TemporaryDirectory() as directory:
            sdk, source, receipt = self.sdk(directory)
            other = source.parents[2] / "windows-x64/cpp_client_wrapper/core_implementations.cc"
            other.parent.mkdir(parents=True)
            other.write_bytes(b"unexpected source")
            with self.assertRaises(ValueError):
                patcher.patch(sdk, receipt)
            self.assertEqual(source.read_bytes(), ORIGINAL.read_bytes())
            self.assertFalse(receipt.exists())

    def test_source_patch_receipt_never_allows_a_silent_repeat(self):
        with tempfile.TemporaryDirectory() as directory:
            sdk, source, receipt = self.sdk(directory)
            result = patcher.patch(sdk, receipt)
            self.assertEqual(result["files"][0]["patched_sha256"], hashlib.sha256(source.read_bytes()).hexdigest())
            self.assertFalse(result["engine_binaries_changed"])
            before = source.read_bytes()
            with self.assertRaises(FileExistsError):
                patcher.patch(sdk, receipt)
            self.assertEqual(source.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
