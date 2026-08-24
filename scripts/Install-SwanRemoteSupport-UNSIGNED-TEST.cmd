@echo off
setlocal
echo ================================================================
echo SWAN REMOTE SUPPORT - UNSIGNED SPARE-LAPTOP TEST ONLY
echo ================================================================
echo.
echo This launcher permits a completely unsigned Swan installer.
echo It does not bypass the Tailscale tag, network, consent, or hash checks.
echo Do not use it on a real customer computer or with customer data.
echo.
choice /C YN /N /M "Continue on an isolated spare laptop? [Y/N] "
if errorlevel 2 exit /b 2

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-SwanRemoteSupport.ps1" -Interactive -AllowUnsignedSwanInstaller
set "SWAN_EXIT=%ERRORLEVEL%"
if not "%SWAN_EXIT%"=="0" echo Swan Remote Support unsigned test setup failed with exit code %SWAN_EXIT%.
exit /b %SWAN_EXIT%
