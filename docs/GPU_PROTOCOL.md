# Gigabyte GPU RGB Fusion 2.0 Protocol & NVAPI Guide

This document details the reverse-engineered I2C communication protocol, packet layouts, register offsets, and Windows NVAPI integration for controlling the RGB lighting on the **Gigabyte GeForce RTX 3060 Ti GAMING OC (LHR Rev 2.0)** graphics card.

---

## 1. Hardware Specifications

| Parameter | Value | Details |
| :--- | :--- | :--- |
| **GPU Name** | `NVIDIA GeForce RTX 3060 Ti` | NVIDIA GA104-202 (Ampere) |
| **PCI Device ID** | `0x248910DE` | Device: `0x2489`, Vendor: `0x10DE` (NVIDIA) |
| **PCI Subsystem ID** | `0x405A1458` | Sub-Device: `0x405A`, Sub-Vendor: `0x1458` (Gigabyte) |
| **Microcontroller** | ITE / Holtek RGB MCU | Attached internally to GPU I2C Port 1 |
| **I2C Address** | `0x32` (Rev 2.0) | 7-bit slave address (`0x62` on Rev 1.0) |

---

## 2. Windows NVAPI I2C Architecture

On 64-bit Windows, direct hardware I2C access on NVIDIA graphics cards is mediated by `C:\Windows\System32\nvapi64.dll`.

### Function Interfaces
Functions are loaded dynamically through `nvapi_QueryInterface(hash_id)`:

* **`NvAPI_Initialize`** (`0x0150E828`): Initializes the NVAPI driver session.
* **`NvAPI_EnumPhysicalGPUs`** (`0xE5AC921F`): Enumerates physical GPU handles.
* **`NvAPI_GPU_GetFullName`** (`0xCEEE8E9F`): Queries human-readable product string.
* **`NvAPI_I2CWriteEx`** (`0x283AC65A`): Executes an I2C transaction write to a GPU port.
* **`NvAPI_I2CReadEx`** (`0x4D7B0709`): Executes an I2C transaction read from a GPU port.

### `NV_I2C_INFO_V3` Struct Definition

```python
class NV_I2C_INFO_V3(ctypes.Structure):
    _fields_ = [
        ("version", ctypes.c_uint32),         # (3 << 16) | sizeof(NV_I2C_INFO_V3) = 0x00030040
        ("display_mask", ctypes.c_uint32),    # 0 (Internal GPU communication port)
        ("is_ddc_port", ctypes.c_uint8),      # 0 (Non-DDC communication)
        ("i2c_dev_address", ctypes.c_uint8),  # 7-bit address shifted left: (0x32 << 1) = 0x64
        ("i2c_reg_address", ctypes.POINTER(ctypes.c_uint8)), # NULL (command bytes inside data)
        ("reg_addr_size", ctypes.c_uint32),   # 0
        ("data", ctypes.POINTER(ctypes.c_uint8)),            # Pointer to byte buffer
        ("size", ctypes.c_uint32),            # Packet size (8 bytes for write, 4 bytes for read)
        ("i2c_speed", ctypes.c_uint32),       # 0xFFFF
        ("i2c_speed_khz", ctypes.c_uint32),   # 0 (NVAPI_I2C_SPEED_DEFAULT)
        ("port_id", ctypes.c_uint8),          # 1 (GPU RGB Controller Port)
        ("is_port_id_set", ctypes.c_uint32),  # 1
    ]
```

---

## 3. Handshake & Chip Verification

To guarantee bus safety and avoid interfering with voltage regulators or VRM sensors sharing the bus, the controller must be verified using the OpenRGB `0xAB` probe handshake:

1. **Probe Write**: Send an 8-byte buffer to `0x32`:
   ```text
   [0xAB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
   ```
2. **Read Response**: Read 4 bytes back from `0x32`:
   ```text
   [0xAB, 0x10, 0x00, 0x01]
   ```
3. If byte 0 equals `0xAB`, the RGB microcontroller presence is confirmed.

---

## 4. Command Packet Formats

All commands are structured as **8-byte I2C block writes**.

### A. Mode & Brightness Command (Register `0x88`)
```text
Byte 0: 0x88           (REG_MODE)
Byte 1: Mode ID        (0x01 = Static, 0x02 = Breathing, 0x03 = Color Cycle, 0x04 = Flash, 0x07 = Wave)
Byte 2: Speed          (0x00 = Slowest, 0x02 = Normal, 0x05 = Fastest)
Byte 3: Brightness     (0x00 = 0% to 0x63 = 100% [99 decimal])
Byte 4: Mystery Flag   (0x00 for static/breathing, 0x08 for gradient/tricolor)
Byte 5: Zone + 1       (0x01 for Zone 0 / side logo)
Byte 6: 0x00           (Padding)
Byte 7: 0x00           (Padding)
```

### B. Color Command (Register `0xB0`)
Sets RGB color for Zone 0 (Side "GIGABYTE" illuminated logo):
```text
Byte 0: 0xB0           (REG_COLOR_LEFT_MID)
Byte 1: Mode ID        (Matches the active mode, e.g. 0x01 for static)
Byte 2: Red (Zone 0)   (0 - 255)
Byte 3: Green (Zone 0) (0 - 255)
Byte 4: Blue (Zone 0)  (0 - 255)
Byte 5: Red (Zone 1)   (0 - 255)
Byte 6: Green (Zone 1) (0 - 255)
Byte 7: Blue (Zone 1)  (0 - 255)
```

### C. EEPROM Permanent Save Command
To persist the configuration into the microcontroller flash across PC reboots:
```text
Byte 0..7: [0xAA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

---

## 5. Performance & Streaming Latency

* **Average I2C Write Latency**: **~0.77 ms** per 8-byte transfer via NVAPI.
* Because the I2C bus is so fast, the GPU logo can stream real-time visualizer color pulses alongside motherboard case fans at 32–60+ FPS with zero latency or frame drops.

---

## 6. CLI Usage Reference

### Check GPU Connection & Status
```powershell
python cli.py gpu status
```

### Set GPU Colors & Presets
```powershell
python cli.py gpu red
python cli.py gpu blue
python cli.py gpu 255 0 0
python cli.py gpu off
```

### GPU Hardware Animation Modes
```powershell
python cli.py gpu mode breathing
python cli.py gpu mode flash
python cli.py gpu mode color_cycle
python cli.py gpu mode wave
```

### Synchronize Motherboard + GPU Together
```powershell
# Set both Motherboard and GPU to Pure Red
python cli.py sync red

# Turn off all RGB across both devices
python cli.py sync off

# Run Pure Red Bass Visualizer pulsing BOTH Motherboard and GPU logo
python cli.py bass --gpu
```
