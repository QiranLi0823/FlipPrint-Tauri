@echo off
chcp 65001 >nul
echo ========================================
echo       FlipPrint PoDoFo Build Script
echo ========================================
echo.

set VCPKG_ROOT=C:\Users\lqr_l\Desktop\vcpkg
set VCPKG_TOOLCHAIN=%VCPKG_ROOT%\scripts\buildsystems\vcpkg.cmake

cd /d "%~dp0"

echo [1/2] Configuring CMake...
cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE="%VCPKG_TOOLCHAIN%"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] CMake configuration failed!
    pause
    exit /b 1
)

echo.
echo [2/2] Building...
cmake --build build --config Release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Build failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo [SUCCESS] Build complete!
echo.
echo Executable: %~dp0build\podofo_test.exe
echo ========================================
echo.

pause
