"""Check the shipped ignore rules with Docker, using public source and canaries.

No local environment files, Git credentials or runtime data are copied into the
fixture. The scratch exporter runs no commands and downloads no base image.
"""
import pathlib
import re
import shutil
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[2]
CANARIES = (
    ".env",
    ".git/config",
    "deployment/linux/company.env",
    "deployment/windows/local-test.env",
    "deployment/windows/custom-company.ps1",
    "product/management/src/local-test.env",
    "product/management/src/company-private.rs",
    "product/target/stale-build.exe",
    "branding/company-private.key",
    "swan-data/profile-key.hex",
    "swan-data/management.sqlite3",
)


def prepare_fixture(context):
    tracked = subprocess.check_output(
        ["git", "ls-files", "-z", "product", "deployment", "branding", "LICENCE"],
        cwd=ROOT,
    ).decode("utf-8").split("\0")
    required = {"LICENCE"}
    sources = []
    for relative in filter(None, tracked):
        source = ROOT / relative
        if source.is_symlink() or not source.is_file():
            raise RuntimeError("Build fixture requires regular tracked source files")
        destination = context / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        if relative.startswith("product/"):
            if source.suffix == ".rs":
                sources.append(source)
                required.add(relative)
            elif source.name in ("Cargo.toml", "Cargo.lock"):
                required.add(relative)
    for source in sources:
        for embedded in re.findall(r'include_(?:str|bytes)!\(\s*"([^"]+)"', source.read_text(encoding="utf-8")):
            relative = (source.parent / embedded).resolve().relative_to(ROOT).as_posix()
            if not (context / relative).is_file():
                raise RuntimeError("Embedded build input is not tracked public source: " + relative)
            required.add(relative)
    for relative in CANARIES:
        destination = context / relative
        if destination.exists():
            raise RuntimeError("Canary would overwrite tracked source")
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text("Swan build context exclusion canary; no actual credentials.\n", encoding="utf-8")
    return required


def main():
    with tempfile.TemporaryDirectory(prefix="swan-docker-context-") as temporary:
        lab = pathlib.Path(temporary)
        context = lab / "context"
        context.mkdir()
        required = prepare_fixture(context)
        probe = lab / "probe.Dockerfile"
        probe.write_text("FROM scratch\nCOPY . /\n", encoding="utf-8")
        shutil.copyfile(ROOT / "deployment/linux/Dockerfile.dockerignore", pathlib.Path(str(probe) + ".dockerignore"))
        output = lab / "export"
        subprocess.run(
            ["docker", "build", "--file", str(probe), "--output", "type=local,dest=" + str(output), str(context)],
            check=True,
            timeout=300,
        )
        actual = {path.relative_to(output).as_posix() for path in output.rglob("*") if path.is_file()}
        missing = required - actual
        unexpected = actual - required
        if missing or unexpected:
            raise RuntimeError("Docker context mismatch; missing=" + repr(sorted(missing)) + "; unexpected=" + repr(sorted(unexpected)))
        print("Docker exported every required input and excluded all configuration/key/cache canaries (" + str(len(actual)) + " public files).")


if __name__ == "__main__":
    main()
