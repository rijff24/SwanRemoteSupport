"""Rehearse pinned company components with isolated loopback HTTPS.

An internal test CA replaces public ACME only in the disposable proxy config.
No system trust store is changed. TCP startup is not native remote-session,
UDP traversal, public reachability or production certificate-renewal evidence.
"""
import base64
import json
import ipaddress
import pathlib
import re
import secrets
import socket
import ssl
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
import uuid

from test_container_startup import authenticator, docker, secret_command


ROOT = pathlib.Path(__file__).resolve().parents[2]


def complete_backup(image, prefix, volumes, lab, proxy_image, transport_image, context, https_port,
                    signed, status, administrator, enrolled, login, secret):
    """Export stopped components and authenticate a restore into a fresh volume.

    Private keys stay inside Docker volumes; the passphrase travels over stdin.
    Restart replacement services using the original HTTPS and transport trust.
    """
    management, transport, tls, _ = volumes
    restored = prefix + "-archive-restore"
    docker("volume", "create", restored)
    replacement_volumes = []
    replacement_containers = []
    network = prefix + "-recovered"
    network_created = False
    password = secrets.token_hex(32)
    common = ["run", "--rm", "-i", "--user", "0", "--read-only", "--network", "none",
              "--tmpfs", "/tmp", "--security-opt", "no-new-privileges:true"]
    read_password = "IFS= read -r SWAN_BACKUP_PASSPHRASE; export SWAN_BACKUP_PASSPHRASE; "
    try:
        # Distinct exports exercise the same stopped, read-only transport/TLS
        # sources repeatedly. A failed export is not retried or overwritten.
        for filename in ["complete.swanbackup", "export-check-1.swanbackup", "export-check-2.swanbackup"]:
            secret_command(common + ["--mount", "type=volume,src=" + management + ",dst=/var/lib/swan",
                "--mount", "type=volume,src=" + transport + ",dst=/transport,readonly",
                "--mount", "type=volume,src=" + tls + ",dst=/tls,readonly",
                "--mount", "type=bind,src=" + str(lab / "local-test.env") + ",dst=/local-test.env,readonly",
                "--entrypoint", "sh", image, "-ec", read_password +
                "exec swan-management backup /var/lib/swan/" + filename + " --transport-directory /transport --tls-directory /tls --deployment-env /local-test.env"], password)
        restore = common + ["--mount", "type=volume,src=" + management + ",dst=/input,readonly",
            "--mount", "type=volume,src=" + restored + ",dst=/var/lib/swan",
            "--env", "SWAN_DATA_DIR=/var/lib/swan/restored", "--entrypoint", "sh", image, "-ec",
            read_password + "exec swan-management restore /input/complete.swanbackup"]
        secret_command(restore, secrets.token_hex(32), expected_success=False)
        docker(*common[:2], "--user", "0", "--network", "none", "--read-only",
               "--mount", "type=volume,src=" + restored + ",dst=/restored,readonly",
               "--entrypoint", "sh", image, "-ec", "test ! -e /restored/restored")
        secret_command(restore, password)
        # Compare actual private artifacts without returning their contents.
        docker("run", "--rm", "--user", "0", "--read-only", "--network", "none",
            "--mount", "type=volume,src=" + restored + ",dst=/restore,readonly",
            "--mount", "type=volume,src=" + transport + ",dst=/transport,readonly",
            "--mount", "type=volume,src=" + tls + ",dst=/tls,readonly",
            "--mount", "type=bind,src=" + str(lab / "local-test.env") + ",dst=/local-test.env,readonly",
            "--entrypoint", "sh", image, "-ec",
            "test -s /restore/restored/management.sqlite3 || { echo missing-database >&2; exit 1; }; "
            "cmp -s /transport/id_ed25519 /restore/restored/transport-id_ed25519 || { echo private-key-mismatch >&2; exit 1; }; "
            "cmp -s /transport/id_ed25519.pub /restore/restored/transport-id_ed25519.pub || { echo public-key-mismatch >&2; exit 1; }; "
            "cmp -s /local-test.env /restore/restored/deployment.env || { echo environment-mismatch >&2; exit 1; }; "
            "find /tls -type f -exec sh -ec 'for file do relative=${file#/tls/}; cmp -s \"$file\" \"/restore/restored/tls-storage/$relative\" || exit 1; done' sh {} +; test $(find /tls -type f | wc -l) = $(find /restore/restored/tls-storage -type f | wc -l); "
            "test $(stat -c %a /restore/restored) = 700 || { echo directory-mode >&2; exit 1; }; "
            "test $(stat -c %a /restore/restored/transport-id_ed25519) = 600 || { echo key-mode >&2; exit 1; }")
        for role in ["transport", "tls", "proxy"]:
            volume = prefix + "-recovered-" + role
            docker("volume", "create", volume)
            replacement_volumes.append(volume)
        recovered_transport, recovered_tls, recovered_proxy = replacement_volumes
        docker("run", "--rm", "--user", "0", "--read-only", "--network", "none",
            "--mount", "type=volume,src=" + restored + ",dst=/restore",
            "--mount", "type=volume,src=" + recovered_transport + ",dst=/transport",
            "--mount", "type=volume,src=" + recovered_tls + ",dst=/tls",
            "--entrypoint", "sh", image, "-ec",
            "cp /restore/restored/transport-id_ed25519 /transport/id_ed25519; "
            "cp /restore/restored/transport-id_ed25519.pub /transport/id_ed25519.pub; "
            "if test -f /restore/restored/transport-db.sqlite3; then cp /restore/restored/transport-db.sqlite3 /transport/db_v2.sqlite3; fi; "
            "chmod 700 /transport /tls; chmod 600 /transport/*; "
            "cp -a /restore/restored/tls-storage/. /tls/; "
            "chown swan:swan /restore/restored /restore/restored/management.sqlite3 /restore/restored/profile-key.hex /restore/restored/setup-token.txt")
        docker("network", "create", network)
        network_created = True
        names = {role: network + "-" + role for role in ["management", "hbbs", "hbbr", "https"]}
        docker("create", "--name", names["management"], "--network", network, "--network-alias", "management",
               "--read-only", "--tmpfs", "/tmp", "--security-opt", "no-new-privileges:true",
               "--mount", "type=volume,src=" + restored + ",dst=/var/lib/swan",
               "--env", "SWAN_DATA_DIR=/var/lib/swan/restored", image)
        replacement_containers.append(names["management"])
        for role, services, command in [("hbbs", ["21115", "21116"], ["hbbs", "-r", "hbbr:21117", "-k", "_"]),
                                         ("hbbr", ["21117"], ["hbbr", "-k", "_"])]:
            arguments = ["create", "--name", names[role], "--network", network, "--network-alias", role,
                         "--mount", "type=volume,src=" + recovered_transport + ",dst=/root"]
            for service in services:
                arguments += ["--publish", "127.0.0.1::" + service + "/tcp"]
            docker(*arguments, transport_image, *command)
            replacement_containers.append(names[role])
        docker("create", "--name", names["https"], "--network", network, "--read-only", "--tmpfs", "/tmp",
               "--security-opt", "no-new-privileges:true", "--env", "SWAN_HOST=localhost",
               "--mount", "type=bind,src=" + str(lab / "Caddyfile") + ",dst=/etc/caddy/Caddyfile,readonly",
               "--mount", "type=volume,src=" + recovered_tls + ",dst=/data",
               "--mount", "type=volume,src=" + recovered_proxy + ",dst=/config",
               "--publish", "127.0.0.1:" + str(https_port) + ":443/tcp", proxy_image)
        replacement_containers.append(names["https"])
        for role in ["management", "hbbs", "hbbr", "https"]:
            docker("start", names[role])
        endpoint = Endpoint(names["https"], context)
        endpoint.ready()
        recovered = verify_profile(endpoint.request("/api/v1/profile"), status["profile_public_key"], lab)
        for key in signed:
            if key not in ["issued_at", "expires_at"] and recovered[key] != signed[key]:
                raise RuntimeError("Replacement changed trusted company profile")
        public_file(names["hbbs"], "/root/id_ed25519.pub", lab / "recovered-transport.pub")
        if (lab / "recovered-transport.pub").read_bytes() != (lab / "transport.pub").read_bytes():
            raise RuntimeError("Replacement changed transport identity")
        for service in ["21115/tcp", "21116/tcp"]:
            container_nat_probe(names["hbbs"], service)
        with socket.create_connection(("127.0.0.1", port(names["hbbr"], "21117/tcp")), timeout=5):
            pass
        endpoint.request("/api/v1/devices", token=administrator, expected=401)
        endpoint.request("/api/v1/login", "POST", body=login, expected=401)
        deadline = time.monotonic() + 65
        while authenticator(secret, 1) == login["totp_code"]:
            if time.monotonic() >= deadline:
                raise RuntimeError("No fresh MFA code available for replacement")
            time.sleep(0.2)
        fresh_login = dict(login, totp_code=authenticator(secret, 1))
        token = endpoint.request("/api/v1/login", "POST", body=fresh_login)["token"]
        endpoint.request("/api/v1/login", "POST", body=fresh_login, expected=401)
        inventory = endpoint.request("/api/v1/devices", token=token)
        if len(inventory) != 1 or inventory[0]["id"] != enrolled["device_id"] or inventory[0]["state"] != "pending" or inventory[0]["unattended"]:
            raise RuntimeError("Replacement lost enrollment or customer consent")
        endpoint.request("/api/v1/devices/" + enrolled["device_id"] + "/state", "PUT", token,
                         {"state": "revoked", "group": "recovery-test"})
        endpoint.request("/api/v1/device/consent", "PUT", enrolled["device_token"],
                         {"unattended": True, "revision": 1}, expected=401)
        endpoint.request("/api/v1/logout", "POST", token)
        endpoint.request("/api/v1/devices", token=token, expected=401)
    finally:
        for name in reversed(replacement_containers):
            docker("rm", "--force", name)
        if network_created:
            docker("network", "rm", network)
        for volume in reversed(replacement_volumes):
            docker("volume", "rm", volume)
        docker("volume", "rm", restored)


def pinned_images():
    compose = (ROOT / "deployment/linux/compose.yaml").read_text(encoding="utf-8")
    images = re.findall(r"^\s+image:\s+(\S+)\s*$", compose, re.MULTILINE)
    proxies = [value for value in images if value.startswith("caddy:")]
    transport = json.loads((ROOT / "deployment/transport-source.json").read_text(encoding="utf-8"))["image"]
    if len(proxies) != 1 or re.fullmatch(r"caddy:[0-9.]+@sha256:[0-9a-f]{64}", proxies[0]) is None:
        raise RuntimeError("HTTPS proxy must use one pinned manifest")
    if re.fullmatch(r"rustdesk/rustdesk-server:[0-9.]+@sha256:[0-9a-f]{64}", transport) is None or images.count(transport) != 2:
        raise RuntimeError("Both transport components must match the source manifest")
    return proxies[0], transport


def port(name, service):
    mapping = docker("port", name, service)
    if re.fullmatch(r"127\.0\.0\.1:[0-9]+", mapping) is None:
        raise RuntimeError("Stack test endpoint must be restricted to loopback")
    return int(mapping.rsplit(":", 1)[1])


def public_file(name, remote, destination):
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline:
        result = subprocess.run(["docker", "cp", name + ":" + remote, str(destination)], capture_output=True, timeout=10)
        if result.returncode == 0 and destination.is_file():
            return
        time.sleep(0.2)
    raise RuntimeError("Component did not publish its public trust file")


def container_nat_probe(name, service):
    # Docker's loopback publisher may present a loopback peer to hbbs, whose
    # NAT-test port interprets that peer as an administrative text connection.
    # Probe the owned bridge endpoint; published host sockets are checked apart.
    address = docker("inspect", "--format", "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}", name)
    parsed = ipaddress.ip_address(address)
    if not parsed.is_private or parsed.is_loopback or parsed.is_unspecified:
        raise RuntimeError("NAT probe requires a disposable private bridge endpoint")
    if service not in ["21115/tcp", "21116/tcp"]:
        raise RuntimeError("Unexpected NAT-test port")
    try:
        deadline = time.monotonic() + 30
        while True:
            try:
                nat_probe(int(service.split("/")[0]), address)
                break
            except ConnectionRefusedError:
                # Published proxy sockets and the public-key file can become
                # ready before hbbs binds its own listener. Wait on the same
                # owned container; malformed replies are never retried.
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.2)
    except OSError as error:
        # Preserve startup evidence before the fixture removes its containers.
        # hbbs output contains its public transport identity, not management
        # credentials; never collect environment/configuration or private files.
        state = docker("inspect", "--format", "{{json .State}}", name)
        logs = docker("logs", "--tail", "30", name)
        raise RuntimeError("Owned hbbs NAT probe failed on " + service + "; state=" + state + "; log=" + logs) from error


def nat_probe(listener, host="127.0.0.1"):
    # TestNatRequest field 20, serial=0, from hbb_common/protos/rendezvous.proto.
    # The one-byte length header follows hbb_common/src/bytes_codec.rs.
    def exact(connection, size):
        result = bytearray()
        while len(result) < size:
            part = connection.recv(size - len(result))
            if not part:
                raise RuntimeError("NAT probe connection closed before a response")
            result.extend(part)
        return bytes(result)

    def integer(data, offset):
        value = 0
        for shift in range(0, 35, 7):
            if offset >= len(data):
                break
            byte = data[offset]
            offset += 1
            value |= (byte & 127) << shift
            if byte < 128:
                return value, offset
        raise RuntimeError("Invalid NAT response integer")

    with socket.create_connection((host, listener), timeout=5) as connection:
        connection.sendall(b"\x0c\xa2\x01\x00")
        for _ in range(3):
            first = exact(connection, 1)
            header = first + exact(connection, first[0] & 3)
            length = int.from_bytes(header, "little") >> 2
            if length > 65536:
                raise RuntimeError("NAT response exceeds probe limit")
            frame = exact(connection, length)
            if frame.startswith(b"\xca\x01"):
                continue  # Native NAT discovery also skips KeyExchange.
            if not frame.startswith(b"\xaa\x01"):
                raise RuntimeError("Rendezvous did not return TestNatResponse")
            size, start = integer(frame, 2)
            if start + size != len(frame) or frame[start:start + 1] != b"\x08":
                raise RuntimeError("Invalid NAT response payload")
            observed, _ = integer(frame, start + 1)
            if not 0 < observed <= 65535:
                raise RuntimeError("NAT probe did not report a peer port")
            return
    raise RuntimeError("NAT probe did not receive its protocol response")


class Endpoint:
    def __init__(self, proxy, context):
        self.base = "https://localhost:" + str(port(proxy, "443/tcp"))
        self.context = context

    def ready(self):
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            try:
                if self.request("/health") == {"status": "ok", "product": "Swan Remote Support"}:
                    return
            except (OSError, RuntimeError):
                pass
            time.sleep(0.2)
        raise RuntimeError("HTTPS proxy did not reach healthy management")

    def request(self, endpoint, method="GET", token=None, body=None, expected=200):
        headers = {"Content-Type": "application/json"}
        if token is not None:
            headers["Authorization"] = "Bearer " + token
        data = None if body is None else json.dumps(body).encode("utf-8")
        query = urllib.request.Request(self.base + endpoint, data=data, headers=headers, method=method)
        try:
            response = urllib.request.urlopen(query, context=self.context, timeout=10)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            if response.status != expected or response.headers.get("Cache-Control") != "no-store":
                raise RuntimeError("Unexpected HTTPS response for " + endpoint)
            if response.headers.get("Strict-Transport-Security") != "max-age=31536000":
                raise RuntimeError("Proxy lost the shipped HTTPS policy")
            content = response.read(2 * 1024 * 1024)
            return json.loads(content) if "application/json" in response.headers.get("Content-Type", "") else content.decode("utf-8")


def verify_profile(envelope, public_key, directory):
    payload = base64.b64decode(envelope["payload"], validate=True)
    signature = base64.b64decode(envelope["signature"], validate=True)
    key = base64.b64decode(public_key, validate=True)
    if len(key) != 32 or len(signature) != 64:
        raise RuntimeError("Invalid profile signing identity")
    key_file, data_file, signature_file = [directory / name for name in ["public.der", "profile.json", "signature.bin"]]
    key_file.write_bytes(bytes.fromhex("302a300506032b6570032100") + key)
    data_file.write_bytes(payload)
    signature_file.write_bytes(signature)
    arguments = ["openssl", "pkeyutl", "-verify", "-rawin", "-pubin", "-keyform", "DER", "-inkey", str(key_file), "-in", str(data_file), "-sigfile", str(signature_file)]
    if subprocess.run(arguments, capture_output=True).returncode != 0:
        raise RuntimeError("Actual company profile signature failed")
    data_file.write_bytes(payload + b" ")
    if subprocess.run(arguments, capture_output=True).returncode == 0:
        raise RuntimeError("Tampered company profile was accepted")
    data_file.write_bytes(payload)
    key_file.write_bytes(bytes.fromhex("302a300506032b6570032100") + bytes([key[0] ^ 1]) + key[1:])
    if subprocess.run(arguments, capture_output=True).returncode == 0:
        raise RuntimeError("Wrong company signing key was accepted")
    return json.loads(payload)


def main():
    image = sys.argv[1] if len(sys.argv) == 2 else "swan-management:test"
    proxy_image, transport_image = pinned_images()
    prefix = "swan-stack-test-" + uuid.uuid4().hex
    names = {role: prefix + "-" + role for role in ["management", "https", "hbbs", "hbbr"]}
    volumes = []
    containers = []
    network_created = False
    try:
        with tempfile.TemporaryDirectory(prefix="swan-stack-config-") as temporary:
            lab = pathlib.Path(temporary)
            original = (ROOT / "deployment/linux/Caddyfile").read_text(encoding="utf-8")
            if original.count("reverse_proxy management:8080") != 1:
                raise RuntimeError("Recheck the test override against the proxy recipe")
            config = lab / "Caddyfile"
            config.write_text("{\n    skip_install_trust\n}\n" + original.replace("reverse_proxy management:8080", "tls internal\n    reverse_proxy management:8080"), encoding="utf-8")
            (lab / "local-test.env").write_text("SWAN_HOST=localhost\nSWAN_MANAGEMENT_IMAGE=" + image + "\n", encoding="utf-8")
            docker("network", "create", prefix)
            network_created = True
            for role in ["management", "transport", "tls", "proxy-config"]:
                volume = prefix + "-" + role + "-data"
                docker("volume", "create", volume)
                volumes.append(volume)
            management_data, transport_data, tls_data, proxy_data = volumes
            docker("create", "--name", names["management"], "--network", prefix, "--network-alias", "management",
                   "--read-only", "--tmpfs", "/tmp", "--security-opt", "no-new-privileges:true",
                   "--mount", "type=volume,src=" + management_data + ",dst=/var/lib/swan", image)
            containers.append(names["management"])
            docker("create", "--name", names["hbbs"], "--network", prefix, "--network-alias", "hbbs",
                   "--mount", "type=volume,src=" + transport_data + ",dst=/root",
                   "--publish", "127.0.0.1::21115/tcp", "--publish", "127.0.0.1::21116/tcp",
                   transport_image, "hbbs", "-r", "hbbr:21117", "-k", "_")
            containers.append(names["hbbs"])
            docker("create", "--name", names["hbbr"], "--network", prefix, "--network-alias", "hbbr",
                   "--mount", "type=volume,src=" + transport_data + ",dst=/root",
                   "--publish", "127.0.0.1::21117/tcp", transport_image, "hbbr", "-k", "_")
            containers.append(names["hbbr"])
            with socket.socket() as available:
                available.bind(("127.0.0.1", 0))
                https_port = available.getsockname()[1]
            with (lab / "local-test.env").open("a", encoding="utf-8") as environment:
                environment.write("SWAN_HTTPS_TEST_PORT=" + str(https_port) + "\n")
            docker("create", "--name", names["https"], "--network", prefix, "--read-only", "--tmpfs", "/tmp",
                   "--security-opt", "no-new-privileges:true", "--env", "SWAN_HOST=localhost",
                   "--mount", "type=bind,src=" + str(config) + ",dst=/etc/caddy/Caddyfile,readonly",
                   "--mount", "type=volume,src=" + tls_data + ",dst=/data",
                   "--mount", "type=volume,src=" + proxy_data + ",dst=/config",
                   "--publish", "127.0.0.1:" + str(https_port) + ":443/tcp", proxy_image)
            containers.append(names["https"])
            docker("start", names["management"])
            docker("start", names["hbbs"])
            transport_public = lab / "transport.pub"
            public_file(names["hbbs"], "/root/id_ed25519.pub", transport_public)
            transport_key = transport_public.read_text().strip()
            if len(base64.b64decode(transport_key, validate=True)) != 32:
                raise RuntimeError("Transport did not generate a valid public identity")
            docker("start", names["hbbr"])
            docker("start", names["https"])
            authority = lab / "root.crt"
            public_file(names["https"], "/data/caddy/pki/authorities/local/root.crt", authority)
            context = ssl.create_default_context(cafile=str(authority))
            endpoint = Endpoint(names["https"], context)
            endpoint.ready()
            try:
                urllib.request.urlopen(endpoint.base + "/health", timeout=10)
            except urllib.error.URLError as error:
                if not isinstance(error.reason, ssl.SSLCertVerificationError):
                    raise RuntimeError("Expected actual untrusted-certificate rejection") from error
            else:
                raise RuntimeError("Lab issuer unexpectedly entered system trust")
            for role, service in [("hbbs", "21115/tcp"), ("hbbs", "21116/tcp"), ("hbbr", "21117/tcp")]:
                with socket.create_connection(("127.0.0.1", port(names[role], service)), timeout=5):
                    pass
            for service in ["21115/tcp", "21116/tcp"]:
                container_nat_probe(names["hbbs"], service)
            status = endpoint.request("/api/v1/status")
            branding = endpoint.request("/api/v1/default-branding")
            branding["display_name"] = "HTTPS Stack Test Company"
            secret = secrets.token_bytes(20)
            password = secrets.token_hex(24)
            profile = {"schema": 1, "company_id": "setup", "revision": 1, "issued_at": int(time.time()), "expires_at": int(time.time()) + 3600,
                       "management_url": endpoint.base, "rendezvous": "hbbs:21116", "relay": "hbbr:21117", "transport_public_key": transport_key,
                       "customer": branding, "technician": branding, "allow_unattended": False, "updates_paused": True, "rollout_percent": 0,
                       "maintenance_start_utc": 0, "maintenance_end_utc": 0, "update_channel": "test", "next_profile_public_key": None}
            setup_token = docker("exec", names["management"], "cat", "/var/lib/swan/setup-token.txt")
            company = endpoint.request("/api/v1/setup", "POST", setup_token,
                                       {"profile": profile, "username": "stack-admin", "password": password, "totp_secret": base64.b32encode(secret).decode("ascii"), "totp_code": authenticator(secret)})
            login = {"username": "stack-admin", "password": password, "totp_code": authenticator(secret, 1)}
            administrator = endpoint.request("/api/v1/login", "POST", body=login)["token"]
            endpoint.request("/api/v1/login", "POST", body=login, expected=401)
            signed = verify_profile(endpoint.request("/api/v1/profile"), status["profile_public_key"], lab)
            if signed["company_id"] != company["company_id"] or signed["transport_public_key"] != transport_key or signed["customer"]["display_name"] != branding["display_name"]:
                raise RuntimeError("Signed HTTPS profile lost company or transport identity")
            enrolled = endpoint.request("/api/v1/enroll", "POST", body={"name": "HTTPS test device", "rustdesk_id": "123456789", "unattended_consent": False})
            if enrolled["state"] != "pending":
                raise RuntimeError("HTTPS enrollment bypassed approval")
            endpoint.request("/api/v1/device/consent", "PUT", enrolled["device_token"], {"unattended": True, "revision": 1}, expected=401)
            endpoint.request("/api/v1/logout", "POST", administrator)
            endpoint.request("/api/v1/devices", token=administrator, expected=401)
            for role in ["https", "management", "hbbs", "hbbr"]:
                docker("stop", "--time", "10", names[role])
            complete_backup(image, prefix, volumes, lab, proxy_image, transport_image, context, https_port,
                            signed, status, administrator, enrolled, login, secret)
            for role in ["management", "hbbs", "hbbr", "https"]:
                docker("start", names[role])
            endpoint = Endpoint(names["https"], context)
            endpoint.ready()
            restored = verify_profile(endpoint.request("/api/v1/profile"), status["profile_public_key"], lab)
            after_public = lab / "transport-after.pub"
            public_file(names["hbbs"], "/root/id_ed25519.pub", after_public)
            if after_public.read_bytes() != transport_public.read_bytes() or restored["company_id"] != company["company_id"] or restored["management_url"] != endpoint.base:
                raise RuntimeError("Stack restart changed company or transport trust")
            endpoint.request("/api/v1/devices", token=administrator, expected=401)
            for service in ["21115/tcp", "21116/tcp"]:
                container_nat_probe(names["hbbs"], service)
            original_login = dict(login, totp_code=authenticator(secret, 1))
            original_token = endpoint.request("/api/v1/login", "POST", body=original_login)["token"]
            original_inventory = endpoint.request("/api/v1/devices", token=original_token)
            if len(original_inventory) != 1 or original_inventory[0]["id"] != enrolled["device_id"] or original_inventory[0]["state"] != "pending":
                raise RuntimeError("Replacement revocation mutated original deployment")
            endpoint.request("/api/v1/logout", "POST", original_token)
            print("PASS: pinned HTTPS/rendezvous/relay startup, local native NAT-test responses, explicit CA validation and untrusted rejection, actual profile signatures, MFA/enrollment/logout over HTTPS, complete encrypted archive recovery and replacement startup with original trust. Native sessions, UDP traversal, public ACME and external reachability remain unverified.")
    finally:
        for name in reversed(containers):
            docker("rm", "--force", name)
        for volume in reversed(volumes):
            docker("volume", "rm", volume)
        if network_created:
            docker("network", "rm", prefix)


if __name__ == "__main__":
    main()
