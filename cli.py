"""
Unified Command-Line Interface for MSI MPG B550 Motherboard & Gigabyte GPU RGB.
"""

import sys
import time
import warnings

warnings.filterwarnings("ignore")


def parse_color(args):
    """Parse a color from CLI args.

    Accepts:
      - A single hex string: ``#RRGGBB`` or ``RRGGBB``  (returns remaining args)
      - Three separate integers: ``R G B``               (returns remaining args)

    Returns:
      ``(r, g, b, remaining_args)`` on success, or raises ``ValueError``.
    """
    if not args:
        raise ValueError("No color arguments provided.")

    first = args[0]
    hex_str = first.lstrip("#")
    if len(hex_str) == 6 and all(c in "0123456789abcdefABCDEF" for c in hex_str):
        r = int(hex_str[0:2], 16)
        g = int(hex_str[2:4], 16)
        b = int(hex_str[4:6], 16)
        return r, g, b, args[1:]

    if len(args) >= 3:
        r, g, b = int(args[0]), int(args[1]), int(args[2])
        return r, g, b, args[3:]

    raise ValueError(f"Cannot parse color from: {args}")


from src.controller import MSIMysticLightB550
from src.gpu_controller import (
    GigabyteGPURGB,
    GPU_MODE_STATIC,
    GPU_MODE_BREATHING,
    GPU_MODE_COLOR_CYCLE,
    GPU_MODE_FLASHING,
    GPU_MODE_DUAL_FLASHING,
    GPU_MODE_GRADIENT,
    GPU_MODE_WAVE,
)
from src.visualizer import BassVisualizer
from src.config import (
    COLOR_PRESETS,
    ANIMATION_MODES,
    MODE_STATIC,
    MODE_DISABLE,
)

GPU_ANIMATION_MODES = {
    "static": GPU_MODE_STATIC,
    "breathing": GPU_MODE_BREATHING,
    "pulse": GPU_MODE_BREATHING,
    "color_cycle": GPU_MODE_COLOR_CYCLE,
    "flash": GPU_MODE_FLASHING,
    "flashing": GPU_MODE_FLASHING,
    "double_flash": GPU_MODE_DUAL_FLASHING,
    "gradient": GPU_MODE_GRADIENT,
    "wave": GPU_MODE_WAVE,
}


def print_help():
    print("""
MSI Motherboard & Gigabyte GPU RGB Controller CLI

Color values can be specified as:
  - A preset name : red, blue, green, off, ...
  - Hex           : #RRGGBB  or  RRGGBB  (e.g. #FF0000 or FF0000)
  - RGB integers  : R G B               (e.g. 255 0 0)

=== Motherboard Commands ===
  python cli.py status
  python cli.py <preset_name>             (red, blue, green, off, etc.)
  python cli.py <#RRGGBB|RRGGBB>         (e.g. #FF0000 or FF0000)
  python cli.py <r> <g> <b>               (e.g. 255 0 0)
  python cli.py mode <animation_mode>     (rainbow_wave, breathing, meteor, etc.)
  python cli.py mode <animation_mode> <#RRGGBB|R G B>

=== GPU Commands ===
  python cli.py gpu status                (Checks NVAPI connection and I2C address)
  python cli.py gpu <preset_name>         (e.g. gpu red, gpu blue, gpu off)
  python cli.py gpu <#RRGGBB|RRGGBB>     (e.g. gpu #FF0000 or gpu FF0000)
  python cli.py gpu <r> <g> <b>           (e.g. gpu 255 0 0)
  python cli.py gpu mode <mode_name>      (breathing, flash, color_cycle, wave)

=== Synchronized Control (Motherboard + GPU) ===
  python cli.py sync <preset_name>        (e.g. sync red, sync off)
  python cli.py sync <#RRGGBB|RRGGBB>    (e.g. sync #FF0000 or sync FF0000)
  python cli.py sync <r> <g> <b>          (e.g. sync 255 0 0)
  python cli.py bass [--gpu]              (Pure red bass visualizer synced to Motherboard & GPU)

Examples:
  python cli.py bass --gpu                # Bass visualizer pulsing Motherboard AND GPU logo
  python cli.py sync red                  # Set entire PC to pure red
  python cli.py sync #FF0000              # Same as above, using hex
  python cli.py gpu mode breathing        # GPU pulsing red breathing mode
  python cli.py sync off                  # Power down all RGB across the system
""")


def run_cli_bass(sync_gpu: bool = False):
    print("=" * 65)
    print(" Pure Red Bass & Kick-Drum Visualizer (Terminal Mode)")
    print("=" * 65)
    print("Audio Source: Default Playback Device (WASAPI Loopback)")
    print("Color: 100% Pure Red (Zero orange, zero amber)")
    if sync_gpu:
        print("Hardware Sync: MSI B550 Motherboard + Gigabyte GPU")
    else:
        print("Hardware Sync: MSI B550 Motherboard (use --gpu to include GPU)")
    print("Press Ctrl+C to stop.\n")

    def on_frame(level: float, red_val: int, fps: float):
        bar_len = 24
        filled = int(bar_len * level)
        bar = "#" * filled + "-" * (bar_len - filled)
        sys.stdout.write(
            f"\r[{bar}] Bass: {int(level * 100):3d}% | Pure Red: {red_val:3d}/255 | FPS: {fps:.1f} "
        )
        sys.stdout.flush()

    vis = BassVisualizer(mode="hybrid", sync_gpu=sync_gpu, on_frame=on_frame)
    vis.start()

    try:
        while vis.running:
            time.sleep(0.1)
    except KeyboardInterrupt:
        print("\n\nStopping visualizer...")
    finally:
        vis.stop()
        print("Done.")


def handle_gpu_command(args):
    if not args:
        print_help()
        return

    sub = args[0].lower()
    gpu = GigabyteGPURGB()

    if sub == "status":
        if gpu.probe_and_connect():
            print(f"[OK] Gigabyte GPU Detected: {gpu.gpu_name}")
            print(f"[OK] Controller I2C Address: 0x{gpu.active_address:02X} (Port 1)")
        else:
            print("[FAIL] Could not detect Gigabyte GPU RGB controller.")
        return

    if not gpu.probe_and_connect():
        print("[FAIL] Unable to connect to GPU RGB controller via NVAPI.")
        return

    if sub == "off":
        gpu.turn_off()
        print("[OK] GPU RGB turned OFF.")
        return

    if sub in COLOR_PRESETS:
        r, g, b = COLOR_PRESETS[sub]
        gpu.apply_color(r, g, b)
        print(f"[OK] GPU set to {sub.upper()} ({r}, {g}, {b})")
        return

    if sub == "mode" and len(args) >= 2:
        mode_name = args[1].lower()
        if mode_name not in GPU_ANIMATION_MODES:
            print(f"Unknown GPU mode: '{mode_name}'. Options: {', '.join(GPU_ANIMATION_MODES.keys())}")
            return
        gpu.apply_mode(GPU_ANIMATION_MODES[mode_name])
        print(f"[OK] GPU mode set to {mode_name.upper()}.")
        return

    try:
        r, g, b, _ = parse_color(args)
        gpu.apply_color(r, g, b)
        print(f"[OK] GPU set to RGB({r}, {g}, {b})")
        return
    except ValueError:
        pass

    print_help()


def handle_sync_command(args):
    if not args:
        print_help()
        return

    sub = args[0].lower()
    r, g, b = 255, 0, 0
    mode = MODE_STATIC

    if sub == "off":
        r, g, b = 0, 0, 0
        mode = MODE_DISABLE
    elif sub in COLOR_PRESETS:
        r, g, b = COLOR_PRESETS[sub]
    else:
        try:
            r, g, b, _ = parse_color(args)
        except ValueError:
            print_help()
            return

    # 1. Update Motherboard
    try:
        with MSIMysticLightB550() as controller:
            controller.apply_color_to_all(r, g, b, mode=mode)
    except Exception as e:
        print(f"[Warning] Motherboard update error: {e}")

    # 2. Update GPU
    try:
        gpu = GigabyteGPURGB()
        if gpu.probe_and_connect():
            if mode == MODE_DISABLE:
                gpu.turn_off()
            else:
                gpu.apply_color(r, g, b)
    except Exception as e:
        print(f"[Warning] GPU update error: {e}")

    print(f"[OK] Synchronized Motherboard + GPU to RGB({r}, {g}, {b})")


def main():
    if len(sys.argv) < 2:
        print_help()
        return

    cmd = sys.argv[1].lower()

    if cmd == "bass":
        sync_gpu = "--gpu" in sys.argv or "-g" in sys.argv
        run_cli_bass(sync_gpu=sync_gpu)
        return

    if cmd == "gpu":
        handle_gpu_command(sys.argv[2:])
        return

    if cmd == "sync":
        handle_sync_command(sys.argv[2:])
        return

    with MSIMysticLightB550() as controller:
        if cmd == "status":
            packet = controller.read_packet()
            print("Current Active Zones (Motherboard):")
            for z in ["j_rgb_1", "j_rainbow_1", "j_rainbow_2", "on_board_led"]:
                info = controller.get_zone_info(packet, z)
                print(f"  {z}: Mode={info['effect']} RGB={info['primary_rgb']} Brightness={info['brightness']}/10")
            return

        if cmd in COLOR_PRESETS:
            r, g, b = COLOR_PRESETS[cmd]
            mode = MODE_DISABLE if cmd == "off" else MODE_STATIC
            controller.apply_color_to_all(r, g, b, mode=mode)
            print(f"[OK] Set all motherboard zones to {cmd.upper()} (R={r}, G={g}, B={b})")
            return

        if cmd == "mode" and len(sys.argv) >= 3:
            mode_name = sys.argv[2].lower()
            if mode_name not in ANIMATION_MODES:
                print(f"Unknown mode: '{mode_name}'. Choose from:\n{', '.join(ANIMATION_MODES.keys())}")
                return

            r, g, b = (255, 0, 0)
            if len(sys.argv) >= 4:
                try:
                    r, g, b, _ = parse_color(sys.argv[3:])
                except ValueError:
                    pass

            controller.apply_color_to_all(r, g, b, mode=ANIMATION_MODES[mode_name])
            print(f"[OK] Switched motherboard mode to {mode_name.upper()} (R={r}, G={g}, B={b})")
            return

        try:
            r, g, b, _ = parse_color(sys.argv[1:])
            controller.apply_color_to_all(r, g, b, mode=MODE_STATIC)
            print(f"[OK] Set all motherboard zones to custom RGB({r}, {g}, {b})")
            return
        except ValueError:
            pass

        print_help()


if __name__ == "__main__":
    main()
