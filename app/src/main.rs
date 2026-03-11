mod app;
mod tabs;

pub use app::ProjectApp;
pub use tabs::{gantt, project, resources, task};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Project Manager",
        options,
        Box::new(|_cc| Ok(Box::new(app::ProjectApp::default()))),
    )
}
