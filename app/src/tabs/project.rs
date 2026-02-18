use crate::ProjectApp;
use eframe::egui::Ui;
use logic::{BasicGettersForStructures, ProjectContainer};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Project info");
    if let Some(project) = app.container.list_project().first() {
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
