"""Hardware diagnostic test for Gigabyte GPU RGB controller."""

import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from src.gpu_controller import (
    GigabyteGPURGB,
    GPU_MODE_STATIC,
    GPU_MODE_BREATHING,
)


def run_gpu_test():
    print("=" * 60)
    print(" Gigabyte GPU RGB Fusion 2.0 Hardware Diagnostic")
    print("=" * 60)

    gpu = GigabyteGPURGB()
    if not gpu._init_nvapi():
        print("[FAIL] NVAPI could not be initialized. Ensure an NVIDIA GPU and driver are installed.")
        return False

    print(f"GPU Model Detected : {gpu.gpu_name}")

    print("\nScanning candidate I2C addresses on GPU Port 1...")
    for addr in [0x32, 0x62, 0x71]:
        probe_pkt = [0xAB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        w_ok = gpu._i2c_write(addr, probe_pkt)
        if w_ok:
            resp = gpu._i2c_read(addr, 4)
            if resp and resp[0] == 0xAB:
                print(f"  [FOUND] Address 0x{addr:02X} responded with valid 0xAB signature: {[hex(x) for x in resp]}")
            else:
                print(f"  [INFO] Address 0x{addr:02X} write OK but invalid read signature: {resp}")
        else:
            print(f"  [INFO] Address 0x{addr:02X} NAK (not responding)")

    if not gpu.probe_and_connect():
        print("\n[FAIL] Could not establish connection to GPU RGB controller.")
        return False

    print(f"\n[OK] Controller verified at I2C address: 0x{gpu.active_address:02X}")

    print("\nTesting: Applying Pure Red (RGB: 255, 0, 0)...")
    ok = gpu.apply_color(255, 0, 0)
    if ok:
        print("[OK] Pure Red color command acknowledged successfully!")
    else:
        print("[FAIL] Failed to apply color.")

    print("\nTesting complete. GPU RGB controller is ready.")
    return True


if __name__ == "__main__":
    success = run_gpu_test()
    sys.exit(0 if success else 1)
