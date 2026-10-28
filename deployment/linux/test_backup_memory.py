"""Run the real backup/restore CLI above its former cap with a bounded address space.

Uses generated, disposable company state only. Never accepts a live data path.
"""
import hashlib
import json
import os
from pathlib import Path
import resource
import secrets
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import uuid


def main():
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2])
    assert not output.exists(), "Preserve the existing qualification receipt"
    memory_limit = 512 * 1024**2
    chunk_size = 1024**2
    assert shutil.disk_usage(tempfile.gettempdir()).free >= 4 * 1024**3

    def bound_memory():
        resource.setrlimit(resource.RLIMIT_AS, (memory_limit, memory_limit))

    with tempfile.TemporaryDirectory(prefix="swan-backup-memory-") as temporary:
        root = Path(temporary)
        data = root / "company"
        data.mkdir(mode=0o700)
        with sqlite3.connect(data / "management.sqlite3") as db:
            db.executescript("CREATE TABLE identity(value TEXT); INSERT INTO identity VALUES('generated company fixture');")
        key = secrets.token_bytes(32)
        (data / "profile-key.hex").write_text(key.hex(), encoding="ascii")
        (data / "setup-token.txt").write_text(secrets.token_urlsafe(32), encoding="ascii")
        artifacts = data / "artifacts"
        artifacts.mkdir(mode=0o700)
        name = str(uuid.uuid4()) + ".zip"
        expected = hashlib.sha256()
        with (artifacts / name).open("xb") as file:
            for _ in range(513):
                chunk = secrets.token_bytes(chunk_size)
                file.write(chunk)
                expected.update(chunk)
        environment = os.environ.copy()
        environment.update(SWAN_DATA_DIR=str(data), SWAN_BACKUP_PASSPHRASE=secrets.token_urlsafe(48))
        backup = root / "company.swan-backup"
        target = root / "restored"

        def run(arguments):
            result = subprocess.run([str(binary), *arguments], env=environment,
                                    capture_output=True, timeout=600, preexec_fn=bound_memory)
            # Do not echo potentially sensitive command output or environment.
            assert result.returncode == 0, "Memory-limited management CLI failed (exit %s)" % result.returncode

        run(["backup", str(backup), "--installer-artifacts-directory", str(artifacts)])
        assert backup.stat().st_size > 512 * 1024**2
        with backup.open("rb") as file:
            assert file.read(7) == b"SWANBK2"
        environment["SWAN_DATA_DIR"] = str(target)
        run(["restore", str(backup)])
        actual = hashlib.sha256()
        received = target / "artifacts" / name
        assert received.stat().st_size == 513 * chunk_size
        with received.open("rb") as file:
            while chunk := file.read(chunk_size):
                actual.update(chunk)
        assert actual.digest() == expected.digest()
        assert (target / "profile-key.hex").read_text(encoding="ascii") == key.hex()
        assert (target / "setup-token.txt").read_bytes() == (data / "setup-token.txt").read_bytes()
        with sqlite3.connect(target / "management.sqlite3") as db:
            assert db.execute("PRAGMA quick_check").fetchone()[0] == "ok"
            assert db.execute("SELECT value FROM identity").fetchone()[0] == "generated company fixture"
        assert target.stat().st_mode & 0o777 == 0o700
        assert (target / "profile-key.hex").stat().st_mode & 0o777 == 0o600
        assert not list(root.glob(".swan-backup-*"))
        assert not list(data.glob(".swan-backup-*"))
        receipt = {"actual_management_cli": True, "format": "SWANBK2",
                   "maximum_virtual_address_space_bytes_per_cli_process": memory_limit,
                   "incompressible_artifact_bytes": received.stat().st_size,
                   "encrypted_backup_bytes": backup.stat().st_size,
                   "artifact_sha256_preserved": actual.hexdigest(),
                   "identity_keys_and_setup_token_preserved": True,
                   "sqlite_integrity_passed": True, "private_unix_permissions_passed": True,
                   "temporary_archive_cleanup_passed": True, "production_qualification": False}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x", encoding="utf-8") as file:
        json.dump(receipt, file, indent=2)
    print("Real backup and restore passed above 512 MiB under a 512 MiB address-space cap.")


if __name__ == "__main__":
    main()
