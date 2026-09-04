"""
Standalone zone-level read/write helpers for the MSI MPG B550 feature report packet.
"""

from typing import Dict, Any

from src.config import ZONE_OFFSETS, SYNC_SETTING_ONBOARD, SPEED_MEDIUM, BRIGHTNESS_100


def get_zone_info(packet: bytearray, zone_name: str) -> Dict[str, Any]:
    """Reads zone state from the 185-byte feature report packet."""
    offset = ZONE_OFFSETS[zone_name]
    effect = packet[offset]
    r1 = packet[offset + 1]
    g1 = packet[offset + 2]
    b1 = packet[offset + 3]
    flags = packet[offset + 4]
    speed = flags & 0x03
    brightness = (flags >> 2) & 0x1F
    r2 = packet[offset + 5]
    g2 = packet[offset + 6]
    b2 = packet[offset + 7]
    color_flags = packet[offset + 8]
    custom_color = bool(color_flags & 0x80)

    info = {
        "zone": zone_name,
        "effect": effect,
        "primary_rgb": (r1, g1, b1),
        "secondary_rgb": (r2, g2, b2),
        "speed": speed,
        "brightness": brightness,
        "custom_color": custom_color,
        "raw_flags": hex(flags),
        "raw_color_flags": hex(color_flags),
    }
    if zone_name in ("j_rainbow_1", "j_rainbow_2"):
        info["led_count_or_cycle"] = packet[offset + 10]
    return info


def set_zone_data(
    packet: bytearray,
    zone_name: str,
    mode: int,
    r: int,
    g: int,
    b: int,
    speed: int = SPEED_MEDIUM,
    brightness: int = BRIGHTNESS_100,
    led_count: int = 100,
) -> None:
    """Writes zone state into the 185-byte feature report packet."""
    offset = ZONE_OFFSETS[zone_name]
    packet[offset] = mode & 0xFF
    packet[offset + 1] = r & 0xFF
    packet[offset + 2] = g & 0xFF
    packet[offset + 3] = b & 0xFF
    packet[offset + 4] = ((brightness & 0x1F) << 2) | (speed & 0x03)
    packet[offset + 5] = r & 0xFF
    packet[offset + 6] = g & 0xFF
    packet[offset + 7] = b & 0xFF

    # Color flags: 0x80 indicates custom RGB color enabled
    # Onboard master zone also requires SYNC_SETTING_ONBOARD (0x01)
    if zone_name == "on_board_led":
        packet[offset + 8] = 0x80 | SYNC_SETTING_ONBOARD
    else:
        packet[offset + 8] = 0x80

    packet[offset + 9] = 0x00

    # Rainbow headers have the 11th byte: led_count / cycle
    if zone_name in ("j_rainbow_1", "j_rainbow_2"):
        packet[offset + 10] = led_count & 0xFF


__all__ = ["get_zone_info", "set_zone_data"]
