use crate::ProjectApp;
use eframe::egui::{self, SliderClamping, Ui};
use egui::Color32;
use egui_extras::{Column, TableBuilder};
use logic::{BasicGettersForStructures, ProjectContainer, Scheduler};
use std::collections::HashMap;
use uuid::Uuid;

struct GanttTaskData {
    id: Uuid,
    name: String,
    start_date: chrono::DateTime<chrono::Utc>,
    end_date: chrono::DateTime<chrono::Utc>,
    is_summary: bool,
    is_critical: bool,
    depth: usize,
}

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Диаграмма Ганта (по дням)");

    if app.container.list_projects().is_empty() {
        ui.label("Нет загруженного проекта. Сначала создайте проект.");
        return;
    }

    let project_id = *app.selected_project_id.as_ref().unwrap();

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Рассчитать критический путь").clicked() {
            let result = {
                let scheduler = Scheduler::new(&app.container);
                scheduler.critical_path(project_id)
            };
            match result {
                Ok(path) => {
                    app.critical_path = Some(path);
                    app.error_message = None;
                }
                Err(e) => {
                    app.error_message = Some(format!("Ошибка расчета критического пути: {}", e));
                }
            }
        }

        ui.label("Масштаб (px/день):");
        ui.add(
            egui::Slider::new(&mut app.gantt_day_width, 8.0..=60.0)
                .clamping(SliderClamping::Always),
        );
        ui.checkbox(&mut app.gantt_only_critical, "Только критический путь");
    });

    let critical_path = app.critical_path.clone().unwrap_or_default();
    let tasks_data = collect_gantt_data(&mut app.container, project_id, &critical_path);
    if tasks_data.is_empty() {
        ui.label("Нет задач. Создайте задачи на вкладке Tasks.");
        return;
    }

    let visible_tasks: Vec<&GanttTaskData> = if app.gantt_only_critical {
        tasks_data.iter().filter(|t| t.is_critical).collect()
    } else {
        tasks_data.iter().collect()
    };

    if visible_tasks.is_empty() {
        ui.label("Нет задач на критическом пути.");
        return;
    }

    let min_date = visible_tasks.iter().map(|t| t.start_date).min().unwrap();
    let max_date = visible_tasks.iter().map(|t| t.end_date).max().unwrap();
    let total_days = (max_date - min_date).num_days() as usize;

    let day_width = app.gantt_day_width.max(8.0);
    let left_col_width = 250.0;

    ui.horizontal(|ui| {
        // Левая часть: диаграмма Ганта
        ui.vertical(|ui| {
            // Постараемся занять большую часть ширины и высоты под диаграмму
            let w = ui.available_width();
            ui.set_min_width((w * 0.7).max(300.0));
            ui.set_min_height(720.0);

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .column(Column::exact(left_col_width))
                        .columns(Column::exact(day_width), total_days + 1)
                        .header(25.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Задача");
                            });
                            for day_offset in 0..=total_days {
                                let date = min_date + chrono::Duration::days(day_offset as i64);
                                header.col(|ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(date.format("%d/%m").to_string());
                                    });
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(25.0, visible_tasks.len(), |mut row| {
                                let task = visible_tasks[row.index()];

                                // Колонка с именем задачи (кликабельная)
                                row.col(|ui| {
                                    let mut clicked = false;
                                    ui.horizontal(|ui| {
                                        ui.add_space(task.depth as f32 * 20.0);
                                        if task.is_critical {
                                            ui.colored_label(Color32::RED, "🔴");
                                        }
                                        let selected = app.selected_task_id == Some(task.id);
                                        let label = if task.is_summary {
                                            format!("📁 {}", task.name)
                                        } else {
                                            task.name.clone()
                                        };
                                        if ui.selectable_label(selected, label).clicked() {
                                            clicked = true;
                                        }
                                    });
                                    if clicked {
                                        app.selected_task_id = Some(task.id);
                                    }
                                });

                                // Колонки для каждого дня
                                for day_offset in 0..=total_days {
                                    row.col(|ui| {
                                        let date =
                                            min_date + chrono::Duration::days(day_offset as i64);
                                        let is_active =
                                            date >= task.start_date && date <= task.end_date;

                                        if is_active {
                                            let color = if task.is_critical {
                                                Color32::RED
                                            } else if task.is_summary {
                                                Color32::from_rgb(255, 255, 200)
                                            } else {
                                                Color32::LIGHT_BLUE
                                            };
                                            ui.painter().rect_filled(ui.max_rect(), 0.0, color);
                                        }
                                    });
                                }

                                // row.response() не используем для клика, т.к. клики съедают виджеты в ячейках
                            });
                        });
                });
        });

        ui.separator();

        // Правая часть: панель деталей задачи
        ui.vertical(|ui| {
            ui.set_min_width(260.0);
            ui.heading("Детали задачи");
            if let Some(selected_id) = app.selected_task_id {
                let project_id = *app.selected_project_id.as_ref().unwrap();

                // Сначала читаем задачу и её стоимость, а нужные данные копируем в локальные переменные.
                let (task_name, task_cost, alloc_ids) = {
                    let task_service = logic::TaskService::new(&mut app.container);
                    if let Some(project) = task_service.get_project(&project_id) {
                        if let Some(task) = project.tasks.get(&selected_id) {
                            let name = task.name.clone();
                            let alloc_ids = task.get_resource_allocations().clone();
                            let cost = task_service
                                .calculate_task_cost(&project_id, &selected_id)
                                .unwrap_or(0.0);
                            (Some(name), Some(cost), alloc_ids)
                        } else {
                            (None, None, Vec::new())
                        }
                    } else {
                        (None, None, Vec::new())
                    }
                };

                if let Some(name) = task_name {
                    ui.label(format!("Имя: {}", name));
                }
                if let Some(cost) = task_cost {
                    ui.label(format!("Стоимость задачи: {:.2}", cost));
                }

                ui.separator();
                ui.strong("Назначенные ресурсы:");

                if let Some(calendar) = app.container.calendar(&project_id) {
                    let pool = app.container.resource_pool();
                    for alloc_id in alloc_ids {
                        if let Some(allocation) = pool.get_allocation(&alloc_id)
                            && let Some(resource) = pool.get_resource(allocation.get_resource_id())
                        {
                            let tw = allocation.get_time_window();
                            let hours = tw.duration_hours(calendar) as f64
                                * allocation.get_engagement_rate();
                            let cost = pool
                                .calculate_allocation_cost(&alloc_id, calendar)
                                .unwrap_or(0.0);
                            ui.separator();
                            ui.label(format!("Ресурс: {}", resource.name));
                            ui.label(format!(
                                "Период: {} - {}",
                                tw.date_start.format("%Y-%m-%d"),
                                tw.date_end.format("%Y-%m-%d")
                            ));
                            ui.label(format!("Часы: {:.1}", hours));
                            ui.label(format!("Стоимость ресурса: {:.2}", cost));
                        }
                    }
                }
            } else {
                ui.label("Выберите задачу на диаграмме для просмотра деталей.");
            }
        });
    });
}

fn collect_gantt_data(
    container: &mut logic::SingleProjectContainer,
    project_id: Uuid,
    critical_path: &[Uuid],
) -> Vec<GanttTaskData> {
    let task_service = logic::TaskService::new(container);
    let all_tasks = task_service.get_all_tasks(project_id);

    let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    let mut tasks_data: HashMap<
        Uuid,
        (
            String,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
            bool,
            Option<Uuid>,
        ),
    > = HashMap::new();

    for task in all_tasks {
        tasks_data.insert(
            *task.get_id(),
            (
                task.name.clone(),
                *task.get_date_start(),
                *task.get_date_end(),
                task.is_summary,
                task.parent_id,
            ),
        );
        if let Some(parent) = task.parent_id {
            children_map.entry(parent).or_default().push(*task.get_id());
        }
    }

    fn add_with_depth(
        id: Uuid,
        depth: usize,
        tasks_data: &mut HashMap<
            Uuid,
            (
                String,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
                bool,
                Option<Uuid>,
            ),
        >,
        children_map: &HashMap<Uuid, Vec<Uuid>>,
        critical_path: &[Uuid],
        result: &mut Vec<GanttTaskData>,
    ) {
        if let Some((name, start, end, is_summary, _)) = tasks_data.remove(&id) {
            let is_critical = critical_path.contains(&id);
            result.push(GanttTaskData {
                id,
                name,
                start_date: start,
                end_date: end,
                is_summary,
                is_critical,
                depth,
            });
            if let Some(children) = children_map.get(&id) {
                let mut sorted_children = children.clone();
                sorted_children.sort_by(|a, b| {
                    let a_data = tasks_data.get(a).unwrap();
                    let b_data = tasks_data.get(b).unwrap();
                    a_data
                        .1
                        .cmp(&b_data.1)
                        .then_with(|| a_data.0.cmp(&b_data.0))
                });
                for &child in &sorted_children {
                    add_with_depth(
                        child,
                        depth + 1,
                        tasks_data,
                        children_map,
                        critical_path,
                        result,
                    );
                }
            }
        }
    }

    let mut root_ids: Vec<Uuid> = tasks_data
        .iter()
        .filter(|(_, (_, _, _, _, parent))| parent.is_none())
        .map(|(id, _)| *id)
        .collect();

    root_ids.sort_by(|a, b| {
        let a_data = tasks_data.get(a).unwrap();
        let b_data = tasks_data.get(b).unwrap();
        a_data
            .1
            .cmp(&b_data.1)
            .then_with(|| a_data.0.cmp(&b_data.0))
    });

    let mut result = Vec::new();
    let mut tasks_data_mut = tasks_data;
    for root in root_ids {
        add_with_depth(
            root,
            0,
            &mut tasks_data_mut,
            &children_map,
            critical_path,
            &mut result,
        );
    }

    result
}
