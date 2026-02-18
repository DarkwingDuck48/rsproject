use crate::ProjectApp;
use eframe::egui::Ui;
use logic::{BasicGettersForStructures, ProjectContainer};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Ресурсы");
    ui.label("Нет ресурсов проекта");
}
