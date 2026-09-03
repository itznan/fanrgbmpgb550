"""
Real-time Audio Visualizer Engine for MSI MPG B550 GAMING PLUS.
Captures system playback via Windows WASAPI loopback, isolates sub-bass / kick transients,
and drives the motherboard RGB in pure red.
"""

import sys
import time
import threading
import warnings
from typing import Callable, Optional, List
import numpy as np

warnings.filterwarnings("ignore")

import soundcard as sc
from src.controller import MSIMysticLightB550
from src.config import MODE_STATIC, SAVE_OFFSET


class BassVisualizer:
    def __init__(
        self,
        mode: str = "hybrid",
        bass_min: float = 28.0,
        bass_max: float = 90.0,
        min_brightness: int = 0,
        threshold: float = 0.18,
        sensitivity: float = 1.0,
        decay: float = 0.70,
        gamma: float = 1.6,
        device_name: Optional[str] = None,
        on_frame: Optional[Callable[[float, int, float], None]] = None,
    ):
        self.mode = mode
        self.bass_min = bass_min
        self.bass_max = bass_max
        self.min_brightness = min_brightness
        self.threshold = threshold
        self.sensitivity = sensitivity
        self.decay = decay
        self.gamma = gamma
        self.device_name = device_name
        self.on_frame = on_frame

        self.running = False
        self._audio_thread = None
        self._render_thread = None

        self._latest_bass = 0.0
        self._latest_flux = 0.0

    @staticmethod
    def get_output_devices() -> List[str]:
        """Returns the list of available speaker / headphone output devices."""
        return [s.name for s in sc.all_speakers()]

    def start(self):
        """Starts the audio capture and RGB rendering threads."""
        if self.running:
            return

        self.running = True
        self._audio_thread = threading.Thread(target=self._audio_worker, daemon=True)
        self._render_thread = threading.Thread(target=self._render_worker, daemon=True)

        self._audio_thread.start()
        self._render_thread.start()

    def stop(self):
        """Stops the visualizer and resets lights to resting state."""
        self.running = False
        if self._render_thread and self._render_thread.is_alive():
            self._render_thread.join(timeout=1.0)
        if self._audio_thread and self._audio_thread.is_alive():
            self._audio_thread.join(timeout=0.5)

    def _audio_worker(self):
        sample_rate = 44100
        block_size = 1024

        try:
            if self.device_name:
                speakers = [s for s in sc.all_speakers() if self.device_name.lower() in s.name.lower()]
                target_speaker = speakers[0] if speakers else sc.default_speaker()
            else:
                target_speaker = sc.default_speaker()

            loopback_mic = sc.get_microphone(id=str(target_speaker.name), include_loopback=True)
            if not loopback_mic:
                self.running = False
                return

            hanning_window = np.hanning(block_size)
            freqs = np.fft.rfftfreq(block_size, 1.0 / sample_rate)
            bass_band = (freqs >= self.bass_min) & (freqs <= self.bass_max)

            prev_bass = 0.0

            with loopback_mic.recorder(samplerate=sample_rate, blocksize=block_size) as rec:
                while self.running:
                    data = rec.record(numframes=block_size)
                    mono = data.mean(axis=1) if data.ndim > 1 else data
                    windowed = mono * hanning_window
                    fft_mag = np.abs(np.fft.rfft(windowed))

                    bass = float(np.mean(fft_mag[bass_band])) if np.any(bass_band) else 0.0
                    flux = max(0.0, bass - prev_bass)
                    prev_bass = bass

                    self._latest_bass = bass
                    self._latest_flux = flux

        except Exception as e:
            print(f"[Visualizer Audio Error]: {e}", file=sys.stderr)
            self.running = False

    def _render_worker(self):
        peak_bass = 20.0
        floor_bass = 1.0
        peak_flux = 8.0
        floor_flux = 0.5

        current_level = 0.0
        frame_count = 0
        t_start = time.perf_counter()
        fps = 0.0

        controller = MSIMysticLightB550()
        try:
            controller.open()
            packet = controller.read_packet()
            packet[SAVE_OFFSET] = 0x00

            while self.running:
                t_frame = time.perf_counter()

                raw_bass = self._latest_bass * self.sensitivity
                raw_flux = self._latest_flux * self.sensitivity

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

                if self.mode == "kick":
                    target_val = flux_norm
                elif self.mode == "rumble":
                    target_val = bass_norm
                else:  # hybrid
                    target_val = (bass_norm * 0.40) + (flux_norm * 0.70)

                # Noise gate
                if target_val < self.threshold:
                    target_val = 0.0
                else:
                    target_val = (target_val - self.threshold) / (1.0 - self.threshold)

                target_val = target_val ** self.gamma

                # Attack / Decay
                if target_val > current_level:
                    current_level = current_level * 0.12 + target_val * 0.88
                else:
                    current_level = current_level * self.decay

                level_clamped = max(0.0, min(1.0, current_level))

                # Pure Red color: G=0, B=0
                min_r = self.min_brightness
                r = int(min_r + (255 - min_r) * level_clamped)
                g = 0
                b = 0

                # Set hardware zones
                controller.set_zone_data(packet, "j_rgb_1", MODE_STATIC, r, g, b)
                controller.set_zone_data(packet, "j_rainbow_1", MODE_STATIC, r, g, b, led_count=100)
                controller.set_zone_data(packet, "j_rainbow_2", MODE_STATIC, r, g, b, led_count=100)
                controller.set_zone_data(packet, "on_board_led", MODE_STATIC, r, g, b)
                for i in range(1, 7):
                    controller.set_zone_data(packet, f"on_board_led_{i}", MODE_STATIC, r, g, b)

                controller.stream_update(packet, pause_sec=0.012)

                frame_count += 1
                if frame_count % 3 == 0:
                    elapsed = time.perf_counter() - t_start
                    if elapsed > 0:
                        fps = frame_count / elapsed
                    if self.on_frame:
                        self.on_frame(level_clamped, r, fps)

                # Regulate frame rate to ~32 FPS
                t_sleep = 0.028 - (time.perf_counter() - t_frame)
                if t_sleep > 0.001:
                    time.sleep(t_sleep)

        except Exception as e:
            print(f"[Visualizer Render Error]: {e}", file=sys.stderr)
        finally:
            try:
                controller.apply_color_to_all(self.min_brightness, 0, 0, mode=MODE_STATIC)
                controller.close()
            except Exception:
                pass
