"""
Color presets and animation mode name mappings.
"""

from src.config.hardware import (
    MODE_BREATHING,
    MODE_FLASHING,
    MODE_DOUBLE_FLASHING,
    MODE_LIGHTNING,
    MODE_METEOR,
    MODE_COLOR_PULSE,
    MODE_COLOR_SHIFT,
    MODE_COLOR_WAVE,
    MODE_MARQUEE,
    MODE_RAINBOW_WAVE,
    MODE_VISOR,
    MODE_STACK,
    MODE_FIRE,
)

COLOR_PRESETS = {
    "red": (255, 0, 0),
    "green": (0, 255, 0),
    "blue": (0, 0, 255),
    "cyan": (0, 255, 255),
    "magenta": (255, 0, 255),
    "yellow": (255, 255, 0),
    "white": (255, 255, 255),
    "orange": (255, 120, 0),
    "purple": (160, 32, 240),
    "off": (0, 0, 0),
}

ANIMATION_MODES = {
    "rainbow_wave": MODE_RAINBOW_WAVE,
    "breathing": MODE_BREATHING,
    "meteor": MODE_METEOR,
    "flashing": MODE_FLASHING,
    "double_flashing": MODE_DOUBLE_FLASHING,
    "lightning": MODE_LIGHTNING,
    "color_pulse": MODE_COLOR_PULSE,
    "color_shift": MODE_COLOR_SHIFT,
    "color_wave": MODE_COLOR_WAVE,
    "marquee": MODE_MARQUEE,
    "visor": MODE_VISOR,
    "stack": MODE_STACK,
    "fire": MODE_FIRE,
}
