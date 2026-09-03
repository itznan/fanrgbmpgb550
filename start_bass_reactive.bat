@echo off
title MSI MPG B550 - Pure Red Bass Reactive RGB
cd /d "%~dp0"

echo ============================================================
echo  MSI MPG B550 GAMING PLUS - Pure Red Bass Reactive RGB
echo ============================================================
echo Audio Source : Default Headphones/Speakers (WASAPI Loopback)
echo Color        : 100%% Pure Red (Zero orange, Zero amber)
echo Status       : Starting visualizer stream...
echo.
echo Tip: Press Ctrl+C at any time in this window to stop.
echo.

python bass_reactive.py

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Visualizer encountered an issue or Python was not found.
    pause
)
