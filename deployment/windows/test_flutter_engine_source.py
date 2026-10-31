import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("engine_source", Path(__file__).with_name("flutter-engine-source.py"))
source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source)


class EngineSourceTests(unittest.TestCase):
    def test_reviewed_source_recipe_and_patch_hashes(self):
        source.load_recipe()

    def test_changed_patch_or_dependency_pin_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("flutter-engine.lock.json", "flutter-engine-windows.patch", "flutter-engine-buildroot.patch"):
                (root / name).write_bytes((source.RECIPE / name).read_bytes())
            with patch.object(source, "RECIPE", root):
                source.load_recipe()
                target = root / "flutter-engine-buildroot.patch"
                original = target.read_bytes()
                target.write_bytes(original + b"unreviewed patch\n")
                with self.assertRaisesRegex(ValueError, "toolset-selection patch"):
                    source.load_recipe()
                target.write_bytes(original)
                lock_path = root / "flutter-engine.lock.json"
                lock = json.loads(lock_path.read_text(encoding="utf-8"))
                lock["dependency_overrides"]["src/flutter/third_party/tinygltf"]["revision"] = "0" * 40
                lock_path.write_text(json.dumps(lock), encoding="utf-8")
                with self.assertRaisesRegex(ValueError, "original DEPS commit"):
                    source.load_recipe()

    def test_compile_commands_fail_closed_on_toolchain_drift(self):
        lock, _ = source.load_recipe()
        valid = ('clang-cl /Brepro -fdebug-compilation-dir=. -imsvc"C:\\VS\\VC\\Tools\\MSVC\\14.38.33130\\include" '
                 '-imsvc"C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\ucrt"\n'
                 '../../flutter/buildtools/windows-x64/clang/bin/lld-link.exe /OUT:./flutter_windows.dll /libpath:"C:\\VS\\VC\\Tools\\MSVC\\14.38.33130\\lib\\x64" '
                 '/libpath:"C:\\Program Files (x86)\\Windows Kits\\10\\lib\\10.0.22621.0\\um\\x64"\n')
        rejected = [
            valid.replace("14.38.33130", "14.44.35207"),
            valid.replace("10.0.22621.0", "10.0.26100.0"),
            valid + valid.replace("14.38.33130", "14.44.35207"),
            valid + " -D_ALLOW_COMPILER_AND_STL_VERSION_MISMATCH=1",
            valid.replace("/Brepro", ""),
            valid.replace("-fdebug-compilation-dir=.", ""),
            valid.replace("../../flutter/buildtools/windows-x64/clang/bin/lld-link.exe", "link.exe"),
            valid.replace("../../flutter/buildtools/windows-x64/clang/bin/lld-link.exe", "C:/unreviewed/lld-link.exe"),
            "clang-cl without auditable toolchain paths",
        ]
        with tempfile.TemporaryDirectory() as directory:
            commands = Path(directory) / "commands.txt"
            commands.write_text(valid, encoding="utf-8")
            source.verify_commands(commands, lock)
            for text in rejected:
                with self.subTest(commands=text):
                    commands.write_text(text, encoding="utf-8")
                    with self.assertRaises(ValueError):
                        source.verify_commands(commands, lock)

    def test_final_link_response_rejects_timestamp_and_incremental_drift(self):
        valid = "objects.obj /Brepro /PDBALTPATH:%_PDB% /INCREMENTAL:NO /DEBUG /threads:1"
        with tempfile.TemporaryDirectory() as directory:
            response = Path(directory) / "link-response.txt"
            response.write_text(valid, encoding="utf-8")
            source.verify_link_response(response)
            for text in (valid.replace("/Brepro", ""), valid.replace("/PDBALTPATH:%_PDB%", ""),
                         valid.replace("/INCREMENTAL:NO", "/INCREMENTAL"), valid + " /INCREMENTAL:YES",
                         valid.replace("/threads:1", ""), valid.replace("/threads:1", "/threads:4"),
                         valid + " /threads:4"):
                with self.subTest(response=text):
                    response.write_text(text, encoding="utf-8")
                    with self.assertRaises(ValueError):
                        source.verify_link_response(response)


if __name__ == "__main__":
    unittest.main()
