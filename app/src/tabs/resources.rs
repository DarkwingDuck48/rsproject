use crate::ProjectApp;
use eframe::egui::{self, Ui};
use logic::{
    BasicGettersForStructures, ExceptionPeriod, ExceptionType, ProjectContainer, RateMeasure,
    ResourceService, TimeWindow,
};

pub fn show(ui: &mut Ui, app: &mut ProjectApp) {
    ui.heading("Resources");

    // Кнопка добавления ресурса
    if ui.button("➕ Add Resource").clicked() {
        app.show_new_resource_dialog = true;
    }

    ui.separator();

    // Получаем текущий проект (если есть) – для календаря и проверок
    if let Some(project) = app.container.list_projects().first() {
        let project_id = *project.get_id();

        // Создаём сервис ресурсов для чтения (не мутабельно)
        let resource_service = ResourceService::new(&mut app.container);
        let resources = resource_service.list_resources();

        if resources.is_empty() {
            ui.label("Ресурсов не создано. Добавьте новый");
        } else {
            egui::Grid::new("resources_grid")
                .num_columns(6)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Наименование");
                    ui.label("Ставка");
                    ui.label("Размерность");
                    ui.label("Утилизация");
                    ui.label("Периоды недоступности");
                    ui.label("Действия");
                    ui.end_row();

                    for resource in resources {
                        let utilization = resource_service.get_resource_utilization(resource.id);

                        ui.label(&resource.name);
                        ui.label(format!("{:.2}", resource.get_base_rate()));
                        ui.label(format!("{:?}", resource.get_rate_measure()));
                        ui.label(format!("{:.1}%", utilization * 100.0));

                        // Отображение количества периодов недоступности
                        // (пока только счётчик, потом можно добавить детали)
                        let unavail_count = resource.get_unavailable_periods().len();
                        ui.label(if unavail_count > 0 {
                            format!("{} periods", unavail_count)
                        } else {
                            "None".to_string()
                        });

                        // Кнопка для добавления периода недоступности
                        if ui.button("➕ Добавить недоступность").clicked() {
                            app.selected_resource_id = Some(resource.id);
                            app.show_unavailable_period_dialog = true;
                        }
                        ui.end_row();
                    }
                });
        }
    } else {
        ui.label("No project loaded. Create a project first.");
    }
}
