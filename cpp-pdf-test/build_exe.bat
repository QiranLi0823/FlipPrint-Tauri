@echo off
chcp 65001 >nul
echo ========================================
echo       Build PDF Test EXE
echo ========================================
echo.

set VCPKG_ROOT=C:\Users\lqr_l\Desktop\vcpkg
set VCPKG_TOOLCHAIN=%VCPKG_ROOT%\scripts\buildsystems\vcpkg.cmake

cd /d "%~dp0"

echo [1/3] Cleaning old build...
if exist build rmdir /s /q build

echo [2/3] Configuring CMake...
cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE="%VCPKG_TOOLCHAIN%" -DCMAKE_BUILD_TYPE=Release -G "Visual Studio 17 2022" -A x64 -T host=x64

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] CMake configuration failed!
    pause
    exit /b 1
)

echo.
echo [3/3] Building EXE...
cmake --build build --config Release --target podofo_test

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Build failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo [SUCCESS] EXE built!
echo.
echo EXE: %~dp0build\Release\podofo_test.exe
echo ========================================
echo.

pause
