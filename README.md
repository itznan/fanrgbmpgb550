# MSI MPG B550 & Gigabyte GPU RGB Controller (Rust + Tauri Edition)

Ultra-fast, lightweight Rust CLI controller and modern Tauri v2 Desktop GUI for:
1. **MSI MPG B550 GAMING PLUS (MS-7C56)** Motherboard (USB HID Feature Report 0x52)
2. **Gigabyte GeForce RTX 3060 Ti GAMING OC** Graphics Card (NVIDIA NVAPI I2C 0xAB Probe)

Features:
* **Dual Execution Mode**: Double-click `fanrgb.exe` to launch the **Tauri GUI**, or pass terminal arguments (`fanrgb sync red`) for instant **CLI mode**.
* **Modern Dark GUI Dashboard**: Control Motherboard & GPU lighting, presets, animation modes, and audio visualizer with a sleek dashboard.
* **Ultra-low Resource Footprint**: ~5 MB RAM footprint in CLI mode and minimal CPU/RAM overhead in GUI mode.
* **Zero Flash Wear Guarantee**: Volatile RAM updates (`save_data = 0x00`) to protect motherboard EEPROM.
* **WASAPI Loopback Pure Red Bass Visualizer**: Real-time 28-90 Hz sub-bass audio reactive lighting.

---

## Building from Source

```powershell
cargo build --release
```

The compiled binary will be generated at `target/release/fanrgb.exe`.

---

## Launching the Application

### 1. Modern Desktop GUI (Tauri)
Simply double-click `fanrgb.exe` or run without arguments:
```powershell
.\target\release\fanrgb.exe
```

### 2. Fast Command-Line Interface (CLI)
```powershell
# Check hardware status
.\target\release\fanrgb.exe status
.\target\release\fanrgb.exe gpu status

# Synchronize system lighting to Red
.\target\release\fanrgb.exe sync red

# Start Pure Red Bass Visualizer synced across Motherboard & GPU
.\target\release\fanrgb.exe bass --gpu

# Power off all RGB
.\target\release\fanrgb.exe sync off
```
