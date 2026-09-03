# Command-Line Interface (CLI) Guide

This guide covers how to operate the **MSI MPG B550 Motherboard** and **Gigabyte GPU RGB** controllers using the unified terminal CLI and batch scripts.

---

## 1. One-Click Desktop Launcher

| File | Action |
| :--- | :--- |
| [`start_bass_reactive.bat`](file:///E:/NAN/Github/fanrgbmpgb550/start_bass_reactive.bat) | Starts the **Pure Red Bass-Reactive mode** immediately in a terminal window. Displays live VU meter, red level, and stream FPS. Press `Ctrl+C` to stop. |

---

## 2. Motherboard Commands (`cli.py`)

Run commands directly from PowerShell, Windows Terminal, or CMD:

### Check Motherboard Status
```powershell
python cli.py status
```
Outputs active effect modes, RGB values, and brightness across all motherboard zones (`JRGB1`, `JRAINBOW1`, `JRAINBOW2`, `ONBOARD`).

### Start Motherboard Pure Red Bass-Reactive Mode
```powershell
python cli.py bass
```
* Captures audio via Windows WASAPI loopback from headphones/speakers.
* Filters out all frequencies outside `28 Hz – 90 Hz`.
* Displays a live ASCII VU meter in your terminal.
* Press `Ctrl+C` to smoothly stop.

### Motherboard Color Presets
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
python cli.py off           # Shuts down all motherboard LEDs
```

### Motherboard Custom RGB Values
```powershell
python cli.py 255 0 0       # Solid Red
python cli.py 255 120 0     # Amber / Warm Orange
python cli.py 0 180 255     # Ice Blue
```

### Motherboard Hardware Animations
```powershell
python cli.py mode rainbow_wave    # Full-spectrum animated wave
python cli.py mode breathing       # Red breathing pulse
python cli.py mode meteor          # Chasing meteor trail on ARGB fans
python cli.py mode flashing        # Flashing
python cli.py mode color_pulse     # Color pulse
python cli.py mode marquee         # Sequential LED chase
python cli.py mode fire            # Flickering flame effect
```

---

## 3. Gigabyte GPU RGB Commands (`cli.py gpu ...`)

Commands to control the illuminated side logo on your Gigabyte graphics card via native NVIDIA NVAPI I2C:

### Check GPU Connection & Status
```powershell
python cli.py gpu status
```

### Set GPU Colors & Presets
```powershell
python cli.py gpu red
python cli.py gpu blue
python cli.py gpu green
python cli.py gpu cyan
python cli.py gpu magenta
python cli.py gpu yellow
python cli.py gpu orange
python cli.py gpu white
python cli.py gpu off               # Turn off GPU RGB
python cli.py gpu 255 0 0           # Custom RGB
```

### Set GPU Hardware Animation Modes
```powershell
python cli.py gpu mode breathing    # Pulsing breath animation
python cli.py gpu mode flash        # Flashing
python cli.py gpu mode double_flash # Double pulse
python cli.py gpu mode color_cycle  # Automatic color cycling
python cli.py gpu mode wave         # Wave effect
python cli.py gpu mode gradient     # Gradient transition
```

---

## 4. Synchronized PC Lighting (Motherboard + GPU)

Control both devices together with a single command:

### Synchronized Colors
```powershell
python cli.py sync red              # Set Motherboard + GPU to Pure Red
python cli.py sync blue             # Set Motherboard + GPU to Blue
python cli.py sync 255 0 0          # Set Motherboard + GPU to custom RGB
python cli.py sync off              # Turn off all RGB across entire PC
```

### Synchronized Pure Red Bass Visualizer
Pulse the Motherboard case fans **AND** the GPU side logo simultaneously to the bass beat:
```powershell
python cli.py bass --gpu
```

---

## 5. Advanced Visualizer Options (`bass_reactive.py`)

```powershell
# Default pure red visualizer (Motherboard only)
python bass_reactive.py

# Synchronize with GPU logo
python bass_reactive.py --gpu

# Strobe-like punchy kicks (snappier decay)
python bass_reactive.py --gpu --mode kick --decay 0.55

# Continuous 808 sub-bass tracking
python bass_reactive.py --gpu --mode rumble

# Higher bass boost
python bass_reactive.py --gpu --sensitivity 1.5

# Keep a subtle resting red glow between beats (instead of pitch black)
python bass_reactive.py --gpu --min-brightness 10

# List available audio output devices
python bass_reactive.py --list-devices
```

---

## 6. Hardware Diagnostics Scripts

Located in [`tests/`](file:///E:/NAN/Github/fanrgbmpgb550/tests/):

```powershell
# Motherboard detection, firmware version query, and color test
python tests/run_test.py

# GPU detection, I2C address scan, 0xAB signature test, and color test
python tests/test_gpu.py

# Scan and enumerate USB HID devices matching MSI VID/PID
python tests/test_enum.py

# Low-level HID Feature Report 0x52 byte dump
python tests/test_read.py
```

---

## 7. Troubleshooting

### Problem: "No device matching VID 0x1462 PID 0x7C56 found"
* Ensure MSI Center or Dragon Center is closed (MSI software locks the USB HID interface).

### Problem: GPU controller not responding / write error
* Ensure Gigabyte Control Center (GCC) or AORUS Engine is closed.
* Run `python tests/test_gpu.py` to confirm the I2C connection and signature.

### Problem: Audio loopback capture error / no sound reaction
* Ensure your headphones/speakers are set as the **Default Windows Playback Device**.
* Run `python bass_reactive.py --list-devices` to verify detected output endpoints.
