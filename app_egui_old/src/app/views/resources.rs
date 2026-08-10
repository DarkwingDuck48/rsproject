use crate::ProjectApp;
use eframe::egui::{self, Ui};
use egui_extras::{Column, TableBuilder};
use logic::{ProjectContainer, RateMeasure, ResourceService};
use uuid::Uuid;

// Структура для хранения данных ресурса для отображения
struct ResourceViewData {
    id: Uuid,
    name: String,
    rate: f64,
    rate_measure: RateMeasure,
    utilization: f64,
    unavail_count: usize,
}

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Ресурсы");

    if ui.button("➕ Добавить ресурс").clicked() {
        app.show_new_resource_dialog = true;
    }
    ui.separator();

    if app.container.list_projects().is_empty() {
        ui.label("Нет загруженного проекта. Сначала создайте проект.");
        return;
    }

    // Собираем все данные для отображения, копируя нужные поля
    let resources_data = {
        let resource_service = ResourceService::new(&mut app.container);
        let resources = resource_service.list_resources();
        let mut data = Vec::with_capacity(resources.len());
        for resource in resources {
            let utilization = resource_service
                .calculate_resource_utilization(
                    resource.id,
                    app.selected_project_id.expect("Не выбран проект"),
                )
                .unwrap_or(0.0);

            let unavail_count = resource.get_unavailable_periods().len();
            data.push(ResourceViewData {
                id: resource.id,
                name: resource.name.clone(),
                rate: *resource.get_base_rate(),
                rate_measure: resource.get_rate_measure().clone(),
                utilization,
                unavail_count,
            });
        }
        data
    }; // resource_service уничтожен, данные скопированы

    if resources_data.is_empty() {
        ui.label("Ресурсов не создано. Нажмите 'Добавить ресурс' для создания.");
        return;
    }

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(Column::auto_with_initial_suggestion(200.0), 1) // Имя
        .columns(Column::auto_with_initial_suggestion(80.0), 1) // Ставка
        .columns(Column::auto_with_initial_suggestion(100.0), 1) // Тип ставки
        .columns(Column::auto_with_initial_suggestion(80.0), 1) // Утилизация
        .columns(Column::auto_with_initial_suggestion(140.0), 1) // Периоды недоступности
        .columns(Column::auto_with_initial_suggestion(180.0), 1) // Действия
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Ресурс");
            });
            header.col(|ui| {
                ui.strong("Ставка");
            });
            header.col(|ui| {
                ui.strong("Тип ставки");
            });
            header.col(|ui| {
                ui.strong("Утилизация");
            });
            header.col(|ui| {
                ui.strong("Недоступность");
            });
            header.col(|ui| {
                ui.strong("Действия");
            });
        })
        .body(|body| {
            body.rows(22.0, resources_data.len(), |mut row| {
                let data = &resources_data[row.index()];

                row.col(|ui| {
                    ui.label(&data.name);
                });
                row.col(|ui| {
                    ui.label(format!("{:.2}", data.rate));
                });
                row.col(|ui| {
                    ui.label(format!("{:?}", data.rate_measure));
                });
                row.col(|ui| {
                    ui.label(format!("{:.1}%", data.utilization * 100.0));
                });
                row.col(|ui| {
                    if data.unavail_count > 0 {
                        ui.label(format!("{} периодов", data.unavail_count));
                    } else {
                        ui.label("нет");
                    }
                });
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("").clicked() {
                            app.selected_resource_id = Some(data.id);
                            app.show_unavailable_period_dialog = true;
                        }
                        if ui.button("").clicked() {
                            app.open_edit_resource_dialog(data.id);
                        }
                        if ui.button("󰩺").clicked() {
                            // Создаём новый сервис для мутабельной операции
                            let mut resource_service = ResourceService::new(&mut app.container);
                            if let Err(e) = resource_service.delete_resource(data.id) {
                                app.error_message = Some(e.to_string());
                            }
                        }
                    });
                });
            });
        });
}
