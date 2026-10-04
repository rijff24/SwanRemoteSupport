"""Generate release installed_files input after final binary signing.

Unsigned CI output is only a draft and must be regenerated after signing.
This script never signs metadata or executes payload files.
"""
import argparse
import hashlib
import json
from pathlib import Path


def digest(file):
    if file.stat().st_size > 512 * 1024 * 1024:
        raise ValueError("Payload file exceeds limit")
    with file.open("rb") as stream:
        result = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(chunk)
        return result.hexdigest()


def manifest(edition, source):
    source = source.resolve(strict=True)
    if edition == "technician":
        if not source.is_file() or source.suffix.lower() != ".exe":
            raise ValueError("Technician source must be the complete portable EXE")
        return [{"path": "SwanRemoteSupport-Technician.exe", "sha256": digest(source)}]
    if not source.is_dir():
        raise ValueError("Customer source must be the final Flutter payload directory")
    files = []
    names = set()
    for file in sorted(source.rglob("*")):
        if file.is_symlink() or getattr(file.lstat(), "st_file_attributes", 0) & 0x400:
            raise ValueError("Payload links are not allowed")
        if not file.is_file():
            continue
        name = file.relative_to(source).as_posix()
        if name.lower() == "rustdesk.exe":
            name = "Swan Remote Support.exe"
        if name.lower() in names or name.lower() == "runtimebroker_rustdesk.exe":
            raise ValueError("Duplicate or Windows-owned payload file")
        names.add(name.lower())
        files.append({"path": name, "sha256": digest(file)})
        if len(files) > 4096:
            raise ValueError("Payload manifest exceeds limit")
    if not {"swan remote support.exe", "librustdesk.dll", "flutter_windows.dll"} <= names:
        raise ValueError("Required customer executable or native libraries missing")
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--edition", choices=("customer", "technician"), required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    files = manifest(args.edition, args.source)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(files, stream, indent=2)
        stream.write("\n")
    print(f"Generated {len(files)} installed payload identities; metadata is unsigned.")


if __name__ == "__main__":
    main()
