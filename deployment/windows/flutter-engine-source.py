"""Verify and package the independent Windows Flutter engine reconstruction."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import zipfile

RECIPE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def load_recipe():
    lock = json.loads((RECIPE / "flutter-engine.lock.json").read_text(encoding="utf-8"))
    if lock["schema"] != 1 or lock["engine"]["repository"] != "https://github.com/flutter-team-archive/engine.git":
        raise ValueError("Unexpected engine source")
    if lock["depot_tools"]["repository"] != "https://chromium.googlesource.com/chromium/tools/depot_tools.git":
        raise ValueError("Unexpected depot-tools source")
    for component in ("engine", "depot_tools"):
        if not re.fullmatch(r"[0-9a-f]{40}", lock[component]["revision"]):
            raise ValueError("Source must use a full immutable revision")
    if lock["patch"]["file"] != "flutter-engine-windows.patch" or set(lock["patch"]["files"]) != {
        "shell/platform/windows/flutter_windows_engine.cc", "fml/hex_codec.h"
    }:
        raise ValueError("Unexpected patch scope")
    patch = RECIPE / lock["patch"]["file"]
    if sha(patch) != lock["patch"]["sha256"]:
        raise ValueError("Engine patch hash differs from reviewed recipe")
    if lock["gn_arguments"] != ["--runtime-mode=release"] or lock["ninja_target"] != "flutter_windows.dll":
        raise ValueError("Unexpected engine build arguments")
    overrides = lock.get("dependency_overrides", {})
    if set(overrides) != {"src/flutter/third_party/tinygltf"}:
        raise ValueError("Unexpected engine dependency override")
    tinygltf = overrides["src/flutter/third_party/tinygltf"]
    if (tinygltf["repository"] != "https://github.com/syoyo/tinygltf.git"
            or tinygltf["revision"] != "9bb5806df4055ac973b970ba5b3e27ce27d98148"):
        raise ValueError("Dependency mirror override must preserve the original DEPS commit")
    return lock, patch


def verify_checkout(engine, lock, patched):
    engine = engine.resolve(strict=True)
    if Path(git(engine, "rev-parse", "--show-toplevel").decode().strip()).resolve() != engine:
        raise ValueError("Expected the engine repository root")
    if git(engine, "rev-parse", "HEAD").decode().strip() != lock["engine"]["revision"]:
        raise ValueError("Engine revision differs from reviewed source")
    if git(engine, "diff", "--cached", "--name-only").strip():
        raise ValueError("Engine has staged changes")
    changed = set(git(engine, "diff", "--name-only").decode().splitlines())
    if changed != (set(lock["patch"]["files"]) if patched else set()):
        raise ValueError("Unexpected engine source modifications")
    if sha(engine / "DEPS") != lock["engine"]["deps_sha256"]:
        raise ValueError("Engine dependency recipe changed")
    for name, expected in lock["patch"]["files"].items():
        path = engine / name
        if path.is_symlink() or sha(path) != expected["patched_sha256" if patched else "original_sha256"]:
            raise ValueError("Engine source bytes changed: " + name)


def apply_patch(engine):
    lock, patch = load_recipe()
    verify_checkout(engine, lock, False)
    # Check all inputs before making any source change. Git applies the reviewed
    # patch; profile fields and network content never supply executable commands.
    subprocess.run(["git", "-C", str(engine), "apply", "--check", str(patch)], check=True)
    subprocess.run(["git", "-C", str(engine), "apply", str(patch)], check=True)
    verify_checkout(engine, lock, True)


def package(build, output):
    lock, patch = load_recipe()
    engine = build / "checkout/src/flutter"
    verify_checkout(engine, lock, True)
    if git(build / "depot_tools", "rev-parse", "HEAD").decode().strip() != lock["depot_tools"]["revision"]:
        raise ValueError("Depot-tools revision changed")
    for name, expected in lock["dependency_overrides"].items():
        dependency = build / "checkout" / name
        if (git(dependency, "rev-parse", "HEAD").decode().strip() != expected["revision"]
                or git(dependency, "config", "--get", "remote.origin.url").decode().strip() != expected["repository"]):
            raise ValueError("Dependency source override changed: " + name)
    if output.exists():
        raise FileExistsError("Do not overwrite an existing engine result")
    release = build / "checkout/src/out/host_release"
    binaries = [release / ("flutter_windows.dll" + suffix) for suffix in ("", ".lib", ".pdb")]
    for path in binaries:
        if path.is_symlink() or not path.is_file() or not 0 < path.stat().st_size < 512 * 1024 * 1024:
            raise ValueError("Missing or unsafe engine output: " + path.name)
    dll = binaries[0].read_bytes()
    if dll[:2] != b"MZ" or len(dll) < 64:
        raise ValueError("Engine is not a Windows PE file")
    pe = struct.unpack_from("<I", dll, 60)[0]
    if (pe + 24 > len(dll) or dll[pe:pe + 4] != b"PE\0\0"
            or struct.unpack_from("<H", dll, pe + 4)[0] != 0x8664
            or not struct.unpack_from("<H", dll, pe + 22)[0] & 0x2000):
        raise ValueError("Engine is not Windows x64")
    for name in ("dependency-revisions.txt", "toolchain.json"):
        if not (build / name).is_file():
            raise ValueError("Missing engine build evidence: " + name)
    if not (release / "args.gn").is_file():
        raise ValueError("Missing generated engine build arguments")
    output.mkdir(parents=True)
    archive = output / "windows-x64-release-source-built.zip"
    with zipfile.ZipFile(archive, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        for path in sorted(binaries):
            info = zipfile.ZipInfo(path.name, (1980, 1, 1, 0, 0, 0))
            info.external_attr = 0o100644 << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            zipped.writestr(info, path.read_bytes())
    subprocess.run(["git", "-C", str(engine), "archive", "--format=zip",
                    "--output=" + str((output / "FlutterEngine-original-source.zip").resolve()), "HEAD"], check=True)
    for path in [RECIPE / "flutter-engine.lock.json", patch, Path(__file__),
                 RECIPE / "Build-FlutterEngine.ps1", build / "dependency-revisions.txt", build / "toolchain.json"]:
        (output / path.name).write_bytes(path.read_bytes())
    (output / "applied-source.patch").write_bytes(git(engine, "diff", "--binary", "--no-ext-diff"))
    (output / "args.gn").write_bytes((release / "args.gn").read_bytes())
    (output / "gclient-config.txt").write_bytes((build / "checkout/.gclient").read_bytes())
    workflow = RECIPE.parents[1] / ".github/workflows/swan-flutter-engine.yml"
    (output / workflow.name).write_bytes(workflow.read_bytes())
    result = {
        "schema": 1, "engine_revision": lock["engine"]["revision"],
        "depot_tools_revision": lock["depot_tools"]["revision"],
        "archive_sha256": sha(archive),
        "binary_files": {path.name: sha(path) for path in binaries},
        "evidence_files": {path.name: sha(path) for path in output.iterdir() if path.is_file()},
        "upstream_binary_match_verified": False, "independent_rebuild_verified": False,
        "runtime_qualified": False, "production_approved": False,
        "scope": "Source-built engine candidate only; does not replace the application workflow's pinned engine",
    }
    (output / "ENGINE-BUILD.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    patch_parser = sub.add_parser("patch")
    patch_parser.add_argument("engine", type=Path)
    package_parser = sub.add_parser("package")
    package_parser.add_argument("build", type=Path)
    package_parser.add_argument("output", type=Path)
    args = parser.parse_args()
    if args.command == "patch":
        apply_patch(args.engine)
        print("Reviewed engine source and patch hashes verified")
    else:
        print(json.dumps(package(args.build, args.output)))
