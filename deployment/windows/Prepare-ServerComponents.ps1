[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$TransportPackage,
    [Parameter(Mandatory)][string]$HttpsPackage,
    [Parameter(Mandatory)][string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$manifestPath = Join-Path $PSScriptRoot 'server-components.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schema -ne 1 -or $manifest.architecture -ne 'x86_64') { throw 'Unsupported server component manifest.' }
$destination = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $destination) { throw 'Component output directory already exists.' }
# Verify both archives before writing anything. No supplier executable is run.
$packages = @(
    @{ Path=(Resolve-Path -LiteralPath $TransportPackage).Path; Pin=$manifest.transport },
    @{ Path=(Resolve-Path -LiteralPath $HttpsPackage).Path; Pin=$manifest.https }
)
foreach ($package in $packages) {
    if ((Get-FileHash -LiteralPath $package.Path -Algorithm SHA256).Hash -ne $package.Pin.sha256) {
        throw 'Pinned server component archive hash mismatch.'
    }
}
$archives = [Collections.Generic.List[IDisposable]]::new()
$selected = @{}
try {
    foreach ($package in $packages) {
        $archive = [IO.Compression.ZipFile]::OpenRead($package.Path)
        $archives.Add($archive)
        foreach ($name in $package.Pin.files) {
            if ($name -notin @('hbbs.exe','hbbr.exe','caddy.exe')) { throw 'Unexpected component executable.' }
            $matches = @($archive.Entries | Where-Object { ($_.FullName -replace '\\','/' -split '/')[-1] -eq $name })
            if ($matches.Count -ne 1 -or $matches[0].Length -lt 2 -or $matches[0].Length -gt 128MB) {
                throw 'Missing, duplicate or oversized server component.'
            }
            $selected[$name] = $matches[0]
        }
    }
    New-Item -ItemType Directory -Path $destination | Out-Null
    foreach ($name in $selected.Keys) {
        # Use a fixed basename, never a supplier-controlled archive path.
        [IO.Compression.ZipFileExtensions]::ExtractToFile($selected[$name], (Join-Path $destination $name), $false)
    }
    Copy-Item -LiteralPath $manifestPath -Destination (Join-Path $destination 'server-components.json')
    Write-Output $destination
} finally {
    foreach ($archive in $archives) { $archive.Dispose() }
}
