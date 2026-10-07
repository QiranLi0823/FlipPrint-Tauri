@echo off
chcp 65001 >nul
echo ========================================
echo       FlipPrint PoDoFo Quick Build
echo ========================================
echo.

cd /d "%~dp0"

REM Check if vcpkg toolchain exists
set VCPKG_TOOLCHAIN=C:\Users\lqr_l\Desktop\vcpkg\scripts\buildsystems\vcpkg.cmake

if not exist "%VCPKG_TOOLCHAIN%" (
    echo [ERROR] vcpkg toolchain not found at:
    echo %VCPKG_TOOLCHAIN%
    echo.
    echo Please update VCPKG_ROOT in build.bat if vcpkg is installed elsewhere.
    pause
    exit /b 1
)

REM Quick build - only rebuild if source changed
echo Building...
cmake --build build --config Release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo [OK] Done!
) else (
    echo.
    echo [ERROR] Build failed!
)
