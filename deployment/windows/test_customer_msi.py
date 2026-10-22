import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import uuid

spec = importlib.util.spec_from_file_location("customer_msi", Path(__file__).with_name("generate-customer-msi.py"))
msi = importlib.util.module_from_spec(spec)
spec.loader.exec_module(msi)


class CustomerMsiTests(unittest.TestCase):
    def test_product_codes_follow_standard_uuid5_across_both_builders(self):
        shell = shutil.which("pwsh") or shutil.which("powershell")
        if not shell:
            self.skipTest("PowerShell required for cross-builder identity validation")
        script = Path(__file__).with_name("Get-MsiProductCode.ps1")
        namespaces = {"customer": msi.UPGRADE, "technician": uuid.UUID("a4374699-436f-4917-9a4e-223f62a9e634")}
        observed = set()
        for edition, namespace in namespaces.items():
            for version in ("1.5.0", "1.5.1", "0.0.0", "255.255.65535"):
                actual = subprocess.check_output([shell, "-NoProfile", "-NonInteractive", "-File", str(script),
                                                  "-Edition", edition, "-Version", version], text=True).strip()
                expected = str(uuid.uuid5(namespace, "windows-x64|" + version)).upper()
                self.assertEqual(actual, expected)
                observed.add(actual)
        self.assertEqual(len(observed), 8)

    def test_version_aliases_and_msi_overflow_are_rejected_by_both_builders(self):
        shell = shutil.which("pwsh") or shutil.which("powershell")
        if not shell:
            self.skipTest("PowerShell required for cross-builder version validation")
        script = Path(__file__).with_name("Get-MsiProductCode.ps1")
        with tempfile.TemporaryDirectory() as folder:
            source, agent = self.fixture(Path(folder))
            for version in ("01.5.0", "1.05.0", "1.5.00", "256.0.0", "1.256.0", "1.0.65536",
                            "1.5.0.1", "1.5.0\n", "١.5.0", "999999999999999999999.0.0"):
                with self.subTest(version=repr(version)):
                    with self.assertRaises(ValueError):
                        msi.author(source, agent, version)
                    for edition in ("customer", "technician"):
                        result = subprocess.run([shell, "-NoProfile", "-NonInteractive", "-File", str(script),
                                                 "-Edition", edition, "-Version", version], capture_output=True)
                        self.assertNotEqual(result.returncode, 0)

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
            self.assertEqual(tree.find("w:Package", ns).attrib["ProductCode"],
                             str(uuid.uuid5(msi.UPGRADE, "windows-x64|1.5.0")).upper())
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
            self.assertEqual(cleanup.attrib["BinaryRef"], "UninstallCommitAgent")
            self.assertEqual(cleanup.attrib["ExeCommand"], 'remove-configuration-task "[STATEFOLDER]."')
            self.assertEqual(cleanup.attrib["Execute"], "deferred")
            self.assertEqual(cleanup.attrib["Impersonate"], "no")
            scheduling = tree.find(".//w:InstallExecuteSequence/w:Custom[@Action='RemoveCompanyConfigurationTask']", ns)
            self.assertEqual(scheduling.attrib["Before"], "RemoveFiles")
            self.assertEqual(scheduling.attrib["Condition"], 'REMOVE = "ALL" AND NOT UPGRADINGPRODUCTCODE AND NOT SWAN_RECOVERY')
            rollback = tree.find(".//w:CustomAction[@Id='RestoreCompanyConfigurationTask']", ns)
            self.assertEqual(rollback.attrib["Execute"], "rollback")
            self.assertEqual(rollback.attrib["BinaryRef"], "UninstallCommitAgent")
            self.assertEqual(rollback.attrib["ExeCommand"], 'restore-configuration-task "[STATEFOLDER]."')
            rollback_scheduling = tree.find(".//w:InstallExecuteSequence/w:Custom[@Action='RestoreCompanyConfigurationTask']", ns)
            self.assertEqual(rollback_scheduling.attrib["Before"], "RemoveCompanyConfigurationTask")
            embedded = tree.find(".//w:Binary[@Id='UninstallCommitAgent']", ns)
            self.assertEqual(embedded.attrib["SourceFile"], str(agent.resolve()))
            commit = tree.find(".//w:CustomAction[@Id='CommitCompanyUninstall']", ns)
            self.assertEqual(commit.attrib["BinaryRef"], "UninstallCommitAgent")
            self.assertEqual(commit.attrib["Execute"], "commit")
            self.assertEqual(commit.attrib["Impersonate"], "no")
            self.assertEqual(commit.attrib["Return"], "check")
            self.assertEqual(commit.attrib["ExeCommand"], 'commit-uninstall customer "[STATEFOLDER]."')
            commit_scheduling = tree.find(".//w:InstallExecuteSequence/w:Custom[@Action='CommitCompanyUninstall']", ns)
            self.assertEqual(commit_scheduling.attrib["After"], "RemoveCompanyConfigurationTask")
            self.assertEqual(commit_scheduling.attrib["Condition"], scheduling.attrib["Condition"])
            preparation = tree.find(".//w:CustomAction[@Id='PrepareCustomerState']", ns)
            self.assertEqual(preparation.attrib["BinaryRef"], "UninstallCommitAgent")
            self.assertEqual(preparation.attrib["ExeCommand"], 'prepare-customer-directory "[STATEFOLDER]."')
            self.assertEqual(preparation.attrib["Execute"], "deferred")
            self.assertEqual(preparation.attrib["Impersonate"], "no")
            preparation_sequence = tree.find(".//w:InstallExecuteSequence/w:Custom[@Action='PrepareCustomerState']", ns)
            self.assertEqual(preparation_sequence.attrib["Before"], "CreateFolders")
            self.assertEqual(preparation_sequence.attrib["Condition"], 'NOT (REMOVE = "ALL")')
            installed_files = tree.findall(".//w:File", ns)
            self.assertEqual(len(installed_files), len(files)+1)

    def test_technician_uninstall_commit_retains_user_context(self):
        import xml.etree.ElementTree as ET
        tree = ET.parse(Path(__file__).with_name("msi") / "Technician.wxs")
        ns = {"w": msi.NAMESPACE}
        action = tree.find(".//w:CustomAction[@Id='CommitTechnicianUninstall']", ns)
        self.assertEqual(action.attrib["BinaryRef"], "UninstallCommitAgent")
        self.assertEqual(action.attrib["ExeCommand"], 'commit-uninstall technician "[INSTALLFOLDER]."')
        self.assertEqual(action.attrib["Execute"], "commit")
        self.assertEqual(action.attrib["Impersonate"], "yes")
        self.assertEqual(action.attrib["Return"], "check")
        sequence = tree.find(".//w:InstallExecuteSequence/w:Custom[@Action='CommitTechnicianUninstall']", ns)
        self.assertEqual(sequence.attrib["After"], "CancelUpdateRecovery")
        self.assertEqual(sequence.attrib["Condition"], 'REMOVE = "ALL" AND NOT UPGRADINGPRODUCTCODE AND NOT SWAN_RECOVERY')
        for name, command in (("CancelUpdateRecovery", "cancel-update-recovery"), ("RestoreUpdateRecovery", "restore-update-recovery")):
            action = tree.find(".//w:CustomAction[@Id='" + name + "']", ns)
            self.assertEqual(action.attrib["BinaryRef"], "UninstallCommitAgent")
            self.assertEqual(action.attrib["ExeCommand"], command + ' "[INSTALLFOLDER]."')
            self.assertEqual(action.attrib["Impersonate"], "yes")

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
