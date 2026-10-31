import hashlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location("engine_candidate", Path(__file__).with_name("select-engine-candidate.py"))
candidate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(candidate)


class EngineCandidateTests(unittest.TestCase):
    def fixture(self, root):
        sdk = root / "sdk"
        (sdk / "bin/internal").mkdir(parents=True)
        (sdk / "bin/internal/engine.version").write_text(candidate.SDK_ENGINE + "\n")
        target = sdk / "bin/cache/artifacts/engine/windows-x64-release"
        target.mkdir(parents=True)
        (target / "gen_snapshot.exe").write_bytes(b"compiler:" + candidate.SNAPSHOT_VERSION)
        (target / "flutter_windows.dll").write_bytes(b"original")
        source = root / "candidate"
        source.mkdir()
        files = {"flutter_windows.dll": b"engine:" + candidate.SNAPSHOT_VERSION,
                 "flutter_windows.dll.lib": b"import library", "flutter_windows.dll.pdb": b"symbols"}
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, "w") as archive:
            for name, data in files.items():
                archive.writestr(name, data)
        raw = stream.getvalue()
        (source / "windows-x64-release-source-built.zip").write_bytes(raw)
        return sdk, target, source, files, raw

    def test_matching_compiler_survives_engine_selection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sdk, target, source, files, raw = self.fixture(root)
            compiler = (target / "gen_snapshot.exe").read_bytes()
            hashes = {name: hashlib.sha256(data).hexdigest() for name, data in files.items()}
            with patch.object(candidate, "ARCHIVE_SHA256", hashlib.sha256(raw).hexdigest()), patch.object(candidate, "BINARIES", hashes):
                receipt = candidate.select(source, sdk, root / "receipt.json")
            self.assertEqual((target / "gen_snapshot.exe").read_bytes(), compiler)
            self.assertEqual((target / "flutter_windows.dll").read_bytes(), files["flutter_windows.dll"])
            self.assertFalse(receipt["runtime_qualified"])
            self.assertFalse(receipt["production_approved"])

    def test_incompatible_compiler_rejected_before_modification(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sdk, target, source, _, _ = self.fixture(root)
            (target / "gen_snapshot.exe").write_bytes(b"other snapshot format")
            with self.assertRaisesRegex(ValueError, "snapshot compiler"):
                candidate.select(source, sdk, root / "receipt.json")
            self.assertEqual((target / "flutter_windows.dll").read_bytes(), b"original")
            self.assertFalse((root / "receipt.json").exists())

    def test_tampered_archive_rejected_before_modification(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sdk, target, source, _, _ = self.fixture(root)
            with self.assertRaisesRegex(ValueError, "archive differs"):
                candidate.select(source, sdk, root / "receipt.json")
            self.assertEqual((target / "flutter_windows.dll").read_bytes(), b"original")
            self.assertFalse((root / "receipt.json").exists())

    def test_unknown_candidate_rejected_before_modification(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sdk, target, source, _, _ = self.fixture(root)
            with self.assertRaisesRegex(ValueError, "Unknown source engine"):
                candidate.select(source, sdk, root / "receipt.json", "unreviewed")
            self.assertEqual((target / "flutter_windows.dll").read_bytes(), b"original")
            self.assertFalse((root / "receipt.json").exists())

    def test_previous_generation_cannot_satisfy_new_candidate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sdk, target, source, files, raw = self.fixture(root)
            hashes = {name: hashlib.sha256(data).hexdigest() for name, data in files.items()}
            with patch.object(candidate, "ARCHIVE_SHA256", hashlib.sha256(raw).hexdigest()), patch.object(candidate, "BINARIES", hashes):
                with self.assertRaisesRegex(ValueError, "archive differs"):
                    candidate.select(source, sdk, root / "receipt.json", "source-candidate-05d5369")
            self.assertEqual((target / "flutter_windows.dll").read_bytes(), b"original")
            self.assertFalse((root / "receipt.json").exists())


if __name__ == "__main__":
    unittest.main()
