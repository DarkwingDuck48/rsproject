use crate::ProjectApp;
use eframe::egui::Ui;
use logic::{BasicGettersForStructures, ProjectContainer};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Project info");

    if ui.button("➕ Новый проект").clicked() {
        app.show_new_project_dialog = true;
    }

    ui.separator();

    if let Some(project) = app.container.list_projects().first() {
        ui.label(format!("Название: {}", project.name));
        ui.label(format!("Описание: {}", project.description));
        ui.label(format!("Дата начала проекта: {}", project.get_date_start()));
        ui.label(format!(
            "Дата окончания проекта: {}",
            project.get_date_end()
        ));
    } else {
        ui.label("Нет загруженных проектов");
    }
}
