# MSI MPG B550 GAMING PLUS RGB Controller

A lightweight, safe, and fast Python RGB controller and real-time audio visualizer for the **MSI MPG B550 GAMING PLUS (MS-7C56)** motherboard.

Built strictly on OpenRGB's reverse-engineered 185-byte HID protocol with **zero flash wear** (`save_data = 0x00` RAM-only mode).

---

## Features

* 🎧 **Pure Red Bass & Kick-Drum Reactive Lighting**: Real-time WASAPI audio loopback from headphones/speakers with dynamic beat onset detection and noise gating. 100% pure red (zero orange, zero amber).
* 🌈 **Hardware Lighting Animations**: Rainbow Wave, Breathing, Meteor, Flashing, Marquee, Fire, etc.
* 🎨 **Static Color & Custom RGB**: Instant color presets and fine-grained RGB channel values.
* ⚡ **Zero Flash Wear Guarantee**: Strictly volatile RAM feature reports (`byte 184 = 0x00`) to protect onboard EEPROM.
* 💻 **CLI & Batch Launchers**: Seamless control via terminal commands and double-click batch files.

---

## Documentation

Comprehensive guides are available in the [`docs/`](file:///E:/NAN/Github/fanrgbmpgb550/docs/) directory:
* [**System Architecture**](file:///E:/NAN/Github/fanrgbmpgb550/docs/ARCHITECTURE.md) - System design, data flow, and threading model.
* [**Hardware & USB Protocol**](file:///E:/NAN/Github/fanrgbmpgb550/docs/HARDWARE_PROTOCOL.md) - 185-byte Feature Report 0x52 specification, memory offsets, and handshake timing.
* [**Audio Visualizer & DSP Engine**](file:///E:/NAN/Github/fanrgbmpgb550/docs/AUDIO_VISUALIZER.md) - 28–90 Hz frequency filtering, spectral flux, AGC, and tuning.
* [**CLI User Guide**](file:///E:/NAN/Github/fanrgbmpgb550/docs/CLI_GUIDE.md) - Complete command-line reference, options, and troubleshooting.

---

## Folder Structure

```text
fanrgbmpgb550/
├── docs/                      # Technical documentation
│   ├── ARCHITECTURE.md        # System design & data flow
│   ├── HARDWARE_PROTOCOL.md   # MSI 185-byte HID packet spec
│   ├── AUDIO_VISUALIZER.md    # Audio DSP & beat detection
│   └── CLI_GUIDE.md           # User guide for CLI & batch runners
├── src/                       # Core application source code
│   ├── config.py              # Constants, zone offsets & presets
│   ├── controller.py          # Core USB HID driver & 185-byte packet handshake
│   └── visualizer.py          # Real-time audio capture, FFT DSP & pure red engine
├── tests/                     # Hardware diagnostics
│   ├── run_test.py            # Diagnostic & board verification test
│   ├── test_enum.py           # USB device discovery & VID/PID check
│   └── test_read.py           # Low-level HID Feature Report 0x52 read test
├── cli.py                     # Unified Command-Line Interface
├── bass_reactive.py           # Quick CLI bass visualizer runner
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
# Check active motherboard lighting status
python cli.py status

# Start Pure Red Bass-Reactive mode in terminal
python cli.py bass

# Set solid color presets
python cli.py red
python cli.py blue
python cli.py off

# Set custom RGB values
python cli.py 255 0 0

# Set hardware animation modes
python cli.py mode rainbow_wave
python cli.py mode breathing
python cli.py mode meteor
```

### 3. Run Hardware Diagnostics

```powershell
python tests/run_test.py
```

---

## Safety Guarantee

MSI Mystic Light microcontrollers store permanent state in flash memory. Writing to flash continuously will degrade or corrupt the chip.

This controller strictly enforces:
```python
packet[184] = 0x00  # Volatile RAM updates only (Zero Flash Wear)
```
You can stream real-time visualizers 24/7 without degrading the motherboard hardware.
