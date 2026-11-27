@echo off
setlocal

title FanRGB - Audio Reactive Red Lights

echo ========================================================
echo   FanRGB: Real-Time Audio Reactive Lights (Red)
echo ========================================================
echo.

:: Locate fanrgb.exe executable
set "FANRGB_EXE="
if exist "%~dp0..\target\release\fanrgb.exe" set "FANRGB_EXE=%~dp0..\target\release\fanrgb.exe"
if not defined FANRGB_EXE if exist "%~dp0..\fanrgb.exe" set "FANRGB_EXE=%~dp0..\fanrgb.exe"
if not defined FANRGB_EXE if exist "%~dp0..\target\debug\fanrgb.exe" set "FANRGB_EXE=%~dp0..\target\debug\fanrgb.exe"
if not defined FANRGB_EXE (
    where fanrgb.exe >nul 2>&1
    if not errorlevel 1 set "FANRGB_EXE=fanrgb.exe"
)

if not defined FANRGB_EXE (
    echo [ERROR] fanrgb.exe could not be found!
    echo Please compile the release binary: cargo build --release
    echo.
    pause
    exit /b 1
)

:: Terminate any running instance to prevent USB HID / GPU I2C bus conflicts
taskkill /F /IM fanrgb.exe >nul 2>&1

echo Starting WASAPI audio reactive visualizer (Red) for Motherboard + GPU...
echo Press Ctrl+C at any time to stop.
echo.

"%FANRGB_EXE%" bass red --gpu %*

if errorlevel 1 (
    echo.
    echo [INFO] Audio visualizer exited with code %errorlevel%.
    pause
)
