"""Reject desktop DLLs eagerly loaded by the Windows service launcher."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct


def verify(path, bundle):
    if not 0 < path.stat().st_size < 512 * 1024 ** 2:
        raise ValueError("Unexpected launcher size")
    data = path.read_bytes()

    def unpack(fmt, at):
        if at < 0 or at + struct.calcsize(fmt) > len(data):
            raise ValueError("Truncated PE imports")
        return struct.unpack_from(fmt, data, at)

    if data[:2] != b"MZ":
        raise ValueError("Expected PE launcher")
    pe, = unpack("<I", 60)
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("Missing PE signature")
    machine, sections = unpack("<HH", pe + 4)
    size, = unpack("<H", pe + 20)
    optional = pe + 24
    magic, = unpack("<H", optional)
    directories, = unpack("<I", optional + 108)
    if machine != 0x8664 or magic != 0x20b or size < 224 or directories < 14 or not 0 < sections <= 96:
        raise ValueError("Expected x64 import directories")

    def raw(rva, length):
        for index in range(sections):
            section = optional + size + index * 40
            virtual_size, address, raw_size, pointer = unpack("<IIII", section + 8)
            if address <= rva and rva + length <= address + max(virtual_size, raw_size):
                delta = rva - address
                if delta + length <= raw_size and pointer + delta + length <= len(data):
                    return pointer + delta
        raise ValueError("Import points outside file-backed sections")

    def name(rva):
        at = raw(rva, 1)
        end = data.find(b"\0", at, min(at + 257, len(data)))
        if end < 0:
            raise ValueError("Unterminated import name")
        raw(rva, end - at + 1)
        value = data[at:end].decode("ascii")
        if not re.fullmatch(r"[A-Za-z0-9_.-]+", value):
            raise ValueError("Invalid import name")
        return value.lower()

    def imports(directory, width, delayed=False):
        rva, length = unpack("<II", optional + 112 + directory * 8)
        if not rva and not length:
            return set()
        if not rva or length < width or length > 1024 * 1024:
            raise ValueError("Invalid import directory")
        result = set()
        for index in range(min(length // width, 512)):
            row = unpack("<" + "I" * (width // 4), raw(rva + index * width, width))
            if not any(row):
                return result
            if delayed and row[0] != 1:
                raise ValueError("Expected RVA-based delay imports")
            result.add(name(row[1] if delayed else row[3]))
        raise ValueError("Unterminated import descriptors")

    eager = imports(1, 20)
    delayed = imports(13, 32, True)
    packaged = {p.name.lower() for p in bundle.glob("*.dll") if p.is_file()}
    if "flutter_windows.dll" not in packaged or "flutter_windows.dll" not in delayed:
        raise ValueError("Flutter engine must be packaged and delay loaded")
    if eager & packaged:
        raise ValueError("Service launcher eagerly loads bundled desktop DLLs: " + ", ".join(sorted(eager & packaged)))
    return {"sha256": hashlib.sha256(data).hexdigest(),
            "normal_imports": sorted(eager), "delay_imports": sorted(delayed),
            "no_eager_bundled_desktop_dlls": True,
            "cold_boot_runtime_verified": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path)
    parser.add_argument("--bundle", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(verify(args.path, args.bundle)))
