"""Exercise native cached-profile time validation without network or installed state.

Requires Python cryptography. The signing key is ephemeral and never serialized.
Use an independently hash-verified swan-agent executable and a fresh private output
directory. This harness does not install software or establish a remote session.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("agent", type=Path)
    parser.add_argument("expected_sha256")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    agent = args.agent.resolve(strict=True)
    digest = hashlib.sha256(agent.read_bytes()).hexdigest()
    if digest != args.expected_sha256.lower():
        raise SystemExit("Native agent hash mismatch")
    output = args.output.resolve()
    output.mkdir(parents=False, exist_ok=False)
    key = Ed25519PrivateKey.generate()
    public = base64.b64encode(key.public_key().public_bytes(
        serialization.Encoding.Raw, serialization.PublicFormat.Raw)).decode()
    bootstrap = dict(schema=1, edition="technician", company_id="time-validation-lab",
                     management_url="https://time-validation.example.invalid",
                     profile_public_key=public, release_public_key=public)
    branding = dict(display_name="Time validation lab", primary_color="#007F82",
                    logo_svg="", support_url="", consent_text="Approve lab support.")
    now = int(time.time())
    cases = [("valid", now - 60, now + 3600, True),
             ("expired", now - 3600, now - 60, False),
             ("future", now + 3600, now + 7200, False),
             ("invalid_lifetime", now - 60, now - 120, False)]
    results = []
    for name, issued, expires, accepted in cases:
        directory = output / name
        directory.mkdir()
        profile = dict(schema=1, company_id=bootstrap["company_id"], revision=1,
                       issued_at=issued, expires_at=expires,
                       management_url=bootstrap["management_url"],
                       rendezvous="transport.example.invalid", relay="transport.example.invalid",
                       transport_public_key=public, customer=branding, technician=branding,
                       allow_unattended=False, updates_paused=True, rollout_percent=0,
                       maintenance_start_utc=0, maintenance_end_utc=23,
                       update_channel="test", next_profile_public_key=None)
        payload = json.dumps(profile, separators=(",", ":")).encode()
        signature = key.sign(payload)
        key.public_key().verify(signature, payload)
        state = dict(bootstrap=bootstrap, profile=dict(
            payload=base64.b64encode(payload).decode(),
            signature=base64.b64encode(signature).decode()), accepted_revision=1,
            device_id=None, device_token=None, unattended_consent=False,
            consent_revision=0, last_release_sequence=0)
        state_path = directory / "managed-state.json"
        state_path.write_text(json.dumps(state), encoding="utf-8")
        bootstrap_path = directory / "bootstrap.json"
        bootstrap_path.write_text(json.dumps(bootstrap), encoding="utf-8")
        before = state_path.read_bytes()
        env = os.environ.copy()
        env["SWAN_STATE_DIR"] = str(directory)
        completed = subprocess.run([str(agent), "verify-bootstrap", str(bootstrap_path)],
                                   env=env, capture_output=True, text=True, timeout=30)
        if (completed.returncode == 0) != accepted:
            raise RuntimeError(f"Unexpected native validation outcome: {name}")
        if not accepted and "Expired or future profile" not in completed.stderr:
            raise RuntimeError(f"Rejection did not exercise profile time validation: {name}")
        if state_path.read_bytes() != before:
            raise RuntimeError(f"Native verification changed fixture state: {name}")
        results.append(dict(case=name, accepted=accepted, state_unchanged=True,
                            signature_independently_verified=True))
    result = dict(agent_sha256=digest, cases=results,
                  scope="Native cached-profile validation; synthetic company; no network or installation")
    (output / "result.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
