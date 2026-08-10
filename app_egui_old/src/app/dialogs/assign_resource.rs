use eframe::egui::{self, Widget};
use logic::ResourceService;

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new("Назначение ресурса на задачу")
        .open(&mut open)
        .show(ctx, |ui| {
            // Выбор ресурса (выпадающий список)
            let resource_service = ResourceService::new(&mut app.container);
            let resources = resource_service.list_resources();

            ui.horizontal(|ui| {
                ui.label("Ресурс:");
                egui::ComboBox::from_id_salt("resource_select")
                    .selected_text(
                        resources
                            .iter()
                            .find(|r| Some(r.id) == app.selected_resource_id)
                            .map(|r| r.name.clone())
                            .unwrap_or_else(|| "Выберите ресурс".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for r in resources {
                            ui.selectable_value(&mut app.selected_resource_id, Some(r.id), &r.name);
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Занятость (0.0-1.0):");
                ui.text_edit_singleline(&mut app.assign_engagement);
            });

            ui.separator();

            ui.checkbox(
                &mut app.assign_use_full_window,
                "Назначить на все время задачи",
            );

            if !app.assign_use_full_window {
                ui.horizontal(|ui| {
                    ui.label("Дата начала работ(измененая):");
                    egui_extras::DatePickerButton::new(&mut app.assign_custom_start)
                        .id_salt("assign_start_picker")
                        .start_end_years(2020..=2035)
                        .ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("Дата окончания работ(измененная):");
                    egui_extras::DatePickerButton::new(&mut app.assign_custom_end)
                        .id_salt("assign_end_picker")
                        .start_end_years(2020..=2035)
                        .ui(ui);
                });
            }

            if ui.button("Назначить").clicked() {
                match app.assing_resource() {
                    Ok(()) => {
                        app.show_assign_resource_dialog = false;
                        app.selected_task_id = None;
                        app.selected_resource_id = None;
                        app.error_message = None;
                        app.assign_engagement = String::from("0.5");
                        app.assign_use_full_window = true;
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            }
        });
    if !open {
        app.show_assign_resource_dialog = false;
    }
}
