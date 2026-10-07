@echo off
chcp 65001 >nul
echo ========================================
echo       FlipPrint Development Mode
echo ========================================
echo.

REM Get base directory
set BASE_DIR=%~dp0
set DLL_SOURCE=%BASE_DIR%FlipPrint\src-tauri\dll
set DLL_DEST=%BASE_DIR%FlipPrint\src-tauri\target\debug

if exist "%DLL_SOURCE%\pdf_dll.dll" (
    echo Copying DLLs to debug folder...
    powershell -NoProfile -Command "Copy-Item -Path '%DLL_SOURCE%\*.dll' -Destination '%DLL_DEST%\' -Force -ErrorAction SilentlyContinue"
    powershell -NoProfile -Command "Copy-Item -Path '%DLL_SOURCE%\*.lib' -Destination '%DLL_DEST%\' -Force -ErrorAction SilentlyContinue"
    echo [OK] DLLs copied
)

cd /d "%BASE_DIR%FlipPrint"

echo.
echo Starting hot-reload development server...
echo Press Ctrl+C to stop
echo.

npm run tauri dev

pause
