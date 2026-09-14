//! Native single-page GUI module for FanRGB.

pub mod actions;
pub mod app;
pub mod panels;
pub mod state;
pub mod theme;
pub mod zone;

use eframe::egui;

pub use state::GuiApp;
pub use theme::configure_custom_styles;

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("FanRGB Hardware Control Center | MSI B550 & GPU"),
        ..Default::default()
    };

    eframe::run_native(
        "FanRGB Control Center",
        options,
        Box::new(|cc| {
            configure_custom_styles(&cc.egui_ctx);
            Ok(Box::new(GuiApp::new()))
        }),
    )
}
