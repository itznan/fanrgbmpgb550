//! Standalone zone-level read/write helpers for the MSI MPG B550 feature report packet.

use crate::config::{get_zone_offset, SYNC_SETTING_ONBOARD};

pub const DEFAULT_ARGB_LED_COUNT: u8 = 100;

pub const ONBOARD_ZONES: &[&str] = &[
    "on_board_led",
    "on_board_led_1",
    "on_board_led_2",
    "on_board_led_3",
    "on_board_led_4",
    "on_board_led_5",
    "on_board_led_6",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneInfo {
    pub effect: u8,
    pub primary_rgb: (u8, u8, u8),
    pub brightness: u8,
}

pub fn get_zone_info(packet: &[u8; 185], zone_name: &str) -> Option<ZoneInfo> {
    let offset = get_zone_offset(zone_name)?;
    let effect = packet[offset];
    let r1 = packet[offset + 1];
    let g1 = packet[offset + 2];
    let b1 = packet[offset + 3];
    let flags = packet[offset + 4];
    let brightness = (flags >> 2) & 0x1F;

    Some(ZoneInfo {
        effect,
        primary_rgb: (r1, g1, b1),
        brightness,
    })
}

pub fn set_zone_data(
    packet: &mut [u8; 185],
    zone_name: &str,
    mode: u8,
    rgb: (u8, u8, u8),
    speed: u8,
    brightness: u8,
) -> bool {
    let offset = match get_zone_offset(zone_name) {
        Some(off) => off,
        None => return false,
    };

    let (r, g, b) = rgb;
    packet[offset] = mode;
    packet[offset + 1] = r;
    packet[offset + 2] = g;
    packet[offset + 3] = b;
    packet[offset + 4] = ((brightness & 0x1F) << 2) | (speed & 0x03);
    packet[offset + 5] = r;
    packet[offset + 6] = g;
    packet[offset + 7] = b;

    if zone_name == "on_board_led" {
        packet[offset + 8] = 0x80 | SYNC_SETTING_ONBOARD;
    } else {
        packet[offset + 8] = 0x80;
    }

    packet[offset + 9] = 0x00;

    if zone_name == "j_rainbow_1" || zone_name == "j_rainbow_2" {
        packet[offset + 10] = DEFAULT_ARGB_LED_COUNT;
    }

    true
}

pub fn set_onboard_zones(
    packet: &mut [u8; 185],
    mode: u8,
    rgb: (u8, u8, u8),
    speed: u8,
    brightness: u8,
) {
    for &zone in ONBOARD_ZONES {
        set_zone_data(packet, zone, mode, rgb, speed, brightness);
    }
}

pub fn set_all_zones(
    packet: &mut [u8; 185],
    mode: u8,
    rgb: (u8, u8, u8),
    speed: u8,
    brightness: u8,
) {
    set_zone_data(packet, "j_rgb_1", mode, rgb, speed, brightness);
    set_zone_data(packet, "j_rainbow_1", mode, rgb, speed, brightness);
    set_zone_data(packet, "j_rainbow_2", mode, rgb, speed, brightness);
    set_onboard_zones(packet, mode, rgb, speed, brightness);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BRIGHTNESS_100, MODE_STATIC, SAVE_OFFSET, SPEED_MEDIUM};

    #[test]
    fn packs_and_unpacks_zones_without_setting_save_bit() {
        let mut packet = [0u8; 185];
        packet[0] = 0x52;
        packet[SAVE_OFFSET] = 0x00;

        set_all_zones(
            &mut packet,
            MODE_STATIC,
            (160, 32, 240),
            SPEED_MEDIUM,
            BRIGHTNESS_100,
        );

        assert_eq!(packet[SAVE_OFFSET], 0x00);
        assert_eq!(
            get_zone_info(&packet, "j_rainbow_1"),
            Some(ZoneInfo {
                effect: MODE_STATIC,
                primary_rgb: (160, 32, 240),
                brightness: BRIGHTNESS_100,
            })
        );
        assert_eq!(
            get_zone_info(&packet, "on_board_led_6").map(|z| z.primary_rgb),
            Some((160, 32, 240))
        );
        assert_eq!(packet[31 + 10], DEFAULT_ARGB_LED_COUNT);
        assert_eq!(packet[74 + 8], 0x80 | SYNC_SETTING_ONBOARD);
    }
}
