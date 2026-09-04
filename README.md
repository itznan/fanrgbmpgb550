# MSI MPG B550 & Gigabyte GPU RGB Controller (Rust Edition)

Ultra-fast, lightweight Rust CLI controller and real-time audio visualizer for:
1. **MSI MPG B550 GAMING PLUS (MS-7C56)** Motherboard (USB HID Feature Report 0x52)
2. **Gigabyte GeForce RTX 3060 Ti GAMING OC** Graphics Card (NVIDIA NVAPI I2C 0xAB Probe)

Re-engineered from Python to Rust for performance & minimal resource usage:
* **Ultra-low ~5 MB RAM footprint** (vs ~80 MB in Python)
* **Instant CLI execution (< 5 ms)**
* **Zero Flash Wear Guarantee** (`save_data = 0x00` RAM volatile updates)
* **WASAPI Loopback Pure Red Bass Visualizer**
* **Single standalone `.exe` binary** with zero runtime dependencies

---

## Building from Source

```powershell
cargo build --release
```

The compiled binary will be generated at `target/release/fanrgb.exe`.

---

## Usage Examples

### Motherboard Control
```powershell
# Check motherboard status
.\target\release\fanrgb.exe status

# Set static color
.\target\release\fanrgb.exe red
.\target\release\fanrgb.exe #FF0000
.\target\release\fanrgb.exe 255 0 0

# Set hardware animation effect mode
.\target\release\fanrgb.exe mode rainbow_wave
.\target\release\fanrgb.exe mode breathing 255 0 0
```

### GPU Control (via NVAPI)
```powershell
# Check GPU NVAPI connection and I2C address
.\target\release\fanrgb.exe gpu status

# Set GPU static color
.\target\release\fanrgb.exe gpu red
.\target\release\fanrgb.exe gpu #FF0000

# Set GPU animation mode
.\target\release\fanrgb.exe gpu mode breathing
```

### Synchronized Control (Motherboard + GPU)
```powershell
# Sync both devices to pure red
.\target\release\fanrgb.exe sync red

# Power down all RGB across the system
.\target\release\fanrgb.exe sync off

# Start real-time Pure Red Bass Visualizer (synced to Motherboard & GPU)
.\target\release\fanrgb.exe bass --gpu
```
