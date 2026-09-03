"""
Low-level HID Feature Report test for MSI MPG B550.
"""

import hid
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from src.config import MSI_USB_VID, MSI_USB_PID


def test_read():
    dev = hid.device()
    try:
        dev.open(MSI_USB_VID, MSI_USB_PID)
        print("Device opened successfully!")

        # 1. Read APROM version
        req = bytearray(64)
        req[0] = 0x01
        req[1] = 0xB0
        for i in range(2, 64):
            req[i] = 0xCC
        dev.write(req)
        resp = dev.read(64, timeout_ms=500)
        if resp and len(resp) >= 3:
            ap_high = resp[2] >> 4
            ap_low = resp[2] & 0x0F
            print(f"APROM Firmware Version: {ap_high}.{ap_low}")

        # 2. Read LDROM version
        req[1] = 0xB6
        dev.write(req)
        resp = dev.read(64, timeout_ms=500)
        if resp and len(resp) >= 3:
            ld_high = resp[2] >> 4
            ld_low = resp[2] & 0x0F
            print(f"LDROM Firmware Version: {ld_high}.{ld_low}")

        # 3. Read Feature Report 0x52
        feat = dev.get_feature_report(0x52, 185)
        print(f"\nFeature Report 0x52 read success! Bytes received: {len(feat)}")
        if len(feat) >= 185:
            print(f"Report ID: 0x{feat[0]:02X}")
            print(f"JRGB1 raw [1..10]: {[hex(b) for b in feat[1:11]]}")
            print(f"JRAINBOW1 raw [31..41]: {[hex(b) for b in feat[31:42]]}")
            print(f"JRAINBOW2 raw [42..52]: {[hex(b) for b in feat[42:53]]}")
            print(f"ONBOARD raw [74..83]: {[hex(b) for b in feat[74:84]]}")
            print(f"Save byte [184]: 0x{feat[184]:02X}")

    except Exception as e:
        print(f"Error during test_read: {e}", file=sys.stderr)
    finally:
        dev.close()
        print("Device closed.")


if __name__ == "__main__":
    test_read()
