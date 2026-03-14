use crate::ProjectApp;
use eframe::egui::{self, Ui};
use egui_extras::{Column, TableBuilder};
use logic::{ProjectContainer, ResourceService};

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

    let resource_service = ResourceService::new(&mut app.container);
    let resources = resource_service.list_resources();

    if resources.is_empty() {
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
        .columns(Column::auto_with_initial_suggestion(140.0), 1) // Действия
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
            body.rows(22.0, resources.len(), |mut row| {
                let resource = &resources[row.index()];
                let utilization = resource_service.get_resource_utilization(resource.id);

                row.col(|ui| {
                    ui.label(&resource.name);
                });
                row.col(|ui| {
                    ui.label(format!("{:.2}", resource.get_base_rate()));
                });
                row.col(|ui| {
                    ui.label(format!("{:?}", resource.get_rate_measure()));
                });
                row.col(|ui| {
                    ui.label(format!("{:.1}%", utilization * 100.0));
                });
                row.col(|ui| {
                    let unavail_count = resource.get_unavailable_periods().len();
                    if unavail_count > 0 {
                        ui.label(format!("{} периодов", unavail_count));
                    } else {
                        ui.label("нет");
                    }
                });
                row.col(|ui| {
                    if ui.button("➕ Недоступность").clicked() {
                        app.selected_resource_id = Some(resource.id);
                        app.show_unavailable_period_dialog = true;
                    }
                });
            });
        });
}
