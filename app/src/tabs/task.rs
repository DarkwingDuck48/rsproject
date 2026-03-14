use crate::ProjectApp;
use chrono::{DateTime, Utc};
use eframe::egui::{self, Ui};
use egui_extras::{Column, TableBuilder};
use logic::{BasicGettersForStructures, ProjectContainer, TaskService};
use std::collections::HashMap;
use uuid::Uuid;

// Структура для хранения данных задачи, необходимых для отрисовки
struct TaskViewData {
    id: Uuid,
    name: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    status: String,
    is_summary: bool,
    parent_id: Option<Uuid>,
    cost: f64,
    depth: usize, // вычисляется заранее
}

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Задачи");

    if ui.button("➕ Новая задача").clicked() {
        app.show_new_task_dialog = true;
    }
    ui.separator();

    if app.container.list_projects().is_empty() {
        ui.label("Нет загруженного проекта. Сначала создайте проект.");
        return;
    }

    let project_id = *app.selected_project_id.as_ref().unwrap();

    // ---- Сбор данных и построение плоского списка с глубиной ----
    let mut flat_tasks: Vec<TaskViewData> = Vec::new();
    {
        let task_service = TaskService::new(&mut app.container);
        let all_tasks = task_service.get_all_tasks(project_id);

        // Карта детей и временное хранилище данных задач
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut tasks_data: HashMap<Uuid, TaskViewData> = HashMap::new();

        for task in all_tasks {
            let cost = task_service
                .calculate_task_cost(&project_id, task.get_id())
                .unwrap_or(0.0);
            let data = TaskViewData {
                id: *task.get_id(),
                name: task.name.clone(),
                start_date: *task.get_date_start(),
                end_date: *task.get_date_end(),
                status: format!("{:?}", task.get_status()),
                is_summary: task.is_summary,
                parent_id: task.parent_id,
                cost,
                depth: 0, // временно
            };
            tasks_data.insert(*task.get_id(), data);
            if let Some(parent) = task.parent_id {
                children_map.entry(parent).or_default().push(*task.get_id());
            }
        }

        // Рекурсивная функция для обхода дерева и добавления в плоский список с глубиной
        fn add_with_depth(
            id: Uuid,
            depth: usize,
            tasks_data: &mut HashMap<Uuid, TaskViewData>,
            children_map: &HashMap<Uuid, Vec<Uuid>>,
            flat: &mut Vec<TaskViewData>,
        ) {
            if let Some(mut data) = tasks_data.remove(&id) {
                data.depth = depth;
                flat.push(data);
                if let Some(children) = children_map.get(&id) {
                    // Сортируем детей для стабильного порядка
                    let mut sorted_children = children.clone();
                    sorted_children.sort_by(|a, b| {
                        let a_data = tasks_data.get(a).expect("child data missing");
                        let b_data = tasks_data.get(b).expect("child data missing");
                        a_data
                            .start_date
                            .cmp(&b_data.start_date)
                            .then_with(|| a_data.name.cmp(&b_data.name))
                    });
                    for &child in &sorted_children {
                        add_with_depth(child, depth + 1, tasks_data, children_map, flat);
                    }
                }
            }
        }

        // Находим корневые задачи (parent_id = None) и сортируем их
        let mut root_ids: Vec<Uuid> = tasks_data
            .values()
            .filter(|d| d.parent_id.is_none())
            .map(|d| d.id)
            .collect();

        root_ids.sort_by(|a, b| {
            let a_data = tasks_data.get(a).expect("missing root data");
            let b_data = tasks_data.get(b).expect("missing root data");
            a_data
                .start_date
                .cmp(&b_data.start_date)
                .then_with(|| a_data.name.cmp(&b_data.name))
        });

        for root in root_ids {
            add_with_depth(root, 0, &mut tasks_data, &children_map, &mut flat_tasks);
        }
    } // task_service уничтожен, контейнер освобождён

    if flat_tasks.is_empty() {
        ui.label("Нет задач. Нажмите 'Новая задача' для создания.");
        return;
    }

    // ---- Отрисовка таблицы с фиксированными колонками ----
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(Column::auto_with_initial_suggestion(200.0), 1) // Задача + отступы
        .columns(Column::auto_with_initial_suggestion(100.0), 1) // Начало
        .columns(Column::auto_with_initial_suggestion(100.0), 1) // Окончание
        .columns(Column::auto_with_initial_suggestion(80.0), 1) // Стоимость
        .columns(Column::auto_with_initial_suggestion(100.0), 1) // Статус
        .columns(Column::auto_with_initial_suggestion(100.0), 1) // Действия
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Задача");
            });
            header.col(|ui| {
                ui.strong("Начало");
            });
            header.col(|ui| {
                ui.strong("Окончание");
            });
            header.col(|ui| {
                ui.strong("Стоимость");
            });
            header.col(|ui| {
                ui.strong("Статус");
            });
            header.col(|ui| {
                ui.strong("Действия");
            });
        })
        .body(|body| {
            body.rows(22.0, flat_tasks.len(), |mut row| {
                let task = &flat_tasks[row.index()];
                row.set_overline(task.is_summary);
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(task.depth as f32 * 20.0);
                        if task.is_summary {
                            ui.colored_label(egui::Color32::PURPLE, &task.name);
                        } else {
                            ui.label(&task.name);
                        }
                    });
                });
                row.col(|ui| {
                    ui.label(task.start_date.format("%Y-%m-%d").to_string());
                });
                row.col(|ui| {
                    ui.label(task.end_date.format("%Y-%m-%d").to_string());
                });
                row.col(|ui| {
                    ui.label(format!("{:.2}", task.cost));
                });
                row.col(|ui| {
                    ui.label(&task.status);
                });
                row.col(|ui| {
                    if !task.is_summary {
                        if ui.button("󰀔").clicked() {
                            app.selected_task_id = Some(task.id);
                            app.assign_custom_start = task.start_date.date_naive();
                            app.assign_custom_end = task.end_date.date_naive();
                            app.show_assign_resource_dialog = true;
                        }
                    } else {
                        ui.label(""); // выравнивание
                    }
                    if ui.button("").clicked() {
                        app.open_edit_task_dialog(task.id);
                    }
                    if ui.button("󰩺").clicked() {
                        // удаление
                        let mut task_service = TaskService::new(&mut app.container);
                        if let Err(e) = task_service.delete_task(project_id, task.id) {
                            app.error_message = Some(e.to_string());
                        }
                    }
                });
            });
        });
}
