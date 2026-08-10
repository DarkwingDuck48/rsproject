use eframe::egui;
use logic::{BasicGettersForStructures, ProjectContainer};

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new("Детали задачи")
        .open(&mut open)
        .show(ctx, |ui| {
            if let Some(task_id) = app.details_task_id {
                let project_id = *app.selected_project_id.as_ref().unwrap();

                let (task_name, task_cost, alloc_ids, task_start, task_end) = {
                    let task_service = logic::TaskService::new(&mut app.container);
                    if let Some(project) = task_service.get_project(&project_id) {
                        if let Some(task) = project.tasks.get(&task_id) {
                            let name = task.name.clone();

                            let alloc_ids = if task.is_summary {
                                task_service.get_task_allocations(&project_id, *task.get_id())
                            } else {
                                task.get_resource_allocations().clone()
                            };

                            let cost = task_service
                                .calculate_task_cost(&project_id, &task_id)
                                .unwrap_or(0.0);
                            (
                                Some(name),
                                Some(cost),
                                alloc_ids,
                                Some(*task.get_date_start()),
                                Some(*task.get_date_end()),
                            )
                        } else {
                            (None, None, Vec::new(), None, None)
                        }
                    } else {
                        (None, None, Vec::new(), None, None)
                    }
                };
                if let Some(name) = task_name {
                    ui.label(format!("Имя: {}", name));
                }
                if let Some(cost) = task_cost {
                    ui.label(format!("Стоимость задачи: {:.2}", cost));
                }
                if let Some(start) = task_start {
                    ui.label(format!("Начало задачи: {}", start.format("%Y-%m-%d")));
                }
                if let Some(end) = task_end {
                    ui.label(format!("Окончание задачи : {}", end.format("%Y-%m-%d")));
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
                                "Период занятости ресурса: {} - {}",
                                tw.date_start.format("%Y-%m-%d"),
                                tw.date_end.format("%Y-%m-%d")
                            ));
                            ui.label(format!(
                                "Занятость: {:.0}%",
                                allocation.get_engagement_rate() * 100.0
                            ));
                            ui.label(format!("Часы: {:.1}", hours));
                            ui.label(format!("Стоимость ресурса: {:.2}", cost));
                        }
                    }
                }
            } else {
                ui.label("Задача не выбрана");
            }
        });
    if !open {
        app.show_task_details_dialog = false;
        app.details_task_id = None;
    }
}
