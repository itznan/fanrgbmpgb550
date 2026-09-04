"""
RGB render worker for BassVisualizer.
Runs in a background thread; reads bass/flux from the visualizer instance
and drives the MSI motherboard (and optionally Gigabyte GPU) RGB.
"""

import sys
import time

from src.controller import MSIMysticLightB550
from src.config import MODE_STATIC, SAVE_OFFSET


def render_worker(vis) -> None:
    """
    RGB render loop. Reads ``vis._latest_bass``, ``vis._latest_flux``, and all
    visualizer parameters; dispatches USB packets to the motherboard controller
    (and optionally the GPU).

    Parameters
    ----------
    vis : BassVisualizer
        The parent visualizer instance.
    """
    peak_bass = 20.0
    floor_bass = 1.0
    peak_flux = 8.0
    floor_flux = 0.5

    current_level = 0.0
    frame_count = 0
    last_sent_r = -1
    t_start = time.perf_counter()
    fps = 0.0

    controller = MSIMysticLightB550()
    gpu = None
    if vis.sync_gpu:
        try:
            from src.gpu_controller import GigabyteGPURGB
            gpu_dev = GigabyteGPURGB()
            if gpu_dev.probe_and_connect():
                gpu = gpu_dev
        except Exception as ex:
            print(f"[Visualizer GPU Warning]: {ex}", file=sys.stderr)
            gpu = None

    try:
        controller.open()
        packet = controller.read_packet()
        packet[SAVE_OFFSET] = 0x00

        while vis.running:
            t_frame = time.perf_counter()

            raw_bass = vis._latest_bass * vis.sensitivity
            raw_flux = vis._latest_flux * vis.sensitivity

            # Dynamic peak / floor follower
            if raw_bass > peak_bass:
                peak_bass = raw_bass
            else:
                peak_bass = max(peak_bass * 0.985, 8.0)

            if raw_bass < floor_bass:
                floor_bass = raw_bass
            else:
                floor_bass = min(floor_bass * 1.01, peak_bass * 0.35)

            if raw_flux > peak_flux:
                peak_flux = raw_flux
            else:
                peak_flux = max(peak_flux * 0.985, 4.0)

            if raw_flux < floor_flux:
                floor_flux = raw_flux
            else:
                floor_flux = min(floor_flux * 1.01, peak_flux * 0.30)

            # Normalized metrics
            bass_norm = max(0.0, min(1.0, (raw_bass - floor_bass) / max(peak_bass - floor_bass, 3.0)))
            flux_norm = max(0.0, min(1.0, (raw_flux - floor_flux) / max(peak_flux - floor_flux, 2.0)))

            if vis.mode == "kick":
                target_val = flux_norm
            elif vis.mode == "rumble":
                target_val = bass_norm
            else:  # hybrid
                target_val = (bass_norm * 0.40) + (flux_norm * 0.70)

            # Noise gate
            if target_val < vis.threshold:
                target_val = 0.0
            else:
                target_val = (target_val - vis.threshold) / (1.0 - vis.threshold)

            target_val = target_val ** vis.gamma

            # Attack / Decay
            if target_val > current_level:
                current_level = current_level * 0.12 + target_val * 0.88
            else:
                current_level = current_level * vis.decay

            level_clamped = max(0.0, min(1.0, current_level))

            # Pure Red color: G=0, B=0
            min_r = vis.min_brightness
            r = int(min_r + (255 - min_r) * level_clamped)
            g = 0
            b = 0

            # Delta check: only dispatch USB packets when color changes or heartbeat (every 30 frames)
            if r != last_sent_r or (frame_count % 30 == 0):
                controller.set_zone_data(packet, "j_rgb_1", MODE_STATIC, r, g, b)
                controller.set_zone_data(packet, "j_rainbow_1", MODE_STATIC, r, g, b, led_count=100)
                controller.set_zone_data(packet, "j_rainbow_2", MODE_STATIC, r, g, b, led_count=100)
                controller.set_zone_data(packet, "on_board_led", MODE_STATIC, r, g, b)
                for i in range(1, 7):
                    controller.set_zone_data(packet, f"on_board_led_{i}", MODE_STATIC, r, g, b)

                controller.stream_update(packet, pause_sec=0.012)
                if gpu:
                    gpu.stream_color_fast(r, 0, 0)
                last_sent_r = r

            frame_count += 1
            if frame_count % 3 == 0:
                elapsed = time.perf_counter() - t_start
                if elapsed > 0:
                    fps = frame_count / elapsed
                if vis.on_frame:
                    vis.on_frame(level_clamped, r, fps)

            # Regulate frame rate to ~32 FPS
            t_sleep = 0.028 - (time.perf_counter() - t_frame)
            if t_sleep > 0.001:
                time.sleep(t_sleep)

    except Exception as e:
        print(f"[Visualizer Render Error]: {e}", file=sys.stderr)
    finally:
        if gpu:
            try:
                gpu.apply_color(vis.min_brightness, 0, 0)
            except Exception:
                pass
        try:
            controller.apply_color_to_all(vis.min_brightness, 0, 0, mode=MODE_STATIC)
            controller.close()
        except Exception:
            pass


__all__ = ["render_worker"]
