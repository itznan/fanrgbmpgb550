# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-27

### Added
- Native USB HID controller (`0x1462:0x7C56`) for MSI MPG B550 GAMING PLUS with volatile-only (`save_data = 0x00`) RAM updates.
- Gigabyte GPU RGB Fusion 2.0 support via NVIDIA NVAPI I2C probing (including multi-GPU scan).
- Custom JSON lighting effects engine (`effects/`) with static profiles, interpolated keyframes, templates, and Ctrl+C state restoration.
- Real-time WASAPI loopback FFT sub-bass audio visualizer (`fanrgb bass`) with idle silence decay.
- Cross-platform Linux HID build support, udev rules (`udev/99-msi-rgb.rules`), `--version`, and unit/mock integration tests.
