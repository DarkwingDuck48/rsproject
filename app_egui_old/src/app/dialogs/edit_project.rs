use eframe::egui::{self, Widget};

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new("Редактировать проект")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Имя проекта");
                ui.text_edit_singleline(&mut app.new_project_name);
            });
            ui.horizontal(|ui| {
                ui.label("Описание проекта");
                ui.text_edit_singleline(&mut app.new_project_desc);
            });
            ui.horizontal(|ui| {
                ui.label("Дата начала:");
                egui_extras::DatePickerButton::new(&mut app.new_project_start)
                    .id_salt("edit_project_start")
                    .ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Дата окончания:");
                egui_extras::DatePickerButton::new(&mut app.new_project_end)
                    .id_salt("edit_project_end")
                    .ui(ui);
            });
            if ui.button("Сохранить").clicked() {
                match app.update_project() {
                    Ok(()) => {
                        app.show_edit_project_dialog = false;
                        app.error_message = None;
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            }
        });
    if !open {
        app.show_edit_project_dialog = false;
    }
}
