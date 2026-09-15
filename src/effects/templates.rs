//! Starter JSON effect templates generator.

pub fn get_template_json(template_name: &str) -> Option<&'static str> {
    match template_name.to_lowercase().as_str() {
        "cyberpunk" | "neon" => Some(CYBERPUNK_TEMPLATE),
        "police" | "strobe" => Some(POLICE_STROBE_TEMPLATE),
        "breath" | "breathing" => Some(BREATHING_TEMPLATE),
        "zones" | "split" => Some(ZONE_SPLIT_TEMPLATE),
        "static" | "profile" => Some(STATIC_PROFILE_TEMPLATE),
        _ => Some(CYBERPUNK_TEMPLATE),
    }
}

pub const CYBERPUNK_TEMPLATE: &str = r##"{
  "name": "Cyberpunk Neon",
  "description": "Smooth fade between vibrant neon cyan and hot magenta/purple",
  "author": "FanRGB",
  "loop": true,
  "fps": 30,
  "sync_gpu": true,
  "keyframes": [
    {
      "duration_ms": 1500,
      "transition": "linear",
      "color": "#00FFFF",
      "brightness": 100
    },
    {
      "duration_ms": 1500,
      "transition": "linear",
      "color": "#FF007F",
      "brightness": 100
    },
    {
      "duration_ms": 1500,
      "transition": "linear",
      "color": "#A020F0",
      "brightness": 100
    }
  ]
}
"##;

pub const POLICE_STROBE_TEMPLATE: &str = r##"{
  "name": "Police Strobe",
  "description": "High-intensity alternating red and blue emergency strobe",
  "loop": true,
  "fps": 30,
  "sync_gpu": true,
  "keyframes": [
    {
      "duration_ms": 100,
      "transition": "instant",
      "color": "red"
    },
    {
      "duration_ms": 50,
      "transition": "instant",
      "color": "off"
    },
    {
      "duration_ms": 100,
      "transition": "instant",
      "color": "red"
    },
    {
      "duration_ms": 150,
      "transition": "instant",
      "color": "off"
    },
    {
      "duration_ms": 100,
      "transition": "instant",
      "color": "blue"
    },
    {
      "duration_ms": 50,
      "transition": "instant",
      "color": "off"
    },
    {
      "duration_ms": 100,
      "transition": "instant",
      "color": "blue"
    },
    {
      "duration_ms": 150,
      "transition": "instant",
      "color": "off"
    }
  ]
}
"##;

pub const BREATHING_TEMPLATE: &str = r##"{
  "name": "Amber Warm Breath",
  "description": "Slow organic breathing from deep ember red to golden amber",
  "loop": true,
  "fps": 30,
  "sync_gpu": true,
  "keyframes": [
    {
      "duration_ms": 2500,
      "transition": "linear",
      "color": "#FF8C00",
      "brightness": 100
    },
    {
      "duration_ms": 1500,
      "transition": "linear",
      "color": "#8B0000",
      "brightness": 25
    }
  ]
}
"##;

pub const ZONE_SPLIT_TEMPLATE: &str = r##"{
  "name": "Dual Zone Wave",
  "description": "Independent colors animated per motherboard header and GPU",
  "loop": true,
  "fps": 30,
  "sync_gpu": true,
  "keyframes": [
    {
      "duration_ms": 1200,
      "transition": "linear",
      "zones": {
        "j_rainbow_1": "cyan",
        "j_rainbow_2": "purple",
        "j_rgb_1": "blue",
        "on_board_led": "magenta",
        "gpu": "cyan"
      }
    },
    {
      "duration_ms": 1200,
      "transition": "linear",
      "zones": {
        "j_rainbow_1": "purple",
        "j_rainbow_2": "cyan",
        "j_rgb_1": "magenta",
        "on_board_led": "blue",
        "gpu": "purple"
      }
    }
  ]
}
"##;

pub const STATIC_PROFILE_TEMPLATE: &str = r##"{
  "name": "Static Studio Profile",
  "description": "Applies fixed custom colors across individual motherboard headers and GPU",
  "sync_gpu": true,
  "zones": {
    "j_rainbow_1": "#50C8FF",
    "j_rainbow_2": "#A020F0",
    "j_rgb_1": "#0014B4",
    "on_board_led": "#4B0082",
    "gpu": "#50C8FF"
  }
}
"##;
