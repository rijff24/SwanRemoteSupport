import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile

import brotli


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


staging = module("customer_exe_staging", "prepare-customer-exe.py")
portable = module("customer_exe_verification", "verify-portable-payload.py")


class CustomerExeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.payload = self.root / "payload"
        self.payload.mkdir()
        self.files = {"rustdesk.exe": b"signed inner executable bytes",
                      "librustdesk.dll": b"native library", "flutter_windows.dll": b"engine"}
        for name, data in self.files.items():
            (self.payload / name).write_bytes(data)
        self.manifest = [{"path": "Swan Remote Support.exe" if n == "rustdesk.exe" else n,
                          "sha256": hashlib.sha256(v).hexdigest()} for n, v in self.files.items()]
        self.revision = "a" * 40

    def source_archive(self, extra=None):
        names = {"Cargo.toml", "Cargo.lock", ".cargo/config.toml", "libs/portable/Cargo.toml",
                 "libs/portable/generate.py", "libs/portable/build.rs", "res/icon.ico", "res/manifest.xml"}
        files = {name: ("source:" + name).encode() for name in names}
        files.update(extra or {})
        metadata = {"repositories": {".": self.revision}, "source_date_epoch": 1793661395,
                    "files": {n: {"sha256": hashlib.sha256(v).hexdigest()} for n, v in files.items()}}
        archive = self.root / "source.zip"
        with zipfile.ZipFile(archive, "x") as z:
            z.writestr("BUILD-SOURCE.json", json.dumps(metadata))
            for name, data in files.items():
                z.writestr("source/" + name, data)
        return archive, hashlib.sha256(archive.read_bytes()).hexdigest()

    def package(self, entries=None, entrypoint=b".\\rustdesk.exe"):
        raw = bytearray(b"not a PE; test the portable container only\0rustdesk")
        for name, content in (entries or self.files).items():
            encoded = ("./" + name).encode()
            compressed = brotli.compress(content)
            raw.extend(struct.pack(">I", len(encoded)) + encoded)
            raw.extend(struct.pack(">I", len(compressed)) + compressed + hashlib.md5(content).hexdigest().encode())
        raw.extend(b"rustdesk" + entrypoint)
        package = self.root / "container.bin"
        package.write_bytes(raw)
        return package

    def test_pinned_source_and_exact_signed_inner_bytes_are_staged_without_execution(self):
        archive, digest = self.source_archive()
        work = self.root / "work"
        record = staging.prepare(archive, digest, self.revision, self.payload, self.manifest, work)
        self.assertFalse(record["executables_run"])
        self.assertEqual(record["payload_files"], 3)
        self.assertEqual((work / "payload/rustdesk.exe").read_bytes(), self.files["rustdesk.exe"])
        self.assertEqual((work / "source/Cargo.lock").read_bytes(), b"source:Cargo.lock")
        with self.assertRaises(FileExistsError):
            staging.prepare(archive, digest, self.revision, self.payload, self.manifest, work)

    def test_wrong_source_hash_or_revision_is_rejected_before_staging(self):
        archive, digest = self.source_archive()
        for pin, revision in (("0" * 64, self.revision), (digest, "b" * 40)):
            work = self.root / "work"
            with self.assertRaises(ValueError):
                staging.prepare(archive, pin, revision, self.payload, self.manifest, work)
            self.assertFalse(work.exists())

    def test_tampered_inner_executable_cannot_be_wrapped_using_the_signed_manifest(self):
        archive, digest = self.source_archive()
        (self.payload / "rustdesk.exe").write_bytes(b"unsigned replacement")
        with self.assertRaises(ValueError):
            staging.prepare(archive, digest, self.revision, self.payload, self.manifest, self.root / "work")

    def test_source_archive_cannot_escape_or_alias_windows_paths(self):
        for name in ("../escape", "a//b", "C:/escape", "a/CON.txt", "a/b."):
            with self.subTest(name=name), self.assertRaises(ValueError):
                staging.safe_path(name)

    def test_exact_embedded_payload_passes_without_running_it(self):
        result = portable.verify(self.package(), self.manifest)
        self.assertTrue(result["embedded_payload_matches_manifest"])
        self.assertFalse(result["payload_executed"])
        self.assertEqual(result["embedded_files"], 3)

    def test_signing_only_outer_does_not_make_old_inner_bytes_match(self):
        changed = dict(self.files, **{"rustdesk.exe": b"earlier unsigned inner executable"})
        with self.assertRaises(ValueError):
            portable.verify(self.package(changed), self.manifest)

    def test_corrupted_payload_or_entrypoint_is_rejected(self):
        package = self.package()
        package.write_bytes(package.read_bytes().replace(hashlib.md5(self.files["rustdesk.exe"]).hexdigest().encode(), b"0" * 32))
        with self.assertRaises(ValueError):
            portable.verify(package, self.manifest)
        with self.assertRaises(ValueError):
            portable.verify(self.package(entrypoint=b"../another.exe"), self.manifest)

    def test_duplicate_installed_name_or_unsafe_embedded_path_is_rejected(self):
        for extra in ({"Swan Remote Support.exe": b"alias"}, {"../escape": b"outside"}):
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                portable.verify(self.package(dict(self.files, **extra)), self.manifest)


if __name__ == "__main__":
    unittest.main()
