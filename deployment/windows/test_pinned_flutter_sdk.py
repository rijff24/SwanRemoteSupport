"""Reject SDK drift and tampering before extraction or CI environment changes."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


@unittest.skipUnless(shutil.which("pwsh"), "PowerShell 7 is required")
class PinnedFlutterSdkTests(unittest.TestCase):
    def reject(self, version, message):
        script = Path(__file__).with_name("install-pinned-flutter.ps1")
        with tempfile.TemporaryDirectory(prefix="swan-sdk-negative-") as folder:
            root = Path(folder)
            archive = root / "untrusted.zip"
            archive.write_bytes(b"An altered or unrelated SDK must not be extracted")
            destination = root / "sdk"
            environment = os.environ.copy()
            exports = [root / "environment", root / "path"]
            for name, path in zip(("GITHUB_ENV", "GITHUB_PATH"), exports):
                path.write_text("unchanged\n")
                environment[name] = str(path)
            result = subprocess.run(
                ["pwsh", "-NoProfile", "-NonInteractive", "-File", str(script),
                 "-Destination", str(destination), "-Archive", str(archive),
                 "-ExpectedVersion", version],
                env=environment, capture_output=True, text=True, timeout=30,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(message, result.stdout + result.stderr)
            self.assertFalse(destination.exists())
            self.assertEqual(archive.read_bytes(), b"An altered or unrelated SDK must not be extracted")
            self.assertTrue(all(path.read_text() == "unchanged\n" for path in exports))

    def test_untrusted_archive_cannot_install_or_export_environment(self):
        self.reject("3.24.5", "Pinned Flutter archive hash mismatch")

    def test_other_version_cannot_fall_back_to_latest(self):
        self.reject("3.47.8", "Requested Flutter version differs from the pinned SDK")


if __name__ == "__main__":
    unittest.main()
