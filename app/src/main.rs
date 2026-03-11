mod tabs;

use chrono::{NaiveDate, Utc};
use eframe::egui::{self, Widget};
use logic::{
    BasicGettersForStructures, Project, ProjectContainer, SingleProjectContainer, TaskService,
};
use tabs::*;
use uuid::Uuid;

#[derive(PartialEq)]
enum Tab {
    Project,
    Tasks,
    Resources,
}

struct ProjectApp {
    container: SingleProjectContainer,
    selected_tab: Tab,
    selected_project_id: Option<Uuid>,
    selected_task_id: Option<Uuid>,
    selected_resource_id: Option<Uuid>,

    // Create project dialog
    show_new_project_dialog: bool,
    new_project_name: String,
    new_project_desc: String,
    new_project_start: NaiveDate,
    new_project_end: NaiveDate,
    error_message: Option<String>,

    // Create task dialog
    show_new_task_dialog: bool,
    new_task_name: String,
    new_task_start: NaiveDate,
    new_task_end: NaiveDate,

    // Create resource dialog
    show_new_resource_dialog: bool,

    // Assign Resource dialog
    show_assign_resource_dialog: bool,
    assign_engagement: String,

    // Add Unavalible period
    show_unavailable_period_dialog: bool,
}

impl Default for ProjectApp {
    fn default() -> Self {
        Self {
            container: SingleProjectContainer::new(),
            selected_tab: Tab::Project,
            show_new_project_dialog: false,
            show_new_task_dialog: false,
            show_new_resource_dialog: false,
            show_assign_resource_dialog: false,
            show_unavailable_period_dialog: false,
            new_project_name: String::new(),
            new_project_desc: String::new(),
            new_project_start: chrono::Utc::now().date_naive(),
            new_project_end: chrono::Utc::now().date_naive(),
            new_task_name: String::new(),
            new_task_start: chrono::Utc::now().date_naive(),
            new_task_end: chrono::Utc::now().date_naive(),
            error_message: None,
            selected_project_id: None,
            selected_task_id: None,
            selected_resource_id: None,
            assign_engagement: String::new(),
        }
    }
}

impl ProjectApp {
    fn show_new_project_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;

        egui::Window::new("Создать новый проект")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Имя проекта");
                    ui.text_edit_singleline(&mut self.new_project_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Описание проекта");
                    ui.text_edit_singleline(&mut self.new_project_desc);
                });

                ui.horizontal(|ui| {
                    ui.label("Дата начала проекта:");
                    egui_extras::DatePickerButton::new(&mut self.new_project_start)
                        .id_salt("start_project_date")
                        .ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("Дата окончания проекта:");
                    egui_extras::DatePickerButton::new(&mut self.new_project_end)
                        .id_salt("end_project_date")
                        .ui(ui);
                });
                if ui.button("Создать проект").clicked() {
                    match self.create_project() {
                        Ok(_) => {
                            self.show_new_project_dialog = false;
                            self.clear_new_project_fields();
                        }
                        Err(e) => self.error_message = Some(e.to_string()),
                    }
                }
            });
        if !open {
            self.show_new_project_dialog = false;
        }
    }

    fn show_new_task_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Create Task")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.new_task_name);
                ui.horizontal(|ui| {
                    ui.label("Start:");
                    egui_extras::DatePickerButton::new(&mut self.new_task_start)
                        .id_salt("task_start_picker")
                        .ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("End:");
                    egui_extras::DatePickerButton::new(&mut self.new_task_end)
                        .id_salt("task_end_picker")
                        .ui(ui);
                });
                if ui.button("Create").clicked() {
                    match self.create_task() {
                        Ok(()) => {
                            self.show_new_task_dialog = false;
                        }
                        Err(e) => self.error_message = Some(e.to_string()),
                    }
                }
            });
        if !open {
            self.show_new_task_dialog = false;
        }
    }
    fn clear_new_project_fields(&mut self) {}
    fn create_project(&mut self) -> anyhow::Result<()> {
        let project = Project::new(
            self.new_project_name.clone(),
            self.new_project_desc.clone(),
            self.new_project_start
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            self.new_project_end.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )?;
        self.container.add_project(project)?;
        Ok(())
    }
    fn create_task(&mut self) -> anyhow::Result<()> {
        eprintln!("create_task called");
        let project = self.container.list_projects().first().cloned();
        if let Some(project) = project {
            let project_id = *project.get_id();
            eprintln!("Project found: {}", project.name);
            let start = self.new_task_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let end = self.new_task_end.and_hms_opt(0, 0, 0).unwrap().and_utc();

            let mut task_service = TaskService::new(&mut self.container);
            task_service.create_task(project_id, self.new_task_name.clone(), start, end)?;

            // Очистить поля
            self.new_task_name.clear();
            self.new_task_start = Utc::now().date_naive();
            self.new_task_end = Utc::now().date_naive();
            Ok(())
        } else {
            eprintln!("No project found");
            Err(anyhow::anyhow!("No project"))
        }
    }
}

impl eframe::App for ProjectApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //Верхняя панель с заголовком
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Новый проект").clicked() {
                    self.show_new_project_dialog = true;
                    ui.close()
                }
                if ui.button("Новый контейнер").clicked() {
                    self.container = SingleProjectContainer::new();
                    ui.close();
                }
                ui.separator();
                if ui.button("Выход").clicked() {
                    std::process::exit(0)
                }
            });

            ui.heading("RS Project");
        });
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Секции");
            ui.separator();
            ui.selectable_value(&mut self.selected_tab, Tab::Project, "📁 Project")
                .context_menu(|ui| {
                    if ui.button("Новый проект").clicked() {
                        self.show_new_project_dialog = true;
                        ui.close();
                    }
                });
            ui.selectable_value(&mut self.selected_tab, Tab::Tasks, "✅ Tasks");
            ui.selectable_value(&mut self.selected_tab, Tab::Resources, "👤 Resources");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::Project => project::show(ui, self),
                Tab::Tasks => task::show(ui, self),
                Tab::Resources => resources::show(ui, self),
            }

            // Отображение ошибки (если есть)
            if let Some(err) = &self.error_message {
                ui.separator();
                ui.colored_label(egui::Color32::RED, err);
            }
        });

        if self.show_new_project_dialog {
            self.show_new_project_dialog(ctx);
        }

        if self.show_new_task_dialog {
            self.show_new_task_dialog(ctx);
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

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Project Manager",
        options,
        Box::new(|_cc| Ok(Box::new(ProjectApp::default()))),
    )
}
