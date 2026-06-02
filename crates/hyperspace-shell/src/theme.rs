use eframe::egui;

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(10, 14, 24);
    visuals.panel_fill = egui::Color32::from_rgb(16, 20, 34);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 26, 42);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(28, 36, 58);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 52, 82);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(56, 78, 120);
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(96, 165, 250, 80);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    ctx.set_style(style);
}
