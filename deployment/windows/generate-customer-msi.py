"""Author a generic customer MSI from the complete pinned Flutter payload.

Company profiles are never executable build input. Sign the final MSI separately.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import uuid
import xml.etree.ElementTree as ET

NAMESPACE = "http://wixtoolset.org/schemas/v4/wxs"
UPGRADE = uuid.UUID("32a585d7-9a78-4ad2-af72-d0266efc709d")
ET.register_namespace("", NAMESPACE)


def element(parent, tag, **attributes):
    return ET.SubElement(parent, "{" + NAMESPACE + "}" + tag, attributes)


def identifier(kind, path):
    return kind + hashlib.sha256(path.encode()).hexdigest()[:30]


def author(source, agent, version):
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError("Numeric major.minor.patch version required")
    source = source.resolve(strict=True)
    agent = agent.resolve(strict=True)
    spec = importlib.util.spec_from_file_location("payload", Path(__file__).with_name("generate-payload-manifest.py"))
    payload = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(payload)
    files = payload.manifest("customer", source)
    root = ET.Element("{" + NAMESPACE + "}Wix")
    package = element(root, "Package", Name="Swan Remote Support", Manufacturer="Swan Remote Support", Version=version,
                      UpgradeCode=str(UPGRADE).upper(), Scope="perMachine", InstallerVersion="500", Language="1033")
    element(package, "MajorUpgrade", DowngradeErrorMessage="A newer Swan Remote Support version is already installed.", Schedule="afterInstallInitialize")
    element(package, "MediaTemplate", EmbedCab="yes")
    element(package, "Property", Id="ARPNOMODIFY", Value="1")
    element(package, "SetProperty", Id="ARPINSTALLLOCATION", Value="[INSTALLFOLDER]", After="CostFinalize", Sequence="execute")
    for prop, name, key in [("WINDOWSBUILD", "CurrentBuildNumber", "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
                            ("INSTALLATIONTYPE", "InstallationType", "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
                            ("OSARCH", "PROCESSOR_ARCHITECTURE", "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")]:
        property_element = element(package, "Property", Id=prop)
        element(property_element, "RegistrySearch", Id=prop+"Search", Root="HKLM", Key=key, Name=name, Type="raw", Bitness="always64")
    element(package, "Launch", Condition='Installed OR (VersionNT64 AND OSARCH = "AMD64" AND WINDOWSBUILD >= 10240 AND (INSTALLATIONTYPE = "Client" OR INSTALLATIONTYPE = "Server"))',
            Message="Windows 10/11 x64 or Windows Server with Desktop Experience is required.")
    programs = element(package, "StandardDirectory", Id="ProgramFiles64Folder")
    install = element(programs, "Directory", Id="INSTALLFOLDER", Name="Swan Remote Support")
    directories = {"": install}
    feature = element(package, "Feature", Id="Customer", Title="Customer support application", Level="1")
    for file in files:
        path = file["path"]
        parts = path.split("/")
        if any(not re.fullmatch(r"[A-Za-z0-9 _\-.@+]+", part) or part in (".", "..") or part.endswith((".", " ")) for part in parts):
            raise ValueError("Unsupported payload path")
        parent = ""
        for name in parts[:-1]:
            current = parent+"/"+name if parent else name
            if current not in directories:
                directories[current] = element(directories[parent], "Directory", Id=identifier("D", current), Name=name)
            parent = current
        component_id = identifier("C", path)
        component = element(directories[parent], "Component", Id=component_id, Guid=str(uuid.uuid5(UPGRADE, path)).upper(), Bitness="always64")
        original = source / ("rustdesk.exe" if path == "Swan Remote Support.exe" else path)
        file_element = element(component, "File", Id=identifier("F", path), Name=parts[-1], Source=str(original), KeyPath="yes", Checksum="yes")
        if path == "Swan Remote Support.exe":
            element(component, "ServiceInstall", Id="CustomerService", Name="Swan Remote Support", DisplayName="Swan Remote Support", Type="ownProcess", Start="auto", ErrorControl="normal", Arguments="--service", Account="LocalSystem", Vital="yes")
            element(component, "ServiceControl", Id="CustomerServiceControl", Name="Swan Remote Support", Start="install", Stop="both", Remove="uninstall", Wait="yes")
            element(component, "CopyFile", Id="WindowsRuntimeBroker", SourceDirectory="System64Folder", SourceName="RuntimeBroker.exe", DestinationDirectory="INSTALLFOLDER", DestinationName="RuntimeBroker_rustdesk.exe")
            element(component, "RemoveFile", Id="RemoveWindowsRuntimeBroker", Name="RuntimeBroker_rustdesk.exe", On="uninstall")
            element(file_element, "Shortcut", Id="CustomerShortcut", Directory="ProgramMenuFolder", Name="Swan Remote Support", Advertise="yes", WorkingDirectory="INSTALLFOLDER")
        element(feature, "ComponentRef", Id=component_id)
    license_path = Path(__file__).resolve().parents[2] / "LICENCE"
    if not any(file["path"].lower() == "license.txt" for file in files):
        component_id = identifier("C", "LICENSE.txt")
        component = element(install, "Component", Id=component_id, Guid=str(uuid.uuid5(UPGRADE, "LICENSE.txt")).upper(), Bitness="always64")
        element(component, "File", Id=identifier("F", "LICENSE.txt"), Name="LICENSE.txt", Source=str(license_path), KeyPath="yes")
        element(feature, "ComponentRef", Id=component_id)
        files.append({"path": "LICENSE.txt", "sha256": hashlib.sha256(license_path.read_bytes()).hexdigest()})
    registration = element(install, "Component", Id="CustomerMsiRegistration", Guid=str(uuid.uuid5(UPGRADE, "msi-registration")).upper(), Bitness="always64")
    element(registration, "RegistryValue", Root="HKLM", Key="Software\\SwanRemoteSupport\\Customer", Name="ProductCode", Type="string", Value="[ProductCode]", KeyPath="yes")
    element(feature, "ComponentRef", Id="CustomerMsiRegistration")
    data = element(package, "StandardDirectory", Id="CommonAppDataFolder")
    state = element(data, "Directory", Id="STATEFOLDER", Name="SwanRemoteSupport")
    component = element(state, "Component", Id="ConfigurationAgent", Guid=str(uuid.uuid5(UPGRADE, "configuration-agent")).upper(), Bitness="always64")
    element(component, "File", Id="ConfigurationAgentExe", Name="swan-agent.exe", Source=str(agent), KeyPath="yes")
    element(feature, "ComponentRef", Id="ConfigurationAgent")
    element(package, "CustomAction", Id="RemoveCompanyConfigurationTask", FileRef="ConfigurationAgentExe",
            ExeCommand="remove-configuration-task", Execute="deferred", Impersonate="no", Return="check")
    element(package, "CustomAction", Id="RestoreCompanyConfigurationTask", FileRef="ConfigurationAgentExe",
            ExeCommand="restore-configuration-task", Execute="rollback", Impersonate="no", Return="check")
    sequence = element(package, "InstallExecuteSequence")
    element(sequence, "Custom", Action="RestoreCompanyConfigurationTask", Before="RemoveCompanyConfigurationTask",
            Condition='REMOVE = "ALL" AND NOT UPGRADINGPRODUCTCODE AND NOT SWAN_RECOVERY')
    element(sequence, "Custom", Action="RemoveCompanyConfigurationTask", Before="RemoveFiles",
            Condition='REMOVE = "ALL" AND NOT UPGRADINGPRODUCTCODE AND NOT SWAN_RECOVERY')
    return ET.ElementTree(root), files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--agent", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--manifest-output", type=Path)
    args = parser.parse_args()
    tree, files = author(args.source, args.agent, args.version)
    ET.indent(tree, space="  ")
    with args.output.open("xb") as stream:
        tree.write(stream, encoding="utf-8", xml_declaration=True)
    if args.manifest_output:
        with args.manifest_output.open("x", encoding="utf-8") as stream:
            json.dump(files, stream, indent=2)
            stream.write("\n")


if __name__ == "__main__":
    main()
