import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("service_imports", Path(__file__).with_name("verify-service-imports.py"))
imports = importlib.util.module_from_spec(spec)
spec.loader.exec_module(imports)


def fixture(eager=b"KERNEL32.dll", delayed=b"flutter_windows.dll"):
    data = bytearray(2048)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 60, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", data, 0x84, 0x8664, 1)
    struct.pack_into("<H", data, 0x94, 240)
    struct.pack_into("<H", data, 0x98, 0x20b)
    struct.pack_into("<I", data, 0x98 + 108, 16)
    struct.pack_into("<IIII", data, 0x98 + 240 + 8, 1536, 0x1000, 1536, 512)
    struct.pack_into("<II", data, 0x98 + 120, 0x1000, 40)
    struct.pack_into("<IIIII", data, 512, 0, 0, 0, 0x1080, 0)
    data[640:640 + len(eager) + 1] = eager + b"\0"
    struct.pack_into("<II", data, 0x98 + 112 + 13 * 8, 0x1100, 64)
    struct.pack_into("<IIIIIIII", data, 768, 1, 0x1180, 0, 0, 0, 0, 0, 0)
    data[896:896 + len(delayed) + 1] = delayed + b"\0"
    return data


class ServiceImportTests(unittest.TestCase):
    def test_delay_loaded_engine_does_not_claim_cold_boot_acceptance(self):
        with tempfile.TemporaryDirectory() as directory:
            bundle = Path(directory)
            (bundle / "flutter_windows.dll").touch()
            path = bundle / "app.exe"
            path.write_bytes(fixture())
            result = imports.verify(path, bundle)
            self.assertTrue(result["no_eager_bundled_desktop_dlls"])
            self.assertFalse(result["cold_boot_runtime_verified"])

    def test_eager_engine_plugins_and_missing_delay_engine_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            bundle = Path(directory)
            for name in ("flutter_windows.dll", "window_manager_plugin.dll"):
                (bundle / name).touch()
            path = bundle / "app.exe"
            for data in (fixture(eager=b"FLUTTER_WINDOWS.dll"),
                         fixture(eager=b"window_manager_plugin.dll"),
                         fixture(delayed=b"other.dll")):
                with self.subTest(data=data[640:665]):
                    path.write_bytes(data)
                    with self.assertRaises(ValueError):
                        imports.verify(path, bundle)

    def test_bad_delay_descriptors_and_file_rvas_are_rejected(self):
        values = []
        data = fixture(); struct.pack_into("<I", data, 768, 0); values.append(data)
        data = fixture(); struct.pack_into("<I", data, 772, 0x9000); values.append(data)
        data = fixture(); struct.pack_into("<I", data, 0x98 + 112 + 13 * 8 + 4, 32); values.append(data)
        values.append(fixture()[:600])
        with tempfile.TemporaryDirectory() as directory:
            bundle = Path(directory)
            (bundle / "flutter_windows.dll").touch()
            path = bundle / "app.exe"
            for data in values:
                path.write_bytes(data)
                with self.assertRaises(ValueError):
                    imports.verify(path, bundle)


if __name__ == "__main__":
    unittest.main()
