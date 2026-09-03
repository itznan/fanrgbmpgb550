"""
Backward-compatibility layer for msi_b550.py.
Forwards all constants, modes, and MSIMysticLightB550 to src.controller and src.config.
"""

from src.config import *
from src.controller import MSIMysticLightB550

__all__ = [
    "MSIMysticLightB550",
    "MSI_USB_VID",
    "MSI_USB_PID",
    "ZONE_OFFSETS",
    "SAVE_OFFSET",
    "COLOR_PRESETS",
    "ANIMATION_MODES",
    "MODE_DISABLE",
    "MODE_STATIC",
    "MODE_BREATHING",
    "MODE_FLASHING",
    "MODE_DOUBLE_FLASHING",
    "MODE_LIGHTNING",
    "MODE_METEOR",
    "MODE_COLOR_RING",
    "MODE_PLANETARY",
    "MODE_DOUBLE_METEOR",
    "MODE_ENERGY",
    "MODE_BLINK",
    "MODE_CLOCK",
    "MODE_COLOR_PULSE",
    "MODE_COLOR_SHIFT",
    "MODE_COLOR_WAVE",
    "MODE_MARQUEE",
    "MODE_RAINBOW_WAVE",
    "MODE_VISOR",
    "MODE_RAINBOW_FLASHING",
    "MODE_COLOR_RING_DOUBLE_FLASHING",
    "MODE_STACK",
    "MODE_FIRE",
    "SPEED_LOW",
    "SPEED_MEDIUM",
    "SPEED_HIGH",
    "BRIGHTNESS_OFF",
    "BRIGHTNESS_10",
    "BRIGHTNESS_50",
    "BRIGHTNESS_100",
]
