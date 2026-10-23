"""Inspect original differing portable payloads without executing or normalizing them."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import struct
import zipfile

import brotli

spec = importlib.util.spec_from_file_location("comparison", Path(__file__).with_name("compare-native-builds.py"))
comparison = importlib.util.module_from_spec(spec)
spec.loader.exec_module(comparison)


def payload(data):
    start = 0
    while True:
        start = data.find(b"rustdesk", start)
        if start < 0:
            raise ValueError("No verified portable payload")
        try:
            position, files, total = start + 8, {}, 0
            while data[position:position + 8] != b"rustdesk":
                length, = struct.unpack_from(">I", data, position)
                position += 4
                if not 0 < length <= 240:
                    raise ValueError("Invalid portable path length")
                name = data[position:position + length].decode("utf-8").replace("\\", "/")
                position += length
                if name.startswith("./"):
                    name = name[2:]
                if (not name or any(not re.fullmatch(r"[A-Za-z0-9 _\-.@+]+", part)
                                   or part in (".", "..") or part.endswith((".", " ")) for part in name.split("/"))
                        or name.casefold() in {n.casefold() for n in files}):
                    raise ValueError("Unsafe portable path")
                size, = struct.unpack_from(">I", data, position)
                position += 4
                if not 0 < size <= 512 * 1024 ** 2 or position + size + 32 > len(data):
                    raise ValueError("Invalid portable size")
                decoder, decoded = brotli.Decompressor(), bytearray()
                for offset in range(0, size, 64):
                    decoded.extend(decoder.process(data[position + offset:position + min(size, offset + 64)]))
                    if len(decoded) > 512 * 1024 ** 2:
                        raise ValueError("Portable decompression bound")
                position += size
                if not decoder.is_finished() or hashlib.md5(decoded).hexdigest().encode() != data[position:position + 32]:
                    raise ValueError("Portable checksum mismatch")
                position += 32
                total += len(decoded)
                if total > 512 * 1024 ** 2 or len(files) >= 4096:
                    raise ValueError("Portable total bound")
                files[name] = bytes(decoded)
            if not data[position + 8:].startswith((b".\\rustdesk.exe", b"./rustdesk.exe")):
                raise ValueError("Portable trailer differs")
            if "rustdesk.exe" not in files:
                raise ValueError("Portable executable missing")
            return files
        except (ValueError, struct.error, brotli.error):
            start += 8


def pe_metadata(data):
    if data[:2] != b"MZ":
        raise ValueError("Expected PE")
    pe, = struct.unpack_from("<I", data, 60)
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("Missing PE header")
    count, = struct.unpack_from("<H", data, pe + 6)
    optional_size, = struct.unpack_from("<H", data, pe + 20)
    optional = pe + 24
    if not 0 < count <= 96 or struct.unpack_from("<H", data, optional)[0] != 0x20b:
        raise ValueError("Expected bounded x64 section table")
    sections, debug = {}, []
    debug_rva, debug_size = struct.unpack_from("<II", data, optional + 112 + 6 * 8)
    for index in range(count):
        offset = optional + optional_size + index * 40
        name = data[offset:offset + 8].rstrip(b"\0").decode("ascii")
        virtual_size, address, size, pointer = struct.unpack_from("<IIII", data, offset + 8)
        if pointer + size > len(data):
            raise ValueError("Section exceeds file")
        sections[name] = {"bytes": size, "sha256": hashlib.sha256(data[pointer:pointer + size]).hexdigest()}
        if address <= debug_rva and debug_rva + debug_size <= address + min(virtual_size, size):
            if debug_size % 28 or debug_size > 28 * 128:
                raise ValueError("Invalid debug directory")
            start = pointer + debug_rva - address
            for entry in range(debug_size // 28):
                kind, length, _, position = struct.unpack_from("<IIII", data, start + entry * 28 + 12)
                if position + length > len(data):
                    raise ValueError("Debug payload exceeds file")
                raw = data[position:position + length]
                item = {"type": kind, "sha256": hashlib.sha256(raw).hexdigest()}
                if kind == 2 and raw.startswith(b"RSDS") and len(raw) >= 25:
                    item.update(guid=raw[4:20].hex(), age=struct.unpack_from("<I", raw, 20)[0],
                                pdb=raw[24:].rstrip(b"\0").decode("utf-8"))
                debug.append(item)
    rich_entries = []
    rich = data.find(b"Rich", 64, pe)
    if rich >= 0 and rich + 8 <= pe:
        key, = struct.unpack_from("<I", data, rich + 4)
        for start in range(64, rich, 4):
            if struct.unpack_from("<I", data, start)[0] ^ key == 0x536e6144:
                for offset in range(start + 16, rich - 7, 8):
                    product, count = struct.unpack_from("<II", data, offset)
                    product ^= key
                    rich_entries.append({"product": product >> 16, "build": product & 65535, "count": count ^ key})
                break
    return {"coff_timestamp": struct.unpack_from("<I", data, pe + 8)[0],
            "linker_version": list(data[optional + 2:optional + 4]), "sections": sections,
            "debug": debug, "rich_entries": rich_entries}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError("Retain previous diagnostic evidence")
    report = json.loads(args.report.read_text())
    names = [o for o in report["outputs"] if o["path"].endswith("-x64-unsigned-install.exe")]
    if len(names) != 1 or report["normalization_applied_to_shipping_files"]:
        raise ValueError("Expected original-byte comparison report")
    files = []
    for index, label in enumerate(("first", "second")):
        archive = args.work / (label + ".zip")
        with archive.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if "sha256:" + digest != report["builds"][index]["archive_digest"]:
            raise ValueError("Original archive digest changed")
        with zipfile.ZipFile(archive) as bundle:
            comparison.validate_entries(bundle)
            raw = bundle.read(names[0]["path"])
            if hashlib.sha256(raw).hexdigest() != names[0][label + "_sha256"]:
                raise ValueError("Original portable executable changed")
            files.append(payload(raw))
    if set(files[0]) != set(files[1]):
        raise ValueError("Payload file sets differ")
    args.output.mkdir(parents=True)
    differences = []
    for name in sorted(files[0]):
        first, second = files[0][name], files[1][name]
        if first == second:
            continue
        item = {"path": name, "first_sha256": hashlib.sha256(first).hexdigest(),
                "second_sha256": hashlib.sha256(second).hexdigest()}
        if name.endswith((".exe", ".dll")):
            item.update(first_metadata=pe_metadata(first), second_metadata=pe_metadata(second))
            for label, raw in (("first", first), ("second", second)):
                target = args.output / label / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(raw)
        differences.append(item)
    result = {"source": report["source_commit"], "different_payloads": differences,
              "shipping_normalized": False, "executables_run": False, "runtime_qualified": False}
    (args.output / "payload-diagnostic.json").write_text(json.dumps(result, indent=2))
    print(json.dumps({"different_payloads": len(differences), "executables_run": False}))


if __name__ == "__main__":
    main()
