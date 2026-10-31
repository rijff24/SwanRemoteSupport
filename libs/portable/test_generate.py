import importlib.util
import hashlib
import os
from pathlib import Path
import re
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("portable_generator", Path(__file__).with_name("generate.py"))
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


class PortablePackageTests(unittest.TestCase):
    def test_cache_identity_follows_payload_without_wall_clock_or_file_times(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = root / "data.bin"
            payload.write_bytes(b"first complete application payload")
            generator.write_app_metadata(directory)
            first = (root / "app_metadata.toml").read_bytes()
            payload.touch()
            generator.write_app_metadata(directory)
            self.assertEqual(first, (root / "app_metadata.toml").read_bytes())
            payload.write_bytes(b"changed complete application payload")
            generator.write_app_metadata(directory)
            second = (root / "app_metadata.toml").read_bytes()
            self.assertNotEqual(first, second)
            for value in (first, second):
                match = re.fullmatch(rb"timestamp = ([1-9][0-9]*)\npayload_sha256 = ([a-f0-9]{64})\n", value)
                self.assertIsNotNone(match)
                self.assertLessEqual(int(match[1]), (1 << 63) - 1)
            self.assertIn(hashlib.sha256(payload.read_bytes()).hexdigest().encode(), second)

    def test_package_bytes_and_cache_identity_ignore_dictionary_insertion_order(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            entries = {"./z.dll": (b"compressed-z", b"0" * 32), "./a.exe": (b"compressed-a", b"1" * 32)}
            generator.write_package_metadata(entries, directory, "./a.exe")
            generator.write_app_metadata(directory)
            first = ((root / "data.bin").read_bytes(), (root / "app_metadata.toml").read_bytes())
            generator.write_package_metadata(dict(reversed(list(entries.items()))), directory, "./a.exe")
            generator.write_app_metadata(directory)
            self.assertEqual(first, ((root / "data.bin").read_bytes(), (root / "app_metadata.toml").read_bytes()))

    def test_real_payload_compression_and_executable_selection_remain_bound(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            source.mkdir()
            (source / "app.exe").write_bytes(b"application bytes")
            (source / "assets").mkdir()
            (source / "assets" / "brand.txt").write_bytes(b"company branding")
            table = generator.generate_md5_table(str(source), 1)
            self.assertEqual(len(table), 2)
            for compressed, _ in table.values():
                self.assertIn(generator.brotli.decompress(compressed), (b"application bytes", b"company branding"))
            generator.write_package_metadata(table, directory, "./app.exe")
            generator.write_app_metadata(directory)
            first = (root / "app_metadata.toml").read_bytes()
            generator.write_package_metadata(table, directory, "./another.exe")
            generator.write_app_metadata(directory)
            self.assertNotEqual(first, (root / "app_metadata.toml").read_bytes())

    def test_failed_compression_restores_the_callers_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            (Path(directory) / "app.exe").write_bytes(b"application bytes")
            previous = os.getcwd()
            with patch.object(generator.brotli, "compress", side_effect=RuntimeError("compression failed")):
                with self.assertRaisesRegex(RuntimeError, "compression failed"):
                    generator.generate_md5_table(directory, 1)
            self.assertEqual(previous, os.getcwd())


if __name__ == "__main__":
    unittest.main()
