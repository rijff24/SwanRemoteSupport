"""Stage pinned corresponding source and an exact, already signed customer payload.

This is a release-maintainer tool, not an installer-worker profile command.
It never executes source, payload files, or downloaded tools.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import zipfile


def safe_path(name):
    path = PurePosixPath(name)
    if (not name or name != path.as_posix() or path.is_absolute() or "\\" in name or ":" in name
            or any(part in (".", "..") or part.endswith((" ", "."))
                   or any(ord(c) < 32 for c in part) for part in path.parts)):
        raise ValueError("Unsafe package path")
    reserved = {"CON", "PRN", "AUX", "NUL"} | {
        prefix + str(i) for prefix in ("COM", "LPT") for i in range(1, 10)}
    if any(part.split(".")[0].upper() in reserved for part in path.parts):
        raise ValueError("Windows-reserved package path")
    return path


def load_payload(source, manifest):
    if source.is_symlink() or getattr(source.lstat(), "st_file_attributes", 0) & 0x400:
        raise ValueError("Payload root must not be a link")
    expected = {}
    for item in manifest:
        if set(item) != {"path", "sha256"} or not re.fullmatch(r"[0-9a-f]{64}", item["sha256"]):
            raise ValueError("Invalid installed payload manifest")
        name = safe_path(item["path"]).as_posix()
        if name.casefold() in {n.casefold() for n in expected}:
            raise ValueError("Duplicate installed payload path")
        expected[name] = item["sha256"]
    if not 3 <= len(expected) <= 4096:
        raise ValueError("Unexpected customer payload count")
    actual = {}
    total = 0
    for file in sorted(source.rglob("*")):
        if file.is_symlink() or getattr(file.lstat(), "st_file_attributes", 0) & 0x400:
            raise ValueError("Payload links are not allowed")
        if not file.is_file():
            continue
        name = safe_path(file.relative_to(source).as_posix()).as_posix()
        installed = "Swan Remote Support.exe" if name == "rustdesk.exe" else name
        if installed not in expected or installed in actual:
            raise ValueError("Unexpected customer payload file")
        total += file.stat().st_size
        if file.stat().st_size > 128 * 1024**2 or total > 512 * 1024**2:
            raise ValueError("Customer payload exceeds limits")
        data = file.read_bytes()
        if hashlib.sha256(data).hexdigest() != expected[installed]:
            raise ValueError("Signed payload hash differs: " + installed)
        # The native installer copies the payload directory without renaming
        # its main executable. Stage its stable installed name before packing.
        actual[installed] = (installed, data)
    if set(actual) != set(expected) or not {
            "Swan Remote Support.exe", "librustdesk.dll", "flutter_windows.dll"} <= set(actual):
        raise ValueError("Incomplete customer payload")
    return actual


def prepare(archive, archive_sha256, revision, source, manifest, work):
    if not re.fullmatch(r"[0-9a-f]{64}", archive_sha256) or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("Exact archive and public source pins are required")
    with archive.open("rb") as stream:
        if hashlib.file_digest(stream, "sha256").hexdigest() != archive_sha256:
            raise ValueError("Corresponding source archive hash differs")
    payload = load_payload(source, manifest)
    staged = {}
    with zipfile.ZipFile(archive) as z:
        metadata = json.loads(z.read("BUILD-SOURCE.json"))
        if metadata["repositories"]["."] != revision:
            raise ValueError("Corresponding source revision differs")
        epoch = metadata["source_date_epoch"]
        if not isinstance(epoch, int) or not 315532800 <= epoch <= 4102444799:
            raise ValueError("Invalid source date epoch")
        expected = metadata["files"]
        if not 3 <= len(expected) <= 20000:
            raise ValueError("Invalid corresponding source manifest size")
        names = z.namelist()
        if len(names) != len(set(names)) or set(names) != {
                "BUILD-SOURCE.json"} | {"source/" + name for name in expected}:
            raise ValueError("Source archive has undeclared or duplicate entries")
        total = 0
        folded = set()
        for entry in z.infolist():
            if entry.filename == "BUILD-SOURCE.json":
                continue
            name = entry.filename[len("source/"):]
            path = safe_path(name)
            if (entry.is_dir() or stat.S_ISLNK(entry.external_attr >> 16)
                    or name.casefold() in folded or entry.file_size > 64 * 1024**2):
                raise ValueError("Unsafe corresponding source entry")
            folded.add(name.casefold())
            total += entry.file_size
            if total > 256 * 1024**2:
                raise ValueError("Corresponding source exceeds size limit")
            data = z.read(entry)
            if hashlib.sha256(data).hexdigest() != expected[name]["sha256"]:
                raise ValueError("Corresponding source file hash differs")
            staged[path.as_posix()] = data
    required = {"Cargo.toml", "Cargo.lock", ".cargo/config.toml", "libs/portable/Cargo.toml",
                "libs/portable/generate.py", "libs/portable/build.rs", "res/icon.ico", "res/manifest.xml"}
    if not required <= set(staged):
        raise ValueError("Incomplete portable packer corresponding source")
    work.mkdir(exist_ok=False)
    for root, entries in ((work / "source", staged),
                          (work / "payload", {name: data for name, data in payload.values()})):
        for name, data in entries.items():
            target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("xb") as output:
                output.write(data)
            os.utime(target, (epoch, epoch))
    # Match the portable-installer resource transformation in the native build
    # workflow; do not accept resource edits or commands from company profiles.
    resource = work / "source/res/manifest.xml"
    resource_before = resource.read_bytes()
    resource_after = b"".join(line for line in resource_before.splitlines(keepends=True)
                              if b"dpiAware" not in line)
    resource.write_bytes(resource_after)
    os.utime(resource, (epoch, epoch))
    record = {"source_revision": revision, "source_archive_sha256": archive_sha256,
              "source_date_epoch": epoch, "payload_files": len(payload),
              "main_sha256": hashlib.sha256(payload["Swan Remote Support.exe"][1]).hexdigest(),
              "payload_manifest": manifest, "source_files": len(staged),
              "portable_manifest_transform": {
                  "operation": "remove dpiAware lines, matching the native portable build workflow",
                  "before_sha256": hashlib.sha256(resource_before).hexdigest(),
                  "after_sha256": hashlib.sha256(resource_after).hexdigest()},
              "executables_run": False}
    (work / "payload-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (work / "prepared.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return record


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--source-archive", type=Path, required=True)
    p.add_argument("--source-sha256", required=True)
    p.add_argument("--source-revision", required=True)
    p.add_argument("--payload", type=Path, required=True)
    p.add_argument("--payload-manifest", type=Path, required=True)
    p.add_argument("--work-directory", type=Path, required=True)
    a = p.parse_args()
    record = prepare(a.source_archive, a.source_sha256, a.source_revision, a.payload,
                     json.loads(a.payload_manifest.read_text(encoding="utf-8-sig")), a.work_directory)
    print(json.dumps({k: v for k, v in record.items() if k != "payload_manifest"}))


if __name__ == "__main__":
    main()
