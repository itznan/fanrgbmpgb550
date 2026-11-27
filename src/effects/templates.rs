//! Starter JSON effect templates generator.

pub fn get_template_json(template_name: &str) -> Option<&'static str> {
    match template_name.to_lowercase().as_str() {
        "cyberpunk" | "neon" => Some(include_str!("../../effects/cyberpunk.json")),
        "police" | "strobe" => Some(include_str!("../../effects/police_strobe.json")),
        "breath" | "breathing" => Some(include_str!("../../effects/amber_breath.json")),
        "zones" | "split" => Some(include_str!("../../effects/zone_split.json")),
        "static" | "profile" => Some(include_str!("../../effects/static_profile.json")),
        _ => None,
    }
}
