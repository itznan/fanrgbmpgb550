//! Color presets and animation mode name mappings.

use super::hardware::*;

pub fn get_color_preset(name: &str) -> Option<(u8, u8, u8)> {
    match name.to_lowercase().as_str() {
        "red" => Some((255, 0, 0)),
        "green" => Some((0, 255, 0)),
        "blue" => Some((0, 0, 255)),
        "cyan" => Some((0, 255, 255)),
        "magenta" => Some((255, 0, 255)),
        "yellow" => Some((255, 255, 0)),
        "white" => Some((255, 255, 255)),
        "orange" => Some((255, 120, 0)),
        "purple" => Some((160, 32, 240)),
        "off" => Some((0, 0, 0)),
        _ => None,
    }
}

pub fn get_animation_mode(name: &str) -> Option<u8> {
    match name.to_lowercase().as_str() {
        "rainbow_wave" => Some(MODE_RAINBOW_WAVE),
        "breathing" => Some(MODE_BREATHING),
        "meteor" => Some(MODE_METEOR),
        "flashing" => Some(MODE_FLASHING),
        "double_flashing" => Some(MODE_DOUBLE_FLASHING),
        "lightning" => Some(MODE_LIGHTNING),
        "color_pulse" => Some(MODE_COLOR_PULSE),
        "color_shift" => Some(MODE_COLOR_SHIFT),
        "color_wave" => Some(MODE_COLOR_WAVE),
        "marquee" => Some(MODE_MARQUEE),
        "visor" => Some(MODE_VISOR),
        "stack" => Some(MODE_STACK),
        "fire" => Some(MODE_FIRE),
        _ => None,
    }
}
