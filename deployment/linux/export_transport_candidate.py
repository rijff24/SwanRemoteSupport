"""Retain public candidate binaries and their exact source from the tested image."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys
import zipfile

from test_container_stack import ROOT, maintained_transport_image
from test_container_startup import docker


def main():
    if len(sys.argv) != 2:
        raise SystemExit("Expected a new candidate artifact directory")
    output = Path(sys.argv[1]).resolve()
    if output.exists():
        raise SystemExit("Candidate artifact directory already exists")
    image = maintained_transport_image()
    revision = subprocess.check_output(["git", "-C", str(ROOT), "rev-parse", "HEAD"], text=True).strip()
    output.mkdir(parents=True)
    container = docker("create", "--network", "none", image)
    # No candidate executable runs during extraction. Remove only this owned,
    # stopped extraction container; failed output remains available for diagnosis.
    try:
        files = {"hbbs": "/usr/local/bin/hbbs", "hbbr": "/usr/local/bin/hbbr"}
        files.update({name: "/usr/share/doc/swan-transport/" + name for name in
                      ["SWAN-SOURCE.json", "transport-build-source.zip", "LICENCE"]})
        for name, remote in files.items():
            docker("cp", container + ":" + remote, str(output / name))
            path = output / name
            if path.is_symlink() or not path.is_file() or not 0 < path.stat().st_size <= 128 * 1024**2:
                raise RuntimeError("Unexpected candidate artifact")
        for name in ["hbbs", "hbbr"]:
            with (output / name).open("rb") as binary:
                header = binary.read(20)
            if header[:6] != b"\x7fELF\x02\x01" or header[18:20] != b"\x3e\x00":
                raise RuntimeError("Expected Linux x86-64 transport executable")
        receipt = json.loads((output / "SWAN-SOURCE.json").read_text())
        with zipfile.ZipFile(output / "transport-build-source.zip") as archive:
            entries = archive.infolist()
            if len(entries) != len({entry.filename for entry in entries}) or sum(entry.file_size for entry in entries) > 64 * 1024**2:
                raise RuntimeError("Duplicate or oversized transport source")
            for entry in entries:
                path = PurePosixPath(entry.filename)
                if path.is_absolute() or ".." in path.parts or "\\" in entry.filename or entry.is_dir():
                    raise RuntimeError("Unsafe transport source archive entry")
            if archive.testzip() is not None:
                raise RuntimeError("Transport source CRC mismatch")
            if json.loads(archive.read("rustdesk-server/SWAN-SOURCE.json")) != receipt:
                raise RuntimeError("Source archive and executable image receipts differ")
            udp = archive.read("rustdesk-server/libs/hbb_common/src/udp.rs").replace(b"\r\n", b"\n")
            if hashlib.sha256(udp).hexdigest() != receipt["patched_udp_sha256"]:
                raise RuntimeError("Archived source lacks the reviewed UDP patch")
        manifest = {"schema": 1, "candidate_only": True, "production_approved": False,
                    "repository": "https://github.com/rijff24/SwanRemoteSupport", "image_id": image,
                    "packaging_revision": revision, "transport_recipe_revision": receipt["project_revision"],
                    "dockerfile_sha256": hashlib.sha256((ROOT / "deployment/transport/Dockerfile").read_bytes()).hexdigest(),
                    "files": {name: {"sha256": hashlib.sha256((output / name).read_bytes()).hexdigest(),
                                     "bytes": (output / name).stat().st_size} for name in files},
                    "scope": "Public unsigned Linux candidate build inputs and outputs; not binary reproducibility or release approval"}
        (output / "SOURCE.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
        print("Retained Linux x86-64 candidate binaries, reviewed patched source, receipt and license.")
    finally:
        docker("rm", container)


if __name__ == "__main__":
    main()
