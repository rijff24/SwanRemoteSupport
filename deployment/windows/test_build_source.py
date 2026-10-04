import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location("build_source", Path(__file__).with_name("archive-build-source.py"))
source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source)


class BuildSourceTests(unittest.TestCase):
    def fixture(self, root):
        subprocess.run(["git", "init", str(root)], check=True, capture_output=True)
        (root / "tracked.rs").write_text("original", encoding="utf-8")
        (root / ".gitignore").write_text("private.env\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(root), "add", "."], check=True, capture_output=True)
        subprocess.run(["git", "-C", str(root), "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                        "commit", "-m", "fixture"], check=True, capture_output=True)
        for name in source.BRIDGE_FILES:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("generated bridge source", encoding="utf-8")

    def test_actual_sources_only_and_repeatable_archive(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "checkout"
            self.fixture(root)
            (root / "tracked.rs").write_text("modified build input", encoding="utf-8")
            (root / "private.env").write_text("secret must never enter archive", encoding="utf-8")
            (root / "untracked.log").write_text("private log", encoding="utf-8")
            first, second = Path(folder) / "first.zip", Path(folder) / "second.zip"
            self.assertEqual(source.archive(root, first), source.archive(root, second))
            with zipfile.ZipFile(first) as package:
                self.assertEqual(package.read("source/tracked.rs"), b"modified build input")
                self.assertNotIn("source/private.env", package.namelist())
                self.assertNotIn("source/untracked.log", package.namelist())
                receipt = json.loads(package.read("BUILD-SOURCE.json"))
                self.assertEqual(len(receipt["generated_bridge_files"]), 6)
                self.assertEqual(set(receipt["files"]), {"tracked.rs", ".gitignore", *source.BRIDGE_FILES})
            with self.assertRaises(FileExistsError):
                source.archive(root, first)

    def test_missing_generated_input_fails_before_output(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "checkout"
            self.fixture(root)
            (root / source.BRIDGE_FILES[0]).unlink()
            output = Path(folder) / "output.zip"
            with self.assertRaisesRegex(ValueError, "Generated bridge source missing"):
                source.archive(root, output)
            self.assertFalse(output.exists())

    def test_submodule_pin_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "checkout"
            self.fixture(root)
            child = root / "dependency"
            self.fixture(child)
            subprocess.run(["git", "-C", str(root), "add", "dependency"], check=True, capture_output=True)
            matching = Path(folder) / "matching.zip"
            source.archive(root, matching)
            with zipfile.ZipFile(matching) as package:
                self.assertEqual(package.read("source/dependency/tracked.rs"), b"original")
                receipt = json.loads(package.read("BUILD-SOURCE.json"))
                self.assertIn("dependency/", receipt["repositories"])
            (child / "tracked.rs").write_text("new child revision", encoding="utf-8")
            subprocess.run(["git", "-C", str(child), "add", "tracked.rs"], check=True, capture_output=True)
            subprocess.run(["git", "-C", str(child), "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                            "commit", "-m", "changed"], check=True, capture_output=True)
            with self.assertRaisesRegex(ValueError, "Submodule does not match"):
                source.archive(root, Path(folder) / "output.zip")

    def test_index_symlink_cannot_publish_its_target(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "checkout"
            self.fixture(root)
            blob = subprocess.check_output(["git", "-C", str(root), "hash-object", "-w", "--stdin"], input=b"private.env").decode().strip()
            subprocess.run(["git", "-C", str(root), "update-index", "--add", "--cacheinfo",
                            "120000," + blob + ",linked.env"], check=True, capture_output=True)
            (root / "linked.env").write_text("sensitive target", encoding="utf-8")
            output = Path(folder) / "output.zip"
            with self.assertRaisesRegex(ValueError, "Unsupported source file mode"):
                source.archive(root, output)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
