@echo off
chcp 65001 >nul
echo ========================================
echo       FlipPrint Portable Packager
echo ========================================
echo.

cd /d "%~dp0"

REM Set paths
set SRC_DIR=%~dp0FlipPrint\src-tauri\target\release
set DLL_DIR=%~dp0FlipPrint\src-tauri\dll
set OUTPUT_DIR=%~dp0FlipPrint-portable
set ZIP_NAME=FlipPrint-portable

echo [1/6] Creating output directory...
if exist "%OUTPUT_DIR%" rmdir /s /q "%OUTPUT_DIR%"
mkdir "%OUTPUT_DIR%"

echo.
echo [2/6] Copying main executable and runtime files...
xcopy /Y /E /I "%SRC_DIR%\*.exe" "%OUTPUT_DIR%\" >nul 2>&1
xcopy /Y /E /I "%SRC_DIR%\*.dll" "%OUTPUT_DIR%\" >nul 2>&1
xcopy /Y /E /I "%SRC_DIR%\resources" "%OUTPUT_DIR%\" >nul 2>&1

REM Copy WebView2Loader.dll if exists
if exist "%SRC_DIR%\WebView2Loader.dll" (
    copy /Y "%SRC_DIR%\WebView2Loader.dll" "%OUTPUT_DIR%\" >nul
)

echo.
echo [3/6] Copying PDF DLLs...
if not exist "%OUTPUT_DIR%\dll" mkdir "%OUTPUT_DIR%\dll"

REM Copy all DLLs from the DLL directory
xcopy /Y /E /I "%DLL_DIR%\*.dll" "%OUTPUT_DIR%\dll\" >nul 2>&1
xcopy /Y /E /I "%DLL_DIR%\*.lib" "%OUTPUT_DIR%\dll\" >nul 2>&1

REM Also copy DLLs directly to root (for easier loading)
xcopy /Y /E /I "%DLL_DIR%\*.dll" "%OUTPUT_DIR%\" >nul 2>&1

echo.
echo [4/6] Creating config folder (if needed)...
if not exist "%OUTPUT_DIR%\config" mkdir "%OUTPUT_DIR%\config"

echo.
echo [5/6] Creating portable info...
(
echo FlipPrint Portable Edition
echo ============================
echo.
echo This is a portable version of FlipPrint.
echo All files are self-contained in this folder.
echo.
echo No installation required - just run FlipPrint.exe!
echo.
echo Version: 1.0.0
) > "%OUTPUT_DIR%\README.txt"

echo.
echo [6/6] Creating ZIP archive...
cd "%~dp0"

REM Remove old ZIP if exists
if exist "%ZIP_NAME%.zip" del /F /Q "%ZIP_NAME%.zip"

REM Use PowerShell to create ZIP (built into Windows)
powershell -NoProfile -Command "Compress-Archive -Path '%OUTPUT_DIR%\*' -DestinationPath '%ZIP_NAME%.zip' -Force"

if exist "%ZIP_NAME%.zip" (
    for %%A in ("%ZIP_NAME%.zip") do echo    Size: %%~zA bytes
)

echo.
echo ========================================
echo [SUCCESS] Portable package created!
echo.
echo Output Folder: %OUTPUT_DIR%
echo Output ZIP:   %~dp0%ZIP_NAME%.zip
echo ========================================
echo.

pause
