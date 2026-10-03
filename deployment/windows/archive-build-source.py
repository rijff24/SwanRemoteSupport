"""Archive tracked build inputs and explicitly named generated bridge sources.

Never sweep the working directory: ignored credentials, lab data, and binaries
are excluded. Run after bridge generation and before application compilation.
The included workflow describes subsequent deterministic packaging transforms.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import zipfile

BRIDGE_FILES = (
    "src/bridge_generated.rs", "src/bridge_generated.io.rs",
    "flutter/lib/generated_bridge.dart", "flutter/lib/generated_bridge.freezed.dart",
    "flutter/macos/Runner/bridge_generated.h", "flutter/ios/Runner/bridge_generated.h",
)


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def safe_path(value):
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or ".git" in path.parts or "\\" in value or ":" in value:
        raise ValueError("Unsafe source path")
    return path


def collect(root):
    files, repositories = {}, {}

    def repository(directory, prefix="", expected=None):
        revision = git(directory, "rev-parse", "HEAD").decode().strip()
        if expected is not None and revision != expected:
            raise ValueError("Submodule does not match pinned source")
        repositories[prefix or "."] = revision
        records = git(directory, "ls-files", "--stage", "-z").split(b"\0")
        for record in filter(None, records):
            header, name = record.split(b"\t", 1)
            mode, object_id, stage = header.decode().split()
            if stage != "0":
                raise ValueError("Unmerged source index")
            relative = name.decode("utf-8")
            safe_path(relative)
            archive_name = prefix + relative
            path = directory / relative
            if path.is_symlink() or not path.resolve().is_relative_to(root):
                raise ValueError("Source link or path escaped checkout")
            if mode == "160000":
                if not path.is_dir():
                    raise ValueError("Uninitialized source submodule")
                repository(path, archive_name + "/", object_id)
            elif mode in ("100644", "100755"):
                if not path.is_file():
                    raise ValueError("Tracked source missing")
                files[archive_name] = (path, mode)
            else:
                raise ValueError("Unsupported source file mode")

    repository(root)
    for name in BRIDGE_FILES:
        path = root / name
        if not path.is_file() or path.is_symlink() or not path.resolve().is_relative_to(root):
            raise ValueError("Generated bridge source missing or unsafe")
        files[name] = (path, "100644")
    return files, repositories


def archive(root, output):
    root = root.resolve(strict=True)
    files, repositories = collect(root)
    # Explicit exclusive creation prevents silently replacing release evidence.
    manifest = {"schema": 1, "snapshot": "before-application-compilation",
                "repositories": repositories, "generated_bridge_files": list(BRIDGE_FILES),
                "files": {}}
    with output.open("xb") as destination, zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as package:
        for name, (path, mode) in sorted(files.items()):
            data = path.read_bytes()
            manifest["files"][name] = {"sha256": hashlib.sha256(data).hexdigest(), "mode": mode}
            info = zipfile.ZipInfo("source/" + name, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = int(mode, 8) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            package.writestr(info, data)
        receipt = zipfile.ZipInfo("BUILD-SOURCE.json", (1980, 1, 1, 0, 0, 0))
        receipt.compress_type = zipfile.ZIP_DEFLATED
        receipt.create_system = 3
        receipt.external_attr = 0o100644 << 16
        package.writestr(receipt, json.dumps(manifest, sort_keys=True, indent=2))
    with zipfile.ZipFile(output) as package:
        if package.testzip() is not None:
            raise ValueError("Source archive CRC failure")
        for name, entry in manifest["files"].items():
            if hashlib.sha256(package.read("source/" + name)).hexdigest() != entry["sha256"]:
                raise ValueError("Source archive hash mismatch")
    return hashlib.sha256(output.read_bytes()).hexdigest()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    print(archive(args.root, args.output))
