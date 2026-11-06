"""Prepare public Windows server inputs; never execute supplied components.

Binary pins and a canonical digest of all corresponding source/recipe files
bind the maintained transport to reviewed public inputs. Build receipts are
checked separately because their project revision varies between CI runs.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import stat
import struct
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SOURCE_RECEIPT = "rustdesk-server/SWAN-SOURCE.json"
TRANSPORT_FILES = {
    "transport-source/target/release/hbbs.exe": 128 * 1024**2,
    "transport-source/target/release/hbbr.exe": 128 * 1024**2,
    "transport-source/SWAN-SOURCE.json": 128 * 1024,
    "transport-artifacts/transport-build-source.zip": 64 * 1024**2,
}


def canonical(value):
    if b"\0" not in value:
        try:
            return value.decode("utf-8").replace("\r\n", "\n").encode("utf-8")
        except UnicodeDecodeError:
            pass
    return value


def entries(archive, limits=None):
    result = {}
    total = 0
    for entry in archive.infolist():
        name = entry.filename
        path = PurePosixPath(name)
        if (entry.is_dir() or path.is_absolute() or "\\" in name or ":" in name
                or any(p in (".", "..") or p.endswith((" ", ".")) for p in path.parts)
                or stat.S_ISLNK(entry.external_attr >> 16) or entry.flag_bits & 1
                or name.casefold() in {p.casefold() for p in result}):
            raise ValueError("Unsafe or duplicate server input")
        limit = limits.get(name, -1) if limits is not None else 16 * 1024**2
        total += entry.file_size
        if not name or not 0 <= entry.file_size <= limit or total > 300 * 1024**2:
            raise ValueError("Unexpected or oversized server input")
        with archive.open(entry) as stream:
            value = stream.read(limit + 1)
        if len(value) != entry.file_size:
            raise ValueError("Server input length changed")
        result[name] = value
    if limits is not None and set(result) != set(limits):
        raise ValueError("Incomplete maintained transport input")
    return result


def read_file(path, limit):
    info = path.lstat()
    if (not stat.S_ISREG(info.st_mode) or getattr(info, "st_file_attributes", 0) & 1024
            or info.st_size > limit):
        raise ValueError("Expected a bounded regular server input")
    with path.open("rb") as stream:
        value = stream.read(limit + 1)
    if len(value) > limit or len(value) != info.st_size:
        raise ValueError("Server input changed or exceeded its size limit")
    return value


def transport_inputs(package=None, directory=None):
    if package is not None:
        with zipfile.ZipFile(io.BytesIO(read_file(package, 300 * 1024**2))) as archive:
            return entries(archive, TRANSPORT_FILES)
    info = directory.lstat()
    if not stat.S_ISDIR(info.st_mode) or getattr(info, "st_file_attributes", 0) & 1024:
        raise ValueError("Expected a regular maintained transport directory")
    directory = directory.resolve(strict=True)
    allowed_directories = {str(parent) for name in TRANSPORT_FILES
                           for parent in PurePosixPath(name).parents if str(parent) != "."}
    found = set()
    for path in directory.rglob("*"):
        info = path.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 1024:
            raise ValueError("Maintained transport inputs must not contain links")
        if stat.S_ISREG(info.st_mode):
            found.add(path.relative_to(directory).as_posix())
        elif not stat.S_ISDIR(info.st_mode) or path.relative_to(directory).as_posix() not in allowed_directories:
            raise ValueError("Unexpected maintained transport directory entry")
    if found != set(TRANSPORT_FILES):
        raise ValueError("Unexpected maintained transport directory contents")
    return {name: read_file(directory / name, limit) for name, limit in TRANSPORT_FILES.items()}


def source_tree_digest(files):
    digest = hashlib.sha256(b"swan-windows-transport-source-v1\0")
    items = sorted((name, canonical(value)) for name, value in files.items() if name != SOURCE_RECEIPT)
    digest.update(struct.pack(">Q", len(items)))
    for name, value in items:
        encoded = name.encode("utf-8")
        digest.update(struct.pack(">Q", len(encoded))); digest.update(encoded)
        digest.update(struct.pack(">Q", len(value))); digest.update(value)
    return digest.hexdigest()


def normalized_source(files):
    # Source-only metadata/line endings become deterministic. Shipping EXEs are
    # never modified. ZIP_STORED avoids compression-library version dependence.
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, value in sorted(files.items()):
            entry = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, canonical(value))
    return output.getvalue()


def prepare(inputs, https_package, destination, root=ROOT):
    manifest_bytes = (root / "deployment/windows/server-components.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    pins = json.loads((root / "deployment/transport/source.json").read_text(encoding="utf-8"))
    if (manifest["schema"] != 2 or manifest["architecture"] != "x86_64"
            or manifest["transport"]["kind"] != "swan-maintained-source-build"):
        raise ValueError("Maintained Windows component policy required")
    for name, expected in manifest["transport"]["binary_sha256"].items():
        if name not in ("hbbs.exe", "hbbr.exe"):
            raise ValueError("Unexpected maintained executable")
        if hashlib.sha256(inputs["transport-source/target/release/" + name]).hexdigest() != expected:
            raise ValueError("Maintained Windows executable hash mismatch")
    if set(manifest["transport"]["binary_sha256"]) != {"hbbs.exe", "hbbr.exe"}:
        raise ValueError("Incomplete maintained binary pins")
    with zipfile.ZipFile(io.BytesIO(inputs["transport-artifacts/transport-build-source.zip"])) as archive:
        source = entries(archive)
    if (len(source) != manifest["transport"]["source_files"]
            or source_tree_digest(source) != manifest["transport"]["source_tree_sha256"]):
        raise ValueError("Maintained transport corresponding source mismatch")
    if canonical(source[SOURCE_RECEIPT]) != canonical(inputs["transport-source/SWAN-SOURCE.json"]):
        raise ValueError("Transport source receipts differ")
    receipt = json.loads(source[SOURCE_RECEIPT])
    if (set(receipt) != {"upstream", "stage", "project_revision", "patched_udp_sha256", "unsigned_test"}
            or receipt["stage"] != "fix" or receipt["upstream"] != pins
            or receipt["patched_udp_sha256"] != pins["patches"][-1]["after_sha256"]
            or receipt["unsigned_test"] is not True
            or not re.fullmatch(r"[a-f0-9]{40}", receipt["project_revision"])):
        raise ValueError("Maintained transport build policy mismatch")
    https = read_file(https_package, 256 * 1024**2)
    if hashlib.sha256(https).hexdigest() != manifest["https"]["sha256"]:
        raise ValueError("Pinned HTTPS archive hash mismatch")
    with zipfile.ZipFile(io.BytesIO(https)) as archive:
        names = archive.namelist()
        if names.count("caddy.exe") != 1 or names.count("LICENSE") != 1:
            raise ValueError("HTTPS executable or license missing")
        if archive.getinfo("caddy.exe").file_size > 128 * 1024**2 or not 0 < archive.getinfo("LICENSE").file_size <= 128 * 1024:
            raise ValueError("HTTPS component exceeds size limit")
        caddy, caddy_license = archive.read("caddy.exe"), archive.read("LICENSE")
    if hashlib.sha256(caddy).hexdigest() != manifest["https"]["binary_sha256"]:
        raise ValueError("Pinned HTTPS executable mismatch")
    outputs = {name: inputs["transport-source/target/release/" + name] for name in ("hbbs.exe", "hbbr.exe")}
    outputs.update({"caddy.exe": caddy, "Caddy-LICENSE.txt": caddy_license,
                    "RustDesk-LICENSE.txt": canonical(source["rustdesk-server/LICENSE"]),
                    "transport-build-source.zip": normalized_source(source),
                    "server-components.json": canonical(manifest_bytes)})
    notices = ("RustDesk rendezvous/relay: AGPL-3.0, upstream RustDesk contributors; Swan maintained UDP normalization.\n"
               f"Upstream source: {manifest['transport']['source']}\n"
               f"Public recipe revision: https://github.com/rijff24/SwanRemoteSupport/tree/{receipt['project_revision']}/deployment/transport\n"
               "Complete corresponding source and recipe: transport-build-source.zip (source ZIP metadata and text checkout endings normalized).\n\n"
               f"Caddy HTTPS server: Apache-2.0, upstream Caddy contributors.\nSource: {manifest['https']['source']}\n"
               "Retain both licenses, notices and the transport source archive with distributions. Test inputs do not establish production signing approval.\n")
    outputs["THIRD-PARTY.txt"] = notices.encode("utf-8")
    destination = destination.absolute()
    if destination.exists() or destination.is_symlink():
        raise ValueError("Component output directory already exists")
    destination.mkdir()  # Exclusive create, after all input verification.
    for name, value in outputs.items():
        with (destination / name).open("xb") as stream:
            stream.write(value)
    return {"maintained_transport": True, "verified_source_files": len(source),
            "source_tree_sha256": source_tree_digest(source), "project_revision": receipt["project_revision"],
            "sha256": {name: hashlib.sha256(value).hexdigest() for name, value in outputs.items()},
            "supplier_executables_run": False, "production_publication": False}


def main():
    parser = argparse.ArgumentParser()
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument("--transport-package", type=Path)
    choice.add_argument("--transport-directory", type=Path)
    parser.add_argument("--https-package", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = prepare(transport_inputs(args.transport_package, args.transport_directory), args.https_package, args.output)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
