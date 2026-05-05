use eframe::egui::{self, Widget};
use logic::ExceptionType;

use crate::ProjectApp;

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    let mut open = true;
    egui::Window::new("Добавить период недоступности")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Причина:");
                egui::ComboBox::from_id_salt("exception_type")
                    .selected_text(format!("{:?}", app.unavailable_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut app.unavailable_type,
                            ExceptionType::Vacation,
                            "Отпуск",
                        );
                        ui.selectable_value(
                            &mut app.unavailable_type,
                            ExceptionType::SickLeave,
                            "Болезнь",
                        );
                        ui.selectable_value(
                            &mut app.unavailable_type,
                            ExceptionType::PersonalDay,
                            "Отгул",
                        );
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Дата начала периода:");
                egui_extras::DatePickerButton::new(&mut app.unavailable_start)
                    .id_salt("unavail_start_picker")
                    .start_end_years(2020..=2035)
                    .ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Дата окончания периода:");
                egui_extras::DatePickerButton::new(&mut app.unavailable_end)
                    .id_salt("unavail_end_picker")
                    .start_end_years(2020..=2035)
                    .ui(ui);
            });
            if ui.button("Добавить").clicked() {
                match app.add_unavailable_period() {
                    Ok(()) => {
                        app.show_unavailable_period_dialog = false;
                        app.error_message = None;
                        app.selected_resource_id = None;
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            }
        });
    if !open {
        app.show_unavailable_period_dialog = false;
    }
}
