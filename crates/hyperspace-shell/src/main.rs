mod app;
mod canvas;
mod palette;
mod theme;

use app::HyperspaceApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OS/3 Hyperspace")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OS/3 Hyperspace",
        native_options,
        Box::new(|cc| Ok(Box::new(HyperspaceApp::new(cc)))),
    )
}
