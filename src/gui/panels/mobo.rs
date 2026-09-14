//! Motherboard and ARGB fan lighting control panel.

use eframe::egui::{self, Color32, RichText, Vec2};

use crate::config::*;
use crate::gui::state::GuiApp;
use crate::gui::theme::card_frame;
use crate::gui::zone::MoboZone;

impl GuiApp {
    pub fn render_mobo_panel(&mut self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🎛 MOTHERBOARD")
                            .color(Color32::from_rgb(96, 165, 250))
                            .strong()
                            .size(13.5),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("MSI MS-7C56")
                                .size(10.0)
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                    });
                });

                ui.separator();

                // Target Zone selector
                ui.label(RichText::new("Target Lighting Zone:").size(11.0).strong());
                let zones = [
                    MoboZone::All,
                    MoboZone::JRgb1,
                    MoboZone::JRainbow1,
                    MoboZone::JRainbow2,
                    MoboZone::OnBoard,
                ];
                egui::ComboBox::from_id_salt("mobo_zone_select")
                    .selected_text(self.mobo_zone.label())
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for z in &zones {
                            ui.selectable_value(&mut self.mobo_zone, *z, z.label());
                        }
                    });

                ui.add_space(4.0);

                // Effect Mode selector
                ui.label(RichText::new("Effect Animation Mode:").size(11.0).strong());
                let mobo_modes: &[(&str, u8)] = &[
                    ("Static Color", MODE_STATIC),
                    ("Breathing", MODE_BREATHING),
                    ("Rainbow Wave", MODE_RAINBOW_WAVE),
                    ("Meteor", MODE_METEOR),
                    ("Flashing", MODE_FLASHING),
                    ("Double Flashing", MODE_DOUBLE_FLASHING),
                    ("Lightning", MODE_LIGHTNING),
                    ("Color Pulse", MODE_COLOR_PULSE),
                    ("Color Shift", MODE_COLOR_SHIFT),
                    ("Color Wave", MODE_COLOR_WAVE),
                    ("Marquee", MODE_MARQUEE),
                    ("Visor", MODE_VISOR),
                    ("Fire", MODE_FIRE),
                    ("Stack", MODE_STACK),
                    ("Disabled / Off", MODE_DISABLE),
                ];

                let current_mode_label = mobo_modes
                    .iter()
                    .find(|(_, code)| *code == self.mobo_mode)
                    .map(|(label, _)| *label)
                    .unwrap_or("Custom / Other");

                egui::ComboBox::from_id_salt("mobo_mode_select")
                    .selected_text(current_mode_label)
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for (label, code) in mobo_modes {
                            if ui.selectable_label(self.mobo_mode == *code, *label).clicked() {
                                self.mobo_mode = *code;
                                if self.live_preview {
                                    self.apply_motherboard();
                                }
                            }
                        }
                    });

                ui.add_space(4.0);

                // Color Picker
                ui.label(RichText::new("Zone Color:").size(11.0).strong());
                ui.horizontal(|ui| {
                    if ui.color_edit_button_srgb(&mut self.mobo_color).changed() && self.live_preview {
                        self.apply_motherboard();
                    }
                    let hex = format!("#{:02X}{:02X}{:02X}", self.mobo_color[0], self.mobo_color[1], self.mobo_color[2]);
                    ui.label(RichText::new(hex).monospace().size(11.0));
                });

                ui.add_space(4.0);

                // Speed Slider
                ui.label(RichText::new("Animation Speed:").size(11.0));
                let speed_labels = ["Slow", "Medium", "High"];
                let mut speed_idx = self.mobo_speed.min(2) as usize;
                ui.horizontal(|ui| {
                    for (i, lbl) in speed_labels.iter().enumerate() {
                        if ui.selectable_label(speed_idx == i, *lbl).clicked() {
                            speed_idx = i;
                            self.mobo_speed = i as u8;
                            if self.live_preview {
                                self.apply_motherboard();
                            }
                        }
                    }
                });

                ui.add_space(4.0);

                // Brightness Slider (0..10)
                ui.label(RichText::new(format!("Brightness: {}%", self.mobo_brightness * 10)).size(11.0));
                if ui
                    .add(egui::Slider::new(&mut self.mobo_brightness, 0..=10).text("Level"))
                    .changed()
                    && self.live_preview
                {
                    self.apply_motherboard();
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Actions
                ui.horizontal(|ui| {
                    let apply_btn = egui::Button::new(RichText::new("✔ Apply MB").strong())
                        .fill(Color32::from_rgb(37, 99, 235))
                        .min_size(Vec2::new(100.0, 28.0));
                    if ui.add(apply_btn).clicked() {
                        self.apply_motherboard();
                    }

                    let off_btn = egui::Button::new(RichText::new("Turn Off").color(Color32::from_rgb(239, 68, 68)))
                        .min_size(Vec2::new(80.0, 28.0));
                    if ui.add(off_btn).clicked() {
                        self.turn_off_motherboard();
                    }
                });
            });
        });
    }
}
