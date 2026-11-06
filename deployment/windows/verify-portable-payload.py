"""Verify the bytes embedded in a portable EXE against its signed-inner manifest.

This reads the package; it never runs or extracts its executable payload.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import struct

import brotli


def read_payload(raw, start):
    position = start + 8
    files = {}
    folded = set()
    total = 0
    while raw[position:position + 8] != b"rustdesk":
        length = struct.unpack_from(">I", raw, position)[0]
        position += 4
        if not 0 < length <= 240:
            raise ValueError("Invalid portable path length")
        name = raw[position:position + length].decode("utf-8").replace("\\", "/")
        position += length
        if name.startswith("./"):
            name = name[2:]
        path = PurePosixPath(name)
        if (not name or name != path.as_posix() or path.is_absolute() or ":" in name or name.casefold() in folded
                or any(part in (".", "..") or part.endswith((" ", "."))
                       or not re.fullmatch(r"[A-Za-z0-9 _\-.@+]+", part) for part in path.parts)):
            raise ValueError("Unsafe or duplicate portable payload path")
        folded.add(name.casefold())
        size = struct.unpack_from(">I", raw, position)[0]
        position += 4
        if not 0 < size <= 512 * 1024**2 or position + size + 32 > len(raw):
            raise ValueError("Invalid portable compressed size")
        decoder = brotli.Decompressor()
        digest = hashlib.sha256()
        checksum = hashlib.md5()
        count = 0
        for offset in range(0, size, 64):
            part = decoder.process(raw[position + offset:position + min(size, offset + 64)])
            count += len(part)
            total += len(part)
            if count > 128 * 1024**2 or total > 512 * 1024**2:
                raise ValueError("Portable payload exceeds limits")
            digest.update(part)
            checksum.update(part)
        position += size
        if not decoder.is_finished() or checksum.hexdigest().encode() != raw[position:position + 32]:
            raise ValueError("Portable payload checksum differs")
        position += 32
        files[name] = digest.hexdigest()
        if len(files) > 4096:
            raise ValueError("Too many portable files")
    entrypoint = raw[position + 8:]
    # The linked PE contains data after the embedded container.
    names = (b"Swan Remote Support.exe", b".\\Swan Remote Support.exe",
             b"./Swan Remote Support.exe")
    if not entrypoint.startswith(names):
        raise ValueError("Unexpected portable entry point")
    return files


def verify(package, manifest):
    if package.stat().st_size > 512 * 1024**2:
        raise ValueError("Portable installer exceeds limit")
    expected = {item["path"]: item["sha256"] for item in manifest}
    if len(expected) != len(manifest) or not {
            "Swan Remote Support.exe", "librustdesk.dll", "flutter_windows.dll"} <= set(expected):
        raise ValueError("Incomplete installed manifest")
    raw = package.read_bytes()
    position = 0
    while True:
        position = raw.find(b"rustdesk", position)
        if position < 0:
            raise ValueError("No embedded payload matches the signed-inner manifest")
        try:
            actual = read_payload(raw, position)
        except (ValueError, UnicodeDecodeError, struct.error, brotli.error):
            position += 8
            continue
        if actual == expected:
            return {"package_sha256": hashlib.sha256(raw).hexdigest(),
                    "embedded_files": len(actual), "embedded_payload_matches_manifest": True,
                    "payload_executed": False}
        position += 8


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--package", type=Path, required=True)
    p.add_argument("--manifest", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    a = p.parse_args()
    record = verify(a.package, json.loads(a.manifest.read_text(encoding="utf-8-sig")))
    with a.output.open("x", encoding="utf-8") as stream:
        json.dump(record, stream, indent=2)
        stream.write("\n")
    print(json.dumps(record))


if __name__ == "__main__":
    main()
