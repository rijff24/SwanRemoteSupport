"""Verify public build provenance and compare original unsigned release bytes.

This verifier never installs or executes artifact contents. Source text checkout
line endings are recorded separately; shipping binaries are never normalized.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tomllib
import urllib.error
import urllib.parse
import urllib.request
import zipfile

BRIDGE = ("src/bridge_generated.rs", "src/bridge_generated.io.rs",
          "flutter/lib/generated_bridge.dart", "flutter/lib/generated_bridge.freezed.dart",
          "flutter/macos/Runner/bridge_generated.h", "flutter/ios/Runner/bridge_generated.h")
MAX_ARCHIVE = 1024 ** 3


class VerificationError(ValueError):
    pass


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], stderr=subprocess.PIPE)


def source_version(root, revision):
    version = tomllib.loads(git(root, "show", revision + ":Cargo.toml").decode("utf-8"))["package"]["version"]
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise VerificationError("Unexpected native version")
    return version


def shipping_names(version):
    return {"SwanRemoteSupport-" + version + suffix for suffix in (
        "-customer-x64-unsigned.msi", "-technician-x64-unsigned.exe",
        "-technician-x64-unsigned.msi", "-x64-unsigned-install.exe")
    } | {"SwanServerSetup-UNSIGNED-TEST.exe", "swan-agent.exe", "swan-management.exe", "swan-worker.exe"}


def source_tree(root, revision):
    files, repositories = {}, {}

    def visit(directory, commit, prefix=""):
        if git(directory, "rev-parse", commit + "^{commit}").decode().strip() != commit:
            raise VerificationError("Pinned source commit is missing or differs")
        repositories[prefix or "."] = commit
        for item in filter(None, git(directory, "ls-tree", "-r", "-z", commit).split(b"\0")):
            header, path = item.split(b"\t", 1)
            mode, kind, object_id = header.decode().split()
            name = path.decode()
            if kind == "commit":
                visit(directory / name, object_id, prefix + name + "/")
            elif kind == "blob" and mode in ("100644", "100755"):
                files[prefix + name] = (mode, object_id)
            else:
                raise VerificationError("Unsupported source tree entry")

    visit(root, revision)
    return files, repositories


def validate_entries(archive):
    entries = archive.infolist()
    names = [e.filename for e in entries]
    if len(names) != len(set(n.casefold() for n in names)) or len(names) > 10000:
        raise VerificationError("Duplicate or excessive archive entries")
    if sum(e.file_size for e in entries) > 2 * MAX_ARCHIVE:
        raise VerificationError("Archive expands beyond verification bound")
    for e in entries:
        path = PurePosixPath(e.filename)
        if (e.is_dir() or path.is_absolute() or "\\" in e.filename or ":" in e.filename
                or not path.parts or any(p in ("", ".", "..") for p in e.filename.split("/"))
                or (e.external_attr >> 16) & 0o170000 == 0o120000
                or e.file_size > 512 * 1024 ** 2):
            raise VerificationError("Unsafe archive entry")
    return set(names)


def entry_hash(archive, name):
    digest = hashlib.sha256()
    with archive.open(name) as stream:
        for block in iter(lambda: stream.read(1024 ** 2), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_source(archive, tracked, repositories, epoch):
    info = archive.getinfo("SwanRemoteSupport-build-source.zip")
    if info.file_size > 256 * 1024 ** 2:
        raise VerificationError("Source archive exceeds bound")
    with zipfile.ZipFile(io.BytesIO(archive.read(info))) as source:
        names = validate_entries(source)
        expected = set(tracked) | set(BRIDGE)
        if names != {"BUILD-SOURCE.json"} | {"source/" + n for n in expected}:
            raise VerificationError("Corresponding source file set differs from public source")
        manifest = json.loads(source.read("BUILD-SOURCE.json"))
        if (manifest["repositories"] != repositories or set(manifest["files"]) != expected
                or manifest.get("source_date_epoch") != epoch
                or set(manifest["generated_bridge_files"]) != set(BRIDGE)):
            raise VerificationError("Source manifest provenance differs")
        conversions, bridges = [], {}
        for name in sorted(expected):
            data = source.read("source/" + name)
            if hashlib.sha256(data).hexdigest() != manifest["files"][name]["sha256"]:
                raise VerificationError("Source manifest content hash differs")
            if name in BRIDGE:
                bridges[name] = hashlib.sha256(data).hexdigest()
                continue
            mode, blob = tracked[name]
            if manifest["files"][name]["mode"] != mode:
                raise VerificationError("Source mode differs")
            def git_hash(value):
                return hashlib.sha1(b"blob " + str(len(value)).encode() + b"\0" + value).hexdigest()
            if git_hash(data) != blob:
                normalized = data.replace(b"\r\n", b"\n")
                if b"\0" in data or (git_hash(normalized) != blob
                        and git_hash(normalized.replace(b"\n", b"\r\n")) != blob):
                    raise VerificationError("Original public source blob differs")
                conversions.append(name)
    return {"tracked_files_verified": len(tracked), "bridge_hashes": bridges,
            "declared_source_text_checkout_conversions": conversions}


def verify_msvc_inputs(archive, required):
    name = "native-msvc-toolchain.json"
    if name not in archive.namelist():
        if required:
            raise VerificationError("Required MSVC input identity is missing")
        return None
    if archive.getinfo(name).file_size > 2 * 1024 ** 2:
        raise VerificationError("MSVC input identity exceeds bound")
    manifest = json.loads(archive.read(name).decode("utf-8-sig"))
    files = manifest.get("files")
    if not isinstance(files, list) or not 6 <= len(files) <= 10000:
        raise VerificationError("Invalid MSVC input file set")
    seen = set()
    for entry in files:
        if not isinstance(entry, dict):
            raise VerificationError("Invalid MSVC input entry")
        path = entry.get("path", "")
        digest = entry.get("sha256", "")
        if (not isinstance(path, str) or not path or path.startswith("/")
                or any(p in ("", ".", "..") for p in path.split("/"))
                or not path.isascii() or any(ord(c) < 32 for c in path)
                or "\\" in path or ":" in path
                or path.casefold() in seen
                or not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest)
                or type(entry.get("bytes")) is not int or entry["bytes"] < 0):
            raise VerificationError("Unsafe or invalid MSVC input entry")
        seen.add(path.casefold())
    required_files = {"bin/hostx64/x64/" + n for n in ("cl.exe", "c1xx.dll", "c2.dll", "link.exe")}
    if not required_files.union({"include/typeinfo", "include/memory"}) <= seen:
        raise VerificationError("Required MSVC inputs missing")
    canonical = "".join(e["path"] + "\0" + e["sha256"] + "\n"
                        for e in sorted(files, key=lambda e: e["path"]))
    identity = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
    if identity != manifest.get("input_identity_sha256"):
        raise VerificationError("MSVC input identity digest differs")
    return {"input_identity_sha256": identity, "files": len(files),
            "visual_studio_version": manifest.get("visual_studio_version"),
            "toolset_directory_version": manifest.get("toolset_directory_version"),
            "scope": "Recorded default x64 MSVC inputs; not Windows SDK or actual compiler-command verification"}


def compare(first, second, checkout, revision):
    tracked, repositories = source_tree(checkout, revision)
    epoch = int(git(checkout, "show", "-s", "--format=%ct", revision))
    with zipfile.ZipFile(first) as a, zipfile.ZipFile(second) as b:
        names = [validate_entries(z) for z in (a, b)]
        proof = [verify_source(z, tracked, repositories, epoch) for z in (a, b)]
        if proof[0]["bridge_hashes"] != proof[1]["bridge_hashes"]:
            raise VerificationError("Independently generated bridge bytes differ")
        toolchains = [verify_msvc_inputs(z, "deployment/windows/Record-MsvcToolchain.ps1" in tracked)
                      for z in (a, b)]
        outputs = [{n for n in ns if "/" not in n and n.lower().endswith((".exe", ".msi", ".dll"))} for ns in names]
        required = shipping_names(source_version(checkout, revision))
        if outputs[0] != required or outputs[1] != required:
            raise VerificationError("Unsigned shipping output sets differ or are incomplete")
        for z, ns in zip((a, b), names):
            for line in z.read("SHA256SUMS.txt").decode().splitlines():
                if not line.strip():
                    continue
                digest, name = line.split(None, 1)
                name = name.strip().lstrip("*")
                if name not in ns or entry_hash(z, name) != digest:
                    raise VerificationError("Declared artifact file hash differs")
        records = []
        for name in sorted(outputs[0]):
            left, right = entry_hash(a, name), entry_hash(b, name)
            records.append({"path": name, "first_sha256": left, "second_sha256": right,
                            "first_bytes": a.getinfo(name).file_size,
                            "second_bytes": b.getinfo(name).file_size,
                            "identical": left == right and a.getinfo(name).file_size == b.getinfo(name).file_size})
    return {"schema": 1, "source_commit": revision, "source_date_epoch": epoch, "source_proofs": proof,
            "msvc_input_proofs": toolchains,
            "recorded_msvc_inputs_identical": (toolchains[0]["input_identity_sha256"] == toolchains[1]["input_identity_sha256"]
                                               if all(toolchains) else None),
            "outputs": records, "all_shipping_bytes_identical": all(r["identical"] for r in records),
            "normalization_applied_to_shipping_files": False,
            "executables_run": False, "runtime_qualified": False, "production_approved": False,
            "scope": "Original unsigned native shipping outputs; no runtime, signing or production qualification"}


def api_json(repository, path, token):
    request = urllib.request.Request("https://api.github.com/repos/" + repository + path,
        headers={"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json"})
    with urllib.request.build_opener(NoRedirect()).open(request, timeout=30) as response:
        data = response.read(2 * 1024 ** 2 + 1)
    if len(data) > 2 * 1024 ** 2:
        raise VerificationError("API metadata exceeds bound")
    return json.loads(data)


def fetch(repository, run_id, revision, artifact_name, token, target):
    run = api_json(repository, "/actions/runs/" + str(run_id), token)
    if (run["head_sha"] != revision or run["status"] != "completed" or run["conclusion"] != "success"
            or run["path"] != ".github/workflows/swan-windows-build.yml"):
        raise VerificationError("Expected a successful exact-source native workflow")
    artifacts = api_json(repository, "/actions/runs/" + str(run_id) + "/artifacts?per_page=100", token)
    matches = [a for a in artifacts["artifacts"] if a["name"] == artifact_name and not a["expired"]]
    if len(matches) != 1:
        raise VerificationError("Expected one complete unexpired native artifact")
    item = matches[0]
    if (item["workflow_run"]["head_sha"] != revision
            or not re.fullmatch(r"sha256:[0-9a-f]{64}", item.get("digest", ""))
            or not 0 < item["size_in_bytes"] < MAX_ARCHIVE):
        raise VerificationError("Invalid artifact identity or digest")
    request = urllib.request.Request("https://api.github.com/repos/" + repository + "/actions/artifacts/" + str(item["id"]) + "/zip",
        headers={"Authorization": "Bearer " + token})
    try:
        urllib.request.build_opener(NoRedirect()).open(request, timeout=30)
    except urllib.error.HTTPError as error:
        if error.code != 302:
            raise VerificationError("Artifact redirect unavailable") from None
        location = error.headers["Location"]
    else:
        raise VerificationError("Expected an artifact download redirect")
    parsed = urllib.parse.urlparse(location)
    if (parsed.scheme != "https" or not (parsed.hostname or "").endswith(".blob.core.windows.net")
            or parsed.username or parsed.password):
        raise VerificationError("Unexpected artifact download origin")
    digest, count = hashlib.sha256(), 0
    # No authorization header is forwarded to the signed storage URL.
    with urllib.request.build_opener(NoRedirect()).open(location, timeout=30) as response, target.open("xb") as output:
        for block in iter(lambda: response.read(1024 ** 2), b""):
            count += len(block)
            if count > item["size_in_bytes"]:
                raise VerificationError("Artifact exceeds declared size")
            output.write(block)
            digest.update(block)
    if count != item["size_in_bytes"] or "sha256:" + digest.hexdigest() != item["digest"]:
        raise VerificationError("Original artifact archive digest differs")
    return {"workflow_run": run_id, "run_attempt": run["run_attempt"], "artifact": item["id"],
            "archive_digest": item["digest"], "bytes": count}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--first-run", required=True, type=int)
    parser.add_argument("--second-run", required=True, type=int)
    parser.add_argument("--source", required=True)
    parser.add_argument("--checkout", required=True, type=Path)
    parser.add_argument("--work", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    repository, token = os.environ.get("GITHUB_REPOSITORY", ""), os.environ.get("GITHUB_TOKEN", "")
    if (not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) or not token
            or not re.fullmatch(r"[0-9a-f]{40}", args.source)
            or min(args.first_run, args.second_run) <= 0 or args.first_run == args.second_run):
        raise VerificationError("Invalid repository, source, token or independent run identities")
    if args.output.exists() or args.work.exists():
        raise VerificationError("Do not replace retained comparison evidence")
    if git(args.checkout, "rev-parse", "HEAD").decode().strip() != args.source:
        raise VerificationError("Comparison checkout differs from the requested source")
    version = source_version(args.checkout, args.source)
    args.work.mkdir(parents=True)
    paths = [args.work / "first.zip", args.work / "second.zip"]
    inputs = [fetch(repository, run, args.source, "SwanRemoteSupport-" + version + "-x64-unsigned", token, path)
              for run, path in zip((args.first_run, args.second_run), paths)]
    if inputs[0]["artifact"] == inputs[1]["artifact"]:
        raise VerificationError("Independent runs reused the same artifact")
    result = compare(*paths, args.checkout, args.source)
    result["builds"] = inputs
    result["independent_rebuild_verified"] = result["all_shipping_bytes_identical"]
    result["comparison_workflow"] = {"source": os.environ.get("GITHUB_SHA"), "run": os.environ.get("GITHUB_RUN_ID")}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x", encoding="utf-8") as output:
        json.dump(result, output, indent=2)
    print(json.dumps({"source": args.source, "outputs": len(result["outputs"]),
                      "all_shipping_bytes_identical": result["all_shipping_bytes_identical"]}))
    return 0 if result["all_shipping_bytes_identical"] else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except VerificationError as error:
        raise SystemExit(str(error)) from None
    except Exception as error:
        # Network exception messages can contain signed URLs. Preserve no such URL in logs.
        raise SystemExit("Comparison verification failed: " + type(error).__name__) from None
