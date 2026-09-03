"""
USB HID device enumeration test for MSI MPG B550.
"""

import hid
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from src.config import MSI_USB_VID, MSI_USB_PID


def scan_msi():
    devices = hid.enumerate(MSI_USB_VID, MSI_USB_PID)
    print(f"Found {len(devices)} device(s) matching VID 0x{MSI_USB_VID:04X}, PID 0x{MSI_USB_PID:04X}:")
    for i, dev_info in enumerate(devices):
        print(f"\nDevice #{i}:")
        for key, val in dev_info.items():
            print(f"  {key}: {val}")
    return devices


if __name__ == "__main__":
    scan_msi()
