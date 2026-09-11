@echo off
chcp 65001 >nul
echo ========================================
echo       FlipPrint Build with Python
echo ========================================
echo.

cd /d "%~dp0FlipPrint"

set PYTHON_EMBED_URL=https://www.python.org/ftp/python/3.11.5/python-3.11.5-embed-amd64.zip
set PYTHON_DIR=%~dp0python-embed
set PYTHON_EXE=%PYTHON_DIR%\python.exe

REM Check if bundled Python exists
if exist "%PYTHON_EXE%" (
    echo [INFO] Found bundled Python at: %PYTHON_EXE%
    goto BUILD
)

echo [1/5] Downloading portable Python...
echo This only needs to be done once.

if not exist "%PYTHON_DIR%" mkdir "%PYTHON_DIR%"

powershell -NoProfile -Command "Invoke-WebRequest -Uri '%PYTHON_EMBED_URL%' -OutFile '%PYTHON_DIR%\python-embed.zip'"

echo.
echo [2/5] Extracting Python...
powershell -NoProfile -Command "Expand-Archive -Path '%PYTHON_DIR%\python-embed.zip' -DestinationPath '%PYTHON_DIR%' -Force"
del "%PYTHON_DIR%\python-embed.zip"

echo.
echo [3/5] Configuring Python for pip...
echo import site >> "%PYTHON_DIR%\python311._pth"

echo.
echo [4/5] Installing pip...
powershell -NoProfile -Command "Invoke-WebRequest -Uri 'https://bootstrap.pypa.io/get-pip.py' -OutFile '%PYTHON_DIR%\get-pip.py'"
%PYTHON_EXE% "%PYTHON_DIR%\get-pip.py"
del "%PYTHON_DIR%\get-pip.py"

echo.
echo [5/5] Installing pypdf...
%PYTHON_EXE% -m pip install pypdf --quiet

:BUILD
echo.
echo ========================================
echo Python setup complete!
echo.

echo [1/3] Installing npm dependencies...
call npm install

echo.
echo [2/3] Building application...
call npm run tauri build

echo.
echo [3/3] Copying Python to release folder...
set RELEASE_DIR=%~dp0FlipPrint\src-tauri\target\release
set PYTHON_TARGET=%RELEASE_DIR%\python

REM Create python folder in release directory
if not exist "%PYTHON_TARGET%" mkdir "%PYTHON_TARGET%"

REM Copy all Python files
xcopy /E /Y "%PYTHON_DIR%\*" "%PYTHON_TARGET%\" >nul

if exist "%PYTHON_TARGET%\python.exe" (
    echo.
    echo [SUCCESS] Python bundled to:
    echo %PYTHON_TARGET%
) else (
    echo.
    echo [WARNING] Python copy failed. Please copy manually:
    echo Source: %PYTHON_DIR%
    echo Target: %PYTHON_TARGET%
)

echo.
echo ========================================
echo Build complete!
echo.
echo Output: %RELEASE_DIR%
echo ========================================
echo.

pause
