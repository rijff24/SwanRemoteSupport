"""Archive tracked transport inputs, patched shared code and the public recipe."""
import hashlib
import json
import subprocess
import sys
import zipfile
from pathlib import Path, PurePosixPath

source = Path(sys.argv[1]).resolve(strict=True)
output = Path(sys.argv[2]).resolve()
project = Path(__file__).resolve().parent
receipt = json.loads((source / "SWAN-SOURCE.json").read_text())
if receipt["stage"] != "fix":
    raise SystemExit("Refusing to archive an unpatched transport candidate")
pins = json.loads((project / "source.json").read_text())
shared = source / "libs/hbb_common"
if (receipt["upstream"] != pins
        or subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip() != pins["commit"]
        or subprocess.check_output(["git", "-C", str(shared), "rev-parse", "HEAD"], text=True).strip() != pins["hbb_common_commit"]):
    raise SystemExit("Transport archive does not use the pinned public revisions")
# Compilation can mutate the tracked SQLite schema database. A correctly
# ordered build must archive first; do not silently call a changed working tree
# the exact public inputs. Only the hash-verified UDP patch is allowed to differ.
for directory, excluded in [(source, "libs/hbb_common"), (shared, "src/udp.rs")]:
    subprocess.run(["git", "-C", str(directory), "diff", "--cached", "--quiet", "HEAD"], check=True)
    subprocess.run(["git", "-C", str(directory), "diff", "--quiet", "HEAD", "--", ".", ":!" + excluded], check=True)
udp = (shared / "src/udp.rs").read_bytes().replace(b"\r\n", b"\n")
if hashlib.sha256(udp).hexdigest() != pins["patches"][-1]["after_sha256"]:
    raise SystemExit("Transport archive UDP source does not match the reviewed patch")
tracked = subprocess.check_output(["git", "-C", str(source), "ls-files", "--recurse-submodules", "-z"])
files = {}
for raw in filter(None, tracked.split(b"\0")):
    name = raw.decode()
    relative = PurePosixPath(name)
    path = source.joinpath(*relative.parts)
    if relative.is_absolute() or any(part in ("..", ".git") for part in relative.parts) or path.is_symlink() or not path.is_file():
        raise SystemExit("Unsafe or missing tracked transport input")
    files["rustdesk-server/" + name] = path
files["rustdesk-server/SWAN-SOURCE.json"] = source / "SWAN-SOURCE.json"
for name in ("source.json", "prepare_source.py", "check_baseline.py", "archive_source.py",
             "patches/0001-udp-address-regression.patch", "patches/0002-udp-address-normalization.patch"):
    files["swan-recipe/" + name] = project / name
output.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED) as archive:
    for name, path in sorted(files.items()):
        archive.writestr(name, path.read_bytes())
print(json.dumps({"source_files": len(files), "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
                  "scope": "Exact transport build inputs and recipe; not binary reproducibility"}))
