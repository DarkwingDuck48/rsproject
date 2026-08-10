use eframe::egui;

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new("Закрыть проект")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("Сохранить изменения перед закрытием?");
            ui.horizontal(|ui| {
                if ui.button("Сохранить и закрыть").clicked() {
                    app.close_project_with_save();
                    app.show_close_project_dialog = false;
                }
                if ui.button("Закрыть без сохранения").clicked() {
                    app.close_project_no_save();
                    app.show_close_project_dialog = false;
                }
                if ui.button("Отмена").clicked() {
                    app.show_close_project_dialog = false;
                }
            });
        });
    if !open {
        app.show_close_project_dialog = false;
    }
}
