//! GUI Theme, dark mode styling, and frame widgets.

use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke};

pub fn configure_custom_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(226, 232, 240));
    visuals.panel_fill = Color32::from_rgb(15, 17, 23);
    visuals.window_fill = Color32::from_rgb(21, 24, 33);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 30, 42);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(32, 37, 51);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 52, 71);
    visuals.widgets.active.bg_fill = Color32::from_rgb(37, 99, 235);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0f32, Color32::from_rgb(42, 48, 66));
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0f32, Color32::from_rgb(42, 48, 66));
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0f32, Color32::from_rgb(59, 130, 246));
    visuals.widgets.active.bg_stroke = Stroke::new(1.0f32, Color32::from_rgb(96, 165, 250));
    visuals.selection.bg_fill = Color32::from_rgb(37, 99, 235);
    visuals.selection.stroke = Stroke::new(1.0f32, Color32::from_rgb(96, 165, 250));

    ctx.set_visuals(visuals);
}

pub fn card_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgb(22, 25, 34))
        .stroke(Stroke::new(1.0f32, Color32::from_rgb(38, 43, 59)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(14.0))
}
