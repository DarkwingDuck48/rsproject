use eframe::egui;

use crate::{ProjectApp, app::views::View};

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.selectable_value(&mut app.selected_tab, View::Project, "📁 Общая информация")
            .context_menu(|ui| {
                if ui.button("Новый проект").clicked() {
                    app.show_new_project_dialog = true;
                    ui.close();
                }
            });
        ui.selectable_value(&mut app.selected_tab, View::Tasks, "✅ Задачи");
        ui.selectable_value(&mut app.selected_tab, View::Resources, "👤 Ресурсы");
        ui.selectable_value(&mut app.selected_tab, View::Gantt, "📊 Диаграмма Ганта")
    });
}
