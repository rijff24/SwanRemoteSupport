@echo off
setlocal DisableDelayedExpansion
set "swan_powershell=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if exist "%SystemRoot%\Sysnative\WindowsPowerShell\v1.0\powershell.exe" set "swan_powershell=%SystemRoot%\Sysnative\WindowsPowerShell\v1.0\powershell.exe"
"%swan_powershell%" -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-Company.ps1" -RequestElevation
set "swan_setup_exit=%errorlevel%"
if not "%swan_setup_exit%"=="0" (
    echo.
    echo Setup did not complete. Keep this folder and contact your support company.
    pause
)
exit /b %swan_setup_exit%
