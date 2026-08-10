use eframe::egui::{self, RichText, Widget};
use logic::{BasicGettersForStructures, DependencyType, ProjectContainer};

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new(if app.edit_resource_id.is_some() {
        "Редактировать задачу"
    } else {
        "Добавление задачи"
    })
    .open(&mut open)
    .show(ctx, |ui| {
        ui.text_edit_singleline(&mut app.new_task_name);
        ui.horizontal(|ui| ui.checkbox(&mut app.new_task_is_summary, "Группирующая задача"));

        ui.add_enabled_ui(!app.new_task_is_summary, |ui| {
            ui.horizontal(|ui| {
                ui.label("Начало задачи:");
                egui_extras::DatePickerButton::new(&mut app.new_task_start)
                    .id_salt("task_start_picker")
                    .start_end_years(2020..=2035)
                    .ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Окончание задачи:");
                egui_extras::DatePickerButton::new(&mut app.new_task_end)
                    .id_salt("task_end_picker")
                    .start_end_years(2020..=2035)
                    .ui(ui);
            })
        });
        ui.separator();
        ui.label(RichText::from("Зависимости").strong());
        if let Some(project) = app.container.list_projects().first() {
            let tasks = project.get_project_tasks();
            ui.vertical(|ui| {
                ui.label("Родительская задача:");
                egui::ComboBox::from_id_salt("parent_task_combo")
                    .selected_text(
                        app.selected_task_parent_id
                            .and_then(|id| tasks.iter().find(|t| t.get_id() == &id))
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| "Нет родителя".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.selected_task_parent_id, None, "Нет родителя");
                        for task in tasks.clone() {
                            // Можно добавить отображение типа задачи (например, 📁 для суммарной)
                            if task.is_summary {
                                let display_name = format!("📁 {}", task.name);

                                ui.selectable_value(
                                    &mut app.selected_task_parent_id,
                                    Some(*task.get_id()),
                                    display_name,
                                );
                            }
                        }
                    });
                ui.separator();

                egui::ComboBox::from_id_salt("dependent_task_combo")
                    .selected_text(
                        app.new_task_dependency_task
                            .and_then(|id| tasks.iter().find(|t| t.get_id() == &id))
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| "Нет зависимой задачи".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut app.new_task_dependency_task,
                            None,
                            "Нет зависимой задачи",
                        );
                        for task in tasks {
                            let display_name = if task.is_summary {
                                format!("📁 {}", task.name)
                            } else {
                                task.name.clone()
                            };
                            ui.selectable_value(
                                &mut app.new_task_dependency_task,
                                Some(*task.get_id()),
                                display_name,
                            );
                        }
                    });
                // TODO: Мы должны отобразить зависимости в виде таблицы, так как у нас может быть не одна зависимость
                ui.label("Тип зависимости задачи:");
                let selected_text = match app.new_task_dependency_type {
                    Some(DependencyType::Blocking) => "Блокирующая",
                    Some(DependencyType::NonBlocking) => "Неблокирующая",
                    None => "Не выбрано", // или "Выберите тип"
                };
                egui::ComboBox::from_id_salt("dependent_type_task_combo")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut app.new_task_dependency_type,
                            Some(DependencyType::Blocking),
                            "Блокирующая",
                        );
                        ui.selectable_value(
                            &mut app.new_task_dependency_type,
                            Some(DependencyType::NonBlocking),
                            "Неблокирующая",
                        );
                    });
            });
        }

        if ui.button("Сохранить").clicked() {
            match app.create_task() {
                Ok(()) => {
                    app.show_new_task_dialog = false;
                    app.error_message = None;
                }
                Err(e) => app.error_message = Some(e.to_string()),
            }
        }
    });
    if !open {
        app.show_new_task_dialog = false;
    }
}
