# MSI MPG B550 & Gigabyte GPU RGB Controller (Rust Native CLI - 2026 Edition)

[![Developer](https://img.shields.io/badge/Developer-itznan-blue.svg)](https://github.com/itznan)
[![GitHub](https://img.shields.io/badge/GitHub-github.com%2Fitznan-181717?logo=github)](https://github.com/itznan)
[![Rust Edition](https://img.shields.io/badge/Rust%20Edition-2021-orange.svg)](Cargo.toml)
[![Release Year](https://img.shields.io/badge/Release-2026-brightgreen.svg)](https://github.com/itznan/fanrgbmpgb550)

Ultra-fast, lightweight native Rust CLI lighting controller for:
1. **MSI MPG B550 GAMING PLUS (MS-7C56)** Motherboard (`USB HID Feature Report 0x52`)
2. **Gigabyte GeForce RTX 3060 Ti GAMING OC** Graphics Card (`NVIDIA NVAPI I2C 0x32/0xAB Probe`)

Developed by **itznan** ([github.com/itznan](https://github.com/itznan)) — 2026.

---

## Table of Contents
- [Key Features](#key-features)
- [Building from Source](#building-from-source)
- [Command-Line Usage (CLI)](#command-line-usage-cli)
  - [Hardware Status](#hardware-status)
  - [Color Formats & PowerShell Quoting](#color-formats--powershell-quoting)
  - [Motherboard & Fans Control](#motherboard--fans-control)
  - [GPU Control](#gpu-control)
  - [Synchronized Control (Mobo + GPU)](#synchronized-control-mobo--gpu)
  - [Real-Time Audio Sub-Bass Visualizer](#real-time-audio-sub-bass-visualizer)
  - [Custom JSON Effects & Profiles](#custom-json-effects--profiles)
- [Animation Modes Reference](#animation-modes-reference)
  - [Motherboard Modes](#motherboard-modes)
  - [GPU Modes](#gpu-modes)
- [Color Presets Reference](#color-presets-reference)
- [Project Architecture & Directory Structure](#project-architecture--directory-structure)
- [Hardware Safety & Architecture](#hardware-safety--architecture)
- [Developer & Credits](#developer--credits)

---

## Key Features
* **Minimal System Footprint**: Pure Rust CLI (< 5 MB binary, tiny memory usage), 0% idle CPU overhead.
* **Deterministic Control**: Direct, low-level hardware control without background bloatware or unrequested transitions.
* **Zero Flash Wear Guarantee**: Volatile RAM updates (`save_data = 0x00`) strictly protect motherboard EEPROM from wear.
* **Real-Time WASAPI Loopback Sub-Bass Visualizer**: High-resolution audio reactivity with terminal ASCII meter.
* **Synchronized Control**: One command syncs motherboard, fans, and graphics card simultaneously.

---

## Building from Source

### Prerequisites
* Rust toolchain (1.70+ recommended): `rustup default stable-x86_64-pc-windows-msvc`
* C++ Build Tools (MSVC)

### Build Command
```powershell
cargo build --release
```
The optimized executable will be created at `.\target\release\fanrgb.exe`.

---

## Command-Line Usage (CLI)

```powershell
.\target\release\fanrgb.exe <command> [arguments]
```

### Hardware Status

Check current active zones, modes, colors, and I2C connection:

```powershell
# Inspect Motherboard zones (J_RGB_1, J_RAINBOW_1, J_RAINBOW_2, ONBOARD_LED)
.\target\release\fanrgb.exe status

# Inspect GPU NVAPI connection and active I2C bus address
.\target\release\fanrgb.exe gpu status
```

---

### Color Formats & PowerShell Quoting

Colors can be specified in three formats:

1. **Hexadecimal** (6-digit hex):
   > **IMPORTANT (PowerShell Users):** In PowerShell, `#` is the comment character!  
   > **Always wrap hex values in quotes** (e.g. `"#A020F0"` or `'#A020F0'`) or **omit the `#`** (e.g. `A020F0`).

   ```powershell
   # Quoted hex (recommended)
   .\target\release\fanrgb.exe "#A020F0"
   .\target\release\fanrgb.exe sync "#800080"
   .\target\release\fanrgb.exe gpu "#FF5500"

   # Or unquoted without '#'
   .\target\release\fanrgb.exe A020F0
   .\target\release\fanrgb.exe sync 00FFCC
   ```

2. **Preset Names**:
   ```powershell
   .\target\release\fanrgb.exe purple
   .\target\release\fanrgb.exe sync red
   .\target\release\fanrgb.exe gpu cyan
   ```

3. **RGB Integers** (`R G B`, 0–255):
   ```powershell
   .\target\release\fanrgb.exe 160 32 240
   .\target\release\fanrgb.exe sync 255 80 0
   .\target\release\fanrgb.exe gpu 0 255 128
   ```

---

### Motherboard & Fans Control

```powershell
# Set static color on all motherboard headers and fans
.\target\release\fanrgb.exe "#A020F0"
.\target\release\fanrgb.exe purple
.\target\release\fanrgb.exe 160 32 240

# Set animation modes
.\target\release\fanrgb.exe mode rainbow
.\target\release\fanrgb.exe mode rainbow_wave
.\target\release\fanrgb.exe mode breathing "#A020F0"
.\target\release\fanrgb.exe mode meteor red
.\target\release\fanrgb.exe mode fire

# Turn off motherboard lighting / fans
.\target\release\fanrgb.exe off
```

---

### GPU Control

```powershell
# Check GPU controller detection
.\target\release\fanrgb.exe gpu status

# Set GPU static color
.\target\release\fanrgb.exe gpu "#A020F0"
.\target\release\fanrgb.exe gpu purple
.\target\release\fanrgb.exe gpu 160 32 240

# Set GPU animation modes
.\target\release\fanrgb.exe gpu mode rainbow
.\target\release\fanrgb.exe gpu mode color_cycle
.\target\release\fanrgb.exe gpu mode wave
.\target\release\fanrgb.exe gpu mode breathing
.\target\release\fanrgb.exe gpu mode gradient
.\target\release\fanrgb.exe gpu mode flash

# Turn off GPU lighting
.\target\release\fanrgb.exe gpu off
```

---

### Synchronized Control (Mobo + GPU)

Synchronize both the motherboard (and all connected fans) and the GPU to match simultaneously:

```powershell
# Sync to static purple
.\target\release\fanrgb.exe sync "#A020F0"
.\target\release\fanrgb.exe sync purple

# Sync to any custom hex or preset
.\target\release\fanrgb.exe sync "#00FFCC"
.\target\release\fanrgb.exe sync red

# Instant blackout (turn off both mobo and GPU)
.\target\release\fanrgb.exe sync off
```

---

### Real-Time Audio Sub-Bass Visualizer

Uses low-latency Windows WASAPI loopback capture with FFT sub-bass extraction to make fans and LEDs pulse dynamically to bass and kick drums:

```powershell
# Visualizer on motherboard fans (default blue)
.\target\release\fanrgb.exe bass

# Visualizer with custom color (e.g. purple, cyan, red)
.\target\release\fanrgb.exe bass purple
.\target\release\fanrgb.exe bass "#A020F0"

# Include GPU in real-time visualizer
.\target\release\fanrgb.exe bass purple --gpu

# Target specific audio endpoint (e.g. "Speakers", "Headphones")
.\target\release\fanrgb.exe bass purple --gpu --device "Speakers"

# Advanced audio tuning:
#   --fmin   Minimum sub-bass frequency in Hz (default: 1.0)
#   --fmax   Maximum sub-bass frequency in Hz (default: 25.0)
#   --min    Minimum idle brightness 0-255 (default: 0)
#   --decay  VU decay smoothing factor 0.0-1.0 (default: 0.82)
.\target\release\fanrgb.exe bass purple --gpu --fmin 20 --fmax 80 --min 10 --decay 0.85
```

---

### Custom JSON Effects & Profiles

Create and execute your own multi-step lighting animations and per-zone static setups using custom JSON files:

```powershell
# Play an animation effect (e.g. Cyberpunk Neon smooth fade)
.\target\release\fanrgb.exe effect cyberpunk

# High-speed alternating police strobe
.\target\release\fanrgb.exe effect police_strobe

# Multi-zone independent animation
.\target\release\fanrgb.exe effect zone_split

# Apply a custom static profile across all headers & GPU
.\target\release\fanrgb.exe effect static_profile

# List all available JSON effect files
.\target\release\fanrgb.exe effect list

# Generate a starter template ready for editing
.\target\release\fanrgb.exe effect init cyberpunk my_custom_effect.json
```

See [usage.md](usage.md) for full JSON schema specifications, supported zones (`j_rainbow_1`, `j_rainbow_2`, `j_rgb_1`, `on_board_led`, `gpu`), transition types, and examples.

---

## Animation Modes Reference

### Motherboard Modes

| Mode CLI Name | Mode Code | Description |
| :--- | :--- | :--- |
| `rainbow` / `rainbow_wave` | `25` (`0x19`) | Addressable rainbow wave across ARGB fans and headers |
| `rainbow_flashing` | `29` (`0x1D`) | Multi-color flashing effect |
| `static` | `1` (`0x01`) | Constant static solid color |
| `breathing` | `2` (`0x02`) | Smooth pulse / breath cycle |
| `flashing` | `3` (`0x03`) | Sharp on/off flashing |
| `double_flashing` | `4` (`0x04`) | Double strobe flash |
| `lightning` | `5` (`0x05`) | Electric storm / lightning strikes |
| `meteor` | `7` (`0x07`) | Shooting meteor trail |
| `color_pulse` | `21` (`0x15`) | Rhythmic color pulsation |
| `color_shift` | `22` (`0x16`) | Gradual color shifting across spectrum |
| `color_wave` | `23` (`0x17`) | Directional traveling color wave |
| `marquee` | `24` (`0x18`) | Chasing theater marquee lights |
| `visor` | `27` (`0x1B`) | Back-and-forth scanner / visor sweep |
| `stack` | `36` (`0x24`) | Sequential stacking LED animation |
| `fire` | `38` (`0x26`) | Realistic flickering fire / flame effect |
| `disable` / `off` | `0` (`0x00`) | LEDs completely turned off |

### GPU Modes

| Mode CLI Name | Register Code | Description |
| :--- | :--- | :--- |
| `rainbow` / `color_cycle` | `0x03` | Smooth RGB color spectrum cycle |
| `wave` / `rainbow_wave` | `0x07` | Dynamic traveling rainbow wave |
| `static` | `0x01` | Solid stationary color |
| `breathing` / `pulse` | `0x02` | Smooth breathing fade |
| `flash` / `flashing` | `0x04` | Single flash pulse |
| `double_flash` | `0x08` | Double flash strobe |
| `gradient` | `0x05` | Gradient color blend |

---

## Color Presets Reference

| Preset Name | Hex Equivalent | RGB Value |
| :--- | :--- | :--- |
| `purple` | `#A020F0` | `(160, 32, 240)` |
| `darkpurple` / `indigo` | `#4B0082` | `(75, 0, 130)` |
| `red` | `#FF0000` | `(255, 0, 0)` |
| `green` | `#00FF00` | `(0, 255, 0)` |
| `blue` | `#0000FF` | `(0, 0, 255)` |
| `cyan` | `#00FFFF` | `(0, 255, 255)` |
| `magenta` | `#FF00FF` | `(255, 0, 255)` |
| `yellow` | `#FFFF00` | `(255, 255, 0)` |
| `white` | `#FFFFFF` | `(255, 255, 255)` |
| `orange` | `#FF7800` | `(255, 120, 0)` |
| `darkblue` / `navy` | `#0014B4` | `(0, 20, 180)` |
| `lightblue` / `skyblue` | `#00B4FF` | `(0, 180, 255)` |
| `iceblue` | `#50C8FF` | `(80, 200, 255)` |
| `off` | `#000000` | `(0, 0, 0)` |

---

## Project Architecture & Directory Structure

The codebase is organized into small, decoupled, and single-responsibility modules:

```text
├── .github/workflows/
│   └── build.yml             # GitHub Actions CI/CD (Windows build & automated release)
├── effects/                  # Bundled custom JSON effect presets & profiles
│   ├── cyberpunk.json        # Smooth fade between neon cyan and purple
│   ├── police_strobe.json    # High-speed alternating red/blue emergency strobe
│   ├── amber_breath.json     # Slow organic amber breathing
│   ├── zone_split.json       # Independent animated zones per header and GPU
│   └── static_profile.json   # Multi-zone static color layout profile
├── src/
│   ├── main.rs               # Lightweight CLI entrypoint & argument dispatcher
│   ├── cli/                  # Command-line interface modules
│   │   ├── mod.rs            # CLI module entrypoint
│   │   ├── args.rs           # Color & argument parser (hex, presets, RGB)
│   │   ├── commands.rs       # Subcommand handlers (status, mode, gpu, sync, effect)
│   │   ├── bass.rs           # Real-time sub-bass visualizer launcher
│   │   └── help.rs           # Formatted CLI help and syntax reference
│   ├── effects/              # Custom JSON effect engine
│   │   ├── mod.rs            # Effects engine exports
│   │   ├── schema.rs         # JSON deserializer, color parser, keyframe interpolation
│   │   ├── player.rs         # High-frequency volatile streaming player
│   │   └── templates.rs      # Starter template generator (cyberpunk, police, etc.)
│   ├── controller/           # MSI Mystic Light Motherboard driver
│   │   ├── mod.rs            # Controller exports
│   │   ├── device.rs         # HID Feature Report 0x52 read/write & handshake
│   │   └── zone.rs           # Zone byte offset packing & unpacking
│   ├── gpu/                  # Gigabyte RGB Fusion 2.0 via NVAPI driver
│   │   ├── mod.rs            # GPU exports
│   │   ├── fusion.rs         # I2C probing (0x32, 0x62, etc.), color & mode registers
│   │   ├── nvapi.rs          # NVIDIA NVAPI 64-bit struct definitions
│   │   └── constants.rs      # Mode codes, speeds, and register addresses
│   ├── visualizer/           # WASAPI Audio DSP engine
│   │   ├── mod.rs            # Visualizer exports
│   │   ├── audio.rs          # WASAPI loopback capture & FFT sub-bass extractor
│   │   └── renderer.rs       # ASCII terminal meter & high-throughput HID streaming
│   └── config/               # Hardware constants & presets
│       ├── mod.rs            # Config exports
│       ├── hardware.rs       # USB VID/PID, zone offsets, report byte constants
│       └── presets.rs        # Named color RGB presets and animation mode mappings
├── usage.md                  # Comprehensive usage and configuration guide
└── Cargo.toml                # Package configuration (Rust Edition 2021, Release 2026)
```

---

## Hardware Safety & Architecture

* **Zero Flash Wear Policy**: Every HID feature report packet explicitly forces `packet[184] = 0x00` (`save_data = 0x00`). All settings reside strictly in the controller's volatile SRAM. No EEPROM/flash cycles are consumed, preventing motherboard firmware degradation.
* **Handshake Compliance**: Emulates the official MSI Mystic Light two-packet handshake (previous state followed by target state) with safe timing delimiters.
* **Safe GPU I2C Communications**: Communicates via official NVIDIA NVAPI `NvAPI_I2CWriteEx` / `NvAPI_I2CReadEx` interfaces with validated register bounds.

---

## Developer & Credits

* **Author / Developer:** [itznan](https://github.com/itznan)
* **GitHub Profile:** [https://github.com/itznan](https://github.com/itznan)
* **Repository:** [https://github.com/itznan/fanrgbmpgb550](https://github.com/itznan/fanrgbmpgb550)
* **Rust Language Edition:** `2021` | **Year / Release:** `2026`
* **License:** MIT / Apache-2.0

