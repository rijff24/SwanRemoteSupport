"""Exercise the real management image with fresh, disposable company storage.

Loopback HTTP is used only inside this CI lab. This checks container deployment
and persistence, not public HTTPS, RustDesk transport or native endpoint sessions.
Credentials stay in memory and are never printed or passed as command arguments.
"""
import base64
import hashlib
import hmac
import json
import re
import secrets
import struct
import subprocess
import sys
import time
import urllib.error
import urllib.request
import uuid


def docker(*arguments):
    return subprocess.check_output(["docker", *arguments], text=True, timeout=90).strip()


def authenticator(secret, offset=0):
    counter = int(time.time()) // 30 + offset
    digest = hmac.new(secret, struct.pack(">Q", counter), hashlib.sha1).digest()
    start = digest[-1] & 15
    value = struct.unpack(">I", digest[start:start + 4])[0] & 0x7FFFFFFF
    return str(value % 1000000).zfill(6)


def request(base, endpoint, method="GET", token=None, body=None, expected=200):
    headers = {"Content-Type": "application/json"}
    if token is not None:
        headers["Authorization"] = "Bearer " + token
    data = None if body is None else json.dumps(body).encode("utf-8")
    query = urllib.request.Request(base + endpoint, data=data, headers=headers, method=method)
    try:
        response = urllib.request.urlopen(query, timeout=10)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        if response.status != expected:
            raise RuntimeError("Unexpected HTTP status for " + endpoint + ": " + str(response.status))
        raw = response.read(1024 * 1024)
        if response.headers.get("Cache-Control") != "no-store":
            raise RuntimeError("Management responses must not be cached")
        return json.loads(raw) if "application/json" in response.headers.get("Content-Type", "") else raw.decode("utf-8")


def ready(base):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        try:
            result = request(base, "/health")
            if result == {"status": "ok", "product": "Swan Remote Support"}:
                return
        except (OSError, RuntimeError):
            time.sleep(0.2)
    raise RuntimeError("Management container did not become healthy")


def container_base(name):
    port = docker("port", name, "8080/tcp")
    if re.fullmatch(r"127\.0\.0\.1:[0-9]+", port) is None:
        raise RuntimeError("Lab container was not restricted to loopback")
    return "http://" + port


def secret_command(arguments, passphrase, expected_success=True):
    # Supply only stdin, never a password in Docker arguments or container metadata.
    result = subprocess.run(["docker", *arguments], input=passphrase + "\n",
                            capture_output=True, text=True, timeout=90)
    if (result.returncode == 0) != expected_success:
        # Report only known error categories, never captured output: diagnostics
        # can contain local paths, deployment settings or Docker arguments.
        categories = {
            "sqlite_readonly": "attempt to write a readonly database",
            "sqlite_open": "unable to open database file",
            "sqlite_locked": "database is locked",
            "sqlite_io": "disk I/O error",
            "readonly_filesystem": "Read-only file system",
            "sqlite_malformed": "database disk image is malformed",
            "permission": "Permission denied",
            "missing_file": "No such file or directory",
            "tls_path": "Unsupported TLS storage path",
            "backup_exists": "Refusing to overwrite an existing backup",
            "authentication": "Backup password incorrect or backup modified",
        }
        matched = [name for name, text in categories.items() if text.casefold() in result.stderr.casefold()]
        codes = re.findall(r"(?:Error code |os error )(\d+)", result.stderr)
        raise RuntimeError("Unexpected encrypted backup/restore command result; exit=" +
                           str(result.returncode) + "; categories=" + repr(matched) + "; codes=" + repr(codes))
    if not expected_success and "Backup password incorrect or backup modified" not in result.stderr:
        raise RuntimeError("Restore failed before demonstrating backup authentication rejection")


def restore_management(image, original, source_volume, status, administrator, enrolled, login_body, secret):
    name = "swan-restore-test-" + uuid.uuid4().hex
    helper = name + "-helper"
    volume = name + "-data"
    created_volume = False
    created_container = False
    try:
        before = request(container_base(original), "/api/v1/profile")
        passphrase = secrets.token_hex(32)
        shell = "IFS= read -r SWAN_BACKUP_PASSPHRASE; export SWAN_BACKUP_PASSPHRASE; exec swan-management "
        secret_command(["exec", "-i", original, "sh", "-ec",
                        shell + "backup /var/lib/swan/rehearsal.swanbackup"], passphrase)
        docker("exec", original, "sh", "-ec",
               'test "$(stat -c %a /var/lib/swan/rehearsal.swanbackup)" = 600; '
               'test "$(head -c 7 /var/lib/swan/rehearsal.swanbackup)" = SWANBK1')
        docker("volume", "create", volume)
        created_volume = True
        restore = ["run", "--rm", "--name", helper, "-i", "--read-only", "--network", "none",
                   "--tmpfs", "/tmp", "--security-opt", "no-new-privileges:true",
                   "--mount", "type=volume,src=" + source_volume + ",dst=/input,readonly",
                   "--mount", "type=volume,src=" + volume + ",dst=/var/lib/swan",
                   "--env", "SWAN_DATA_DIR=/var/lib/swan/restored", "--entrypoint", "sh", image,
                   "-ec", shell + "restore /input/rehearsal.swanbackup"]
        secret_command(restore, secrets.token_hex(32), expected_success=False)
        docker("run", "--rm", "--name", helper, "--read-only", "--network", "none",
               "--mount", "type=volume,src=" + volume + ",dst=/var/lib/swan", "--entrypoint", "sh",
               image, "-ec", "test ! -e /var/lib/swan/restored")
        secret_command(restore, passphrase)
        docker("create", "--name", name, "--read-only", "--tmpfs", "/tmp",
               "--security-opt", "no-new-privileges:true", "--mount", "type=volume,src=" + volume + ",dst=/var/lib/swan",
               "--env", "SWAN_DATA_DIR=/var/lib/swan/restored", "--publish", "127.0.0.1::8080", image)
        created_container = True
        docker("start", name)
        base = container_base(name)
        ready(base)
        if request(base, "/api/v1/status") != status:
            raise RuntimeError("Restored company trust or configuration state changed")
        after = request(base, "/api/v1/profile")
        prior_profile = json.loads(base64.b64decode(before["payload"], validate=True))
        restored_profile = json.loads(base64.b64decode(after["payload"], validate=True))
        for profile in [prior_profile, restored_profile]:
            profile.pop("issued_at")
            profile.pop("expires_at")
        if restored_profile != prior_profile:
            raise RuntimeError("Restore changed company branding or policy")
        inventory = request(base, "/api/v1/devices", token=administrator)
        if len(inventory) != 1 or inventory[0]["id"] != enrolled["device_id"] or inventory[0]["state"] != "approved" or inventory[0]["unattended"] is not False:
            raise RuntimeError("Restore lost enrollment, approval or consent")
        docker("exec", name, "sh", "-ec",
               'test "$(id -u)" = 10001; test "$(stat -c %a /var/lib/swan/restored)" = 700; '
               'test "$(stat -c %a /var/lib/swan/restored/profile-key.hex)" = 600; '
               'test "$(stat -c %a /var/lib/swan/restored/setup-token.txt)" = 600')
        request(base, "/api/v1/login", "POST", body=login_body, expected=401)
        deadline = time.monotonic() + 65
        while authenticator(secret, 1) == login_body["totp_code"]:
            if time.monotonic() >= deadline:
                raise RuntimeError("No fresh authenticator code available for restore verification")
            time.sleep(0.2)
        restored_login = dict(login_body, totp_code=authenticator(secret, 1))
        token = request(base, "/api/v1/login", "POST", body=restored_login)["token"]
        request(base, "/api/v1/login", "POST", body=restored_login, expected=401)
        request(base, "/api/v1/devices/" + enrolled["device_id"] + "/state", "PUT", token, {"state": "revoked", "group": "container-test"})
        request(base, "/api/v1/device/consent", "PUT", enrolled["device_token"], {"unattended": False, "revision": 1}, expected=401)
        original_inventory = request(container_base(original), "/api/v1/devices", token=administrator)
        if len(original_inventory) != 1 or original_inventory[0]["state"] != "approved":
            raise RuntimeError("Restored deployment mutated the original company volume")
        request(base, "/api/v1/logout", "POST", token)
        request(base, "/api/v1/logout", "POST", administrator)
        request(base, "/api/v1/devices", token=administrator, expected=401)
        print("PASS: encrypted management backup, wrong-password rejection without writes, restored company/device/consent identity, private keys, MFA/replay and revocation. Transport and TLS backup remain separate tests.")
    finally:
        if docker("ps", "-aq", "--filter", "name=^/" + helper + "$"):
            docker("rm", "--force", helper)
        if created_container:
            docker("rm", "--force", name)
        if created_volume:
            docker("volume", "rm", volume)


def main():
    image = sys.argv[1] if len(sys.argv) == 2 else "swan-management:test"
    name = "swan-company-test-" + uuid.uuid4().hex
    volume = name + "-data"
    created = False
    started = False
    try:
        docker("volume", "create", volume)
        created = True
        docker("create", "--name", name, "--read-only", "--tmpfs", "/tmp",
               "--security-opt", "no-new-privileges:true", "--mount", "type=volume,src=" + volume + ",dst=/var/lib/swan",
               "--publish", "127.0.0.1::8080", image)
        started = True
        docker("start", name)
        base = container_base(name)
        ready(base)
        if docker("exec", name, "id", "-u") != "10001":
            raise RuntimeError("Management container must run as the service account")
        docker("exec", name, "sh", "-ec",
               'test "$SSL_CERT_FILE" = /etc/ssl/certs/ca-certificates.crt; '
               'test -s "$SSL_CERT_FILE"; test -r "$SSL_CERT_FILE"; '
               'test "$(stat -c %a /var/lib/swan)" = 700; '
               'test "$(stat -c %a /var/lib/swan/profile-key.hex)" = 600; '
               'test "$(stat -c %a /var/lib/swan/setup-token.txt)" = 600; '
               'test -f /var/lib/swan/management.sqlite3')
        status = request(base, "/api/v1/status")
        if status["configured"] is not False or len(base64.b64decode(status["profile_public_key"], validate=True)) != 32:
            raise RuntimeError("Expected a fresh company with its own public trust key")
        if "Swan Remote Support" not in request(base, "/"):
            raise RuntimeError("Administration assets were not embedded in the image")
        request(base, "/api/v1/devices", expected=401)
        branding = request(base, "/api/v1/default-branding")
        branding["display_name"] = "Container Test Company"
        secret = secrets.token_bytes(20)
        password = secrets.token_hex(24)
        # A known public RFC 8032 test key; no signing private key is supplied.
        transport_key = base64.b64encode(bytes.fromhex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")).decode("ascii")
        profile = {"schema": 1, "company_id": "setup", "revision": 1, "issued_at": int(time.time()),
                   "expires_at": int(time.time()) + 3600, "management_url": "https://support.example.com",
                   "rendezvous": "support.example.com:21116", "relay": "support.example.com:21117",
                   "transport_public_key": transport_key, "customer": branding, "technician": branding,
                   "allow_unattended": False, "updates_paused": True, "rollout_percent": 0,
                   "maintenance_start_utc": 0, "maintenance_end_utc": 0, "update_channel": "test",
                   "next_profile_public_key": None}
        setup_token = docker("exec", name, "cat", "/var/lib/swan/setup-token.txt")
        company = request(base, "/api/v1/setup", "POST", setup_token,
                          {"profile": profile, "username": "container-admin", "password": password,
                           "totp_secret": base64.b32encode(secret).decode("ascii"), "totp_code": authenticator(secret)})
        login_body = {"username": "container-admin", "password": password, "totp_code": "invalid"}
        request(base, "/api/v1/login", "POST", body=login_body, expected=401)
        login_body["totp_code"] = authenticator(secret, 1)
        administrator = request(base, "/api/v1/login", "POST", body=login_body)["token"]
        request(base, "/api/v1/login", "POST", body=login_body, expected=401)
        enrolled = request(base, "/api/v1/enroll", "POST", body={"name": "Container test device", "rustdesk_id": "987654321", "unattended_consent": False})
        if enrolled["state"] != "pending" or enrolled["company_id"] != company["company_id"]:
            raise RuntimeError("Fresh enrollment must require company approval")
        request(base, "/api/v1/device/consent", "PUT", enrolled["device_token"], {"unattended": True, "revision": 1}, expected=401)
        request(base, "/api/v1/devices/" + enrolled["device_id"] + "/state", "PUT", administrator, {"state": "approved", "group": "container-test"})
        request(base, "/api/v1/device/consent", "PUT", enrolled["device_token"], {"unattended": True, "revision": 1}, expected=403)
        docker("stop", "--time", "10", name)
        if docker("inspect", "--format", "{{.State.ExitCode}}", name) != "0":
            raise RuntimeError("Management did not shut down cleanly on Docker SIGTERM")
        docker("start", name)
        base = container_base(name)
        ready(base)
        after = request(base, "/api/v1/status")
        if after["configured"] is not True or after["profile_public_key"] != status["profile_public_key"]:
            raise RuntimeError("Container restart changed company identity")
        inventory = request(base, "/api/v1/devices", token=administrator)
        if len(inventory) != 1 or inventory[0]["id"] != enrolled["device_id"] or inventory[0]["state"] != "approved" or inventory[0]["unattended"] is not False:
            raise RuntimeError("Container restart lost device, permission or consent state")
        restore_management(image, name, volume, after, administrator, enrolled, login_body, secret)
        request(base, "/api/v1/logout", "POST", administrator)
        request(base, "/api/v1/devices", token=administrator, expected=401)
        print("PASS: non-root/read-only container startup, private storage, company setup, MFA/replay denial, enrollment approval and restart persistence. HTTPS and native transport remain separate tests.")
    finally:
        if started:
            docker("rm", "--force", name)
        if created:
            docker("volume", "rm", volume)


if __name__ == "__main__":
    main()
