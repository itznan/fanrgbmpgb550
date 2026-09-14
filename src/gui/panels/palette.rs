//! Quick swatch palette bar panel.

use eframe::egui::{self, Color32, RichText, Rounding, Stroke};

use crate::gui::state::GuiApp;
use crate::gui::theme::card_frame;

impl GuiApp {
    pub fn render_quick_palette(&mut self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("PRESETS:")
                        .size(11.0)
                        .color(Color32::from_rgb(148, 163, 184))
                        .strong(),
                );

                let presets: &[(&str, [u8; 3])] = &[
                    ("Red", [255, 0, 0]),
                    ("Fire Orange", [255, 110, 0]),
                    ("Amber Gold", [255, 190, 0]),
                    ("Neon Green", [0, 255, 0]),
                    ("Cyber Cyan", [0, 255, 255]),
                    ("Sky Blue", [0, 180, 255]),
                    ("Ice Blue", [80, 200, 255]),
                    ("Deep Blue", [0, 0, 255]),
                    ("Purple", [160, 32, 240]),
                    ("Dark Purple", [75, 0, 130]),
                    ("Magenta", [255, 0, 255]),
                    ("Pure White", [255, 255, 255]),
                    ("Warm White", [255, 210, 160]),
                ];

                for (name, col) in presets {
                    let c32 = Color32::from_rgb(col[0], col[1], col[2]);
                    let btn = egui::Button::new(RichText::new("  ").background_color(c32))
                        .fill(c32)
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0f32, Color32::from_rgb(80, 80, 90)));
                    if ui.add(btn).on_hover_text(*name).clicked() {
                        self.master_color = *col;
                        self.mobo_color = *col;
                        self.gpu_color = *col;
                        if self.live_preview {
                            self.sync_all(*col);
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hex = format!("#{:02X}{:02X}{:02X}", self.master_color[0], self.master_color[1], self.master_color[2]);
                    ui.label(RichText::new(hex).monospace().size(11.0).color(Color32::from_rgb(148, 163, 184)));
                    ui.color_edit_button_srgb(&mut self.master_color);
                    ui.label(RichText::new("Master Color:").size(11.0));
                });
            });
        });
    }
}
