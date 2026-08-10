use eframe::egui::{self, Widget};

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;

    egui::Window::new("Создать новый проект")
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
                ui.label("Дата начала проекта:");
                egui_extras::DatePickerButton::new(&mut app.new_project_start)
                    .id_salt("start_project_date")
                    .start_end_years(2020..=2035)
                    .ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Дата окончания проекта:");
                egui_extras::DatePickerButton::new(&mut app.new_project_end)
                    .id_salt("end_project_date")
                    .start_end_years(2020..=2035)
                    .ui(ui);
            });
            if ui.button("Создать проект").clicked() {
                match app.create_project() {
                    Ok(_) => {
                        app.show_new_project_dialog = false;
                        app.clear_new_project_fields();
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            }
        });
    if !open {
        app.show_new_project_dialog = false;
    }
}
