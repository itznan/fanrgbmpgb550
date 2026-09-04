# MSI MPG B550 & Gigabyte GPU RGB Controller (Rust CLI Edition)

Ultra-fast, lightweight Rust CLI controller for:
1. **MSI MPG B550 GAMING PLUS (MS-7C56)** Motherboard (USB HID Feature Report 0x52)
2. **Gigabyte GeForce RTX 3060 Ti GAMING OC** Graphics Card (NVIDIA NVAPI I2C 0xAB Probe)

## Key Features
* **Pure Fast CLI**: Instant execution with ~5 MB RAM footprint and zero webview/GUI overhead.
* **Zero Flash Wear Guarantee**: Volatile RAM updates (`save_data = 0x00`) protect motherboard EEPROM from wear.
* **WASAPI Loopback Pure Red Bass Visualizer**: Real-time 28–90 Hz sub-bass reactive lighting with terminal ASCII meter.
* **Synchronized Control**: Easily match colors and effects across motherboard and graphics card with a single command.

---

## Building from Source

```powershell
cargo build --release
```

The compiled binary will be generated at `target/release/fanrgb.exe`.

---

## Command-Line Usage

```powershell
# Display help and available commands
.\target\release\fanrgb.exe help

# Check hardware status
.\target\release\fanrgb.exe status
.\target\release\fanrgb.exe gpu status

# Set Motherboard color
.\target\release\fanrgb.exe red
.\target\release\fanrgb.exe #FF5500
.\target\release\fanrgb.exe 255 80 0
.\target\release\fanrgb.exe mode breathing 255 0 0
.\target\release\fanrgb.exe off

# Set GPU color & modes
.\target\release\fanrgb.exe gpu red
.\target\release\fanrgb.exe gpu #00FFCC
.\target\release\fanrgb.exe gpu mode breathing
.\target\release\fanrgb.exe gpu off

# Synchronize Motherboard + GPU
.\target\release\fanrgb.exe sync red
.\target\release\fanrgb.exe sync #00FF00
.\target\release\fanrgb.exe sync off

# Real-Time Pure Red Sub-Bass Visualizer (WASAPI Loopback)
.\target\release\fanrgb.exe bass
.\target\release\fanrgb.exe bass --gpu
```
