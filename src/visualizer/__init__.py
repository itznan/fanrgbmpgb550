"""
Real-time Audio Visualizer Engine for MSI MPG B550 GAMING PLUS.
Captures system playback via Windows WASAPI loopback, isolates sub-bass / kick transients,
and drives the motherboard RGB in pure red.
"""

import threading
import warnings
from typing import Callable, Optional, List

warnings.filterwarnings("ignore")

import soundcard as sc

from src.visualizer.audio import audio_worker
from src.visualizer.renderer import render_worker


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
        sync_gpu: bool = False,
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
        self.sync_gpu = sync_gpu
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
        audio_worker(self)

    def _render_worker(self):
        render_worker(self)
