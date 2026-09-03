# MSI MPG B550 Motherboard & Gigabyte GPU RGB Controller

A lightweight, safe, and ultra-fast Python RGB controller and real-time audio visualizer for:
1. **MSI MPG B550 GAMING PLUS (MS-7C56)** Motherboard
2. **Gigabyte GeForce RTX 3060 Ti GAMING OC** Graphics Card

Built strictly on reverse-engineered USB HID & NVAPI I2C protocols with **zero flash wear** (`save_data = 0x00` RAM-only updates on motherboard, safe I2C block writes on GPU).

---

## Features

* 🎧 **Pure Red Bass & Kick-Drum Reactive Lighting**: Real-time WASAPI audio loopback from headphones/speakers with dynamic beat onset detection and noise gating. 100% pure red (zero orange, zero amber).
* 🔄 **Synchronized PC Lighting**: Sync your Motherboard fans and GPU side illuminated logo simultaneously (`python cli.py sync red` or `python cli.py bass --gpu`).
* ⚡ **GPU RGB Fusion 2.0 via NVAPI**: Native NVIDIA driver I2C communication (~0.77 ms latency) with 0xAB chip verification.
* 🌈 **Hardware Lighting Animations**: Rainbow Wave, Breathing, Meteor, Flashing, Marquee, Fire, Wave, etc.
* 🎨 **Static Color & Custom RGB**: Instant presets and fine-grained RGB channel values.
* ⚡ **Zero Flash Wear Guarantee**: Volatile RAM updates only to protect motherboard EEPROM.
* 💻 **CLI & Batch Launchers**: Seamless control via terminal commands and double-click batch files.

---

## Documentation

Comprehensive guides are available in the [`docs/`](file:///E:/NAN/Github/fanrgbmpgb550/docs/) directory:
* [**System Architecture**](file:///E:/NAN/Github/fanrgbmpgb550/docs/ARCHITECTURE.md) - System design, data flow, and threading model.
* [**Hardware & USB Protocol**](file:///E:/NAN/Github/fanrgbmpgb550/docs/HARDWARE_PROTOCOL.md) - Motherboard 185-byte Feature Report 0x52 specification and handshake.
* [**GPU RGB Protocol & NVAPI**](file:///E:/NAN/Github/fanrgbmpgb550/docs/GPU_PROTOCOL.md) - Gigabyte RGB Fusion 2.0 I2C registers, NVAPI integration, and 0xAB verification.
* [**Audio Visualizer & DSP Engine**](file:///E:/NAN/Github/fanrgbmpgb550/docs/AUDIO_VISUALIZER.md) - 28–90 Hz frequency filtering, spectral flux, AGC, and tuning.
* [**CLI User Guide**](file:///E:/NAN/Github/fanrgbmpgb550/docs/CLI_GUIDE.md) - Complete command-line reference, options, and troubleshooting.

---

## Folder Structure

```text
fanrgbmpgb550/
├── docs/                      # Technical documentation
│   ├── ARCHITECTURE.md        # System design & data flow
│   ├── HARDWARE_PROTOCOL.md   # MSI motherboard HID packet spec
│   ├── GPU_PROTOCOL.md        # Gigabyte GPU NVAPI I2C protocol
│   ├── AUDIO_VISUALIZER.md    # Audio DSP & beat detection
│   └── CLI_GUIDE.md           # User guide for CLI & batch runners
├── src/                       # Core application source code
│   ├── config.py              # Constants, zone offsets & presets
│   ├── controller.py          # Motherboard USB HID driver
│   ├── gpu_controller.py      # Gigabyte GPU NVAPI I2C driver
│   └── visualizer.py          # Real-time audio capture, FFT DSP & synchronized engine
├── tests/                     # Hardware diagnostics
│   ├── run_test.py            # Motherboard detection & verification test
│   ├── test_gpu.py            # GPU NVAPI I2C probe & signature test
│   ├── test_enum.py           # USB device discovery
│   └── test_read.py           # Low-level HID Feature Report 0x52 test
├── cli.py                     # Unified Command-Line Interface
├── bass_reactive.py           # Standalone CLI bass visualizer runner
├── start_bass_reactive.bat    # Windows one-click desktop runner
├── msi_b550.py                # Backward-compatibility module
├── requirements.txt           # Python dependencies
└── README.md                  # Project overview
```

---

## Quick Start

### 1. One-Click Desktop Launcher (.bat)

* **Start Pure Red Bass-Reactive mode**: Double-click [`start_bass_reactive.bat`](file:///E:/NAN/Github/fanrgbmpgb550/start_bass_reactive.bat)

---

### 2. Run from the Command Line (CLI)

```powershell
# Check motherboard and GPU status
python cli.py status
python cli.py gpu status

# Synchronize Motherboard + GPU to Pure Red
python cli.py sync red

# Start Pure Red Bass Visualizer synced across Motherboard AND GPU logo
python cli.py bass --gpu

# Turn off all RGB across both devices
python cli.py sync off

# Set GPU animation modes
python cli.py gpu mode breathing
python cli.py gpu mode color_cycle
```

### 3. Run Hardware Diagnostics

```powershell
# Motherboard diagnostic
python tests/run_test.py

# GPU diagnostic
python tests/test_gpu.py
```

---

## Safety Guarantee

* **Motherboard**: Volatile RAM updates only (`packet[184] = 0x00`).
* **GPU**: Safety probe `0xAB` verification before issuing any I2C write commands to prevent touching VRM or sensor addresses.
