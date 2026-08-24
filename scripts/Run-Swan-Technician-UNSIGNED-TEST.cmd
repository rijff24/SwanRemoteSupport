@echo off
setlocal
echo ================================================================
echo SWAN REMOTE SUPPORT - UNSIGNED TECHNICIAN TEST CLIENT
echo ================================================================
echo.
echo This prepares a portable technician-side copy of the unsigned build.
echo Use it only on the Swan technician laptop for the isolated test.
echo.
choice /C YN /N /M "Prepare and run the unsigned technician client? [Y/N] "
if errorlevel 2 exit /b 2

set "SWAN_SOURCE="
set "SWAN_TARGET=%~dp0SwanRemoteSupport-Technician-UNSIGNED-TEST.exe"

for %%F in ("%~dp0SwanRemoteSupport-*-x64-unsigned-install.exe") do if exist "%%~fF" set "SWAN_SOURCE=%%~fF"

if not defined SWAN_SOURCE (
  echo No SwanRemoteSupport x64 unsigned installer was found in this folder.
  exit /b 3
)

copy /B /Y "%SWAN_SOURCE%" "%SWAN_TARGET%" >nul
if errorlevel 1 (
  echo Could not create the technician-side test executable.
  exit /b 4
)

start "" "%SWAN_TARGET%"
exit /b 0
