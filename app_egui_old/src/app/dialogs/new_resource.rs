use eframe::egui;
use logic::RateMeasure;

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new(if app.edit_resource_id.is_some() {
        "Редактировать ресурс"
    } else {
        "Добавление ресурса"
    })
    .open(&mut open)
    .show(ctx, |ui| {
        ui.text_edit_singleline(&mut app.new_resource_name);
        ui.horizontal(|ui| {
            ui.label("Ставка");
            ui.text_edit_singleline(&mut app.new_resource_rate);
        });
        ui.horizontal(|ui| {
            ui.label("Тип ставки");
            egui::ComboBox::from_id_salt("rate_measure")
                .selected_text(format!("{:?}", app.new_resource_measure))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.new_resource_measure,
                        RateMeasure::Hourly,
                        "Почасовая",
                    );
                    ui.selectable_value(
                        &mut app.new_resource_measure,
                        RateMeasure::Daily,
                        "Ежедевная",
                    );
                    ui.selectable_value(
                        &mut app.new_resource_measure,
                        RateMeasure::Monthly,
                        "Помесячная",
                    );
                });
        });
        if ui.button("Сохранить").clicked() {
            match app.create_resource() {
                Ok(()) => {
                    app.show_new_resource_dialog = false;
                    app.error_message = None
                }
                Err(e) => app.error_message = Some(e.to_string()),
            }
        }
    });
    if !open {
        app.show_new_resource_dialog = false;
        app.edit_resource_id = None;
    }
}
