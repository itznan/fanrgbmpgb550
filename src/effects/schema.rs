//! Schema and deserialization for custom JSON lighting effects and profiles.

use crate::config::get_color_preset;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

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

                Err(de::Error::custom(format!(
                    "Unknown color string '{}'",
                    value
                )))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<ColorRGB, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let r: u8 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g: u8 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let b: u8 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::invalid_length(4, &self));
                }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn apply_zones(&mut self, zones: &HashMap<String, ColorRGB>) {
        // Apply "all" before specific zones so HashMap randomization cannot clobber overrides.
        for (key, color) in zones {
            if canonical_zone_name(key) == Some("all") {
                *self = Self::solid(*color);
            }
        }
        for (key, color) in zones {
            match canonical_zone_name(key) {
                Some("j_rgb_1") => self.j_rgb_1 = *color,
                Some("j_rainbow_1") => self.j_rainbow_1 = *color,
                Some("j_rainbow_2") => self.j_rainbow_2 = *color,
                Some("on_board_led") => self.on_board_led = *color,
                Some("gpu") => self.gpu = *color,
                _ => {}
            }
        }
    }

    pub fn from_keyframe(kf: &EffectKeyframe, unscaled_base: &ZoneState) -> (Self, Self) {
        let mut unscaled = *unscaled_base;

        if let Some(c) = kf.color {
            unscaled = Self::solid(c);
        }

        if let Some(ref zones) = kf.zones {
            unscaled.apply_zones(zones);
        }

        let mut scaled = unscaled;
        if let Some(b) = kf.brightness {
            let factor = if b > 100 {
                (b as f32) / 255.0
            } else {
                (b as f32) / 100.0
            };
            scaled.j_rgb_1 = scaled.j_rgb_1.scale(factor);
            scaled.j_rainbow_1 = scaled.j_rainbow_1.scale(factor);
            scaled.j_rainbow_2 = scaled.j_rainbow_2.scale(factor);
            scaled.on_board_led = scaled.on_board_led.scale(factor);
            scaled.gpu = scaled.gpu.scale(factor);
        }

        (unscaled, scaled)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::get_template_json;

    #[test]
    fn parses_all_color_formats_and_rejects_bad_arrays() {
        assert_eq!(
            serde_json::from_str::<ColorRGB>(r##""#00FFFF""##).unwrap(),
            ColorRGB::new(0, 255, 255)
        );
        assert_eq!(
            serde_json::from_str::<ColorRGB>(r#""purple""#).unwrap(),
            ColorRGB::new(160, 32, 240)
        );
        assert_eq!(
            serde_json::from_str::<ColorRGB>("[10, 20, 30]").unwrap(),
            ColorRGB::new(10, 20, 30)
        );
        assert_eq!(
            serde_json::from_str::<ColorRGB>(r#"{"r":1,"g":2,"b":3}"#).unwrap(),
            ColorRGB::new(1, 2, 3)
        );
        assert!(serde_json::from_str::<ColorRGB>("[1, 2, 3, 4]").is_err());
    }

    #[test]
    fn applies_all_before_specific_zones_and_avoids_brightness_compounding() {
        let mut zones = HashMap::new();
        zones.insert("gpu".to_string(), ColorRGB::new(200, 0, 0));
        zones.insert("all".to_string(), ColorRGB::new(0, 0, 100));

        let kf1 = EffectKeyframe {
            duration_ms: 100,
            transition: "linear".to_string(),
            color: None,
            zones: Some(zones),
            brightness: Some(50),
        };
        let (unscaled1, scaled1) = ZoneState::from_keyframe(&kf1, &ZoneState::default());
        assert_eq!(scaled1.j_rainbow_1, ColorRGB::new(0, 0, 50));
        assert_eq!(scaled1.gpu, ColorRGB::new(100, 0, 0));

        let mut zones2 = HashMap::new();
        zones2.insert("j_rainbow_1".to_string(), ColorRGB::new(200, 200, 200));
        let kf2 = EffectKeyframe {
            duration_ms: 100,
            transition: "linear".to_string(),
            color: None,
            zones: Some(zones2),
            brightness: Some(50),
        };
        let (_, scaled2) = ZoneState::from_keyframe(&kf2, &unscaled1);
        assert_eq!(scaled2.gpu, ColorRGB::new(100, 0, 0));
        assert_eq!(scaled2.j_rainbow_1, ColorRGB::new(100, 100, 100));
    }

    #[test]
    fn bundled_effects_and_templates_parse_and_unknown_template_is_none() {
        for name in ["cyberpunk", "police", "breath", "zones", "static"] {
            let raw = get_template_json(name).unwrap();
            assert!(serde_json::from_str::<CustomEffect>(raw).is_ok());
        }
        assert!(get_template_json("does_not_exist").is_none());
        let fade = include_str!("../../effects/blue_purple_fade.json");
        assert!(serde_json::from_str::<CustomEffect>(fade).is_ok());
    }
}
