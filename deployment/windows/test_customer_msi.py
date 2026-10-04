import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("customer_msi", Path(__file__).with_name("generate-customer-msi.py"))
msi = importlib.util.module_from_spec(spec)
spec.loader.exec_module(msi)


class CustomerMsiTests(unittest.TestCase):
    def fixture(self, root):
        source = root / "payload"
        source.mkdir()
        for name in ("rustdesk.exe", "librustdesk.dll", "flutter_windows.dll"):
            (source / name).write_bytes(name.encode())
        assets = source / "data" / "flutter_assets"
        assets.mkdir(parents=True)
        (assets / "test.txt").write_bytes(b"asset")
        agent = root / "agent.exe"
        agent.write_bytes(b"agent")
        return source, agent

    def test_full_manifest_license_and_service_lifecycle(self):
        with tempfile.TemporaryDirectory() as folder:
            source, agent = self.fixture(Path(folder))
            tree, files = msi.author(source, agent, "1.5.0")
            paths = {file["path"] for file in files}
            self.assertEqual(paths, {"Swan Remote Support.exe", "librustdesk.dll", "flutter_windows.dll", "data/flutter_assets/test.txt", "LICENSE.txt"})
            ns = {"w": msi.NAMESPACE}
            service = tree.find(".//w:ServiceInstall", ns)
            self.assertEqual(service.attrib["Name"], "Swan Remote Support")
            self.assertEqual(service.attrib["Arguments"], "--service")
            control = tree.find(".//w:ServiceControl", ns)
            self.assertEqual(control.attrib["Stop"], "both")
            self.assertEqual(control.attrib["Remove"], "uninstall")
            copy = tree.find(".//w:CopyFile", ns)
            self.assertEqual(copy.attrib["SourceDirectory"], "System64Folder")
            self.assertEqual(copy.attrib["DestinationName"], "RuntimeBroker_rustdesk.exe")
            registration = tree.find(".//w:Component[@Id='CustomerMsiRegistration']/w:RegistryValue", ns)
            self.assertEqual(registration.attrib["Root"], "HKLM")
            self.assertEqual(registration.attrib["Value"], "[ProductCode]")
            self.assertEqual(registration.attrib["Key"], "Software\\SwanRemoteSupport\\Customer")
            location = tree.find(".//w:SetProperty[@Id='ARPINSTALLLOCATION']", ns)
            self.assertEqual(location.attrib["Value"], "[INSTALLFOLDER]")
            cleanup = tree.find(".//w:CustomAction[@Id='RemoveCompanyConfigurationTask']", ns)
            self.assertEqual(cleanup.attrib["FileRef"], "ConfigurationAgentExe")
            self.assertEqual(cleanup.attrib["Execute"], "deferred")
            self.assertEqual(cleanup.attrib["Impersonate"], "no")
            scheduling = tree.find(".//w:InstallExecuteSequence/w:Custom[@Action='RemoveCompanyConfigurationTask']", ns)
            self.assertEqual(scheduling.attrib["Before"], "RemoveFiles")
            self.assertEqual(scheduling.attrib["Condition"], 'REMOVE = "ALL" AND NOT UPGRADINGPRODUCTCODE AND NOT SWAN_RECOVERY')
            rollback = tree.find(".//w:CustomAction[@Id='RestoreCompanyConfigurationTask']", ns)
            self.assertEqual(rollback.attrib["Execute"], "rollback")
            rollback_scheduling = tree.find(".//w:InstallExecuteSequence/w:Custom[@Action='RestoreCompanyConfigurationTask']", ns)
            self.assertEqual(rollback_scheduling.attrib["Before"], "RemoveCompanyConfigurationTask")
            installed_files = tree.findall(".//w:File", ns)
            self.assertEqual(len(installed_files), len(files)+1)

    def test_component_identity_survives_version_and_payload_changes(self):
        with tempfile.TemporaryDirectory() as folder:
            source, agent = self.fixture(Path(folder))
            first, _ = msi.author(source, agent, "1.5.0")
            (source / "rustdesk.exe").write_bytes(b"new release")
            second, _ = msi.author(source, agent, "1.6.0")
            tag = "{"+msi.NAMESPACE+"}Component"
            first_ids = {item.attrib["Id"]: item.attrib["Guid"] for item in first.iter(tag)}
            second_ids = {item.attrib["Id"]: item.attrib["Guid"] for item in second.iter(tag)}
            self.assertEqual(first_ids, second_ids)

    def test_unsafe_names_versions_and_incomplete_payloads_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            source, agent = self.fixture(Path(folder))
            with self.assertRaises(ValueError):
                msi.author(source, agent, "1.5.0;command")
            bad = source / "bad'.dll"
            bad.write_bytes(b"unsafe")
            with self.assertRaises(ValueError):
                msi.author(source, agent, "1.5.0")
            bad.unlink()
            (source / "librustdesk.dll").unlink()
            with self.assertRaises(ValueError):
                msi.author(source, agent, "1.5.0")


if __name__ == "__main__":
    unittest.main()
