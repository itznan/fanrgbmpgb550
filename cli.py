"""
Interactive CLI for MSI MPG B550 GAMING PLUS RGB.
"""

import sys
import time

from src.controller import MSIMysticLightB550
from src.visualizer import BassVisualizer
from src.config import (
    COLOR_PRESETS,
    ANIMATION_MODES,
    MODE_STATIC,
    MODE_DISABLE,
)


def print_help():
    print("""
MSI MPG B550 RGB Controller CLI

Usage:
  python cli.py status
  python cli.py bass [--kick | --rumble | --hybrid]
  python cli.py <preset_name>
  python cli.py <r> <g> <b>
  python cli.py mode <animation_mode> [r g b]

Examples:
  python cli.py bass                 # Start pure red bass visualizer in terminal
  python cli.py red                  # Solid red
  python cli.py off                  # Turn off all LEDs
  python cli.py 255 0 0              # Custom RGB
  python cli.py mode rainbow_wave    # Rainbow wave animation
  python cli.py mode breathing       # Red breathing animation

Presets:
  red, green, blue, cyan, magenta, yellow, orange, purple, white, off

Animation Modes:
  rainbow_wave, breathing, meteor, flashing, double_flashing,
  lightning, color_pulse, color_shift, color_wave, marquee, visor,
  stack, fire
""")


def run_cli_bass():
    print("=" * 60)
    print(" Pure Red Bass & Kick-Drum Visualizer (Terminal Mode)")
    print("=" * 60)
    print("Audio Source: Default Headphones/Speakers (WASAPI Loopback)")
    print("Color: 100% Pure Red (Zero orange, zero amber)")
    print("Press Ctrl+C to stop.\n")

    def on_frame(level: float, red_val: int, fps: float):
        bar_len = 24
        filled = int(bar_len * level)
        bar = "#" * filled + "-" * (bar_len - filled)
        sys.stdout.write(
            f"\r[{bar}] Bass: {int(level * 100):3d}% | Pure Red: {red_val:3d}/255 | FPS: {fps:.1f} "
        )
        sys.stdout.flush()

    vis = BassVisualizer(mode="hybrid", on_frame=on_frame)
    vis.start()

    try:
        while vis.running:
            time.sleep(0.1)
    except KeyboardInterrupt:
        print("\n\nStopping visualizer...")
    finally:
        vis.stop()
        print("Done.")


def main():
    if len(sys.argv) < 2:
        print_help()
        return

    cmd = sys.argv[1].lower()

    if cmd == "bass":
        run_cli_bass()
        return

    with MSIMysticLightB550() as controller:
        if cmd == "status":
            packet = controller.read_packet()
            print("Current Active Zones:")
            for z in ["j_rgb_1", "j_rainbow_1", "j_rainbow_2", "on_board_led"]:
                info = controller.get_zone_info(packet, z)
                print(f"  {z}: Mode={info['effect']} RGB={info['primary_rgb']} Brightness={info['brightness']}/10")
            return

        if cmd in COLOR_PRESETS:
            r, g, b = COLOR_PRESETS[cmd]
            mode = MODE_DISABLE if cmd == "off" else MODE_STATIC
            controller.apply_color_to_all(r, g, b, mode=mode)
            print(f"[OK] Set all zones to {cmd.upper()} (R={r}, G={g}, B={b})")
            return

        if cmd == "mode" and len(sys.argv) >= 3:
            mode_name = sys.argv[2].lower()
            if mode_name not in ANIMATION_MODES:
                print(f"Unknown mode: '{mode_name}'. Choose from:\n{', '.join(ANIMATION_MODES.keys())}")
                return

            r, g, b = (255, 0, 0)
            if len(sys.argv) >= 6:
                try:
                    r = int(sys.argv[3])
                    g = int(sys.argv[4])
                    b = int(sys.argv[5])
                except ValueError:
                    pass

            controller.apply_color_to_all(r, g, b, mode=ANIMATION_MODES[mode_name])
            print(f"[OK] Switched mode to {mode_name.upper()} (R={r}, G={g}, B={b})")
            return

        if len(sys.argv) >= 4:
            try:
                r = int(sys.argv[1])
                g = int(sys.argv[2])
                b = int(sys.argv[3])
                controller.apply_color_to_all(r, g, b, mode=MODE_STATIC)
                print(f"[OK] Set all zones to custom RGB({r}, {g}, {b})")
            except ValueError:
                print_help()
            return

        print_help()


if __name__ == "__main__":
    main()
