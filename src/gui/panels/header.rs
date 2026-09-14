//! Top header and hardware telemetry status bar panel.

use eframe::egui::{self, Color32, RichText};

use crate::gui::state::GuiApp;
use crate::gui::theme::card_frame;

impl GuiApp {
    pub fn render_header(&mut self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                // Title + Version
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(
                            RichText::new("FANRGB")
                                .color(Color32::from_rgb(59, 130, 246))
                                .strong(),
                        );
                        ui.heading(
                            RichText::new("CONTROL CENTER")
                                .color(Color32::from_rgb(241, 245, 249))
                                .strong(),
                        );
                    });
                    ui.label(
                        RichText::new("MSI MPG B550 GAMING PLUS & GIGABYTE GPU RGB CONTROLLER")
                            .size(10.5)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Global Blackout button
                    let off_btn = ui.button(
                        RichText::new("⏹ BLACKOUT ALL")
                            .color(Color32::from_rgb(239, 68, 68))
                            .strong(),
                    );
                    if off_btn.clicked() {
                        self.blackout_all();
                    }

                    // Global Quick Sync button
                    let sync_btn = ui.button(
                        RichText::new("⚡ SYNC ALL")
                            .color(Color32::from_rgb(96, 165, 250))
                            .strong(),
                    );
                    if sync_btn.clicked() {
                        self.sync_all(self.master_color);
                    }

                    // Rescan Hardware button
                    if ui.button(RichText::new("🔄 RESCAN").size(11.0)).clicked() {
                        self.reconnect_hardware();
                    }

                    // Safety badge
                    ui.label(
                        RichText::new("🛡 RAM VOLATILE (0 WEAR)")
                            .size(10.0)
                            .color(Color32::from_rgb(52, 211, 153)),
                    );
                });
            });

            ui.separator();

            // Device Status Indicators Bar
            ui.horizontal(|ui| {
                // Motherboard status pill
                if self.mobo_connected {
                    ui.label(
                        RichText::new("● MSI B550 CONNECTED")
                            .size(11.0)
                            .color(Color32::from_rgb(34, 197, 94))
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("({})", self.mobo_fw_info))
                            .size(10.5)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                } else {
                    ui.label(
                        RichText::new("○ MSI B550 OFFLINE")
                            .size(11.0)
                            .color(Color32::from_rgb(239, 68, 68))
                            .strong(),
                    );
                }

                ui.label(RichText::new("|").color(Color32::from_rgb(51, 65, 85)));

                // GPU status pill
                if self.gpu_connected {
                    ui.label(
                        RichText::new("● GPU CONNECTED")
                            .size(11.0)
                            .color(Color32::from_rgb(34, 197, 94))
                            .strong(),
                    );
                    let addr_str = self
                        .gpu_i2c_addr
                        .map(|a| format!("0x{:02X}", a))
                        .unwrap_or_else(|| "Unknown".to_string());
                    ui.label(
                        RichText::new(format!("{} (I2C {})", self.gpu_model_name, addr_str))
                            .size(10.5)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                } else {
                    ui.label(
                        RichText::new("○ GPU OFFLINE")
                            .size(11.0)
                            .color(Color32::from_rgb(239, 68, 68))
                            .strong(),
                    );
                }

                // Live Preview toggle
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.live_preview, RichText::new("Instant Live Apply").size(11.5));
                });
            });
        });
    }
}
