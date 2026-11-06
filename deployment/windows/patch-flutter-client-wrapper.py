"""Patch only the reviewed SDK reply callable; never normalize shipping binaries."""
import argparse
import hashlib
import json
from pathlib import Path

ENGINE = "a18df97ca57a249df5d8d68cd0820600223ce262"
ORIGINAL_SHA256 = "48214b6261812361168f49d7a073a079b188709e8b29fdd1785307f9eef43989"
PATCHED_SHA256 = "4ecdd3b3ace7672043ffbcddd157324915f21854d2c7885cbdaf73fdac6731a0"
UPSTREAM = ("https://github.com/flutter/engine/blob/" + ENGINE
            + "/shell/platform/common/client_wrapper/core_implementations.cc")

# This type has external named linkage. An anonymous-namespace type would again
# include a physical-source-path identifier in MSVC RTTI.
NAMED_REPLY = r'''
namespace internal {
class SwanBinaryReplyHandler {
 public:
  SwanBinaryReplyHandler(
      const std::shared_ptr<FlutterDesktopMessenger>& messenger,
      const FlutterDesktopMessageResponseHandle* response_handle)
      : messenger_(messenger), response_handle_(response_handle) {}

  void operator()(const uint8_t* reply, size_t reply_size) {
    auto lock = std::unique_ptr<FlutterDesktopMessenger,
                               decltype(&FlutterDesktopMessengerUnlock)>(
        FlutterDesktopMessengerLock(messenger_.get()),
        &FlutterDesktopMessengerUnlock);
    if (!FlutterDesktopMessengerIsAvailable(messenger_.get())) {
      return;
    }
    if (!response_handle_) {
      std::cerr << "Error: Response can be set only once. Ignoring "
                   "duplicate response."
                << std::endl;
      return;
    }
    FlutterDesktopMessengerSendResponse(messenger_.get(), response_handle_,
                                        reply, reply_size);
    response_handle_ = nullptr;
  }

 private:
  std::shared_ptr<FlutterDesktopMessenger> messenger_;
  const FlutterDesktopMessageResponseHandle* response_handle_;
};
}  // namespace internal
'''

def transform(data):
    # Only source line endings may be canonicalized; this is not a PE rewrite.
    source = data.replace(b"\r\n", b"\n")
    if hashlib.sha256(source).hexdigest() != ORIGINAL_SHA256:
        raise ValueError("Flutter client-wrapper preimage differs from reviewed source")
    text = source.decode("utf-8")
    start = text.index("  BinaryReply reply_handler = [messenger_ptr, response_handle](")
    end = text.index("\n  };", start) + len("\n  };")
    replacement = ("  BinaryReply reply_handler =\n"
                   "      internal::SwanBinaryReplyHandler(messenger_ptr, response_handle);")
    text = text[:start] + replacement + text[end:]
    marker = "namespace flutter {\n"
    if text.count(marker) != 1:
        raise ValueError("Unexpected Flutter wrapper namespace")
    result = text.replace(marker, marker + NAMED_REPLY, 1).encode("utf-8")
    if hashlib.sha256(result).hexdigest() != PATCHED_SHA256:
        raise ValueError("Patched Flutter wrapper differs from reviewed output")
    return result


def patch(sdk, receipt):
    if receipt.exists():
        raise FileExistsError("Preserve the existing wrapper patch receipt")
    if (sdk / "bin/internal/engine.version").read_text().strip() != ENGINE:
        raise ValueError("Expected the pinned Flutter 3.24.5 SDK engine")
    root = sdk / "bin/cache/artifacts/engine"
    paths = sorted(root.rglob("core_implementations.cc"))
    if not 1 <= len(paths) <= 4:
        raise ValueError("Expected a bounded set of cached client-wrapper sources")
    changes = []
    for path in paths:
        for ancestor in (path, *path.parents):
            if ancestor == sdk.parent:
                break
            if ancestor.is_symlink() or getattr(ancestor, "is_junction", lambda: False)():
                raise ValueError("Unexpected wrapper source link")
        if "cpp_client_wrapper" not in path.parts:
            raise ValueError("Unexpected wrapper source location")
        original = path.read_bytes()
        patched = transform(original)
        changes.append((path, patched, hashlib.sha256(original).hexdigest()))
    # Validate every input before writing any SDK source.
    for path, patched, _ in changes:
        path.write_bytes(patched)
    result = {"engine_revision": ENGINE, "upstream": UPSTREAM,
              "original_canonical_sha256": ORIGINAL_SHA256,
              "patch_script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "files": [{"path": path.relative_to(sdk).as_posix(),
                         "original_sha256": original,
                         "patched_sha256": hashlib.sha256(patched).hexdigest()}
                        for path, patched, original in changes],
              "engine_binaries_changed": False, "shipping_pe_normalization": False,
              "full_native_reproduction_and_runtime_required": True}
    receipt.parent.mkdir(parents=True, exist_ok=True)
    with receipt.open("x", encoding="utf-8") as output:
        json.dump(result, output, indent=2)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sdk", required=True, type=Path)
    parser.add_argument("--receipt", required=True, type=Path)
    arguments = parser.parse_args()
    print(json.dumps(patch(arguments.sdk, arguments.receipt), indent=2))
