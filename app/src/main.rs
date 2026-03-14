mod app;
mod tabs;

pub use app::ProjectApp;
pub use tabs::{gantt, project, resources, task};

fn main() -> eframe::Result<()> {
    // Большее и удобное стартовое окно приложения
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([1024.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Project Manager",
        options,
        Box::new(|_cc| Ok(Box::new(app::ProjectApp::default()))),
    )
}
