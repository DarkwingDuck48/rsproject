use crate::ProjectApp;
use eframe::egui::{self, Ui};
use logic::{BasicGettersForStructures, ProjectContainer};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Задачи");
    if ui.button("➕ Новая задача").clicked() {
        app.show_new_task_dialog = true
    }
    ui.separator();

    // Получаем текущий проект (если есть)
    if let Some(project) = app.container.list_projects().first() {
        // Создаём сервис задач для чтения (не мутабельно, так как мы только читаем)
        // Для чтения можно использовать &container, но TaskService требует &mut, поэтому создадим временный сервис
        // Можно обойтись без сервиса, просто получить задачи из проекта.
        let tasks = project.get_project_tasks();
        let project_name = project.name.clone();

        if tasks.is_empty() {
            ui.label("No tasks yet. Click 'Add Task' to create one.");
        } else {
            egui::Grid::new("tasks_grid")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Project");
                    ui.label("Name");
                    ui.label("Start");
                    ui.label("End");
                    ui.label("Status");
                    ui.label("Actions");
                    ui.end_row();

                    for task in tasks {
                        ui.label(&project_name);
                        ui.label(&task.name);
                        ui.label(task.get_date_start().format("%Y-%m-%d").to_string());
                        ui.label(task.get_date_end().format("%Y-%m-%d").to_string());
                        ui.label(format!("{:?}", task.get_status()));

                        // Кнопка назначения ресурса (пока заглушка)
                        if ui.button("Assign").clicked() {
                            app.selected_task_id = Some(*task.get_id());
                            app.show_assign_resource_dialog = true;
                        }
                        ui.end_row();
                    }
                });
        }
    } else {
        ui.label("No project loaded. Create a project first.");
    }
}
