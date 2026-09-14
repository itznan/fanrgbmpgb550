# MSI MPG B550 & Gigabyte GPU RGB Controller (Rust Native Edition)

Ultra-fast, lightweight native Rust GUI & CLI lighting controller for:
1. **MSI MPG B550 GAMING PLUS (MS-7C56)** Motherboard (`USB HID Feature Report 0x52`)
2. **Gigabyte GeForce RTX 3060 Ti GAMING OC** Graphics Card (`NVIDIA NVAPI I2C 0x32/0xAB Probe`)

---

## Table of Contents
- [Key Features](#key-features)
- [Building from Source](#building-from-source)
- [Graphical User Interface (GUI)](#graphical-user-interface-gui)
- [Command-Line Usage (CLI)](#command-line-usage-cli)
  - [Hardware Status](#hardware-status)
  - [Color Formats & PowerShell Quoting](#color-formats--powershell-quoting)
  - [Motherboard & Fans Control](#motherboard--fans-control)
  - [GPU Control](#gpu-control)
  - [Synchronized Control (Mobo + GPU)](#synchronized-control-mobo--gpu)
  - [Real-Time Audio Sub-Bass Visualizer](#real-time-audio-sub-bass-visualizer)
- [Animation Modes Reference](#animation-modes-reference)
  - [Motherboard Modes](#motherboard-modes)
  - [GPU Modes](#gpu-modes)
- [Color Presets Reference](#color-presets-reference)
- [Hardware Safety & Architecture](#hardware-safety--architecture)

---

## Key Features
* **Professional Single-Page GUI**: Native `egui`/`eframe` dashboard with full hardware controls, live preview, and quick swatches on one clean page.
* **Minimal System Footprint**: Pure Rust (~18 MB RAM), 0% idle CPU overhead, zero Webview or Electron bloat.
* **Deterministic Control**: Direct, low-level hardware control without background bloatware or unrequested transitions.
* **Zero Flash Wear Guarantee**: Volatile RAM updates (`save_data = 0x00`) strictly protect motherboard EEPROM from wear.
* **Real-Time WASAPI Loopback Sub-Bass Visualizer**: Integrated in GUI with animated live VU meter and terminal ASCII meter in CLI.
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

## Graphical User Interface (GUI)

Launch the dashboard by running without arguments or passing `gui`:

```powershell
# Launch GUI
.\target\release\fanrgb.exe

# Or explicitly
.\target\release\fanrgb.exe gui
```

> Double-clicking `fanrgb.exe` in Windows Explorer also opens the GUI automatically.

### GUI Features
* **Per-Zone Selection**: Target `All Zones`, `J_RAINBOW_1`, `J_RAINBOW_2` (ARGB headers), `J_RGB_1` (12V RGB), or `Onboard LEDs`.
* **Live Interactive Color Picker**: Full HSV/RGB picker with instant preview.
* **Quick Color Swatches**: One-click presets (Red, Green, Blue, Purple, Orange, Cyan, White, Blackout).
* **Speed & Brightness Sliders**: Intuitive hardware speed (Slow, Normal, Fast) and brightness (0–100%).
* **Integrated Sub-Bass Visualizer**: Real-time VU meter showing live audio reactivity.

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
src/
├── main.rs                   # Lightweight entrypoint (routes to GUI or CLI)
├── cli/                      # Command-line interface modules
│   ├── mod.rs                # CLI module entrypoint
│   ├── args.rs               # Color & argument parser (hex, presets, RGB)
│   ├── commands.rs           # Subcommand handlers (status, mode, gpu, sync)
│   ├── bass.rs               # Real-time sub-bass visualizer launcher
│   └── help.rs               # Formatted CLI help and syntax reference
├── gui/                      # Graphical user interface (egui / eframe)
│   ├── mod.rs                # GUI entrypoint and native window configuration
│   ├── app.rs                # eframe::App lifecycle and update loop
│   ├── state.rs              # GuiApp state, hardware handles, audio device list
│   ├── actions.rs            # Hardware operations (apply, sync, off, reconnect)
│   ├── theme.rs              # Dark theme styling, visuals, and card containers
│   ├── zone.rs               # MoboZone enum and label/key mappings
│   └── panels/               # Modular UI component cards
│       ├── mod.rs            # Panels module exports
│       ├── header.rs         # Top bar, status pills, and global sync buttons
│       ├── palette.rs        # Quick color swatch presets & master picker
│       ├── mobo.rs           # Motherboard zones, animation modes, speed/brightness
│       ├── gpu.rs            # GPU NVAPI status, mode selector, sliders
│       ├── visualizer.rs     # WASAPI audio capture, live VU meter, frequency filters
│       └── footer.rs         # Status feedback bar and engine footprint info
├── controller/               # MSI Mystic Light Motherboard driver
│   ├── mod.rs                # Controller exports
│   ├── device.rs             # HID Feature Report 0x52 read/write & handshake
│   └── zone.rs               # Zone byte offset packing & unpacking
├── gpu/                      # Gigabyte RGB Fusion 2.0 via NVAPI driver
│   ├── mod.rs                # GPU exports
│   ├── fusion.rs             # I2C probing (0x32, 0x62, etc.), color & mode registers
│   ├── nvapi.rs              # NVIDIA NVAPI 64-bit struct definitions
│   └── constants.rs          # Mode codes, speeds, and register addresses
├── visualizer/               # WASAPI Audio DSP engine
│   ├── mod.rs                # Visualizer exports
│   ├── audio.rs              # WASAPI loopback capture & FFT sub-bass extractor
│   └── renderer.rs           # ASCII terminal meter & high-throughput HID streaming
└── config/                   # Hardware constants & presets
    ├── mod.rs                # Config exports
    ├── hardware.rs           # USB VID/PID, zone offsets, report byte constants
    └── presets.rs            # Named color RGB presets and animation mode mappings
```

---

## Hardware Safety & Architecture

* **Zero Flash Wear Policy**: Every HID feature report packet explicitly forces `packet[184] = 0x00` (`save_data = 0x00`). All settings reside strictly in the controller's volatile SRAM. No EEPROM/flash cycles are consumed, preventing motherboard firmware degradation.
* **Handshake Compliance**: Emulates the official MSI Mystic Light two-packet handshake (previous state followed by target state) with safe timing delimiters.
* **Safe GPU I2C Communications**: Communicates via official NVIDIA NVAPI `NvAPI_I2CWriteEx` / `NvAPI_I2CReadEx` interfaces with validated register bounds.

