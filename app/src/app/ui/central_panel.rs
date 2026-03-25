use eframe::egui;
use logic::ProjectContainer;

use crate::{
    ProjectApp,
    app::views::{View, gantt, project, resources, task},
};

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.container.list_projects().is_empty() {
            // Приветственный экран, если пусто в контейнере
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("RS Project - Добро пожаловать");
                ui.label("Нет активных проектов");
                if ui.button("Создать проект").clicked() {
                    app.show_new_project_dialog = true;
                }
            });
        } else {
            match app.selected_tab {
                View::Project => project::show(ui, app),
                View::Tasks => task::show(ui, app),
                View::Resources => resources::show(ui, app),
                View::Gantt => gantt::show(ui, app),
            }
        }
        // Отображение ошибки (если есть)
        if let Some(err) = &app.error_message {
            ui.separator();
            ui.colored_label(egui::Color32::RED, err);
        }
    });
}
