@echo off
REM ClawdShell installer for Windows CMD
REM Usage: curl -fsSL https://clawdshell.dev/install.cmd -o install.cmd && install.cmd && del install.cmd

setlocal

if "%CLAWDSHELL_VERSION%"=="" set CLAWDSHELL_VERSION=latest
if "%CLAWDSHELL_REPO%"=="" set CLAWDSHELL_REPO=coffecup25/clawdshell

set ARCH=x86_64
set BINARY_NAME=clawdshell-windows-%ARCH%.exe

if "%CLAWDSHELL_VERSION%"=="latest" (
    set DOWNLOAD_URL=https://github.com/%CLAWDSHELL_REPO%/releases/latest/download/%BINARY_NAME%
) else (
    set DOWNLOAD_URL=https://github.com/%CLAWDSHELL_REPO%/releases/download/%CLAWDSHELL_VERSION%/%BINARY_NAME%
)

set INSTALL_DIR=%USERPROFILE%\.local\bin
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

set DEST=%INSTALL_DIR%\clawdshell.exe

echo.
echo   clawdshell installer
echo   windows-%ARCH%
echo.
echo   Downloading...

curl -fsSL "%DOWNLOAD_URL%" -o "%DEST%"
if %ERRORLEVEL% neq 0 (
    echo   Download failed: %DOWNLOAD_URL%
    exit /b 1
)

echo   Downloaded to %DEST%
echo.

REM Add to PATH for this session
set PATH=%INSTALL_DIR%;%PATH%

REM Run the interactive installer
"%DEST%" --install

endlocal
