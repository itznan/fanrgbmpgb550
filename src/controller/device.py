"""
MSI MPG B550 GAMING PLUS (MS-7C56) Hardware Controller.
Based strictly on OpenRGB's MSIMotherboard185Controller implementation.

SAFETY RULES:
- Never write save_data = 0x01 (always 0x00 for RAM-only volatile updates).
- Feature report size must be exactly 185 bytes (Report ID 0x52 + 184 bytes payload).
"""

import hid
import time
from typing import Dict, Any, Tuple

from src.config import (
    MSI_USB_VID,
    MSI_USB_PID,
    ZONE_OFFSETS,
    SAVE_OFFSET,
    MODE_STATIC,
    SPEED_MEDIUM,
    BRIGHTNESS_100,
    SYNC_SETTING_ONBOARD,
)
from src.controller.zone import get_zone_info as _get_zone_info, set_zone_data as _set_zone_data


class MSIMysticLightB550:
    def __init__(self):
        self.dev = None
        self._stream_cached_packet = None

    def open(self):
        self.dev = hid.device()
        self.dev.open(MSI_USB_VID, MSI_USB_PID)

    def close(self):
        if self.dev:
            try:
                self.dev.close()
            except Exception:
                pass
            self.dev = None

    def __enter__(self):
        self.open()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    def read_firmware_versions(self) -> Tuple[str, str]:
        # APROM (0xB0)
        req = bytearray(64)
        req[0] = 0x01
        req[1] = 0xB0
        req[2:] = b"\xCC" * 62
        self.dev.write(req)
        resp_ap = self.dev.read(64, timeout_ms=500)
        ap_ver = f"{resp_ap[2] >> 4}.{resp_ap[2] & 0x0F}" if len(resp_ap) >= 3 else "Unknown"

        # LDROM (0xB6)
        req[1] = 0xB6
        self.dev.write(req)
        resp_ld = self.dev.read(64, timeout_ms=500)
        ld_ver = f"{resp_ld[2] >> 4}.{resp_ld[2] & 0x0F}" if len(resp_ld) >= 3 else "Unknown"

        return ap_ver, ld_ver

    def read_packet(self) -> bytearray:
        """Reads the current 185-byte feature report."""
        feat = self.dev.get_feature_report(0x52, 185)
        if len(feat) != 185:
            raise ValueError(f"Expected 185 bytes from HID get_feature_report, got {len(feat)}")
        return bytearray(feat)

    def get_zone_info(self, packet: bytearray, zone_name: str) -> Dict[str, Any]:
        return _get_zone_info(packet, zone_name)

    def set_zone_data(
        self,
        packet: bytearray,
        zone_name: str,
        mode: int,
        r: int,
        g: int,
        b: int,
        speed: int = SPEED_MEDIUM,
        brightness: int = BRIGHTNESS_100,
        led_count: int = 100,
    ):
        _set_zone_data(packet, zone_name, mode, r, g, b, speed, brightness, led_count)

    def update_hardware(self, target_packet: bytearray) -> bool:
        """
        Sends the standard two-packet update handshake to the device.
        STRICT SAFETY: save_data byte 184 is forced to 0x00 (RAM only).
        """
        if len(target_packet) != 185:
            raise ValueError(f"Packet must be exactly 185 bytes, got {len(target_packet)}")

        # ALWAYS enforce volatile RAM-only flag (no flash writes)
        target_packet[SAVE_OFFSET] = 0x00

        # Step 1: Read current state from board
        old_state = self.read_packet()
        old_state[SAVE_OFFSET] = 0x00

        # Step 2: Send old state first (Mystic Light transition handshake)
        res1 = self.dev.send_feature_report(old_state)
        time.sleep(0.02)  # 20ms pause

        # Step 3: Send target state
        res2 = self.dev.send_feature_report(target_packet)
        return res1 == 185 and res2 == 185

    def stream_update(self, target_packet: bytearray, pause_sec: float = 0.012) -> bool:
        """
        High-throughput streaming update for real-time visualizers.
        Maintains the two-packet handshake without an expensive read_packet() on every frame.
        STRICT SAFETY: save_data byte 184 is forced to 0x00 (RAM only).
        """
        if len(target_packet) != 185:
            raise ValueError(f"Packet must be exactly 185 bytes, got {len(target_packet)}")

        target_packet[SAVE_OFFSET] = 0x00

        if self._stream_cached_packet is None:
            self._stream_cached_packet = self.read_packet()
            self._stream_cached_packet[SAVE_OFFSET] = 0x00

        # Step 1: Send cached previous state
        res1 = self.dev.send_feature_report(self._stream_cached_packet)
        if pause_sec > 0:
            time.sleep(pause_sec)

        # Step 2: Send target state
        res2 = self.dev.send_feature_report(target_packet)

        # Step 3: Cache target state for next frame
        self._stream_cached_packet[:] = target_packet
        return res1 == 185 and res2 == 185

    def apply_color_to_all(self, r: int, g: int, b: int, mode: int = MODE_STATIC) -> bool:
        """
        Applies an RGB color to JRGB1, JRAINBOW1, JRAINBOW2, and ONBOARD LEDs.
        """
        packet = self.read_packet()

        # Update JRGB1 (12V 4-pin header)
        self.set_zone_data(packet, "j_rgb_1", mode, r, g, b)

        # Update JRAINBOW1 (5V 3-pin ARGB header - up to 120 LEDs)
        self.set_zone_data(packet, "j_rainbow_1", mode, r, g, b, led_count=100)

        # Update JRAINBOW2 (5V 3-pin ARGB header - up to 120 LEDs)
        self.set_zone_data(packet, "j_rainbow_2", mode, r, g, b, led_count=100)

        # Update Master ONBOARD
        self.set_zone_data(packet, "on_board_led", mode, r, g, b)

        # Update individual ONBOARD LEDs 1..6
        for i in range(1, 7):
            zone_key = f"on_board_led_{i}"
            self.set_zone_data(packet, zone_key, mode, r, g, b)

        return self.update_hardware(packet)
