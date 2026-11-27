//! CLI argument parsing helpers.

use crate::config::get_color_preset;

/// Parses a color from all tokens in `args`.
/// Supports hex (`"#RRGGBB"` or `RRGGBB`), presets (`red`, `dark blue`, etc.), and RGB integers (`255 0 0`).
pub fn parse_color(args: &[String]) -> Result<(u8, u8, u8), String> {
    match args.len() {
        0 => Err("No color arguments provided.".to_string()),
        1 => {
            let hex_str = args[0].trim_start_matches('#');
            if hex_str.len() == 6 && hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
                let r = u8::from_str_radix(&hex_str[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex_str[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex_str[4..6], 16).map_err(|e| e.to_string())?;
                return Ok((r, g, b));
            }
            get_color_preset(&args[0]).ok_or_else(|| format!("Unknown color '{}'", args[0]))
        }
        2 => {
            let two_words_under = format!("{}_{}", args[0], args[1]);
            if let Some(rgb) = get_color_preset(&two_words_under) {
                return Ok(rgb);
            }
            let two_words_concat = format!("{}{}", args[0], args[1]);
            get_color_preset(&two_words_concat)
                .ok_or_else(|| format!("Unknown color '{} {}'", args[0], args[1]))
        }
        3 => {
            let r = args[0]
                .parse::<u8>()
                .map_err(|_| "Invalid R component (expected 0-255)".to_string())?;
            let g = args[1]
                .parse::<u8>()
                .map_err(|_| "Invalid G component (expected 0-255)".to_string())?;
            let b = args[2]
                .parse::<u8>()
                .map_err(|_| "Invalid B component (expected 0-255)".to_string())?;
            Ok((r, g, b))
        }
        _ => Err(format!("Cannot parse color from: {:?}", args)),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_color;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parses_hex_preset_two_word_and_rgb() {
        assert_eq!(parse_color(&s(&["#A020F0"])), Ok((160, 32, 240)));
        assert_eq!(parse_color(&s(&["A020F0"])), Ok((160, 32, 240)));
        assert_eq!(parse_color(&s(&["purple"])), Ok((160, 32, 240)));
        assert_eq!(parse_color(&s(&["dark", "blue"])), Ok((0, 20, 180)));
        assert_eq!(parse_color(&s(&["160", "32", "240"])), Ok((160, 32, 240)));
    }

    #[test]
    fn rejects_invalid_and_trailing_color_args() {
        assert!(parse_color(&[]).is_err());
        assert!(parse_color(&s(&["red", "extra"])).is_err());
        assert!(parse_color(&s(&["300", "0", "0"])).is_err());
        assert!(parse_color(&s(&["notacolor"])).is_err());
    }
}
