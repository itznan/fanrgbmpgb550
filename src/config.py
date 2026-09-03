"""
Configuration, Hardware constants, Zone offsets, and Color presets
for MSI MPG B550 GAMING PLUS (MS-7C56).
"""

# USB Hardware IDs
MSI_USB_VID = 0x1462
MSI_USB_PID = 0x7C56  # MS-7C56 (MPG B550 GAMING PLUS)

# Effect Modes from MSIMotherboardCommon.h
MODE_DISABLE = 0
MODE_STATIC = 1
MODE_BREATHING = 2
MODE_FLASHING = 3
MODE_DOUBLE_FLASHING = 4
MODE_LIGHTNING = 5
MODE_METEOR = 7
MODE_COLOR_RING = 15
MODE_PLANETARY = 16
MODE_DOUBLE_METEOR = 17
MODE_ENERGY = 18
MODE_BLINK = 19
MODE_CLOCK = 20
MODE_COLOR_PULSE = 21
MODE_COLOR_SHIFT = 22
MODE_COLOR_WAVE = 23
MODE_MARQUEE = 24
MODE_RAINBOW_WAVE = 25
MODE_VISOR = 27
MODE_RAINBOW_FLASHING = 29
MODE_COLOR_RING_DOUBLE_FLASHING = 35
MODE_STACK = 36
MODE_FIRE = 38
MODE_DIRECT_DUMMY = 100

# Speed settings
SPEED_LOW = 0
SPEED_MEDIUM = 1
SPEED_HIGH = 2

# Brightness settings
BRIGHTNESS_OFF = 0
BRIGHTNESS_10 = 1
BRIGHTNESS_50 = 5
BRIGHTNESS_100 = 10

# Sync flags
SYNC_SETTING_ONBOARD = 0x01
SYNC_SETTING_JRAINBOW1 = 0x02
SYNC_SETTING_JRAINBOW2 = 0x04
SYNC_SETTING_JCORSAIR = 0x08
SYNC_SETTING_JPIPE1 = 0x10
SYNC_SETTING_JPIPE2 = 0x20
SYNC_SETTING_JRGB = 0x80

# Byte offsets in the 185-byte Feature Report (Report ID 0x52)
ZONE_OFFSETS = {
    "j_rgb_1": 1,           # 10 bytes: 1..10 (12V 4-pin RGB)
    "j_pipe_1": 11,         # 10 bytes: 11..20
    "j_pipe_2": 21,         # 10 bytes: 21..30
    "j_rainbow_1": 31,      # 11 bytes: 31..41 (5V 3-pin ARGB)
    "j_rainbow_2": 42,      # 11 bytes: 42..52 (5V 3-pin ARGB)
    "j_corsair": 53,        # 11 bytes: 53..63
    "j_corsair_outer": 64,  # 10 bytes: 64..73
    "on_board_led": 74,     # 10 bytes: 74..83 (Master onboard)
    "on_board_led_1": 84,   # 10 bytes: 84..93
    "on_board_led_2": 94,   # 10 bytes: 94..103
    "on_board_led_3": 104,  # 10 bytes: 104..113
    "on_board_led_4": 114,  # 10 bytes: 114..123
    "on_board_led_5": 124,  # 10 bytes: 124..133
    "on_board_led_6": 134,  # 10 bytes: 134..143
    "on_board_led_7": 144,  # 10 bytes: 144..153
    "on_board_led_8": 154,  # 10 bytes: 154..163
    "on_board_led_9": 164,  # 10 bytes: 164..173
    "j_rgb_2": 174,         # 10 bytes: 174..183
}

# Strict safety rule: byte 184 MUST always be 0x00 (RAM volatile only, no flash writes)
SAVE_OFFSET = 184

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
