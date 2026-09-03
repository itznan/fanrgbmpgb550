# Command-Line Interface (CLI) Guide

This guide covers how to operate the **MSI MPG B550 GAMING PLUS RGB Controller** using the terminal CLI and batch scripts.

---

## 1. One-Click Desktop Launcher

| File | Action |
| :--- | :--- |
| [`start_bass_reactive.bat`](file:///E:/NAN/Github/fanrgbmpgb550/start_bass_reactive.bat) | Starts the **Pure Red Bass-Reactive mode** immediately in a terminal window. Displays live VU meter, red level, and stream FPS. Press `Ctrl+C` to stop. |

---

## 2. CLI Reference (`cli.py`)

Run commands directly from PowerShell, Windows Terminal, or CMD:

### Check Status
```powershell
python cli.py status
```
Outputs active effect modes, RGB values, and brightness across all motherboard zones (`JRGB1`, `JRAINBOW1`, `JRAINBOW2`, `ONBOARD`).

### Start Pure Red Bass-Reactive Mode
```powershell
python cli.py bass
```
* Captures headphone audio via Windows WASAPI loopback.
* Filters out all frequencies outside `28 Hz – 90 Hz`.
* Displays a live ASCII VU meter in your terminal.
* Press `Ctrl+C` at any time to smoothly stop and return lights to resting state.

### Color Presets
```powershell
python cli.py red
python cli.py green
python cli.py blue
python cli.py cyan
python cli.py magenta
python cli.py yellow
python cli.py orange
python cli.py purple
python cli.py white
python cli.py off           # Shuts down all LEDs
```

### Custom RGB Values
```powershell
# Syntax: python cli.py <red> <green> <blue>
python cli.py 255 0 0       # Solid Red
python cli.py 255 120 0     # Amber / Warm Orange
python cli.py 0 180 255     # Ice Blue
```

### Hardware Animations
```powershell
# Syntax: python cli.py mode <animation_mode> [r g b]
python cli.py mode rainbow_wave    # Full-spectrum animated wave
python cli.py mode breathing       # Red breathing pulse
python cli.py mode meteor          # Chasing meteor trail on ARGB fans
python cli.py mode flashing        # Flashing
python cli.py mode color_pulse     # Color pulse
python cli.py mode marquee         # Sequential LED chase
python cli.py mode fire            # Flickering flame effect
```

---

## 3. Dedicated Bass Visualizer Runner (`bass_reactive.py`)

For advanced command-line tuning of the audio visualizer:

```powershell
# Run with default settings (Hybrid mode, pure red)
python bass_reactive.py

# Strobe-like punchy kicks (snappier decay)
python bass_reactive.py --mode kick --decay 0.55

# Continuous 808 sub-bass tracking
python bass_reactive.py --mode rumble

# Higher bass boost
python bass_reactive.py --sensitivity 1.5

# Keep a subtle resting red glow between beats (instead of pitch black)
python bass_reactive.py --min-brightness 10

# List all available audio output devices
python bass_reactive.py --list-devices
```

---

## 4. Hardware Diagnostics Scripts

Located in [`tests/`](file:///E:/NAN/Github/fanrgbmpgb550/tests/):

```powershell
# Full detection, firmware version query, and test color application
python tests/run_test.py

# Scan and enumerate USB HID devices matching MSI VID/PID
python tests/test_enum.py

# Low-level HID Feature Report 0x52 byte dump
python tests/test_read.py
```

---

## 5. Troubleshooting

### Problem: "No device matching VID 0x1462 PID 0x7C56 found"
* Ensure MSI Center or Dragon Center is closed (MSI software locks the USB HID interface).
* Run `python tests/test_enum.py` to confirm Windows detects the onboard Nuvoton USB controller.

### Problem: Audio loopback capture error / no sound reaction
* Ensure your headphones/speakers are set as the **Default Windows Playback Device**.
* Run `python bass_reactive.py --list-devices` to verify detected output endpoints.
* Specify a custom device if needed:
  ```powershell
  python bass_reactive.py --device "Headphones"
  ```
