# FanRGB CLI Usage Guide

`fanrgb` is a fast, lightweight native Rust command-line tool to control RGB lighting on:
- **MSI MPG B550 GAMING PLUS (MS-7C56)** Motherboard headers (`J_RGB_1`, `J_RAINBOW_1`, `J_RAINBOW_2`, `ONBOARD_LED`) via USB HID.
- **Gigabyte GeForce RTX 3060 Ti GAMING OC** (and compatible Gigabyte GPUs) via NVIDIA NVAPI I2C.
- **WASAPI Sub-Bass Audio Visualizer** for real-time sound reactivity.

---

## Table of Contents
1. [Quick Start & Binary Setup](#1-quick-start--binary-setup)
2. [Color Formats & PowerShell Rules](#2-color-formats--powershell-rules)
3. [Motherboard & Fans Control](#3-motherboard--fans-control)
4. [GPU Lighting Control](#4-gpu-lighting-control)
5. [Synchronized System Control (Mobo + GPU)](#5-synchronized-system-control-mobo--gpu)
6. [Real-Time Audio Visualizer (`fanrgb bass`)](#6-real-time-audio-visualizer-fanrgb-bass)
7. [Custom JSON Effects & Profiles (`fanrgb effect`)](#7-custom-json-effects--profiles-fanrgb-effect)
8. [Color Presets Reference](#8-color-presets-reference)
9. [Hardware Animation Modes Reference](#9-hardware-animation-modes-reference)
10. [Automation, Startup & Scripting Recipes](#10-automation-startup--scripting-recipes)
11. [Hardware Safety & Troubleshooting](#11-hardware-safety--troubleshooting)

---

## 1. Quick Start & Binary Setup

### Building the Binary
From the repository root directory:
```powershell
cargo build --release
```
The compiled executable will be located at:
```text
.\target\release\fanrgb.exe
```

### Optional: Add to PATH or Create an Alias
To invoke `fanrgb` directly from any terminal:
```powershell
# In your PowerShell profile ($PROFILE):
Set-Alias fanrgb "E:\NAN\Github\fanrgbmpgb550\target\release\fanrgb.exe"
```

### Quick Commands Cheat Sheet
```powershell
# Check motherboard zone status
.\target\release\fanrgb.exe status

# Check GPU detection and I2C address
.\target\release\fanrgb.exe gpu status

# Turn everything off (instant blackout)
.\target\release\fanrgb.exe sync off

# Set both motherboard and GPU to solid purple
.\target\release\fanrgb.exe sync purple

# Start real-time sub-bass visualizer
.\target\release\fanrgb.exe bass purple --gpu
```

---

## 2. Color Formats & PowerShell Rules

`fanrgb` accepts colors in three distinct formats:

| Format | Example | Syntax Notes |
| :--- | :--- | :--- |
| **Preset Name** | `purple`, `cyan`, `darkblue` | Case-insensitive. See [Color Presets](#7-color-presets-reference). |
| **Hex Code (Quoted)** | `"#A020F0"`, `'#00FFCC'` | **Mandatory quotes** in PowerShell (otherwise `#` starts a comment). |
| **Hex Code (Bare)** | `A020F0`, `00FFCC` | Safe to omit `#` without quoting. |
| **RGB Integers** | `160 32 240`, `0 255 128` | Three space-separated integers between `0` and `255`. |

> [!WARNING]
> **PowerShell Quoting Rule:** In PowerShell, `#` is treated as the start of a comment!
> - ❌ `.\fanrgb.exe #A020F0` *(PowerShell discards `#A020F0`)*
> - ✔️ `.\fanrgb.exe "#A020F0"`
> - ✔️ `.\fanrgb.exe A020F0`

---

## 3. Motherboard & Fans Control

Controls all onboard LEDs and connected 12V RGB (`J_RGB_1`) and 5V ARGB headers (`J_RAINBOW_1`, `J_RAINBOW_2`).

### Check Motherboard Status
```powershell
.\target\release\fanrgb.exe status
```
*Outputs current effect mode, primary RGB values, and brightness level (1–10) for each zone.*

### Set Static Color
```powershell
# Using preset
.\target\release\fanrgb.exe purple
.\target\release\fanrgb.exe iceblue
.\target\release\fanrgb.exe red

# Using hex
.\target\release\fanrgb.exe "#A020F0"
.\target\release\fanrgb.exe 00FFCC

# Using RGB decimal values
.\target\release\fanrgb.exe 160 32 240
```

### Apply Hardware Animation Modes
Hardware-level animation modes execute autonomously on the motherboard controller:
```powershell
# Standard addressable rainbow wave
.\target\release\fanrgb.exe mode rainbow

# Breathing pulse with specific color
.\target\release\fanrgb.exe mode breathing purple
.\target\release\fanrgb.exe mode breathing "#FF3300"

# Meteor / shooting star effect
.\target\release\fanrgb.exe mode meteor cyan

# Realistic fire / flame simulation
.\target\release\fanrgb.exe mode fire

# Lightning strobe effect
.\target\release\fanrgb.exe mode lightning white
```

### Turn Off Motherboard LEDs
```powershell
.\target\release\fanrgb.exe off
```

---

## 4. GPU Lighting Control

Directly controls the RGB Fusion controller on Gigabyte graphics cards through official NVIDIA NVAPI I2C calls.

### Check GPU Connection & I2C Bus Address
```powershell
.\target\release\fanrgb.exe gpu status
```
*Expected output: detected GPU model and active I2C bus address (e.g. `0x32`).*

### Set GPU Static Color
```powershell
# Preset
.\target\release\fanrgb.exe gpu purple
.\target\release\fanrgb.exe gpu red

# Hex
.\target\release\fanrgb.exe gpu "#A020F0"
.\target\release\fanrgb.exe gpu FF5500

# RGB decimal
.\target\release\fanrgb.exe gpu 160 32 240
```

### GPU Hardware Animation Modes
```powershell
# Full spectrum color cycling
.\target\release\fanrgb.exe gpu mode color_cycle

# Traveling rainbow wave
.\target\release\fanrgb.exe gpu mode wave

# Smooth breathing fade
.\target\release\fanrgb.exe gpu mode breathing

# Color gradient blend
.\target\release\fanrgb.exe gpu mode gradient

# Flash strobe
.\target\release\fanrgb.exe gpu mode flash
```

### Turn Off GPU Lighting
```powershell
.\target\release\fanrgb.exe gpu off
```

---

## 5. Synchronized System Control (Mobo + GPU)

The `sync` command targets both the MSI motherboard headers and the Gigabyte graphics card in a single atomic action.

### Sync Static Color
```powershell
# Synchronize both devices to purple
.\target\release\fanrgb.exe sync purple
.\target\release\fanrgb.exe sync "#A020F0"
.\target\release\fanrgb.exe sync 160 32 240

# Synchronize both to neon cyan
.\target\release\fanrgb.exe sync cyan
```

### System-Wide Blackout
```powershell
# Turns off both Motherboard and GPU LEDs
.\target\release\fanrgb.exe sync off
```

---

## 6. Real-Time Audio Visualizer (`fanrgb bass`)

Captures Windows system audio via low-latency WASAPI loopback, runs high-speed FFT frequency analysis targeting sub-bass and kick-drum frequencies (default: 1.0 Hz – 25.0 Hz), and streams brightness updates to your hardware in real time with an ASCII VU meter.

```powershell
.\target\release\fanrgb.exe bass [color] [options]
```

### Command Flags & Parameters

| Flag | Description | Default | Example |
| :--- | :--- | :--- | :--- |
| `[color]` | Base reactive color (preset, hex, or RGB) | `blue` | `fanrgb bass purple` |
| `--gpu`, `-g` | Include GPU RGB along with motherboard fans | Disabled | `fanrgb bass purple --gpu` |
| `--device <name>`, `-d <name>` | Select specific audio output endpoint | Default device | `--device "Headphones"` |
| `--min <0-255>`, `-m <0-255>` | Minimum idle brightness floor | `0` (off when silent) | `--min 20` |
| `--fmin <hz>`, `--min-freq` | Lower sub-bass frequency bound | `1.0` Hz | `--fmin 20.0` |
| `--fmax <hz>`, `--max-freq` | Upper sub-bass frequency bound | `25.0` Hz | `--fmax 80.0` |
| `--decay <0.0-1.0>` | Falloff smoothing / release factor | `0.82` | `--decay 0.90` (slower) |

### Example Usage Scenarios

```powershell
# 1. Basic sub-bass visualizer on fans (default blue)
.\target\release\fanrgb.exe bass

# 2. Visualizer in purple synced to both Motherboard fans and GPU
.\target\release\fanrgb.exe bass purple --gpu

# 3. Ambient idle glow (never goes fully black when no bass hits)
.\target\release\fanrgb.exe bass red --gpu --min 15

# 4. Target specific audio device (e.g. USB DAC or Headset)
.\target\release\fanrgb.exe bass cyan --gpu --device "Realtek HD Audio"

# 5. Electronic / EDM punchy kick tuning (20 Hz - 70 Hz with quick snap)
.\target\release\fanrgb.exe bass "#FF0055" --gpu --fmin 20 --fmax 70 --decay 0.75
```

> [!TIP]
> Press `Ctrl + C` in the terminal at any time to stop the visualizer safely.

---

## 7. Custom JSON Effects & Profiles (`fanrgb effect`)

Create, customize, and execute dynamic lighting animations and multi-zone static profiles using custom JSON files.

### Commands

```powershell
# Play an effect file
.\target\release\fanrgb.exe effect effects/cyberpunk.json

# Shorthand: run by effect name directly from ./effects/
.\target\release\fanrgb.exe effect cyberpunk
.\target\release\fanrgb.exe effect police_strobe
.\target\release\fanrgb.exe effect amber_breath
.\target\release\fanrgb.exe effect zone_split

# Apply a multi-zone static profile once
.\target\release\fanrgb.exe effect static_profile

# List all available JSON effects in ./effects/
.\target\release\fanrgb.exe effect list

# Generate a starter template ready for editing
.\target\release\fanrgb.exe effect init cyberpunk my_custom_effect.json
.\target\release\fanrgb.exe effect init police my_police.json
.\target\release\fanrgb.exe effect init breath my_breath.json
.\target\release\fanrgb.exe effect init zones my_zones.json
.\target\release\fanrgb.exe effect init static my_static.json
```

---

### JSON Schema & Format Reference

#### Top-Level Fields

| Field | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `name` | String | No | `"Custom Effect"` | Friendly name shown in the terminal banner |
| `description` | String | No | `""` | Description of the effect or purpose |
| `author` | String | No | `""` | Author or credit |
| `loop` | Boolean | No | `true` | When `true`, loops the animation indefinitely |
| `repeat` | Integer | No | `null` | Limit animation to N full cycles (e.g. `5`), then stops |
| `fps` | Integer | No | `30` | Interpolation frame rate (5 to 60 FPS) |
| `sync_gpu` | Boolean | No | Auto | Whether to sync Gigabyte GPU (auto-detected if `gpu` or global color is used) |
| `keyframes` | Array | No | `null` | Sequence of animated frames/steps (see below) |
| `zones` | Object | No | `null` | Direct static zone color map (used for static profiles or fallback) |

---

#### Keyframe Fields (`keyframes: [...]`)

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `duration_ms` | Integer | **Required** | Frame duration in milliseconds (e.g. `1000`) |
| `transition` | String | `"linear"` | `"linear"` (smooth color fade) or `"instant"` / `"step"` / `"hold"` (jump and hold) |
| `color` | Color | Optional | Global color applied across all zones |
| `zones` | Object | Optional | Per-zone color map (overrides global `color` for specific headers) |
| `brightness` | Integer | `100` | Brightness percentage scale (0 to 100) |

---

#### Supported Zone Identifiers

Use any of these names in the `"zones"` object:

| Zone Key | Hardware Header / Device |
| :--- | :--- |
| `j_rainbow_1` | 5V 3-pin ARGB Header 1 (e.g. front case ARGB fans) |
| `j_rainbow_2` | 5V 3-pin ARGB Header 2 (e.g. rear ARGB fan / pump) |
| `j_rgb_1` | 12V 4-pin RGB Header (analog RGB light strips / fans) |
| `on_board_led` | Motherboard Chipset LEDs |
| `gpu` | Gigabyte Graphics Card Logo / Shroud RGB |
| `all` | All zones simultaneously |

---

#### Accepted Color Formats

Colors in JSON can be specified in any of the following formats:
- **Hex String**: `"#00FFFF"`, `"#A020F0"`, `"FF0055"`
- **Preset Name**: `"purple"`, `"cyan"`, `"red"`, `"iceblue"`, `"darkblue"`, `"white"`, `"off"`
- **RGB Array**: `[0, 255, 255]`
- **RGB Object**: `{"r": 0, "g": 255, "b": 255}`

---

### Custom Effect Examples

#### 1. Smooth Fade (Cyberpunk Neon)
Smoothly interpolates between neon cyan, pink, and purple at 30 FPS:
```json
{
  "name": "Cyberpunk Neon",
  "description": "Smooth fade between vibrant neon cyan and hot magenta/purple",
  "loop": true,
  "fps": 30,
  "sync_gpu": true,
  "keyframes": [
    {
      "duration_ms": 1500,
      "transition": "linear",
      "color": "#00FFFF"
    },
    {
      "duration_ms": 1500,
      "transition": "linear",
      "color": "#FF007F"
    },
    {
      "duration_ms": 1500,
      "transition": "linear",
      "color": "#A020F0"
    }
  ]
}
```

#### 2. High-Speed Police Strobe
Uses `"instant"` transitions to flash red and blue:
```json
{
  "name": "Police Strobe",
  "loop": true,
  "fps": 30,
  "sync_gpu": true,
  "keyframes": [
    { "duration_ms": 100, "transition": "instant", "color": "red" },
    { "duration_ms": 50,  "transition": "instant", "color": "off" },
    { "duration_ms": 100, "transition": "instant", "color": "red" },
    { "duration_ms": 150, "transition": "instant", "color": "off" },
    { "duration_ms": 100, "transition": "instant", "color": "blue" },
    { "duration_ms": 50,  "transition": "instant", "color": "off" },
    { "duration_ms": 100, "transition": "instant", "color": "blue" },
    { "duration_ms": 150, "transition": "instant", "color": "off" }
  ]
}
```

#### 3. Dual-Zone Wave (Independent Fan & GPU Colors)
Animates each header and the GPU with contrasting colors:
```json
{
  "name": "Dual Zone Wave",
  "loop": true,
  "fps": 30,
  "sync_gpu": true,
  "keyframes": [
    {
      "duration_ms": 1200,
      "transition": "linear",
      "zones": {
        "j_rainbow_1": "cyan",
        "j_rainbow_2": "purple",
        "j_rgb_1": "blue",
        "on_board_led": "magenta",
        "gpu": "cyan"
      }
    },
    {
      "duration_ms": 1200,
      "transition": "linear",
      "zones": {
        "j_rainbow_1": "purple",
        "j_rainbow_2": "cyan",
        "j_rgb_1": "magenta",
        "on_board_led": "blue",
        "gpu": "purple"
      }
    }
  ]
}
```

#### 4. Multi-Zone Static Profile
Sets specific colors across all hardware headers once and exits cleanly without running a background loop:
```json
{
  "name": "Static Studio Profile",
  "sync_gpu": true,
  "zones": {
    "j_rainbow_1": "#50C8FF",
    "j_rainbow_2": "#A020F0",
    "j_rgb_1": "#0014B4",
    "on_board_led": "#4B0082",
    "gpu": "#50C8FF"
  }
}
```

---

## 8. Color Presets Reference

The following names are built directly into `fanrgb`:

| Preset Name | Aliases | Hex Code | RGB (R, G, B) |
| :--- | :--- | :--- | :--- |
| `purple` | — | `#A020F0` | `(160, 32, 240)` |
| `darkpurple` | `dark_purple`, `deeppurple`, `deep_purple`, `indigo` | `#4B0082` | `(75, 0, 130)` |
| `red` | — | `#FF0000` | `(255, 0, 0)` |
| `green` | — | `#00FF00` | `(0, 255, 0)` |
| `blue` | — | `#0000FF` | `(0, 0, 255)` |
| `darkblue` | `dark_blue`, `navy`, `deepblue`, `deep_blue` | `#0014B4` | `(0, 20, 180)` |
| `lightblue` | `light_blue`, `skyblue`, `sky_blue` | `#00B4FF` | `(0, 180, 255)` |
| `iceblue` | `ice_blue` | `#50C8FF` | `(80, 200, 255)` |
| `cyan` | — | `#00FFFF` | `(0, 255, 255)` |
| `magenta` | — | `#FF00FF` | `(255, 0, 255)` |
| `yellow` | — | `#FFFF00` | `(255, 255, 0)` |
| `white` | — | `#FFFFFF` | `(255, 255, 255)` |
| `orange` | — | `#FF7800` | `(255, 120, 0)` |
| `off` | — | `#000000` | `(0, 0, 0)` |

---

## 9. Hardware Animation Modes Reference

### Motherboard Modes (`fanrgb mode <name> [color]`)

| Mode Name | Mode Code | Supported Color | Description |
| :--- | :--- | :--- | :--- |
| `rainbow` / `rainbow_wave` | `0x19` (`25`) | Spectrum | Flowing multi-color ARGB wave |
| `rainbow_flashing` | `0x1D` (`29`) | Spectrum | Rapid alternating rainbow flash |
| `static` | `0x01` (`1`) | Custom | Solid static color |
| `breathing` | `0x02` (`2`) | Custom | Smooth cyclic breathing pulse |
| `flashing` | `0x03` (`3`) | Custom | Sharp strobe / on-off flash |
| `double_flashing` | `0x04` (`4`) | Custom | Double pulse strobe |
| `lightning` | `0x05` (`5`) | Custom | Random electric lightning strikes |
| `meteor` | `0x07` (`7`) | Custom | Chasing shooting star meteor effect |
| `color_pulse` | `0x15` (`21`) | Custom | Rhythmic color pulse |
| `color_shift` | `0x16` (`22`) | Spectrum | Gradual smooth spectrum transition |
| `color_wave` | `0x17` (`23`) | Custom | Single-color directional wave |
| `marquee` | `0x18` (`24`) | Custom | Theater marquee chasing lights |
| `visor` | `0x1B` (`27`) | Custom | Back-and-forth scanning visor sweep |
| `stack` | `0x24` (`36`) | Custom | Segment stacking LED animation |
| `fire` | `0x26` (`38`) | Auto | Flickering fireplace flame simulation |
| `disable` / `off` | `0x00` (`0`) | None | Turns off LEDs completely |

### GPU Modes (`fanrgb gpu mode <name>`)

| Mode Name | Register Code | Description |
| :--- | :--- | :--- |
| `rainbow` / `color_cycle` | `0x03` | Continuous RGB spectrum rotation |
| `wave` / `rainbow_wave` | `0x07` | Directional rainbow color wave |
| `static` | `0x01` | Fixed solid color |
| `breathing` / `pulse` | `0x02` | Smooth fade in and out |
| `flash` / `flashing` | `0x04` | Single flash interval |
| `double_flash` | `0x08` | Dual-pulse strobe |
| `gradient` | `0x05` | Linear gradient color shift |

---

## 10. Automation, Startup & Scripting Recipes

### Run at Windows Login (Task Scheduler)
Create a scheduled task to automatically apply your preferred profile on user login without keeping any background apps running:

1. Open **Task Scheduler** (`taskschd.msc`).
2. Click **Create Basic Task...**
3. Set Trigger to **When I log on**.
4. Set Action to **Start a program**:
   - **Program/script:** `E:\NAN\Github\fanrgbmpgb550\target\release\fanrgb.exe`
   - **Add arguments:** `sync purple` *(or your preferred color or `effect static_profile`)*
5. Check **Run with highest privileges** if required by your GPU NVAPI configuration.

### PowerShell Day / Night Profile Script
Save as `rgb_profile.ps1`:
```powershell
param (
    [string]$Time = "day"
)

$fanrgb = "E:\NAN\Github\fanrgbmpgb550\target\release\fanrgb.exe"

switch ($Time.ToLower()) {
    "day"        { & $fanrgb sync cyan }
    "night"      { & $fanrgb sync darkpurple }
    "cyberpunk"  { & $fanrgb effect cyberpunk }
    "police"     { & $fanrgb effect police_strobe }
    "off"        { & $fanrgb sync off }
    "party"      { & $fanrgb bass purple --gpu --min 10 }
    default      { & $fanrgb sync purple }
}
```

---

## 11. Hardware Safety & Troubleshooting

### Zero EEPROM Flash Wear
The MSI motherboard controller stores settings in either EEPROM (flash memory) or SRAM (volatile RAM).  
- `fanrgb` **strictly targets SRAM** (`save_data = 0x00` at packet byte 184).
- Custom JSON animations and visualizers stream real-time updates directly to volatile RAM with **zero risk of flash memory wear or EEPROM degradation**.

### Troubleshooting Checklist

1. **"Could not open MSI Motherboard"**:
   - Ensure MSI Center / Mystic Light / Dragon Center is closed and not locking the USB HID interface.
   - Run PowerShell or Command Prompt **As Administrator**.
   - Check device manager for HID devices under Vendor ID `0x0DB0` and Product ID `0x550` or `0x7C56`.

2. **"Could not detect Gigabyte GPU RGB controller"**:
   - Ensure official NVIDIA drivers are installed and NVAPI is accessible.
   - Ensure Gigabyte RGB Fusion / AORUS Engine background services are stopped so they do not contend for the I2C bus.
   - Check `fanrgb gpu status` to verify if I2C bus detection completes successfully.

3. **Audio Visualizer produces no light or reaction**:
   - Check that the active audio playback device matches `--device`.
   - Ensure audio is playing at an audible volume level (sub-bass below 80 Hz).
   - If using quiet audio, lower the frequency filter (`--fmin 20 --fmax 100`) or adjust the decay factor.
