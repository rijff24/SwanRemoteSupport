"""Generate the compiler fixture from the actual SDK patch callable, not a copy."""
import argparse
import importlib.util
from pathlib import Path

root = Path(__file__).parent
spec = importlib.util.spec_from_file_location("reply_patch", root / "patch-flutter-client-wrapper.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", required=True, type=Path)
args = parser.parse_args()
template = (root / "fixtures/flutter_reply_handler.cpp.in").read_text(encoding="utf-8")
if template.count("@NAMED_REPLY@") != 1 or args.output.exists():
    raise ValueError("Unexpected template or existing fixture source")
args.output.write_text(template.replace("@NAMED_REPLY@", module.NAMED_REPLY), encoding="utf-8")
