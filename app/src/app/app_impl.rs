use eframe::egui::{self};

use crate::{
    ProjectApp,
    app::{AppTheme, dialogs, ui},
};

impl eframe::App for ProjectApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_theme {
            AppTheme::Light => ctx.set_visuals(egui::Visuals::light()),
            AppTheme::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }
        ui::top_panel::show(ctx, self);
        ui::side_panel::show(ctx, self);
        ui::central_panel::show(ctx, self);
        if self.show_new_project_dialog {
            dialogs::new_project::show(ctx, self);
        }

        if self.show_new_task_dialog {
            dialogs::new_task::show(ctx, self);
        }

        if self.show_new_resource_dialog {
            dialogs::new_resource::show(ctx, self)
        }
        if self.show_unavailable_period_dialog {
            dialogs::unavailable_period::show(ctx, self)
        }
        if self.show_assign_resource_dialog {
            dialogs::assign_resource::show(ctx, self);
        }

        if self.show_close_project_dialog {
            dialogs::close_project::show(ctx, self);
        }

        if self.show_task_details_dialog {
            dialogs::task_details::show(ctx, self);
        }
        if self.show_edit_project_dialog {
            dialogs::edit_project::show(ctx, self);
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()

        // _visuals.window_fill() would also be a natural choice
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {}
}
