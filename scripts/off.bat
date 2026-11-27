@echo off
setlocal

title FanRGB - Turn Off RGB

:: Locate fanrgb.exe
set "FANRGB_EXE="
if exist "%~dp0..\target\release\fanrgb.exe" set "FANRGB_EXE=%~dp0..\target\release\fanrgb.exe"
if not defined FANRGB_EXE if exist "%~dp0..\fanrgb.exe" set "FANRGB_EXE=%~dp0..\fanrgb.exe"
if not defined FANRGB_EXE if exist "%~dp0..\target\debug\fanrgb.exe" set "FANRGB_EXE=%~dp0..\target\debug\fanrgb.exe"
if not defined FANRGB_EXE (
    where fanrgb.exe >nul 2>&1
    if not errorlevel 1 set "FANRGB_EXE=fanrgb.exe"
)

if not defined FANRGB_EXE (
    echo [ERROR] fanrgb.exe not found. Run "cargo build --release" first.
    pause
    exit /b 1
)

:: Stop any running visualizer/effect instance so it cannot override the off state
taskkill /F /IM fanrgb.exe >nul 2>&1

"%FANRGB_EXE%" off
set "MOBO_ERR=%errorlevel%"
"%FANRGB_EXE%" gpu off
set "GPU_ERR=%errorlevel%"

if %MOBO_ERR% neq 0 exit /b %MOBO_ERR%
if %GPU_ERR% neq 0 exit /b %GPU_ERR%
exit /b 0
