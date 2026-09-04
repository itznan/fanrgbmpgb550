//! Hardware constants for MSI MPG B550 GAMING PLUS (MS-7C56).
#![allow(dead_code)]

pub const MSI_USB_VID: u16 = 0x1462;
pub const MSI_USB_PID: u16 = 0x7C56;

// Effect Modes
pub const MODE_DISABLE: u8 = 0;
pub const MODE_STATIC: u8 = 1;
pub const MODE_BREATHING: u8 = 2;
pub const MODE_FLASHING: u8 = 3;
pub const MODE_DOUBLE_FLASHING: u8 = 4;
pub const MODE_LIGHTNING: u8 = 5;
pub const MODE_METEOR: u8 = 7;
pub const MODE_COLOR_RING: u8 = 15;
pub const MODE_PLANETARY: u8 = 16;
pub const MODE_DOUBLE_METEOR: u8 = 17;
pub const MODE_ENERGY: u8 = 18;
pub const MODE_BLINK: u8 = 19;
pub const MODE_CLOCK: u8 = 20;
pub const MODE_COLOR_PULSE: u8 = 21;
pub const MODE_COLOR_SHIFT: u8 = 22;
pub const MODE_COLOR_WAVE: u8 = 23;
pub const MODE_MARQUEE: u8 = 24;
pub const MODE_RAINBOW_WAVE: u8 = 25;
pub const MODE_VISOR: u8 = 27;
pub const MODE_RAINBOW_FLASHING: u8 = 29;
pub const MODE_COLOR_RING_DOUBLE_FLASHING: u8 = 35;
pub const MODE_STACK: u8 = 36;
pub const MODE_FIRE: u8 = 38;

// Speed settings
pub const SPEED_LOW: u8 = 0;
pub const SPEED_MEDIUM: u8 = 1;
pub const SPEED_HIGH: u8 = 2;

// Brightness settings
pub const BRIGHTNESS_OFF: u8 = 0;
pub const BRIGHTNESS_10: u8 = 1;
pub const BRIGHTNESS_50: u8 = 5;
pub const BRIGHTNESS_100: u8 = 10;

// Sync flags
pub const SYNC_SETTING_ONBOARD: u8 = 0x01;

// Byte offsets in the 185-byte Feature Report (Report ID 0x52)
pub struct ZoneOffset {
    pub name: &'static str,
    pub offset: usize,
}

pub const ZONE_OFFSETS: &[ZoneOffset] = &[
    ZoneOffset { name: "j_rgb_1", offset: 1 },
    ZoneOffset { name: "j_pipe_1", offset: 11 },
    ZoneOffset { name: "j_pipe_2", offset: 21 },
    ZoneOffset { name: "j_rainbow_1", offset: 31 },
    ZoneOffset { name: "j_rainbow_2", offset: 42 },
    ZoneOffset { name: "j_corsair", offset: 53 },
    ZoneOffset { name: "j_corsair_outer", offset: 64 },
    ZoneOffset { name: "on_board_led", offset: 74 },
    ZoneOffset { name: "on_board_led_1", offset: 84 },
    ZoneOffset { name: "on_board_led_2", offset: 94 },
    ZoneOffset { name: "on_board_led_3", offset: 104 },
    ZoneOffset { name: "on_board_led_4", offset: 114 },
    ZoneOffset { name: "on_board_led_5", offset: 124 },
    ZoneOffset { name: "on_board_led_6", offset: 134 },
    ZoneOffset { name: "on_board_led_7", offset: 144 },
    ZoneOffset { name: "on_board_led_8", offset: 154 },
    ZoneOffset { name: "on_board_led_9", offset: 164 },
    ZoneOffset { name: "j_rgb_2", offset: 174 },
];

pub fn get_zone_offset(zone_name: &str) -> Option<usize> {
    ZONE_OFFSETS.iter().find(|z| z.name == zone_name).map(|z| z.offset)
}

/// Strict safety rule: byte 184 MUST always be 0x00 (RAM volatile only, no flash writes)
pub const SAVE_OFFSET: usize = 184;
