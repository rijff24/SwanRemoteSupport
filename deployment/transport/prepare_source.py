"""Apply reviewed transport patches only to the exact public source pins."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path


def git(directory, *arguments):
    return subprocess.check_output(["git", "-C", str(directory), *arguments])


def canonical(path):
    if not path.is_file() or path.is_symlink():
        raise ValueError("Expected a regular pinned source file")
    return path.read_bytes().replace(b"\r\n", b"\n")


def prepare(source, stage):
    project = Path(__file__).resolve().parent
    manifest = json.loads((project / "source.json").read_text())
    source = source.resolve(strict=True)
    shared = source / "libs/hbb_common"
    if git(source, "rev-parse", "HEAD").decode().strip() != manifest["commit"]:
        raise ValueError("Wrong transport source revision")
    if git(shared, "rev-parse", "HEAD").decode().strip() != manifest["hbb_common_commit"]:
        raise ValueError("Wrong shared transport source revision")
    if hashlib.sha256(canonical(source / "Cargo.lock")).hexdigest() != manifest["cargo_lock_sha256"]:
        raise ValueError("Pinned transport dependency lock changed")
    if stage == "tests" and git(source, "status", "--porcelain").strip():
        raise ValueError("Baseline checkout must be clean before adding the regression")
    entry = manifest["patches"][0 if stage == "tests" else 1]
    patch = (project / entry["path"]).resolve(strict=True)
    if patch.parent != project / "patches" or hashlib.sha256(patch.read_bytes()).hexdigest() != entry["sha256"]:
        raise ValueError("Transport patch identity differs")
    target = shared / "src/udp.rs"
    before = canonical(target)
    if hashlib.sha256(before).hexdigest() != entry["before_sha256"]:
        raise ValueError("Transport source differs from the reviewed patch input")
    # Use a canonical checkout representation for identical patch application on
    # Windows and Linux; Git still records the same public source content.
    target.write_bytes(before)
    git(shared, "apply", "--check", "--whitespace=error-all", str(patch))
    git(shared, "apply", "--whitespace=error-all", str(patch))
    if hashlib.sha256(canonical(target)).hexdigest() != entry["after_sha256"]:
        raise ValueError("Patched transport source differs from reviewed output")
    receipt = {"upstream": manifest, "stage": stage,
               "project_revision": git(project, "rev-parse", "HEAD").decode().strip(),
               "patched_udp_sha256": entry["after_sha256"], "unsigned_test": True}
    (source / "SWAN-SOURCE.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"stage": stage, "source": manifest["commit"],
                      "shared_source": manifest["hbb_common_commit"], "patch_verified": True}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("stage", choices=("tests", "fix"))
    arguments = parser.parse_args()
    prepare(arguments.source, arguments.stage)
