import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import urllib.error
import zipfile

spec = importlib.util.spec_from_file_location("native_comparison", Path(__file__).with_name("compare-native-builds.py"))
comparison = importlib.util.module_from_spec(spec)
spec.loader.exec_module(comparison)


class NativeBuildComparisonTests(unittest.TestCase):
    def repository(self, root, source=b"public source\n"):
        root.mkdir()
        hooks = root / "empty-hooks"
        hooks.mkdir()
        args = ["git", "-C", str(root), "-c", "core.autocrlf=false", "-c", "core.hooksPath=" + str(hooks),
                "-c", "commit.gpgsign=false", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid"]
        subprocess.run(args + ["init", "-q"], check=True, capture_output=True)
        (root / "source.txt").write_bytes(source)
        (root / "Cargo.toml").write_bytes(b'[package]\nversion = "1.5.0"\n')
        subprocess.run(args + ["add", "source.txt", "Cargo.toml"], check=True, capture_output=True)
        subprocess.run(args + ["commit", "-q", "-m", "fixture"], check=True, capture_output=True)
        revision = comparison.git(root, "rev-parse", "HEAD").decode().strip()
        return revision, int(comparison.git(root, "show", "-s", "--format=%ct", revision))

    def package(self, path, revision, epoch, source=b"public source\n", binary=b"MZ executable\n", bad_sum=False, omit=None):
        files = {"source.txt": source, "Cargo.toml": b'[package]\nversion = "1.5.0"\n', **{n: ("generated:" + n).encode() for n in comparison.BRIDGE}}
        manifest = {"repositories": {".": revision}, "source_date_epoch": epoch,
                    "generated_bridge_files": list(comparison.BRIDGE),
                    "files": {n: {"sha256": hashlib.sha256(v).hexdigest(), "mode": "100644"} for n, v in files.items()}}
        memory = io.BytesIO()
        with zipfile.ZipFile(memory, "w") as z:
            z.writestr("BUILD-SOURCE.json", json.dumps(manifest))
            for name, data in files.items():
                z.writestr("source/" + name, data)
        shipping = {n: b"MSI fixture" if n.endswith(".msi") else b"MZ fixture" for n in comparison.shipping_names("1.5.0")}
        shipping["SwanRemoteSupport-1.5.0-x64-unsigned-install.exe"] = binary
        if omit is not None:
            shipping.pop(omit)
        with zipfile.ZipFile(path, "w") as z:
            z.writestr("SwanRemoteSupport-build-source.zip", memory.getvalue())
            for name, data in shipping.items():
                z.writestr(name, data)
            sums = "\n".join(("0" * 64 if bad_sum else hashlib.sha256(v).hexdigest()) + "  " + n for n, v in shipping.items())
            z.writestr("SHA256SUMS.txt", sums)

    def test_source_checkout_line_endings_do_not_normalize_shipping_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision, epoch = self.repository(root / "source")
            a, b = root / "first.zip", root / "second.zip"
            self.package(a, revision, epoch)
            self.package(b, revision, epoch, source=b"public source\r\n", binary=b"MZ executable\r\n")
            report = comparison.compare(a, b, root / "source", revision)
            self.assertFalse(report["all_shipping_bytes_identical"])
            self.assertEqual(report["source_proofs"][1]["declared_source_text_checkout_conversions"], ["source.txt"])
            self.assertFalse(report["normalization_applied_to_shipping_files"])
            self.assertFalse(report["executables_run"])

    def test_original_source_cannot_be_replaced_by_a_self_consistent_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision, epoch = self.repository(root / "source")
            a, b = root / "first.zip", root / "second.zip"
            self.package(a, revision, epoch)
            self.package(b, revision, epoch, source=b"altered source\n")
            with self.assertRaisesRegex(comparison.VerificationError, "public source blob"):
                comparison.compare(a, b, root / "source", revision)

    def test_binary_source_checkout_bytes_are_not_normalized(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision, epoch = self.repository(root / "source", b"binary\0\n")
            a, b = root / "first.zip", root / "second.zip"
            self.package(a, revision, epoch, source=b"binary\0\n")
            self.package(b, revision, epoch, source=b"binary\0\r\n")
            with self.assertRaisesRegex(comparison.VerificationError, "public source blob"):
                comparison.compare(a, b, root / "source", revision)

    def test_declared_shipping_hash_mismatch_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision, epoch = self.repository(root / "source")
            a, b = root / "first.zip", root / "second.zip"
            self.package(a, revision, epoch)
            self.package(b, revision, epoch, bad_sum=True)
            with self.assertRaisesRegex(comparison.VerificationError, "artifact file hash"):
                comparison.compare(a, b, root / "source", revision)

    def test_exact_original_shipping_bytes_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision, epoch = self.repository(root / "source")
            a, b = root / "first.zip", root / "second.zip"
            self.package(a, revision, epoch)
            self.package(b, revision, epoch)
            report = comparison.compare(a, b, root / "source", revision)
            self.assertTrue(report["all_shipping_bytes_identical"])
            self.assertEqual(len(report["outputs"]), 8)
            self.assertFalse(report["production_approved"])

    def test_matching_incomplete_output_sets_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision, epoch = self.repository(root / "source")
            a, b = root / "first.zip", root / "second.zip"
            self.package(a, revision, epoch, omit="swan-management.exe")
            self.package(b, revision, epoch, omit="swan-management.exe")
            with self.assertRaisesRegex(comparison.VerificationError, "output sets"):
                comparison.compare(a, b, root / "source", revision)

    def test_untrusted_download_origin_rejected_before_storage_request(self):
        revision = "a" * 40
        metadata = [
            {"head_sha": revision, "status": "completed", "conclusion": "success", "path": ".github/workflows/swan-windows-build.yml"},
            {"artifacts": [{"name": "candidate", "expired": False, "id": 5, "workflow_run": {"head_sha": revision},
                            "digest": "sha256:" + "0" * 64, "size_in_bytes": 1}]},
        ]
        with tempfile.TemporaryDirectory() as directory, patch.object(comparison, "api_json", side_effect=metadata), patch.object(comparison.urllib.request, "build_opener") as factory:
            factory.return_value.open.side_effect = urllib.error.HTTPError("https://api.github.com/example", 302, "redirect", {"Location": "https://untrusted.example/archive"}, None)
            target = Path(directory) / "original.zip"
            with self.assertRaisesRegex(comparison.VerificationError, "download origin"):
                comparison.fetch("owner/repository", 1, revision, "candidate", "not-a-real-token", target)
            self.assertFalse(target.exists())
            self.assertEqual(factory.call_count, 1)

    def test_download_digest_mismatch_retains_rejected_original_bytes(self):
        revision = "a" * 40
        metadata = [
            {"head_sha": revision, "status": "completed", "conclusion": "success", "path": ".github/workflows/swan-windows-build.yml"},
            {"artifacts": [{"name": "candidate", "expired": False, "id": 5, "workflow_run": {"head_sha": revision},
                            "digest": "sha256:" + hashlib.sha256(b"expected").hexdigest(), "size_in_bytes": 8}]},
        ]
        with tempfile.TemporaryDirectory() as directory, patch.object(comparison, "api_json", side_effect=metadata), patch.object(comparison.urllib.request, "build_opener") as factory:
            factory.return_value.open.side_effect = [
                urllib.error.HTTPError("https://api.github.com/example", 302, "redirect", {"Location": "https://example.blob.core.windows.net/archive"}, None),
                io.BytesIO(b"tampered"),
            ]
            target = Path(directory) / "original.zip"
            with self.assertRaisesRegex(comparison.VerificationError, "archive digest"):
                comparison.fetch("owner/repository", 1, revision, "candidate", "not-a-real-token", target)
            self.assertEqual(target.read_bytes(), b"tampered")
            storage_argument = factory.return_value.open.call_args_list[1].args[0]
            self.assertIsInstance(storage_argument, str)


if __name__ == "__main__":
    unittest.main()
