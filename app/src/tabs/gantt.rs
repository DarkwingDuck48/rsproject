use crate::ProjectApp;
use eframe::egui::{self, Ui};
use egui::Color32;
use logic::{BasicGettersForStructures, ProjectContainer};
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
    ui.heading("Диаграмма Ганта");

    if app.container.list_projects().is_empty() {
        ui.label("Нет загруженного проекта. Сначала создайте проект.");
        return;
    }

    let project_id = *app.selected_project_id.as_ref().unwrap();
    let critical_path = app.critical_path.clone().unwrap_or_default();

    let tasks_data = collect_gantt_data(&mut app.container, project_id, &critical_path);
    if tasks_data.is_empty() {
        ui.label("Нет задач. Создайте задачи на вкладке Tasks.");
        return;
    }

    let min_date = tasks_data.iter().map(|t| t.start_date).min().unwrap();
    let max_date = tasks_data.iter().map(|t| t.end_date).max().unwrap();
    let total_days = (max_date - min_date).num_days() as f32;

    // Увеличиваем масштаб и размеры для лучшей читаемости
    let pixels_per_day = 8.0;
    let row_height = 30.0;
    let left_panel_width = 250.0;
    let chart_width = total_days * pixels_per_day;
    let chart_height = tasks_data.len() as f32 * row_height;
    let scale_height = 30.0; // увеличиваем высоту для шкалы

    ui.horizontal(|ui| {
        // Левая панель – список задач
        ui.vertical(|ui| {
            ui.set_width(left_panel_width);
            // Отступ сверху, чтобы сравнять с правой панелью
            ui.add_space(scale_height);
            for task in &tasks_data {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(left_panel_width, row_height),
                    egui::Sense::hover(),
                );
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                child_ui.horizontal(|ui| {
                    ui.add_space(task.depth as f32 * 15.0);
                    if task.is_summary {
                        ui.colored_label(egui::Color32::GOLD, &task.name);
                    } else {
                        ui.label(&task.name);
                    }
                });
            }
        });
        // Правая панель – диаграмма + шкала
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(chart_width, chart_height + scale_height),
            egui::Sense::hover(),
        );

        let origin = response.rect.min;
        let chart_top = origin.y + scale_height;

        // Рисуем сетку (вертикальные линии каждые 5 дней)
        let days_total = (max_date - min_date).num_days();
        for day in 0..=days_total {
            let x = origin.x + (day as f32 * pixels_per_day);
            if day % 5 == 0 {
                painter.line_segment(
                    [
                        egui::pos2(x, chart_top),
                        egui::pos2(x, origin.y + chart_height + scale_height),
                    ],
                    egui::Stroke::new(1.0, Color32::GRAY), // увеличиваем толщину линии
                );
                // Подпись даты – используем средний шрифт
                let date = min_date + chrono::Duration::days(day);
                let text = date.format("%d/%m").to_string();
                painter.text(
                    egui::pos2(x, origin.y + scale_height - 5.0),
                    egui::Align2::CENTER_BOTTOM,
                    text,
                    egui::TextStyle::Body.resolve(ui.style()),
                    Color32::WHITE,
                );
            }
        }

        // Рисуем полоски задач
        for (i, task) in tasks_data.iter().enumerate() {
            let x_start = (task.start_date - min_date).num_days() as f32 * pixels_per_day;
            let width = (task.end_date - task.start_date).num_days() as f32 * pixels_per_day;
            let y = chart_top + i as f32 * row_height + 4.0; // увеличиваем отступ сверху

            let rect = egui::Rect::from_min_size(
                origin + egui::vec2(x_start, y),
                egui::vec2(width, row_height - 8.0),
            );

            let color = if task.is_critical {
                Color32::RED
            } else if task.is_summary {
                Color32::from_rgb(200, 200, 0)
            } else {
                Color32::LIGHT_BLUE
            };

            painter.rect_filled(rect, 4.0, color); // увеличиваем скругление
        }
        painter.rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(1.0, Color32::GRAY),
            egui::StrokeKind::Outside,
        )
    });
}

// collect_gantt_data остается без изменений (копируем из предыдущего ответа)
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
