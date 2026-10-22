"""Select the reviewed experimental engine without replacing the SDK compiler."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import zipfile

ARCHIVE_SHA256 = "96a19e1b2f838e11a375235c29ada407adcff5e4e124c60f0bc54eb527e8c978"
BINARIES = {
    "flutter_windows.dll": "8f6153cb66e9dd6cb354c94089e8c00fa6f3041b50bc593c275b4cb55f8a1a07",
    "flutter_windows.dll.lib": "b8bd040e25393a103caaff762f3a95ab48d4f96c09c03dcd6cec9f01d8acca00",
    "flutter_windows.dll.pdb": "7542bcc7cf7ef96d08fed1a2a4b5082932def0f68f933c01eaeba7f9c1893cb0",
}
SNAPSHOT_VERSION = b"80a49c7111088100a233b2ae788e1f48"
SDK_ENGINE = "a18df97ca57a249df5d8d68cd0820600223ce262"


def select(candidate, sdk, receipt, source_candidate="source-candidate-9f97ef0"):
    if source_candidate == "source-candidate-9f97ef0":
        archive_sha256, binaries = ARCHIVE_SHA256, BINARIES
        source_commit, workflow_run, artifact = "9f97ef0053d822bcf5cf3f5f03bf5dc571247d16", 39716739857, 12401286693
        independent_runs = []
    elif source_candidate == "source-candidate-05d5369":
        archive_sha256 = "2c84aedf35e0a93a86013b0c4903450dc740e4c8d049d971ed31b214e3b60500"
        binaries = {
            "flutter_windows.dll": "c87ba5a71afa7b7100c1c9ccc1289009dab812f9de2acc38f9d5103e48c1f204",
            "flutter_windows.dll.lib": "f7183633a4053b822ad6e353484c76528ffe98242cbe5bef04923ca40373eafc",
            "flutter_windows.dll.pdb": "64317fb6c9248d35fd3b2ed4c72147d44676b27c3d17bee0e74b986942abe521",
        }
        source_commit, workflow_run, artifact = "05d5369c04804b9a9b2dc06df0cb85b0efe6f274", 39753153773, 12416969754
        independent_runs = [39753153773, 39761977920]
    else:
        raise ValueError("Unknown source engine candidate")
    if receipt.exists():
        raise FileExistsError("Do not overwrite candidate selection evidence")
    if (sdk / "bin/internal/engine.version").read_text().strip() != SDK_ENGINE:
        raise ValueError("Expected the reviewed Flutter 3.24.5 SDK")
    target = sdk / "bin/cache/artifacts/engine/windows-x64-release"
    compiler = target / "gen_snapshot.exe"
    compiler_bytes = compiler.read_bytes()
    if SNAPSHOT_VERSION not in compiler_bytes:
        raise ValueError("SDK snapshot compiler is incompatible with the candidate engine")
    data = (candidate / "windows-x64-release-source-built.zip").read_bytes()
    if hashlib.sha256(data).hexdigest() != archive_sha256:
        raise ValueError("Candidate archive differs from the reviewed build")
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        if len(archive.infolist()) != 3 or set(archive.namelist()) != set(binaries):
            raise ValueError("Unexpected candidate engine files")
        files = {name: archive.read(name) for name in binaries}
    for name, content in files.items():
        if hashlib.sha256(content).hexdigest() != binaries[name]:
            raise ValueError("Candidate engine hash mismatch: " + name)
        if (target / name).is_symlink():
            raise ValueError("Engine destination cannot be a symlink")
    if SNAPSHOT_VERSION not in files["flutter_windows.dll"]:
        raise ValueError("Candidate runtime snapshot version is incompatible")
    # Validate the complete input before changing the disposable SDK cache.
    # Keep its matching gen_snapshot and SDK snapshots; do not copy arbitrary ZIP entries.
    for name, content in files.items():
        (target / name).write_bytes(content)
    if compiler.read_bytes() != compiler_bytes:
        raise ValueError("SDK compiler changed during engine selection")
    result = {
        "source_commit": source_commit,
        "workflow_run": workflow_run, "artifact": artifact,
        "archive_sha256": archive_sha256, "binary_files": binaries,
        "independent_rebuild_verified": bool(independent_runs),
        "independent_rebuild_runs": independent_runs,
        "sdk_engine_revision": SDK_ENGINE,
        "snapshot_compiler_sha256": hashlib.sha256(compiler_bytes).hexdigest(),
        "snapshot_version": SNAPSHOT_VERSION.decode(),
        "runtime_qualified": False, "production_approved": False,
        "scope": "Experimental native rebuild using the source engine and matching SDK snapshot compiler",
    }
    receipt.parent.mkdir(parents=True, exist_ok=True)
    receipt.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True, type=Path)
    parser.add_argument("--sdk", required=True, type=Path)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--source-candidate", choices=("source-candidate-9f97ef0", "source-candidate-05d5369"), default="source-candidate-9f97ef0")
    args = parser.parse_args()
    print(json.dumps(select(args.candidate, args.sdk, args.receipt, args.source_candidate), indent=2))
