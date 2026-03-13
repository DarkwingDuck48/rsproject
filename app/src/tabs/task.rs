use crate::ProjectApp;
use eframe::egui::{self, Ui};
use logic::{BasicGettersForStructures, ProjectContainer, TaskService};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Задачи");
    if ui.button("➕ Новая задача").clicked() {
        app.show_new_task_dialog = true
    }
    ui.separator();

    // Получаем текущий проект (если есть)
    if !app.container.list_projects().is_empty() {
        let task_service = TaskService::new(&mut app.container);
        let project = task_service
            .get_project(&app.selected_project_id.unwrap())
            .unwrap();
        let project_name = project.name.clone();
        let tasks = task_service.get_tasks(project.get_id());
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
                    ui.label("Task Cost");
                    ui.label("Status");
                    ui.label("Actions");
                    ui.end_row();

                    for task in tasks {
                        ui.label(&project_name);
                        ui.label(&task.name);
                        ui.label(task.get_date_start().format("%Y-%m-%d").to_string());
                        ui.label(task.get_date_end().format("%Y-%m-%d").to_string());
                        match task_service.calculate_task_cost(project.get_id(), task.get_id()) {
                            Ok(cost) => ui.label(format!("{:.2}", cost)),
                            Err(e) => ui.colored_label(egui::Color32::RED, format!("ERR: {}", e)),
                        };
                        ui.label(format!("{:?}", task.get_status()));

                        // Кнопка назначения ресурса (пока заглушка)
                        if ui.button("Assign").clicked() {
                            app.selected_task_id = Some(*task.get_id());
                            app.assign_custom_start = task.get_date_start().date_naive();
                            app.assign_custom_end = task.get_date_end().date_naive();
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
