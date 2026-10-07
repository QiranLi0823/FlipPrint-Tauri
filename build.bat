@echo off
chcp 65001 >nul
echo ========================================
echo       FlipPrint Build (C++ DLL)
echo ========================================
echo.

cd /d "%~dp0FlipPrint"

REM Set DLL source directory (C++ build output)
set DLL_SOURCE=%~dp0cpp-pdf-test\dll_output\Release
set DLL_DEST=%~dp0src-tauri\dll

echo [1/4] Installing npm dependencies...
call npm install

echo.
echo [2/4] Building Tauri application...
call npm run tauri build

echo.
echo [3/4] Copying PDF DLL and dependencies...
if not exist "%DLL_DEST%" mkdir "%DLL_DEST%"

REM Copy PDF DLL
if exist "%DLL_SOURCE%\pdf_dll.dll" (
    copy /Y "%DLL_SOURCE%\pdf_dll.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\pdf_dll.lib" "%DLL_DEST%\"
    echo [OK] PDF DLL copied
) else (
    echo [WARNING] PDF DLL not found at: %DLL_SOURCE%
    echo Please run cpp-pdf-test\build_dll.bat first
)

REM Copy PoDoFo DLLs
if exist "%DLL_SOURCE%\podofo.dll" (
    copy /Y "%DLL_SOURCE%\podofo.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\brotlicommon.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\brotlidec.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\bz2.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\freetype.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\iconv-2.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\jpeg62.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\libcrypto-3-x64.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\liblzma.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\libpng16.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\libxml2.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\tiff.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\utf8proc.dll" "%DLL_DEST%\"
    copy /Y "%DLL_SOURCE%\z.dll" "%DLL_DEST%\"
    echo [OK] PoDoFo DLLs copied
) else (
    echo [WARNING] PoDoFo DLLs not found
)

REM Copy header file
copy /Y "%~dp0cpp-pdf-test\include\PdfDll.h" "%DLL_DEST%\" >nul 2>&1

echo.
echo [4/4] Copying DLLs to release folder...
set RELEASE_DIR=%~dp0FlipPrint\src-tauri\target\release

if exist "%DLL_DEST%\pdf_dll.dll" (
    copy /Y "%DLL_DEST%\*.dll" "%RELEASE_DIR%\" >nul 2>&1
    echo [OK] DLLs copied to release folder
)

echo.
echo ========================================
echo Build complete!
echo.
echo Output: %RELEASE_DIR%
echo DLL: %DLL_DEST%
echo ========================================
echo.

pause
