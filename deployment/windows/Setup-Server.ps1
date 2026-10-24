[CmdletBinding()]
param([string]$RenderPreview,[string]$BundledDirectory,[switch]$UnsignedTestPackage)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[Windows.Forms.Application]::EnableVisualStyles()
if ([Threading.Thread]::CurrentThread.ApartmentState -ne 'STA') { throw 'Run this wizard using Windows PowerShell with -STA.' }
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$administrator = ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$script:operation = $null
$script:setupUrl = $null
$script:installerText = $null
if (Get-Variable -Name SwanPackagedInstallerScript -ErrorAction SilentlyContinue) {
    $script:installerText = $SwanPackagedInstallerScript
} else { $script:installerText = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'Install-Server.ps1') -Raw }
$form = [Windows.Forms.Form]::new()
$form.Text = 'Swan Remote Support - Company server setup'
if ($UnsignedTestPackage) {$form.Text += ' [UNSIGNED TEST]'}
$form.ClientSize = [Drawing.Size]::new(820,740)
$form.MinimumSize = $form.Size
$form.MaximumSize = $form.Size
$form.StartPosition = 'CenterScreen'
$form.Font = [Drawing.Font]::new('Segoe UI',10)
$form.BackColor = [Drawing.Color]::White
$form.MaximizeBox = $false
$title = [Windows.Forms.Label]::new()
$title.Text = 'Swan Remote Support'
$title.Font = [Drawing.Font]::new('Segoe UI',22,[Drawing.FontStyle]::Bold)
$title.ForeColor = [Drawing.Color]::FromArgb(0,125,135)
$title.SetBounds(24,18,760,45)
$form.Controls.Add($title)
$intro = [Windows.Forms.Label]::new()
$intro.Text = 'Set up your company server, then configure your company in the HTTPS setup page.'
$intro.SetBounds(26,70,760,38)
$form.Controls.Add($intro)
function Add-Field([string]$Label,[int]$Y,[string]$Help,[bool]$Browse=$false) {
    $caption = [Windows.Forms.Label]::new()
    $caption.Text = $Label
    $caption.SetBounds(26,$Y,760,22)
    $form.Controls.Add($caption)
    $input = [Windows.Forms.TextBox]::new()
    $width = 768
    if ($Browse) {$width=674}
    $input.SetBounds(26,($Y+24),$width,28)
    $form.Controls.Add($input)
    $hint = [Windows.Forms.Label]::new()
    $hint.Text = $Help
    $hint.ForeColor = [Drawing.Color]::DimGray
    $hint.Font = [Drawing.Font]::new('Segoe UI',9)
    $hint.SetBounds(26,($Y+54),768,24)
    $form.Controls.Add($hint)
    return $input
}
$hostInput = Add-Field 'Company public hostname' 116 'For example, support.example.com. Configure DNS and public port forwarding for this server.'
$keyInput = Add-Field 'Trusted project release public key' 195 'Use the base64 release key supplied through your trusted project or company release policy.'
$publisherInput = Add-Field 'Trusted management publisher certificate thumbprint' 274 'Use the certificate fingerprint from your trusted release policy, not from an unverified download.'
$executableInput = Add-Field 'Signed management executable' 353 'Select the signed swan-management.exe from your reviewed release.' $true
$componentsInput = Add-Field 'Prepared server components folder' 432 'Contains pinned rendezvous, relay, HTTPS executables and their license/source notices.' $true
if ($BundledDirectory) {
    $executableInput.Text = Join-Path $BundledDirectory 'swan-management.exe'
    $componentsInput.Text = Join-Path $BundledDirectory 'components'
}
$fileButton = [Windows.Forms.Button]::new()
$fileButton.Text = 'Browse...'
$fileButton.SetBounds(714,377,80,28)
$fileButton.Add_Click({
    $dialog = [Windows.Forms.OpenFileDialog]::new()
    $dialog.Filter = 'Management executable (*.exe)|*.exe'
    try {if ($dialog.ShowDialog($form) -eq 'OK') {$executableInput.Text=$dialog.FileName}} finally {$dialog.Dispose()}
})
$form.Controls.Add($fileButton)
$folderButton = [Windows.Forms.Button]::new()
$folderButton.Text = 'Browse...'
$folderButton.SetBounds(714,456,80,28)
$folderButton.Add_Click({
    $dialog = [Windows.Forms.FolderBrowserDialog]::new()
    $dialog.Description = 'Select the prepared server components folder'
    try {if ($dialog.ShowDialog($form) -eq 'OK') {$componentsInput.Text=$dialog.SelectedPath}} finally {$dialog.Dispose()}
})
$form.Controls.Add($folderButton)
$status = [Windows.Forms.Label]::new()
$status.Text = 'Company branding, administrator MFA, device policy and update settings follow in web setup.'
if (-not $administrator) {$status.Text='Open this wizard as administrator to install. You can review setup here without installing.'}
if ($RenderPreview) {$status.Text='Preview only. Installation and browser actions are disabled.'}
$status.SetBounds(26,520,768,42)
$form.Controls.Add($status)
$log = [Windows.Forms.TextBox]::new()
$log.Multiline = $true
$log.ReadOnly = $true
$log.ScrollBars = 'Vertical'
$log.SetBounds(26,568,768,80)
$form.Controls.Add($log)
$progress = [Windows.Forms.ProgressBar]::new()
$progress.SetBounds(26,661,768,8)
$form.Controls.Add($progress)
$installButton = [Windows.Forms.Button]::new()
$installButton.Text = 'Install company server'
$installButton.SetBounds(26,687,205,32)
$installButton.BackColor = [Drawing.Color]::FromArgb(0,125,135)
$installButton.ForeColor = [Drawing.Color]::White
$installButton.FlatStyle = 'Flat'
$installButton.Enabled = $administrator -and -not $RenderPreview
$form.Controls.Add($installButton)
$webButton = [Windows.Forms.Button]::new()
$webButton.Text = 'Open company setup'
$webButton.SetBounds(244,687,205,32)
$webButton.Enabled = $false
$webButton.Add_Click({
    if ($script:setupUrl -and -not $RenderPreview) {
        $start = [Diagnostics.ProcessStartInfo]::new($script:setupUrl)
        $start.UseShellExecute = $true
        [Diagnostics.Process]::Start($start) | Out-Null
    }
})
$form.Controls.Add($webButton)
$closeButton = [Windows.Forms.Button]::new()
$closeButton.Text = 'Close'
$closeButton.SetBounds(689,687,105,32)
$closeButton.Add_Click({$form.Close()})
$form.Controls.Add($closeButton)
$timer = [Windows.Forms.Timer]::new()
$timer.Interval = 250
$installButton.Add_Click({
    try {
        $hostname = $hostInput.Text.Trim()
        if ($hostname.Length -gt 253 -or $hostname -notmatch '\.' -or @($hostname.Split('.') | Where-Object {$_ -notmatch '^[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?$'}).Count) {throw 'Enter a valid public company hostname.'}
        if ([Convert]::FromBase64String($keyInput.Text.Trim()).Length -ne 32) {throw 'The release public key must decode to 32 bytes.'}
        if ($publisherInput.Text.Trim() -notmatch '^[A-Fa-f0-9]{40}$') {throw 'Enter the trusted certificate thumbprint from your release policy.'}
        if (-not (Test-Path -LiteralPath $executableInput.Text -PathType Leaf)) {throw 'Select the signed management executable.'}
        if (-not (Test-Path -LiteralPath $componentsInput.Text -PathType Container)) {throw 'Select the prepared server components folder.'}
        $parameters = @{Executable=$executableInput.Text;ReleasePublicKey=$keyInput.Text.Trim();PublisherThumbprint=$publisherInput.Text.Trim();ComponentsDirectory=$componentsInput.Text;PublicHostname=$hostname}
        $worker = [Management.Automation.PowerShell]::Create()
        $worker.AddScript($script:installerText).AddParameters($parameters) | Out-Null
        $handle = $worker.BeginInvoke()
        $script:operation = @{Worker=$worker;Handle=$handle;Hostname=$hostname}
        foreach ($control in @($hostInput,$keyInput,$publisherInput,$executableInput,$componentsInput,$fileButton,$folderButton,$installButton,$closeButton)) {$control.Enabled=$false}
        $log.Text = 'Installing company components. Public certificate issuance depends on DNS and reachability.'
        $status.Text = 'Installation is running. Keep this wizard open until it finishes.'
        $progress.Style = 'Marquee'
        $timer.Start()
    } catch {$log.Text=$_.Exception.Message}
})
$timer.Add_Tick({
    if (-not $script:operation -or -not $script:operation.Handle.IsCompleted) {return}
    $timer.Stop()
    $operation = $script:operation
    try {
        $operation.Worker.EndInvoke($operation.Handle) | Out-Null
        if ($operation.Worker.HadErrors) {throw ($operation.Worker.Streams.Error | Out-String)}
        $log.Text = ($operation.Worker.Streams.Information | Out-String).Trim()
        $script:setupUrl = 'https://'+$operation.Hostname
        $webButton.Enabled = $true
        $status.Text = 'Server installed. Complete company setup over HTTPS. Retrieve the one-time token from protected server data.'
    } catch {
        $log.Text = $_.Exception.Message
        $status.Text = 'Installation failed. Review the error and retained files before retrying.'
        foreach ($control in @($hostInput,$keyInput,$publisherInput,$executableInput,$componentsInput,$fileButton,$folderButton)) {$control.Enabled=$true}
        $installButton.Enabled = $administrator
    } finally {
        $operation.Worker.Dispose()
        $script:operation = $null
        $closeButton.Enabled = $true
        $progress.Style = 'Blocks'
    }
})
$form.Add_FormClosing({param($sender,$event)
    if ($script:operation) {$event.Cancel=$true;$status.Text='Wait for the current installation to finish before closing.'}
})
try {
    if ($RenderPreview) {
        $form.CreateControl()
        $form.PerformLayout()
        $bitmap = [Drawing.Bitmap]::new($form.Width,$form.Height)
        try {
            $form.DrawToBitmap($bitmap,[Drawing.Rectangle]::new(0,0,$form.Width,$form.Height))
            # Hidden forms suppress child drawing. Render each control without
            # showing the form or enabling installation/browser actions.
            $graphics = [Drawing.Graphics]::FromImage($bitmap)
            $left = [int](($form.Width-$form.ClientSize.Width)/2)
            $top = $form.Height-$form.ClientSize.Height-$left
            try {
                foreach ($control in $form.Controls) {
                    $child = [Drawing.Bitmap]::new($control.Width,$control.Height)
                    try {
                        $control.DrawToBitmap($child,[Drawing.Rectangle]::new(0,0,$control.Width,$control.Height))
                        $graphics.DrawImageUnscaled($child,($left+$control.Left),($top+$control.Top))
                    } finally {$child.Dispose()}
                }
            } finally {$graphics.Dispose()}
            $bitmap.Save([IO.Path]::GetFullPath($RenderPreview),[Drawing.Imaging.ImageFormat]::Png)
        } finally {$bitmap.Dispose()}
    } else {[Windows.Forms.Application]::Run($form)}
} finally {$timer.Dispose();$form.Dispose()}
