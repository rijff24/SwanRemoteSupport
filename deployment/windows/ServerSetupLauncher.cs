using System;
using System.Diagnostics;
using System.IO;
using System.Management.Automation;
using System.Management.Automation.Runspaces;
using System.Reflection;
using System.Security.AccessControl;
using System.Security.Principal;
using System.Threading;

internal static class ServerSetupLauncher
{
    private static readonly string[] PayloadFiles = {
        "swan-management.exe", "components/hbbs.exe", "components/hbbr.exe", "components/caddy.exe",
        "components/Caddy-LICENSE.txt", "components/RustDesk-LICENSE.txt", "components/THIRD-PARTY.txt",
        "components/server-components.json", "SOURCE.json", "LICENCE"
    };

    private static Stream Resource(string name)
    {
        Stream resource = Assembly.GetExecutingAssembly().GetManifestResourceStream("Swan." + name);
        if (resource == null) throw new InvalidOperationException("Missing packaged resource: " + name);
        return resource;
    }

    private static string Script(string name)
    {
        using (Stream input = Resource(name))
        using (StreamReader reader = new StreamReader(input)) return reader.ReadToEnd();
    }

    private static string CreatePayloadDirectory()
    {
        WindowsIdentity identity = WindowsIdentity.GetCurrent();
        bool elevated = new WindowsPrincipal(identity).IsInRole(WindowsBuiltInRole.Administrator);
        SecurityIdentifier administrators = new SecurityIdentifier(WellKnownSidType.BuiltinAdministratorsSid, null);
        SecurityIdentifier system = new SecurityIdentifier(WellKnownSidType.LocalSystemSid, null);
        SecurityIdentifier owner = elevated ? administrators : identity.User;
        DirectorySecurity access = new DirectorySecurity();
        access.SetAccessRuleProtection(true, false);
        access.SetOwner(owner);
        InheritanceFlags inheritance = InheritanceFlags.ContainerInherit | InheritanceFlags.ObjectInherit;
        access.AddAccessRule(new FileSystemAccessRule(owner, FileSystemRights.FullControl, inheritance, PropagationFlags.None, AccessControlType.Allow));
        access.AddAccessRule(new FileSystemAccessRule(system, FileSystemRights.FullControl, inheritance, PropagationFlags.None, AccessControlType.Allow));
        string parent = elevated ? Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData) : Path.GetTempPath();
        string root = Path.Combine(parent, "SwanServerSetup-" + Guid.NewGuid().ToString("N"));
        if (Directory.Exists(root)) throw new IOException("Payload directory already exists.");
        Directory.CreateDirectory(root, access);
        DirectoryInfo created = new DirectoryInfo(root);
        if ((created.Attributes & FileAttributes.ReparsePoint) != 0 ||
            !created.GetAccessControl().GetOwner(typeof(SecurityIdentifier)).Equals(owner))
            throw new IOException("Payload directory ownership is invalid.");
        return root;
    }

    private static void Extract(string root)
    {
        foreach (string relative in PayloadFiles)
        {
            string target = Path.Combine(root, relative.Replace('/', Path.DirectorySeparatorChar));
            Directory.CreateDirectory(Path.GetDirectoryName(target));
            using (Stream input = Resource(relative))
            {
                if (input.Length > 256L * 1024 * 1024) throw new IOException("Packaged component too large.");
                using (FileStream output = new FileStream(target, FileMode.CreateNew, FileAccess.Write, FileShare.None)) input.CopyTo(output);
            }
            // Explicitly retain the protected root owner for payload children,
            // rather than depending on an elevated token's default owner.
            IdentityReference owner = new DirectoryInfo(root).GetAccessControl().GetOwner(typeof(SecurityIdentifier));
            FileSecurity fileAccess = File.GetAccessControl(target);
            fileAccess.SetOwner(owner);
            File.SetAccessControl(target, fileAccess);
            string directory = Path.GetDirectoryName(target);
            if (directory != root)
            {
                DirectorySecurity directoryAccess = Directory.GetAccessControl(directory);
                directoryAccess.SetOwner(owner);
                Directory.SetAccessControl(directory, directoryAccess);
            }
        }
    }

    private static void Cleanup(string root)
    {
        if (root == null) return;
        // Fixed basenames only. Retain unexpected files; never recurse.
        foreach (string relative in PayloadFiles)
        {
            string path = Path.Combine(root, relative.Replace('/', Path.DirectorySeparatorChar));
            if (File.Exists(path)) File.Delete(path);
        }
        string components = Path.Combine(root, "components");
        if (Directory.Exists(components) && Directory.GetFileSystemEntries(components).Length == 0) Directory.Delete(components);
        if (Directory.Exists(root) && Directory.GetFileSystemEntries(root).Length == 0) Directory.Delete(root);
    }

    [STAThread]
    private static int Main(string[] arguments)
    {
        string payload = null;
        try
        {
            bool preview = arguments.Length == 2 && arguments[0] == "--render-preview";
            if (arguments.Length != 0 && !preview) throw new ArgumentException("Use no arguments, or --render-preview OUTPUT.png.");
            payload = CreatePayloadDirectory();
            Extract(payload);
            using (Runspace runspace = RunspaceFactory.CreateRunspace())
            {
                runspace.ApartmentState = ApartmentState.STA;
                runspace.ThreadOptions = PSThreadOptions.UseCurrentThread;
                runspace.Open();
                // Installer code stays in the executable resource, not a
                // mutable temporary script executed by an elevated process.
                runspace.SessionStateProxy.SetVariable("SwanPackagedInstallerScript", Script("Install-Server.ps1"));
                using (PowerShell shell = PowerShell.Create())
                {
                    shell.Runspace = runspace;
                    shell.AddScript(Script("Setup-Server.ps1"))
                        .AddParameter("BundledDirectory", payload)
                        .AddParameter("UnsignedTestPackage", true);
                    if (preview) shell.AddParameter("RenderPreview", Path.GetFullPath(arguments[1]));
                    shell.Invoke();
                    if (shell.HadErrors) throw new InvalidOperationException("Server setup failed: " + shell.Streams.Error[0]);
                }
            }
            return 0;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine(error.Message);
            return 1;
        }
        finally
        {
            try { Cleanup(payload); }
            catch (Exception error) { Console.Error.WriteLine("Retained setup payload: " + error.Message); }
        }
    }
}
