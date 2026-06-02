use eframe::egui;

/// Applies a premium, futuristic "Hyperspace" dark-neon theme inspired by high-end
/// sci-fi OS mockups (deep space blacks, vibrant purple/cyan/magenta neon accents,
/// subtle glassmorphism via layered fills + borders, clean modern typography).
/// This is the foundation for making the prototype look modern and attractive
/// rather than "basic egui app".
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Deep space / nebula background palette (matches the reference image's cosmic darks)
    visuals.window_fill = egui::Color32::from_rgb(6, 8, 16);          // Very deep navy/black
    visuals.panel_fill = egui::Color32::from_rgb(10, 12, 22);         // Slightly lighter panels for glass effect
    visuals.extreme_bg_color = egui::Color32::from_rgb(4, 5, 12);     // Near-black for cards/inner content

    // Widget states with neon-tinged hover/active for premium feel
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(14, 16, 28);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(18, 20, 36);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(28, 32, 58);   // Subtle lift
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(38, 44, 78);

    // Strong neon selection (cyan-purple vibe from the mockup logo/accents)
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(120, 80, 255, 70); // Purple glow
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 140, 255));

    // Text and icon colors for high contrast on dark
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 225, 240));
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 205, 225));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 245, 255));
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 200, 255)); // Neon pop

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();

    // Tighter, modern spacing like the reference UI
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin { left: 8, right: 8, top: 6, bottom: 6 };

    // Premium typography (larger, clean headings + good body)
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(12.0, egui::FontFamily::Monospace),
    );

    // Rounding uses egui 0.31+ defaults / CornerRadius (direct assignment changed in this version).
    // The overall dark-neon glass aesthetic is still achieved via fills, strokes, and spacing.

    ctx.set_style(style);
}
