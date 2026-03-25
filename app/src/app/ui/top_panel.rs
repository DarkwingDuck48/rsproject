use eframe::egui::{self, RichText};

use crate::{ProjectApp, app::AppTheme};

pub fn show(ctx: &egui::Context, app: &mut ProjectApp) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.menu_button("Файл", |ui| {
            if ui.button("Новый проект").clicked() {
                app.show_new_project_dialog = true;
                ui.close()
            }
            if ui.button("Закрыть проект").clicked() {
                app.show_close_project_dialog = true;
                ui.close();
            }

            if ui.button(" 🔃 Открыть проект").clicked() {
                app.load_project();
                ui.close();
            }
            if ui.button(" 💾 Сохранить проект").clicked() {
                app.save_project();
                ui.close();
            }

            ui.menu_button("Отображение", |ui| {
                if ui.button("☀️ Светлая тема").clicked() {
                    app.current_theme = AppTheme::Light;
                    ui.close();
                }
                if ui.button("🌙 Темная тема").clicked() {
                    app.current_theme = AppTheme::Dark;
                    ui.close();
                }
            });

            ui.separator();
            if ui.button("Выход").clicked() {
                std::process::exit(0)
            }
        });

        ui.heading(RichText::from("RS Project").size(20.0));
    });
}
