"""Exercise native cached-profile trust and time validation without network or installed state.

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
from cryptography.exceptions import InvalidSignature


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
    time_error = "Expired or future profile"
    cases = [("valid", now - 60, now + 3600, None),
             ("expired", now - 3600, now - 60, time_error),
             ("future", now + 3600, now + 7200, time_error),
             ("invalid_lifetime", now - 60, now - 120, time_error)]
    cases += [(name, now - 60, now + 3600, error) for name, error in [
        ("tampered_branding", "Invalid signature"),
        ("wrong_signing_key", "Invalid signature"),
        ("wrong_company", "Wrong company or schema"),
        ("replayed_revision", "Replayed profile"),
        ("bootstrap_company", "Installer belongs to another company or edition"),
        ("bootstrap_edition", "Installer belongs to another company or edition"),
        ("bootstrap_trust_key", "Installer cannot replace configured trust keys"),
        ("bootstrap_endpoint", "Installer endpoint differs from trusted company configuration"),
    ]]
    results = []
    for name, issued, expires, expected_error in cases:
        accepted = expected_error is None
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
        if name == "wrong_company":
            profile["company_id"] = "another-company"
        payload = json.dumps(profile, separators=(",", ":")).encode()
        signer = Ed25519PrivateKey.generate() if name == "wrong_signing_key" else key
        signature = signer.sign(payload)
        signer.public_key().verify(signature, payload)
        if name == "tampered_branding":
            profile["technician"] = dict(branding, display_name="Tampered company")
            payload = json.dumps(profile, separators=(",", ":")).encode()
        signature_matches_pin = True
        try:
            key.public_key().verify(signature, payload)
        except InvalidSignature:
            signature_matches_pin = False
        if signature_matches_pin != (name not in ("tampered_branding", "wrong_signing_key")):
            raise RuntimeError(f"Unexpected fixture signature: {name}")
        state = dict(bootstrap=bootstrap, profile=dict(
            payload=base64.b64encode(payload).decode(),
            signature=base64.b64encode(signature).decode()),
            accepted_revision=2 if name == "replayed_revision" else 1,
            device_id=None, device_token=None, unattended_consent=False,
            consent_revision=0, last_release_sequence=0)
        state_path = directory / "managed-state.json"
        state_path.write_text(json.dumps(state), encoding="utf-8")
        bootstrap_path = directory / "bootstrap.json"
        installer_bootstrap = dict(bootstrap)
        if name == "bootstrap_company":
            installer_bootstrap["company_id"] = "another-company"
        elif name == "bootstrap_edition":
            installer_bootstrap["edition"] = "customer"
        elif name == "bootstrap_trust_key":
            installer_bootstrap["profile_public_key"] = base64.b64encode(
                Ed25519PrivateKey.generate().public_key().public_bytes(
                    serialization.Encoding.Raw, serialization.PublicFormat.Raw)).decode()
        elif name == "bootstrap_endpoint":
            installer_bootstrap["management_url"] = "https://another-company.example.invalid"
        bootstrap_path.write_text(json.dumps(installer_bootstrap), encoding="utf-8")
        before = state_path.read_bytes()
        bootstrap_before = bootstrap_path.read_bytes()
        env = os.environ.copy()
        env["SWAN_STATE_DIR"] = str(directory)
        completed = subprocess.run([str(agent), "verify-bootstrap", str(bootstrap_path)],
                                   env=env, capture_output=True, text=True, timeout=30)
        if (completed.returncode == 0) != accepted:
            raise RuntimeError(f"Unexpected native validation outcome: {name}")
        if not accepted and expected_error not in completed.stderr:
            raise RuntimeError(f"Rejection did not exercise expected trust validation: {name}")
        if state_path.read_bytes() != before or bootstrap_path.read_bytes() != bootstrap_before:
            raise RuntimeError(f"Native verification changed fixture state: {name}")
        results.append(dict(case=name, accepted=accepted, state_unchanged=True,
                            signature_matches_pinned_key=signature_matches_pin,
                            expected_rejection_verified=not accepted))
    result = dict(agent_sha256=digest, cases=results,
                  scope="Native cached-profile validation; synthetic company; no network or installation")
    (output / "result.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
