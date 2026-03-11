use crate::app::ProjectApp;
use eframe::egui::{self, Ui};
use logic::{BasicGettersForStructures, ProjectContainer, Scheduler};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Gantt Chart (Table View)");

    if let Some(project) = app.container.list_projects().first() {
        let project_id = *project.get_id();
        let tasks = project.get_project_tasks();

        if tasks.is_empty() {
            ui.label("No tasks yet.");
            return;
        }

        // Кнопка для расчёта критического пути
        if ui.button("Calculate Critical Path").clicked() {
            let scheduler = Scheduler::new(&app.container);
            match scheduler.critical_path(project_id) {
                Ok(path) => {
                    app.critical_path = Some(path);
                    app.error_message = None;
                }
                Err(e) => {
                    app.error_message = Some(e.to_string());
                    app.critical_path = None;
                }
            }
        }

        ui.separator();

        // Таблица задач
        egui::Grid::new("gantt_grid")
            .num_columns(6)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Task");
                ui.label("Start");
                ui.label("End");
                ui.label("Duration (days)");
                ui.label("Resources");
                ui.label("Critical");
                ui.end_row();

                for task in tasks {
                    // Вычисляем длительность в днях
                    let duration_days = task.get_duration().num_days();

                    // Собираем имена ресурсов, назначенных на задачу
                    let resource_names: Vec<String> = task
                        .get_resource_allocations()
                        .iter()
                        .filter_map(|alloc_id| {
                            // Получаем аллокацию из пула ресурсов (нужен доступ к ResourcePool)
                            // Пока упростим: будем хранить ID, но для отображения имён нужен доступ к ресурсам.
                            // Лучше передать ResourceService, но для простоты пока оставим заглушку.
                            // В реальном коде нужно получить ResourcePool из контейнера.
                            None // временно
                        })
                        .collect();

                    let resources_str = if resource_names.is_empty() {
                        format!("{} alloc(s)", task.get_resource_allocations().len())
                    } else {
                        format!("{} alloc(s)", task.get_resource_allocations().len())
                    };

                    // Проверка, находится ли задача на критическом пути
                    let is_critical = app
                        .critical_path
                        .as_ref()
                        .map(|path| path.contains(task.get_id()))
                        .unwrap_or(false);

                    ui.label(&task.name);
                    ui.label(task.get_date_start().format("%Y-%m-%d").to_string());
                    ui.label(task.get_date_end().format("%Y-%m-%d").to_string());
                    ui.label(duration_days.to_string());
                    ui.label(resources_str);
                    if is_critical {
                        ui.colored_label(egui::Color32::RED, "✔");
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            });
    } else {
        ui.label("No project loaded.");
    }
}
