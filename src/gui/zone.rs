//! Motherboard lighting zone enum.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoboZone {
    All,
    JRgb1,
    JRainbow1,
    JRainbow2,
    OnBoard,
}

impl MoboZone {
    pub fn label(&self) -> &'static str {
        match self {
            MoboZone::All => "All Motherboard Zones",
            MoboZone::JRgb1 => "J_RGB_1 (12V 4-Pin)",
            MoboZone::JRainbow1 => "J_RAINBOW_1 (5V ARGB)",
            MoboZone::JRainbow2 => "J_RAINBOW_2 (5V ARGB)",
            MoboZone::OnBoard => "On-Board IO / Chipset",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            MoboZone::All => "all",
            MoboZone::JRgb1 => "j_rgb_1",
            MoboZone::JRainbow1 => "j_rainbow_1",
            MoboZone::JRainbow2 => "j_rainbow_2",
            MoboZone::OnBoard => "on_board_led",
        }
    }
}
