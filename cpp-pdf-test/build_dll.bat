@echo off
chcp 65001 >nul
echo ========================================
echo       Build PDF DLL
echo ========================================
echo.

set VCPKG_ROOT=C:\Users\lqr_l\Desktop\vcpkg
set VCPKG_TOOLCHAIN=%VCPKG_ROOT%\scripts\buildsystems\vcpkg.cmake

cd /d "%~dp0"

echo [1/3] Cleaning old DLL build...
if exist dll_output rmdir /s /q dll_output
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
echo [3/3] Building DLL...
cmake --build build --config Release --target pdf_dll

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Build failed!
    pause
    exit /b 1
)

REM Copy DLL files to FlipPrint
set SRC_DIR=%~dp0dll_output\Release
set DEST_DIR=%~dp0..\FlipPrint\src-tauri\dll
if not exist "%DEST_DIR%" mkdir "%DEST_DIR%"

echo.
echo Copying DLLs to FlipPrint...

REM Copy pdf_dll.dll and lib
copy /Y "%SRC_DIR%\pdf_dll.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\pdf_dll.lib" "%DEST_DIR%\"
copy /Y include\PdfDll.h "%DEST_DIR%\"

REM Copy PoDoFo and dependencies
copy /Y "%SRC_DIR%\podofo.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\brotlicommon.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\brotlidec.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\bz2.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\freetype.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\iconv-2.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\jpeg62.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\libcrypto-3-x64.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\liblzma.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\libpng16.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\libxml2.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\tiff.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\utf8proc.dll" "%DEST_DIR%\"
copy /Y "%SRC_DIR%\z.dll" "%DEST_DIR%\"

echo.
echo ========================================
echo [SUCCESS] DLL and dependencies copied!
echo.
echo Files in: %DEST_DIR%
echo.
dir "%DEST_DIR%\"
echo ========================================
echo.

pause
