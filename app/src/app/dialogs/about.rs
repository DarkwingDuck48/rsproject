use crate::app::ProjectApp;
use eframe::egui;
use std::env;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new("О программе")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("О программе!");
            ui.label(format!(
                "Версия: {}",
                env::var("CARGO_PKG_VERSION").unwrap_or_default()
            ));
        });
    if !open {
        app.show_about_dialog = false;
    }
}
