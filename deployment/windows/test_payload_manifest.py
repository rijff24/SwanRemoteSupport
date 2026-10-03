import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("generate-payload-manifest.py")
SPEC = importlib.util.spec_from_file_location("payload_manifest", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class PayloadManifestTests(unittest.TestCase):
    def test_customer_manifest_maps_executable_and_covers_libraries_and_assets(self):
        with tempfile.TemporaryDirectory(prefix="swan-manifest-") as directory:
            root = Path(directory)
            for name in ["rustdesk.exe", "librustdesk.dll", "flutter_windows.dll", "data/assets.json"]:
                file = root / name
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_bytes(name.encode())
            files = {file["path"]: file["sha256"] for file in MODULE.manifest("customer", root)}
            self.assertEqual(len(files), 4)
            self.assertEqual(files["Swan Remote Support.exe"], MODULE.digest(root / "rustdesk.exe"))
            self.assertIn("data/assets.json", files)
            (root / "flutter_windows.dll").unlink()
            with self.assertRaises(ValueError):
                MODULE.manifest("customer", root)

    def test_technician_manifest_covers_complete_packed_executable(self):
        with tempfile.TemporaryDirectory(prefix="swan-manifest-") as directory:
            file = Path(directory) / "technician.exe"
            file.write_bytes(b"portable packed executable fixture")
            self.assertEqual(MODULE.manifest("technician", file), [{"path": "SwanRemoteSupport-Technician.exe", "sha256": MODULE.digest(file)}])

    def test_cli_generates_json_without_overwriting_previous_output(self):
        with tempfile.TemporaryDirectory(prefix="swan-manifest-") as directory:
            file = Path(directory) / "technician.exe"
            output = Path(directory) / "manifest.json"
            file.write_bytes(b"portable fixture")
            command = [sys.executable, str(SCRIPT), "--edition", "technician", "--source", str(file), "--output", str(output)]
            self.assertEqual(subprocess.run(command, capture_output=True).returncode, 0)
            original = output.read_bytes()
            self.assertEqual(json.loads(original)[0]["sha256"], MODULE.digest(file))
            self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)
            self.assertEqual(output.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
