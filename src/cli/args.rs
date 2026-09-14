//! CLI argument parsing helpers.

use crate::config::get_color_preset;

/// Parses a color from command-line arguments.
/// Supports hex (`"#RRGGBB"` or `RRGGBB`), presets (`red`, `purple`, etc.), and RGB integers (`255 0 0`).
pub fn parse_color(args: &[String]) -> Result<((u8, u8, u8), &[String]), String> {
    if args.is_empty() {
        return Err("No color arguments provided.".to_string());
    }

    let first = &args[0];
    let hex_str = first.trim_start_matches('#');
    if hex_str.len() == 6 && hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        let r = u8::from_str_radix(&hex_str[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex_str[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex_str[4..6], 16).map_err(|e| e.to_string())?;
        return Ok(((r, g, b), &args[1..]));
    }

    if args.len() >= 2 {
        let two_words_under = format!("{}_{}", args[0], args[1]);
        if let Some(rgb) = get_color_preset(&two_words_under) {
            return Ok((rgb, &args[2..]));
        }
        let two_words_concat = format!("{}{}", args[0], args[1]);
        if let Some(rgb) = get_color_preset(&two_words_concat) {
            return Ok((rgb, &args[2..]));
        }
    }

    if let Some(rgb) = get_color_preset(first) {
        return Ok((rgb, &args[1..]));
    }

    if args.len() >= 3 {
        let r = args[0].parse::<u8>().map_err(|_| "Invalid R component")?;
        let g = args[1].parse::<u8>().map_err(|_| "Invalid G component")?;
        let b = args[2].parse::<u8>().map_err(|_| "Invalid B component")?;
        return Ok(((r, g, b), &args[3..]));
    }

    Err(format!("Cannot parse color from: {:?}", args))
}
