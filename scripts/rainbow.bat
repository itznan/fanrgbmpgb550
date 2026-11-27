@echo off
setlocal

title FanRGB - Rainbow Mode

echo ========================================================
echo   FanRGB: Setting all devices to Rainbow Mode
echo ========================================================
echo.

:: Locate the fanrgb executable
set "FANRGB_EXE="

if exist "%~dp0..\target\release\fanrgb.exe" set "FANRGB_EXE=%~dp0..\target\release\fanrgb.exe"
if not defined FANRGB_EXE if exist "%~dp0..\fanrgb.exe" set "FANRGB_EXE=%~dp0..\fanrgb.exe"
if not defined FANRGB_EXE if exist "%~dp0..\target\debug\fanrgb.exe" set "FANRGB_EXE=%~dp0..\target\debug\fanrgb.exe"
if not defined FANRGB_EXE (
    where fanrgb.exe >nul 2>&1
    if not errorlevel 1 set "FANRGB_EXE=fanrgb.exe"
)

:: If not found, try building release binary if cargo is available
if not defined FANRGB_EXE (
    echo [INFO] fanrgb.exe not found. Checking for Rust cargo compiler...
    where cargo.exe >nul 2>&1
    if errorlevel 1 goto :not_found
    echo [INFO] Building fanrgb release binary...
    pushd "%~dp0.."
    cargo build --release
    popd
    if exist "%~dp0..\target\release\fanrgb.exe" set "FANRGB_EXE=%~dp0..\target\release\fanrgb.exe"
)

if not defined FANRGB_EXE goto :not_found

:: GPU rainbow style: default is rainbow (color_cycle), or wave if specified
set "GPU_MODE=rainbow"
set "DO_PAUSE=0"

:parse_args
if "%~1"=="" goto :done_args
if /i "%~1"=="wave" set "GPU_MODE=wave"
if /i "%~1"=="--wave" set "GPU_MODE=wave"
if /i "%~1"=="-w" set "GPU_MODE=wave"
if /i "%~1"=="pause" set "DO_PAUSE=1"
if /i "%~1"=="--pause" set "DO_PAUSE=1"
if /i "%~1"=="-p" set "DO_PAUSE=1"
shift
goto :parse_args
:done_args

echo [1/2] Setting Motherboard (Headers, ARGB Fans, Onboard LEDs)...
"%FANRGB_EXE%" mode rainbow
set "MOBO_ERR=%errorlevel%"

echo.
echo [2/2] Setting GPU RGB controller [%GPU_MODE%]...
"%FANRGB_EXE%" gpu mode %GPU_MODE%
set "GPU_ERR=%errorlevel%"

echo.
echo ========================================================
if %MOBO_ERR% neq 0 (
    echo   [WARNING] Motherboard returned error code %MOBO_ERR%.
    echo   Make sure MSI Center is closed and try running as Administrator.
)
if %GPU_ERR% neq 0 (
    echo   [WARNING] GPU returned error code %GPU_ERR%.
    echo   Ensure official NVIDIA drivers and NVAPI are accessible.
)
if %MOBO_ERR% equ 0 if %GPU_ERR% equ 0 (
    echo   [SUCCESS] Everything is now running in Rainbow Mode!
)
echo ========================================================
echo.

if "%DO_PAUSE%"=="1" (
    pause
)

if %MOBO_ERR% neq 0 exit /b %MOBO_ERR%
if %GPU_ERR% neq 0 exit /b %GPU_ERR%
exit /b 0

:not_found
echo [ERROR] fanrgb.exe could not be found or built!
echo Please run "cargo build --release" first.
echo ========================================================
echo.
pause
exit /b 1
