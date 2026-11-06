"""Verifier boundary tests with inert fixtures; no executable is run."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import warnings
import zipfile

spec = importlib.util.spec_from_file_location("windows_components", Path(__file__).with_name("prepare_windows_components.py"))
components = importlib.util.module_from_spec(spec)
spec.loader.exec_module(components)


def zipped(files, date=(2024, 1, 1, 0, 0, 0)):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w") as archive:
        for name, value in files.items():
            archive.writestr(zipfile.ZipInfo(name, date_time=date), value)
    return stream.getvalue()


class WindowsComponentsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="swan-maintained-package-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        for directory in ("deployment/windows", "deployment/transport"):
            (self.root / directory).mkdir(parents=True)
        self.pins = {"patches": [{"after_sha256": "c" * 64}], "scope": "inert verifier fixture"}
        self.receipt = {"upstream": self.pins, "stage": "fix", "project_revision": "a" * 40,
                        "patched_udp_sha256": "c" * 64, "unsigned_test": True}
        self.source = {"rustdesk-server/Cargo.toml": b"inert source fixture\n",
                       "rustdesk-server/LICENSE": b"inert license fixture\n",
                       "swan-recipe/recipe.py": b"# inert recipe fixture\n",
                       components.SOURCE_RECEIPT: json.dumps(self.receipt).encode()}
        self.inputs = {"transport-source/target/release/hbbs.exe": b"inert rendezvous fixture",
                       "transport-source/target/release/hbbr.exe": b"inert relay fixture",
                       "transport-source/SWAN-SOURCE.json": self.source[components.SOURCE_RECEIPT],
                       "transport-artifacts/transport-build-source.zip": zipped(self.source)}
        https = zipped({"caddy.exe": b"inert HTTPS fixture", "LICENSE": b"inert HTTPS license"})
        self.https = self.root / "https.zip"
        self.https.write_bytes(https)
        self.manifest = {"schema": 2, "architecture": "x86_64",
                         "transport": {"kind": "swan-maintained-source-build", "source": "https://example.test/source",
                                       "binary_sha256": {n: hashlib.sha256(self.inputs["transport-source/target/release/" + n]).hexdigest() for n in ("hbbs.exe", "hbbr.exe")},
                                       "source_files": len(self.source), "source_tree_sha256": components.source_tree_digest(self.source)},
                         "https": {"source": "https://example.test/https", "sha256": hashlib.sha256(https).hexdigest(),
                                   "binary_sha256": hashlib.sha256(b"inert HTTPS fixture").hexdigest()}}
        (self.root / "deployment/windows/server-components.json").write_text(json.dumps(self.manifest), encoding="utf-8")
        (self.root / "deployment/transport/source.json").write_text(json.dumps(self.pins), encoding="utf-8")
        self.output = self.root / "prepared"

    def prepare(self, inputs=None, destination=None):
        return components.prepare(self.inputs if inputs is None else inputs, self.https,
                                  self.output if destination is None else destination, self.root)

    def reject(self, inputs):
        with self.assertRaises(ValueError):
            self.prepare(inputs)
        self.assertFalse(self.output.exists(), "Rejected inputs must not create output")

    def test_verified_bytes_source_and_licenses_retained_without_execution(self):
        result = self.prepare()
        self.assertFalse(result["supplier_executables_run"])
        self.assertEqual(len(result["sha256"]), 8)
        for name in ("hbbs.exe", "hbbr.exe"):
            self.assertEqual((self.output / name).read_bytes(), self.inputs["transport-source/target/release/" + name])
        with zipfile.ZipFile(self.output / "transport-build-source.zip") as source:
            self.assertEqual(set(source.namelist()), set(self.source))
            for entry in source.infolist():
                self.assertEqual(entry.date_time, (1980, 1, 1, 0, 0, 0))
        self.assertIn("AGPL-3.0", (self.output / "THIRD-PARTY.txt").read_text())

    def test_source_zip_timestamps_and_checkout_endings_do_not_change_shipping_inputs(self):
        first = self.prepare()
        source = {name: value.replace(b"\n", b"\r\n") for name, value in self.source.items()}
        inputs = dict(self.inputs)
        inputs["transport-artifacts/transport-build-source.zip"] = zipped(source, (2025, 2, 2, 2, 2, 2))
        second = self.prepare(inputs, self.root / "second")
        self.assertEqual(first, second)

    def test_changed_binary_rejected_before_any_output(self):
        inputs = dict(self.inputs)
        inputs["transport-source/target/release/hbbs.exe"] += b"changed"
        self.reject(inputs)

    def test_changed_source_and_extra_file_rejected(self):
        for change in ("changed", "extra"):
            source = dict(self.source)
            source["rustdesk-server/Cargo.toml" if change == "changed" else "unexpected.py"] = b"unreviewed input"
            inputs = dict(self.inputs)
            inputs["transport-artifacts/transport-build-source.zip"] = zipped(source)
            self.reject(inputs)

    def test_forged_receipt_cannot_replace_verified_source(self):
        for field, value in [("stage", "tests"), ("project_revision", "not-a-revision"),
                             ("unsigned_test", False), ("patched_udp_sha256", "d" * 64),
                             ("upstream", {"different": True}), ("unexpected", "unreviewed")]:
            receipt = copy.deepcopy(self.receipt)
            receipt[field] = value
            source = dict(self.source)
            source[components.SOURCE_RECEIPT] = json.dumps(receipt).encode()
            inputs = dict(self.inputs)
            inputs["transport-source/SWAN-SOURCE.json"] = source[components.SOURCE_RECEIPT]
            inputs["transport-artifacts/transport-build-source.zip"] = zipped(source)
            self.reject(inputs)

    def test_outer_and_inner_receipt_must_agree(self):
        inputs = dict(self.inputs)
        inputs["transport-source/SWAN-SOURCE.json"] += b" "
        self.reject(inputs)

    def test_wrong_https_archive_rejected(self):
        self.https.write_bytes(b"wrong archive")
        self.reject(self.inputs)

    def test_unsafe_and_duplicate_source_paths_rejected(self):
        for name in ("../outside", "/absolute", "back\\slash", "drive:name"):
            source = dict(self.source)
            source[name] = b"unreviewed"
            inputs = dict(self.inputs)
            inputs["transport-artifacts/transport-build-source.zip"] = zipped(source)
            self.reject(inputs)
        stream = io.BytesIO()
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            with zipfile.ZipFile(stream, "w") as archive:
                for _ in range(2):
                    archive.writestr("duplicate", b"fixture")
        inputs = dict(self.inputs)
        inputs["transport-artifacts/transport-build-source.zip"] = stream.getvalue()
        self.reject(inputs)

    def test_existing_output_is_never_overwritten(self):
        self.output.mkdir()
        marker = self.output / "owned-marker"
        marker.write_bytes(b"preserve")
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertEqual(marker.read_bytes(), b"preserve")

    def test_artifact_zip_and_extracted_directory_have_same_strict_input_set(self):
        package = self.root / "transport.zip"
        package.write_bytes(zipped(self.inputs))
        self.assertEqual(components.transport_inputs(package=package), self.inputs)
        directory = self.root / "artifact"
        directory.mkdir()
        for name, value in self.inputs.items():
            path = directory / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(value)
        self.assertEqual(components.transport_inputs(directory=directory), self.inputs)
        (directory / "unexpected").write_bytes(b"unreviewed")
        with self.assertRaises(ValueError):
            components.transport_inputs(directory=directory)


if __name__ == "__main__":
    unittest.main()
