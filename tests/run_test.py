"""
Diagnostic and detection test script for MSI MPG B550 GAMING PLUS.
"""

import sys
import os

# Allow running directly from tests/ or project root
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from src.controller import MSIMysticLightB550
from src.config import MODE_STATIC


def inspect_current_state():
    print("=" * 50)
    print(" MSI MPG B550 GAMING PLUS (MS-7C56) Detection Test")
    print("=" * 50)
    with MSIMysticLightB550() as controller:
        ap_ver, ld_ver = controller.read_firmware_versions()
        serial = controller.dev.get_serial_number_string()
        mfg = controller.dev.get_manufacturer_string()
        prod = controller.dev.get_product_string()

        print(f"Device   : {mfg} {prod}")
        print(f"Serial   : {serial}")
        print(f"Firmware : APROM v{ap_ver} | LDROM v{ld_ver}")
        print("\nReading active configuration (185-byte Feature Report 0x52)...")

        packet = controller.read_packet()
        for zone in ["j_rgb_1", "j_rainbow_1", "j_rainbow_2", "on_board_led"]:
            info = controller.get_zone_info(packet, zone)
            print(f"\n--- Zone: {zone.upper()} ---")
            print(f"  Mode/Effect: {info['effect']}")
            print(f"  Primary RGB: {info['primary_rgb']}")
            print(f"  Secondary RGB: {info['secondary_rgb']}")
            print(f"  Brightness: {info['brightness']}/10")
            print(f"  Speed: {info['speed']}")
            if "led_count_or_cycle" in info:
                print(f"  LED Count/Cycle: {info['led_count_or_cycle']}")
    print("\n[OK] Device detected and readable without errors!")


def apply_color_change(r: int, g: int, b: int, color_name: str):
    print(f"\nApplying test color: {color_name} (R={r}, G={g}, B={b})...")
    with MSIMysticLightB550() as controller:
        success = controller.apply_color_to_all(r, g, b, mode=MODE_STATIC)
        if success:
            print(f"[OK] Successfully applied {color_name} to JRGB1, JRAINBOW1, JRAINBOW2, and ONBOARD!")
        else:
            print(f"[FAIL] Hardware report did not return expected size.")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "color":
        r = int(sys.argv[2])
        g = int(sys.argv[3])
        b = int(sys.argv[4])
        name = sys.argv[5] if len(sys.argv) > 5 else f"RGB({r},{g},{b})"
        apply_color_change(r, g, b, name)
    else:
        inspect_current_state()
