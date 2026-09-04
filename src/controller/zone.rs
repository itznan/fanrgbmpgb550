//! Standalone zone-level read/write helpers for the MSI MPG B550 feature report packet.
#![allow(dead_code)]

use crate::config::{
    get_zone_offset, SYNC_SETTING_ONBOARD,
};

#[derive(Debug, Clone)]
pub struct ZoneInfo {
    pub zone: String,
    pub effect: u8,
    pub primary_rgb: (u8, u8, u8),
    pub secondary_rgb: (u8, u8, u8),
    pub speed: u8,
    pub brightness: u8,
    pub custom_color: bool,
    pub raw_flags: u8,
    pub raw_color_flags: u8,
    pub led_count_or_cycle: Option<u8>,
}

pub fn get_zone_info(packet: &[u8; 185], zone_name: &str) -> Option<ZoneInfo> {
    let offset = get_zone_offset(zone_name)?;
    let effect = packet[offset];
    let r1 = packet[offset + 1];
    let g1 = packet[offset + 2];
    let b1 = packet[offset + 3];
    let flags = packet[offset + 4];
    let speed = flags & 0x03;
    let brightness = (flags >> 2) & 0x1F;
    let r2 = packet[offset + 5];
    let g2 = packet[offset + 6];
    let b2 = packet[offset + 7];
    let color_flags = packet[offset + 8];
    let custom_color = (color_flags & 0x80) != 0;

    let led_count_or_cycle = if zone_name == "j_rainbow_1" || zone_name == "j_rainbow_2" {
        Some(packet[offset + 10])
    } else {
        None
    };

    Some(ZoneInfo {
        zone: zone_name.to_string(),
        effect,
        primary_rgb: (r1, g1, b1),
        secondary_rgb: (r2, g2, b2),
        speed,
        brightness,
        custom_color,
        raw_flags: flags,
        raw_color_flags: color_flags,
        led_count_or_cycle,
    })
}

pub fn set_zone_data(
    packet: &mut [u8; 185],
    zone_name: &str,
    mode: u8,
    r: u8,
    g: u8,
    b: u8,
    speed: u8,
    brightness: u8,
    led_count: u8,
) -> bool {
    let offset = match get_zone_offset(zone_name) {
        Some(off) => off,
        None => return false,
    };

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
        packet[offset + 10] = led_count;
    }

    true
}
