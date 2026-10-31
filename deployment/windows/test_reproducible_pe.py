import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("repro_pe", Path(__file__).with_name("verify-reproducible-pe.py"))
pe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pe)


def fixture():
    data = bytearray(1024)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 60, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", data, 0x84, 0x8664, 1)
    struct.pack_into("<HH", data, 0x94, 240, 0x2000)
    struct.pack_into("<H", data, 0x98, 0x20b)
    struct.pack_into("<I", data, 0x98 + 108, 16)
    struct.pack_into("<II", data, 0x98 + 160, 0x1000, 56)
    struct.pack_into("<IIII", data, 0x98 + 240 + 8, 512, 0x1000, 512, 512)
    struct.pack_into("<IIII", data, 512 + 12, 16, 0, 0, 0)
    codeview = b"RSDS" + b"0" * 16 + struct.pack("<I", 1) + b"WindowInjection.pdb\0"
    struct.pack_into("<IIII", data, 540 + 12, 2, len(codeview), 0x1040, 576)
    data[576:576 + len(codeview)] = codeview
    return data


class ReproduciblePeTests(unittest.TestCase):
    def test_marker_and_pdb_basename_do_not_claim_a_rebuild(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "helper.dll"
            path.write_bytes(fixture())
            result = pe.verify(path, True)
            self.assertTrue(result["repro_marker"])
            self.assertEqual(result["pdb_names"], ["WindowInjection.pdb"])
            self.assertFalse(result["independent_rebuild_verified"])

    def test_missing_marker_absolute_paths_and_malformed_pe_are_rejected(self):
        changes = []
        data = fixture(); struct.pack_into("<I", data, 524, 14); changes.append(data)
        data = fixture(); data[600:602] = b"C:"; changes.append(data)
        data = fixture(); struct.pack_into("<H", data, 0x84, 0x14c); changes.append(data)
        data = fixture(); struct.pack_into("<II", data, 0x98 + 160, 0x2000, 56); changes.append(data)
        changes.extend((fixture()[:300], b"not a PE"))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "helper.dll"
            for value in changes:
                with self.subTest(length=len(value)):
                    path.write_bytes(value)
                    with self.assertRaises(ValueError):
                        pe.verify(path, True)


if __name__ == "__main__":
    unittest.main()
