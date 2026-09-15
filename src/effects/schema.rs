//! Schema and deserialization for custom JSON lighting effects and profiles.

use std::collections::HashMap;
use std::fmt;
use serde::{de, Deserialize, Deserializer};
use crate::config::get_color_preset;

fn default_true() -> bool {
    true
}

fn default_fps() -> u32 {
    30
}

fn default_transition() -> String {
    "linear".to_string()
}

/// Represents an RGB color that can be parsed from:
/// - Hex string: `"#A020F0"` or `"A020F0"`
/// - Preset string: `"purple"`, `"cyan"`, `"red"`, `"darkblue"`, etc.
/// - Integer array: `[160, 32, 240]`
/// - Object: `{"r": 160, "g": 32, "b": 240}`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorRGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_tuple(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let r = (self.r as f32 + (other.r as f32 - self.r as f32) * t).round() as u8;
        let g = (self.g as f32 + (other.g as f32 - self.g as f32) * t).round() as u8;
        let b = (self.b as f32 + (other.b as f32 - self.b as f32) * t).round() as u8;
        Self { r, g, b }
    }

    pub fn scale(self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: ((self.r as f32) * factor).round() as u8,
            g: ((self.g as f32) * factor).round() as u8,
            b: ((self.b as f32) * factor).round() as u8,
        }
    }
}

impl<'de> Deserialize<'de> for ColorRGB {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> de::Visitor<'de> for ColorVisitor {
            type Value = ColorRGB;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex string ('#RRGGBB'), a preset name ('purple'), an array [R, G, B], or an object {r, g, b}")
            }

            fn visit_str<E>(self, value: &str) -> Result<ColorRGB, E>
            where
                E: de::Error,
            {
                let trimmed = value.trim();
                let hex_clean = trimmed.trim_start_matches('#');
                if hex_clean.len() == 6 && hex_clean.chars().all(|c| c.is_ascii_hexdigit()) {
                    let r = u8::from_str_radix(&hex_clean[0..2], 16).map_err(de::Error::custom)?;
                    let g = u8::from_str_radix(&hex_clean[2..4], 16).map_err(de::Error::custom)?;
                    let b = u8::from_str_radix(&hex_clean[4..6], 16).map_err(de::Error::custom)?;
                    return Ok(ColorRGB { r, g, b });
                }

                if let Some((r, g, b)) = get_color_preset(trimmed) {
                    return Ok(ColorRGB { r, g, b });
                }

                Err(de::Error::custom(format!("Unknown color string '{}'", value)))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<ColorRGB, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let r: u8 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g: u8 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let b: u8 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(ColorRGB { r, g, b })
            }

            fn visit_map<M>(self, mut map: M) -> Result<ColorRGB, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut r = None;
                let mut g = None;
                let mut b = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.to_lowercase().as_str() {
                        "r" | "red" => r = Some(map.next_value()?),
                        "g" | "green" => g = Some(map.next_value()?),
                        "b" | "blue" => b = Some(map.next_value()?),
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let r = r.ok_or_else(|| de::Error::missing_field("r"))?;
                let g = g.ok_or_else(|| de::Error::missing_field("g"))?;
                let b = b.ok_or_else(|| de::Error::missing_field("b"))?;
                Ok(ColorRGB { r, g, b })
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}

/// Represents one step / keyframe in a custom lighting effect.
#[derive(Debug, Clone, Deserialize)]
pub struct EffectKeyframe {
    /// Duration of this frame in milliseconds (e.g. 500, 1000).
    #[serde(alias = "duration", alias = "time_ms")]
    pub duration_ms: u64,

    /// Transition type: "linear" / "fade" or "instant" / "step" / "hold".
    #[serde(default = "default_transition")]
    pub transition: String,

    /// Optional global color for all zones in this keyframe.
    pub color: Option<ColorRGB>,

    /// Optional per-zone color mapping.
    pub zones: Option<HashMap<String, ColorRGB>>,

    /// Brightness scale for this keyframe (0 to 100). Default is 100.
    pub brightness: Option<u8>,
}

/// Custom effect or static lighting profile defined via JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomEffect {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,

    /// Whether the animation loops indefinitely (default true).
    #[serde(default = "default_true")]
    pub r#loop: bool,

    /// Optional number of repetitions (if loop is false or to limit cycles).
    pub repeat: Option<u32>,

    /// Frame rate for smooth interpolation (default 30).
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Whether to synchronize the GPU. If not specified, auto-detected.
    pub sync_gpu: Option<bool>,

    /// Static zone colors (if not using keyframes, or base fallback).
    pub zones: Option<HashMap<String, ColorRGB>>,

    /// Sequence of animated keyframes.
    #[serde(alias = "frames", alias = "steps")]
    pub keyframes: Option<Vec<EffectKeyframe>>,
}

/// Normalized canonical zone names.
pub fn canonical_zone_name(name: &str) -> Option<&'static str> {
    match name.to_lowercase().replace('-', "_").as_str() {
        "j_rgb_1" | "jrgb1" | "j_rgb" | "jrgb" | "rgb" | "12v" => Some("j_rgb_1"),
        "j_rainbow_1" | "jrainbow1" | "rainbow1" | "argb1" | "fans" | "fan" => Some("j_rainbow_1"),
        "j_rainbow_2" | "jrainbow2" | "rainbow2" | "argb2" => Some("j_rainbow_2"),
        "on_board_led" | "onboard_led" | "on_board" | "onboard" | "chipset" => Some("on_board_led"),
        "gpu" | "gpu_rgb" | "graphics" => Some("gpu"),
        "all" | "sync" | "system" => Some("all"),
        _ => None,
    }
}

/// A resolved state mapping canonical zone names to RGB colors.
#[derive(Debug, Clone, Copy)]
pub struct ZoneState {
    pub j_rgb_1: ColorRGB,
    pub j_rainbow_1: ColorRGB,
    pub j_rainbow_2: ColorRGB,
    pub on_board_led: ColorRGB,
    pub gpu: ColorRGB,
}

impl Default for ZoneState {
    fn default() -> Self {
        Self {
            j_rgb_1: ColorRGB::new(0, 0, 0),
            j_rainbow_1: ColorRGB::new(0, 0, 0),
            j_rainbow_2: ColorRGB::new(0, 0, 0),
            on_board_led: ColorRGB::new(0, 0, 0),
            gpu: ColorRGB::new(0, 0, 0),
        }
    }
}

impl ZoneState {
    pub fn solid(color: ColorRGB) -> Self {
        Self {
            j_rgb_1: color,
            j_rainbow_1: color,
            j_rainbow_2: color,
            on_board_led: color,
            gpu: color,
        }
    }

    pub fn from_keyframe(kf: &EffectKeyframe, base: &ZoneState) -> Self {
        let mut state = *base;

        if let Some(c) = kf.color {
            state = Self::solid(c);
        }

        if let Some(ref zones) = kf.zones {
            for (key, color) in zones {
                match canonical_zone_name(key) {
                    Some("all") => state = Self::solid(*color),
                    Some("j_rgb_1") => state.j_rgb_1 = *color,
                    Some("j_rainbow_1") => state.j_rainbow_1 = *color,
                    Some("j_rainbow_2") => state.j_rainbow_2 = *color,
                    Some("on_board_led") => state.on_board_led = *color,
                    Some("gpu") => state.gpu = *color,
                    _ => {}
                }
            }
        }

        if let Some(b) = kf.brightness {
            let factor = if b > 100 {
                (b as f32) / 255.0
            } else {
                (b as f32) / 100.0
            };
            state.j_rgb_1 = state.j_rgb_1.scale(factor);
            state.j_rainbow_1 = state.j_rainbow_1.scale(factor);
            state.j_rainbow_2 = state.j_rainbow_2.scale(factor);
            state.on_board_led = state.on_board_led.scale(factor);
            state.gpu = state.gpu.scale(factor);
        }

        state
    }

    pub fn lerp(&self, other: &ZoneState, t: f32) -> Self {
        Self {
            j_rgb_1: self.j_rgb_1.lerp(other.j_rgb_1, t),
            j_rainbow_1: self.j_rainbow_1.lerp(other.j_rainbow_1, t),
            j_rainbow_2: self.j_rainbow_2.lerp(other.j_rainbow_2, t),
            on_board_led: self.on_board_led.lerp(other.on_board_led, t),
            gpu: self.gpu.lerp(other.gpu, t),
        }
    }
}
