//! Real-time audio sub-bass visualizer panel.

use std::sync::atomic::Ordering;
use eframe::egui::{self, Color32, RichText, Vec2};

use crate::gui::state::GuiApp;
use crate::gui::theme::card_frame;

impl GuiApp {
    pub fn render_visualizer_panel(&mut self, ui: &mut egui::Ui) {
        let is_running = self.viz_running.load(Ordering::SeqCst);

        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🔊 AUDIO VISUALIZER")
                            .color(Color32::from_rgb(244, 114, 182))
                            .strong()
                            .size(13.5),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_running {
                            ui.label(
                                RichText::new("● ACTIVE")
                                    .size(10.0)
                                    .color(Color32::from_rgb(34, 197, 94))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                RichText::new("○ IDLE")
                                    .size(10.0)
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );
                        }
                    });
                });

                ui.separator();

                // Live Audio Bass VU Meter
                let (bass_val, flux_val) = if is_running {
                    let m = self.viz_metrics.lock().unwrap();
                    (m.bass, m.flux)
                } else {
                    (0.0, 0.0)
                };

                let norm_bass = (bass_val / 40.0).clamp(0.0, 1.0);

                ui.label(RichText::new("Real-Time Sub-Bass Level:").size(11.0).strong());
                let progress_bar = egui::ProgressBar::new(norm_bass)
                    .show_percentage()
                    .fill(Color32::from_rgb(self.viz_color[0], self.viz_color[1], self.viz_color[2]))
                    .animate(is_running);
                ui.add(progress_bar);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Sub-Bass: {:.1} | Flux: {:.1}", bass_val, flux_val))
                            .size(10.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });

                ui.add_space(4.0);

                // Audio Output Device
                ui.label(RichText::new("Capture Device (WASAPI):").size(11.0));
                let dev_label = self
                    .audio_devices
                    .get(self.selected_device_idx)
                    .cloned()
                    .unwrap_or_else(|| "Default Device".to_string());

                egui::ComboBox::from_id_salt("viz_audio_device")
                    .selected_text(&dev_label)
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for (idx, name) in self.audio_devices.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_device_idx, idx, name);
                        }
                    });

                ui.add_space(4.0);

                // Reactive Color
                ui.label(RichText::new("Bass Reactive Color:").size(11.0));
                ui.horizontal(|ui| {
                    ui.color_edit_button_srgb(&mut self.viz_color);
                    let hex = format!("#{:02X}{:02X}{:02X}", self.viz_color[0], self.viz_color[1], self.viz_color[2]);
                    ui.label(RichText::new(hex).monospace().size(11.0));
                });

                ui.add_space(4.0);

                // Frequency Window Sliders
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("F-Min: {:.1} Hz", self.viz_fmin)).size(10.5));
                        ui.add(egui::Slider::new(&mut self.viz_fmin, 1.0..=40.0));
                    });
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("F-Max: {:.1} Hz", self.viz_fmax)).size(10.5));
                        ui.add(egui::Slider::new(&mut self.viz_fmax, 10.0..=100.0));
                    });
                });

                ui.add_space(4.0);

                // Decay speed & Min brightness
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("Decay: {:.2}", self.viz_decay)).size(10.5));
                        ui.add(egui::Slider::new(&mut self.viz_decay, 0.50..=0.98));
                    });
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("Min Idle: {}/255", self.viz_min_brightness)).size(10.5));
                        ui.add(egui::Slider::new(&mut self.viz_min_brightness, 0..=128));
                    });
                });

                ui.add_space(4.0);

                // Sync GPU toggle
                ui.checkbox(&mut self.viz_sync_gpu, RichText::new("Sync GPU with Visualizer").size(11.0));

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                // Big Start / Stop Button
                let (btn_text, btn_color) = if is_running {
                    ("⏹ STOP VISUALIZER", Color32::from_rgb(239, 68, 68))
                } else {
                    ("▶ START VISUALIZER", Color32::from_rgb(236, 72, 153))
                };

                let viz_btn = egui::Button::new(RichText::new(btn_text).strong())
                    .fill(btn_color)
                    .min_size(Vec2::new(ui.available_width() - 8.0, 30.0));

                if ui.add(viz_btn).clicked() {
                    self.toggle_visualizer();
                }
            });
        });
    }
}
