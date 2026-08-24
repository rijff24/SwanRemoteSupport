@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-SwanRemoteSupport.ps1" -Interactive
set "SWAN_EXIT=%ERRORLEVEL%"
if not "%SWAN_EXIT%"=="0" echo Swan Remote Support express setup failed with exit code %SWAN_EXIT%.
exit /b %SWAN_EXIT%
