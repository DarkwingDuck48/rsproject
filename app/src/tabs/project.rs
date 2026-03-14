use crate::ProjectApp;
use eframe::egui::{self, Ui};
use logic::{BasicGettersForStructures, ProjectContainer, TaskService};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.horizontal(|ui| {
        ui.heading("📋 Информация о проекте");
        if ui.button("✏️ Редактировать").clicked() {
            app.open_edit_project_dialog();
        }
    });
    ui.separator();

    let resources_count = app.container.resource_pool().get_resources().len();
    let project_id = *app.selected_project_id.as_ref().unwrap();
    let (regular_count, summary_count, total_cost) = {
        let task_service = TaskService::new(&mut app.container);
        let all_tasks = task_service.get_all_tasks(project_id);
        let regular = all_tasks.iter().filter(|t| !t.is_summary).count();
        let summary = all_tasks.iter().filter(|t| t.is_summary).count();
        let cost = task_service
            .calculate_project_cost(project_id)
            .unwrap_or(0.0);
        (regular, summary, cost)
    };

    if app.selected_project_id.is_some() {
        let project = app.container.get_project(&project_id).unwrap();
        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.heading("📌 Основное");
                egui::Grid::new("project_main_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("📁 Название:");
                        ui.label(&project.name);
                        ui.end_row();
                        ui.label("📝 Описание:");
                        ui.label(&project.description);
                        ui.end_row();
                        ui.label("📅 Дата начала:");
                        ui.label(project.get_date_start().format("%Y-%m-%d").to_string());
                        ui.end_row();
                        ui.label("📅 Дата окончания:");
                        ui.label(project.get_date_end().format("%Y-%m-%d").to_string());
                        ui.end_row();
                    });
            });

            ui.add_space(8.0);
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.heading("📊 Статистика");
                egui::Grid::new("project_stats_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("📋 Всего задач:");
                        ui.label(format!("{}", regular_count + summary_count));
                        ui.end_row();
                        ui.label("📄 Обычных задач:");
                        ui.label(format!("{}", regular_count));
                        ui.end_row();
                        ui.label("📁 Групп:");
                        ui.label(format!("{}", summary_count));
                        ui.end_row();
                        ui.label("👤 Ресурсов:");
                        ui.label(format!("{}", resources_count));
                        ui.end_row();
                        ui.label("💰 Общая стоимость:");
                        ui.label(
                            egui::RichText::new(format!("{:.2}", total_cost))
                                .color(egui::Color32::DARK_GREEN)
                                .strong(),
                        );
                        ui.end_row();
                    });
            });
        });
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label("⚠️ Нет загруженного проекта.");
            ui.add_space(8.0);
            ui.label("Создайте или загрузите проект через меню File.");
        });
    }
}
