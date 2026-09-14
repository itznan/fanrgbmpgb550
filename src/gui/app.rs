//! eframe::App main lifecycle and frame rendering.

use std::sync::atomic::Ordering;
use eframe::egui::{self, Vec2};

use super::state::GuiApp;

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_viz_running = self.viz_running.load(Ordering::SeqCst);
        if is_viz_running {
            ctx.request_repaint(); // Smooth 60 FPS repaint while audio visualizer is running
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

                    // 1. TOP HEADER & TELEMETRY
                    self.render_header(ui);

                    ui.add_space(2.0);

                    // 2. QUICK PALETTE BAR
                    self.render_quick_palette(ui);

                    ui.add_space(2.0);

                    // 3. MAIN DASHBOARD: 3-COLUMN CARDS
                    ui.columns(3, |columns| {
                        // COLUMN 1: Motherboard Control
                        self.render_mobo_panel(&mut columns[0]);

                        // COLUMN 2: GPU Control
                        self.render_gpu_panel(&mut columns[1]);

                        // COLUMN 3: WASAPI Audio Visualizer
                        self.render_visualizer_panel(&mut columns[2]);
                    });

                    ui.add_space(4.0);

                    // 4. FOOTER & STATUS BAR
                    self.render_footer(ui);
                });
        });
    }
}
