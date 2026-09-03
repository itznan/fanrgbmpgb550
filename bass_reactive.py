"""
Standalone CLI Runner for Pure Red Bass-Reactive Lighting.
"""

import sys
import time
import argparse
from src.visualizer import BassVisualizer


def main():
    parser = argparse.ArgumentParser(
        description="Pure Red Bass-Reactive RGB Visualizer for MSI MPG B550"
    )
    parser.add_argument(
        "--mode",
        type=str,
        choices=["hybrid", "kick", "rumble"],
        default="hybrid",
        help="Bass detection mode: 'hybrid' (body + kicks), 'kick' (punchy kicks), 'rumble' (deep sub-bass)",
    )
    parser.add_argument(
        "--min-brightness",
        type=int,
        default=0,
        help="Resting brightness (0-255) between kicks (default: 0)",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.18,
        help="Noise gate threshold (default: 0.18)",
    )
    parser.add_argument(
        "--sensitivity",
        type=float,
        default=1.0,
        help="Sensitivity multiplier (default: 1.0)",
    )
    parser.add_argument(
        "--decay",
        type=float,
        default=0.70,
        help="Decay factor (default: 0.70)",
    )
    parser.add_argument(
        "--gamma",
        type=float,
        default=1.6,
        help="Contrast exponent (default: 1.6)",
    )
    parser.add_argument(
        "--device",
        type=str,
        default=None,
        help="Audio device name (default: system default headphones/speakers)",
    )
    parser.add_argument(
        "--gpu",
        action="store_true",
        help="Synchronize GPU RGB logo with motherboard bass visualizer",
    )
    parser.add_argument(
        "--list-devices",
        action="store_true",
        help="List available audio output devices and exit",
    )

    args = parser.parse_args()

    if args.list_devices:
        print("\nAvailable Audio Output Devices (Headphones/Speakers):")
        for i, name in enumerate(BassVisualizer.get_output_devices()):
            print(f"  [{i}] {name}")
        print()
        return

    print("=" * 64)
    print(" MSI B550 Pure Red Bass Visualizer")
    print("=" * 64)
    print("Audio Source   : Default Headphones/Speakers (WASAPI Loopback)")
    print("Color Scheme   : 100% Pure Red (Zero orange / Zero amber)")
    print(f"Detection Mode : {args.mode.upper()}")
    print(f"Min Brightness : {args.min_brightness}/255")
    print(f"Sensitivity    : x{args.sensitivity:.1f} | Decay: {args.decay:.2f}")
    print("Press Ctrl+C to stop.\n")

    def on_frame(level: float, red_val: int, fps: float):
        bar_len = 24
        filled = int(bar_len * level)
        bar = "#" * filled + "-" * (bar_len - filled)
        sys.stdout.write(
            f"\r[{bar}] Bass: {int(level * 100):3d}% | Pure Red: {red_val:3d}/255 | FPS: {fps:.1f} "
        )
        sys.stdout.flush()

    vis = BassVisualizer(
        mode=args.mode,
        min_brightness=args.min_brightness,
        threshold=args.threshold,
        sensitivity=args.sensitivity,
        decay=args.decay,
        gamma=args.gamma,
        device_name=args.device,
        sync_gpu=args.gpu,
        on_frame=on_frame,
    )
    vis.start()

    try:
        while vis.running:
            time.sleep(0.1)
    except KeyboardInterrupt:
        print("\n\nStopping visualizer...")
    finally:
        vis.stop()
        print("Done.")


if __name__ == "__main__":
    main()
