"""Check Windows x64 build metadata; a REPRO marker alone is not a rebuild comparison."""
import argparse
import hashlib
import json
from pathlib import Path
import struct


def verify(path, require_dll=False):
    if not 0 < path.stat().st_size < 512 * 1024 * 1024:
        raise ValueError("Unexpected Windows binary size")
    data = path.read_bytes()

    def unpack(fmt, offset):
        if offset < 0 or offset + struct.calcsize(fmt) > len(data):
            raise ValueError("Truncated Windows binary metadata")
        return struct.unpack_from(fmt, data, offset)

    if data[:2] != b"MZ":
        raise ValueError("Expected a Windows PE binary")
    pe, = unpack("<I", 60)
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("Missing PE signature")
    machine, sections = unpack("<HH", pe + 4)
    optional_size, characteristics = unpack("<HH", pe + 20)
    optional = pe + 24
    magic, = unpack("<H", optional)
    if machine != 0x8664 or magic != 0x20b or optional_size < 168 or not 0 < sections <= 96:
        raise ValueError("Expected Windows x64 metadata")
    if require_dll and not characteristics & 0x2000:
        raise ValueError("Expected a DLL")
    directories, = unpack("<I", optional + 108)
    if directories < 7:
        raise ValueError("Missing PE debug directory")
    rva, size = unpack("<II", optional + 112 + 6 * 8)
    if not rva or not size or size % 28 or size > 28 * 128:
        raise ValueError("Invalid PE debug directory")
    debug = None
    for index in range(sections):
        section = optional + optional_size + index * 40
        virtual_size, virtual_address, raw_size, raw_pointer = unpack("<IIII", section + 8)
        if virtual_address <= rva and rva + size <= virtual_address + max(virtual_size, raw_size):
            delta = rva - virtual_address
            if delta + size > raw_size:
                raise ValueError("Debug directory exceeds file-backed section")
            debug = raw_pointer + delta
            break
    if debug is None or debug + size > len(data):
        raise ValueError("PE debug directory is outside the binary")
    types, pdb_names = [], []
    for index in range(size // 28):
        kind, length, _, pointer = unpack("<IIII", debug + index * 28 + 12)
        types.append(kind)
        if length and (pointer + length > len(data) or pointer < 0):
            raise ValueError("Debug payload is outside the binary")
        if kind == 2:
            if length < 25 or data[pointer:pointer + 4] != b"RSDS":
                raise ValueError("Expected CodeView debug information")
            name = data[pointer + 24:pointer + length].rstrip(b"\0").decode("utf-8")
            if not name.lower().endswith(".pdb") or any(character in name for character in ("/", "\\", ":", "\0")):
                raise ValueError("Debug information embeds a build-machine PDB path")
            pdb_names.append(name)
    if 16 not in types:
        raise ValueError("Missing IMAGE_DEBUG_TYPE_REPRO build marker")
    return {"sha256": hashlib.sha256(data).hexdigest(), "machine": "x64",
            "is_dll": bool(characteristics & 0x2000), "repro_marker": True,
            "pdb_names": pdb_names, "independent_rebuild_verified": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path)
    parser.add_argument("--dll", action="store_true")
    arguments = parser.parse_args()
    print(json.dumps(verify(arguments.path, arguments.dll)))
