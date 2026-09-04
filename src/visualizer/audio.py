"""
Audio capture worker for BassVisualizer.
Runs in a background thread; writes bass/flux readings to the visualizer instance.
"""

import sys
import numpy as np
import soundcard as sc


def audio_worker(vis) -> None:
    """
    Audio capture loop. Reads ``vis.bass_min``, ``vis.bass_max``, ``vis.device_name``
    and ``vis.running``; writes ``vis._latest_bass`` and ``vis._latest_flux``.

    Parameters
    ----------
    vis : BassVisualizer
        The parent visualizer instance.
    """
    sample_rate = 44100
    block_size = 1024

    try:
        if vis.device_name:
            speakers = [s for s in sc.all_speakers() if vis.device_name.lower() in s.name.lower()]
            target_speaker = speakers[0] if speakers else sc.default_speaker()
        else:
            target_speaker = sc.default_speaker()

        loopback_mic = sc.get_microphone(id=str(target_speaker.name), include_loopback=True)
        if not loopback_mic:
            vis.running = False
            return

        hanning_window = np.hanning(block_size)
        freqs = np.fft.rfftfreq(block_size, 1.0 / sample_rate)
        bass_band = (freqs >= vis.bass_min) & (freqs <= vis.bass_max)

        prev_bass = 0.0

        with loopback_mic.recorder(samplerate=sample_rate, blocksize=block_size) as rec:
            while vis.running:
                data = rec.record(numframes=block_size)
                mono = data.mean(axis=1) if data.ndim > 1 else data
                windowed = mono * hanning_window
                fft_mag = np.abs(np.fft.rfft(windowed))

                bass = float(np.mean(fft_mag[bass_band])) if np.any(bass_band) else 0.0
                flux = max(0.0, bass - prev_bass)
                prev_bass = bass

                vis._latest_bass = bass
                vis._latest_flux = flux

    except Exception as e:
        print(f"[Visualizer Audio Error]: {e}", file=sys.stderr)
        vis.running = False


__all__ = ["audio_worker"]
