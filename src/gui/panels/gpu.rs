//! Graphics card lighting control panel.

use eframe::egui::{self, Color32, RichText, Vec2};

use crate::gpu::*;
use crate::gui::state::GuiApp;
use crate::gui::theme::card_frame;

impl GuiApp {
    pub fn render_gpu_panel(&mut self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🎮 GRAPHICS CARD")
                            .color(Color32::from_rgb(52, 211, 153))
                            .strong()
                            .size(13.5),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("NVAPI I2C")
                                .size(10.0)
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                    });
                });

                ui.separator();

                // GPU Model & Status
                ui.label(RichText::new("Device Profile:").size(11.0).strong());
                ui.label(
                    RichText::new(&self.gpu_model_name)
                        .size(11.5)
                        .color(Color32::from_rgb(226, 232, 240)),
                );

                ui.add_space(4.0);

                // GPU Modes
                ui.label(RichText::new("GPU Lighting Mode:").size(11.0).strong());
                let gpu_modes: &[(&str, u8)] = &[
                    ("Static Color", GPU_MODE_STATIC),
                    ("Breathing / Pulse", GPU_MODE_BREATHING),
                    ("Color Cycle", GPU_MODE_COLOR_CYCLE),
                    ("Flashing", GPU_MODE_FLASHING),
                    ("Dual Flashing", GPU_MODE_DUAL_FLASHING),
                    ("Gradient", GPU_MODE_GRADIENT),
                    ("Wave", GPU_MODE_WAVE),
                ];

                let current_gpu_mode_label = gpu_modes
                    .iter()
                    .find(|(_, code)| *code == self.gpu_mode)
                    .map(|(label, _)| *label)
                    .unwrap_or("Custom");

                egui::ComboBox::from_id_salt("gpu_mode_select")
                    .selected_text(current_gpu_mode_label)
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for (label, code) in gpu_modes {
                            if ui.selectable_label(self.gpu_mode == *code, *label).clicked() {
                                self.gpu_mode = *code;
                                if self.live_preview {
                                    self.apply_gpu();
                                }
                            }
                        }
                    });

                ui.add_space(4.0);

                // Color Picker
                ui.label(RichText::new("GPU Color:").size(11.0).strong());
                ui.horizontal(|ui| {
                    if ui.color_edit_button_srgb(&mut self.gpu_color).changed() && self.live_preview {
                        self.apply_gpu();
                    }
                    let hex = format!("#{:02X}{:02X}{:02X}", self.gpu_color[0], self.gpu_color[1], self.gpu_color[2]);
                    ui.label(RichText::new(hex).monospace().size(11.0));
                });

                ui.add_space(4.0);

                // Speed Slider (0..5)
                ui.label(RichText::new("Effect Speed:").size(11.0));
                let gpu_speed_labels = ["Slowest", "Slow", "Normal", "Fast", "Fastest"];
                let speed_val = self.gpu_speed.min(4) as usize;
                ui.horizontal(|ui| {
                    for (i, lbl) in gpu_speed_labels.iter().enumerate() {
                        if ui.selectable_label(speed_val == i, *lbl).clicked() {
                            self.gpu_speed = match i {
                                0 => GPU_SPEED_SLOWEST,
                                1 => GPU_SPEED_SLOW,
                                2 => GPU_SPEED_NORMAL,
                                3 => GPU_SPEED_FAST,
                                _ => GPU_SPEED_FASTEST,
                            };
                            if self.live_preview {
                                self.apply_gpu();
                            }
                        }
                    }
                });

                ui.add_space(4.0);

                // Brightness Slider (0..99)
                ui.label(RichText::new(format!("Brightness: {} / 99", self.gpu_brightness)).size(11.0));
                if ui
                    .add(egui::Slider::new(&mut self.gpu_brightness, 0..=99).text("Value"))
                    .changed()
                    && self.live_preview
                {
                    self.apply_gpu();
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Actions
                ui.horizontal(|ui| {
                    let apply_btn = egui::Button::new(RichText::new("✔ Apply GPU").strong())
                        .fill(Color32::from_rgb(16, 185, 129))
                        .min_size(Vec2::new(100.0, 28.0));
                    if ui.add(apply_btn).clicked() {
                        self.apply_gpu();
                    }

                    let off_btn = egui::Button::new(RichText::new("Turn Off").color(Color32::from_rgb(239, 68, 68)))
                        .min_size(Vec2::new(80.0, 28.0));
                    if ui.add(off_btn).clicked() {
                        self.turn_off_gpu();
                    }
                });
            });
        });
    }
}
