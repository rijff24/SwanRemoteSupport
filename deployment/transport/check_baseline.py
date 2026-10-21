"""Verify the baseline failure or the fixed real UDP paths, retaining logs."""
import json
import os
import subprocess
import sys
from pathlib import Path

source = Path(sys.argv[1]).resolve(strict=True)
stage = sys.argv[2] if len(sys.argv) == 3 else "baseline"
if stage not in ("baseline", "fixed"):
    raise SystemExit("Select baseline or fixed verification")
result = subprocess.run(["cargo", "+1.90.0", "test", "--release", "--locked", "-p", "hbb_common",
                         "swan_udp_tests", "--", "--nocapture"], cwd=source,
                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
log = result.stdout.decode(errors="replace")
evidence = source / "swan-evidence"
evidence.mkdir(exist_ok=stage == "fixed")
(evidence / (stage + ".log")).write_bytes(result.stdout)
windows = os.name == "nt"
expected_failure = (result.returncode != 0 and "os error 10014" in log
                    and "dual_stack_sends_ipv4_message_and_raw_datagrams ... FAILED" in log
                    and "ipv4_sender_keeps_ipv4_destination ... ok" in log
                    and "ipv6_sender_keeps_native_ipv6_destination ... ok" in log)
all_passed = result.returncode == 0 and all(name + " ... ok" in log for name in (
    "dual_stack_sends_ipv4_message_and_raw_datagrams",
    "ipv4_sender_keeps_ipv4_destination", "ipv6_sender_keeps_native_ipv6_destination"))
passed = expected_failure if windows and stage == "baseline" else all_passed
receipt = {"stage": stage, "windows": windows, "exit_code": result.returncode,
           "expected_winsock_failure_reproduced": expected_failure,
           "baseline_check_passed": passed}
(evidence / (stage + ".json")).write_text(json.dumps(receipt, indent=2) + "\n")
print("\n".join(log.splitlines()[-60:]))
print(json.dumps(receipt))
if not passed:
    raise SystemExit("Baseline did not demonstrate the required platform behavior; no fix accepted")
