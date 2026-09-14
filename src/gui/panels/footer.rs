//! Bottom footer and hardware status bar panel.

use eframe::egui::{self, Color32, RichText};

use crate::gui::state::GuiApp;
use crate::gui::theme::card_frame;

impl GuiApp {
    pub fn render_footer(&mut self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                let status_color = if self.status_is_error {
                    Color32::from_rgb(248, 113, 113)
                } else {
                    Color32::from_rgb(74, 222, 128)
                };

                ui.label(RichText::new(&self.status_message).color(status_color).size(11.5));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Rust Native | Low Latency | ~18 MB Footprint")
                            .size(10.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                });
            });
        });
    }
}
