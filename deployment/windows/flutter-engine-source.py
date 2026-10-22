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
    if lock["buildroot"]["repository"] != "https://github.com/flutter/buildroot.git":
        raise ValueError("Unexpected Flutter buildroot source")
    for component in ("engine", "depot_tools", "buildroot"):
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
    buildroot = lock["buildroot"]
    if (buildroot["patch_file"] != "flutter-engine-buildroot.patch"
            or set(buildroot["files"]) != {"build/toolchain/win/setup_toolchain.py", "build/config/compiler/BUILD.gn"}
            or sha(RECIPE / buildroot["patch_file"]) != buildroot["patch_sha256"]):
        raise ValueError("Unexpected buildroot toolset-selection patch")
    if lock["windows_toolchain"] != {
        "msvc_component": "Microsoft.VisualStudio.Component.VC.14.38.17.8.x86.x64",
        "msvc_atl_component": "Microsoft.VisualStudio.Component.VC.14.38.17.8.ATL",
        "msvc_tools_version": "14.38.33130", "vcvars_selector": "14.38",
        "windows_sdk_version": "10.0.22621.0",
    }:
        raise ValueError("Unexpected Windows engine toolset")
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


def verify_buildroot(root, lock, patched):
    root = root.resolve(strict=True)
    expected = lock["buildroot"]
    if Path(git(root, "rev-parse", "--show-toplevel").decode().strip()).resolve() != root:
        raise ValueError("Expected the Flutter buildroot repository root")
    if git(root, "rev-parse", "HEAD").decode().strip() != expected["revision"]:
        raise ValueError("Buildroot source revision changed")
    if git(root, "diff", "--cached", "--name-only").strip():
        raise ValueError("Buildroot has staged changes")
    changed = set(git(root, "diff", "--name-only").decode().splitlines())
    if changed != (set(expected["files"]) if patched else set()):
        raise ValueError("Unexpected buildroot source modifications")
    for name, hashes in expected["files"].items():
        path = root / name
        if path.is_symlink() or sha(path) != hashes["patched_sha256" if patched else "original_sha256"]:
            raise ValueError("Buildroot source bytes changed: " + name)


def patch_buildroot(root):
    lock, _ = load_recipe()
    verify_buildroot(root, lock, False)
    patch = RECIPE / lock["buildroot"]["patch_file"]
    subprocess.run(["git", "-C", str(root), "apply", "--check", str(patch)], check=True)
    subprocess.run(["git", "-C", str(root), "apply", str(patch)], check=True)
    verify_buildroot(root, lock, True)


def verify_commands(path, lock):
    commands = path.read_text(encoding="utf-8-sig").replace("\\", "/")
    if "_ALLOW_COMPILER_AND_STL_VERSION_MISMATCH" in commands:
        raise ValueError("Compiler and standard-library compatibility checks must remain enabled")
    msvc = set(re.findall(r'/VC/Tools/MSVC/([^/\s"]+)/', commands, re.IGNORECASE))
    sdk = set(re.findall(r'/Windows Kits/10/(?:include|lib)/([^/\s"]+)/', commands, re.IGNORECASE))
    if msvc != {lock["windows_toolchain"]["msvc_tools_version"]}:
        raise ValueError("Generated compile/link commands do not use only the pinned MSVC toolset")
    if sdk != {lock["windows_toolchain"]["windows_sdk_version"]}:
        raise ValueError("Generated compile/link commands do not use only the pinned Windows SDK")
    compiles = [line for line in commands.splitlines() if re.search(r'clang-cl(?:\.exe)?\b', line)]
    if not compiles or any("/Brepro" not in line or "-fdebug-compilation-dir=." not in line for line in compiles):
        raise ValueError("Generated compiler commands lack deterministic timestamp/path controls")


def verify_link_response(path):
    response = path.read_text(encoding="utf-8-sig")
    for flag in ("/Brepro", "/PDBALTPATH:%_PDB%", "/INCREMENTAL:NO"):
        if flag.lower() not in response.lower().split():
            raise ValueError("Final engine link lacks deterministic flag: " + flag)
    switches = re.findall(r'/INCREMENTAL(?::(?:YES|NO))?(?=\s|$)', response, re.IGNORECASE)
    if any(switch.lower() != "/incremental:no" for switch in switches):
        raise ValueError("Incremental linking cannot be used for reproducible engine builds")


def package(build, output):
    lock, patch = load_recipe()
    engine = build / "checkout/src/flutter"
    verify_checkout(engine, lock, True)
    verify_buildroot(build / "checkout/src", lock, True)
    verify_commands(build / "ninja-commands.txt", lock)
    verify_link_response(build / "link-response.txt")
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
    for name in ("dependency-revisions.txt", "toolchain.json", "ninja-commands.txt", "link-response.txt"):
        if not (build / name).is_file():
            raise ValueError("Missing engine build evidence: " + name)
    toolchain = json.loads((build / "toolchain.json").read_text(encoding="utf-8-sig"))
    for field in ("msvc_tools_version", "windows_sdk_version"):
        if toolchain[field] != lock["windows_toolchain"][field]:
            raise ValueError("Recorded toolchain differs from reviewed recipe")
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
    subprocess.run(["git", "-C", str(build / "checkout/src"), "archive", "--format=zip",
                    "--output=" + str((output / "FlutterBuildroot-original-source.zip").resolve()), "HEAD"], check=True)
    for path in [RECIPE / "flutter-engine.lock.json", patch, Path(__file__),
                 RECIPE / lock["buildroot"]["patch_file"], RECIPE / "Build-FlutterEngine.ps1",
                 RECIPE / "Install-FlutterEngineToolchain.ps1", build / "dependency-revisions.txt",
                 build / "toolchain.json", build / "ninja-commands.txt", build / "link-response.txt"]:
        (output / path.name).write_bytes(path.read_bytes())
    (output / "applied-source.patch").write_bytes(git(engine, "diff", "--binary", "--no-ext-diff"))
    (output / "applied-buildroot.patch").write_bytes(git(build / "checkout/src", "diff", "--binary", "--no-ext-diff"))
    (output / "args.gn").write_bytes((release / "args.gn").read_bytes())
    (output / "gclient-config.txt").write_bytes((build / "checkout/.gclient").read_bytes())
    workflow = RECIPE.parents[1] / ".github/workflows/swan-flutter-engine.yml"
    (output / workflow.name).write_bytes(workflow.read_bytes())
    result = {
        "schema": 1, "engine_revision": lock["engine"]["revision"],
        "depot_tools_revision": lock["depot_tools"]["revision"],
        "buildroot_revision": lock["buildroot"]["revision"],
        "msvc_tools_version": lock["windows_toolchain"]["msvc_tools_version"],
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
    root_parser = sub.add_parser("patch-buildroot")
    root_parser.add_argument("root", type=Path)
    command_parser = sub.add_parser("verify-commands")
    command_parser.add_argument("commands", type=Path)
    link_parser = sub.add_parser("verify-link-response")
    link_parser.add_argument("response", type=Path)
    package_parser = sub.add_parser("package")
    package_parser.add_argument("build", type=Path)
    package_parser.add_argument("output", type=Path)
    args = parser.parse_args()
    if args.command == "patch":
        apply_patch(args.engine)
        print("Reviewed engine source and patch hashes verified")
    elif args.command == "patch-buildroot":
        patch_buildroot(args.root)
        print("Reviewed buildroot source and toolset-selection patch verified")
    elif args.command == "verify-commands":
        verify_commands(args.commands, load_recipe()[0])
        print("Generated commands use the pinned Microsoft toolset and Windows SDK")
    elif args.command == "verify-link-response":
        verify_link_response(args.response)
        print("Final engine link uses deterministic timestamp/path controls and excludes incremental linking")
    else:
        print(json.dumps(package(args.build, args.output)))
