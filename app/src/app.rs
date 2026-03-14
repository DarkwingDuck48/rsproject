use crate::tabs::*;
use chrono::{NaiveDate, Utc};
use eframe::egui::{self, Widget};
use logic::{
    BasicGettersForStructures, ExceptionPeriod, ExceptionType, Project, ProjectContainer,
    RateMeasure, ResourceService, SingleProjectContainer, TaskService, TimeWindow,
};
use rfd::FileDialog;
use uuid::Uuid;

#[derive(PartialEq)]
enum Tab {
    Project,
    Tasks,
    Resources,
    Gantt,
}

pub struct ProjectApp {
    pub container: SingleProjectContainer,
    selected_tab: Tab,
    pub selected_project_id: Option<Uuid>,
    pub(crate) selected_task_id: Option<Uuid>,
    pub selected_resource_id: Option<Uuid>,
    pub critical_path: Option<Vec<Uuid>>,

    // Create project dialog
    pub show_new_project_dialog: bool,
    new_project_name: String,
    new_project_desc: String,
    new_project_start: NaiveDate,
    new_project_end: NaiveDate,
    pub error_message: Option<String>,

    // Create task dialog
    pub show_new_task_dialog: bool,
    new_task_name: String,
    new_task_start: NaiveDate,
    new_task_end: NaiveDate,
    new_task_is_summary: bool,
    selected_task_parent_id: Option<Uuid>,

    // Create resource dialog
    pub show_new_resource_dialog: bool,
    new_resource_name: String,
    new_resource_rate: String,
    new_resource_measure: RateMeasure,

    // Assign Resource dialog
    pub(crate) show_assign_resource_dialog: bool,
    assign_engagement: String,
    assign_use_full_window: bool,
    pub(crate) assign_custom_start: NaiveDate,
    pub(crate) assign_custom_end: NaiveDate,

    pub(crate) show_unavailable_period_dialog: bool,
    unavailable_start: NaiveDate,
    unavailable_end: NaiveDate,
    unavailable_type: ExceptionType,

    // Gantt chart state
    pub gantt_day_width: f32,
    pub gantt_only_critical: bool,
    pub details_task_id: Option<Uuid>,
    pub show_task_details_dialog: bool,
}

impl Default for ProjectApp {
    fn default() -> Self {
        let now = Utc::now().date_naive();
        Self {
            container: SingleProjectContainer::new(),
            selected_tab: Tab::Project,
            critical_path: None,
            show_new_project_dialog: false,
            show_new_task_dialog: false,
            show_new_resource_dialog: false,
            show_assign_resource_dialog: false,
            show_unavailable_period_dialog: false,
            new_project_name: String::new(),
            new_project_desc: String::new(),
            new_project_start: now,
            new_project_end: now,
            new_task_name: String::new(),
            new_task_start: now,
            new_task_end: now,
            error_message: None,
            selected_project_id: None,
            selected_task_id: None,
            selected_resource_id: None,
            assign_engagement: String::from("0.5"),
            new_resource_name: String::new(),
            new_resource_rate: String::from("1000"),
            new_resource_measure: RateMeasure::Hourly,
            unavailable_start: now,
            unavailable_end: now,
            unavailable_type: ExceptionType::Vacation,
            assign_use_full_window: false,
            assign_custom_start: now,
            assign_custom_end: now,
            new_task_is_summary: false,
            selected_task_parent_id: None,
            gantt_day_width: 36.0,
            gantt_only_critical: false,
            details_task_id: None,
            show_task_details_dialog: false,
        }
    }
}

impl ProjectApp {
    pub fn with_container(container: SingleProjectContainer) -> Self {
        let project_id = container
            .list_projects()
            .first()
            .map(|p| *p.get_id())
            .unwrap_or_else(Uuid::new_v4);
        Self {
            container,
            selected_tab: Tab::Project,
            selected_project_id: Some(project_id),
            show_new_project_dialog: false,
            new_project_name: String::new(),
            new_project_desc: String::new(),
            new_project_start: Utc::now().date_naive(),
            new_project_end: Utc::now().date_naive(),
            error_message: None,
            show_new_task_dialog: false,
            new_task_name: String::new(),
            new_task_start: Utc::now().date_naive(),
            new_task_end: Utc::now().date_naive(),
            show_new_resource_dialog: false,
            new_resource_name: String::new(),
            new_resource_rate: String::from("1000"),
            new_resource_measure: RateMeasure::Hourly,
            show_assign_resource_dialog: false,
            selected_task_id: None,
            selected_resource_id: None,
            assign_engagement: String::from("0.5"),
            assign_use_full_window: true,
            assign_custom_start: Utc::now().date_naive(),
            assign_custom_end: Utc::now().date_naive(),
            show_unavailable_period_dialog: false,
            unavailable_start: Utc::now().date_naive(),
            unavailable_end: Utc::now().date_naive(),
            unavailable_type: ExceptionType::Vacation,
            critical_path: None,
            new_task_is_summary: false,
            selected_task_parent_id: None,
            gantt_day_width: 36.0,
            gantt_only_critical: false,
            details_task_id: None,
            show_task_details_dialog: false,
        }
    }
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
                ui.horizontal(|ui| ui.checkbox(&mut self.new_task_is_summary, "Is Summary Task?"));

                ui.add_enabled_ui(!self.new_task_is_summary, |ui| {
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
                    })
                });

                if let Some(project) = self.container.list_projects().first() {
                    let tasks = project.get_project_tasks();
                    ui.horizontal(|ui| {
                        ui.label("Родительская задача:");
                        egui::ComboBox::from_id_salt("parent_task_combo")
                            .selected_text(
                                self.selected_task_parent_id
                                    .and_then(|id| tasks.iter().find(|t| t.get_id() == &id))
                                    .map(|t| t.name.clone())
                                    .unwrap_or_else(|| "Нет родителя".to_string()),
                            )
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.selected_task_parent_id,
                                    None,
                                    "Нет родителя",
                                );
                                for task in tasks {
                                    // Можно добавить отображение типа задачи (например, 📁 для суммарной)
                                    let display_name = if task.is_summary {
                                        format!("📁 {}", task.name)
                                    } else {
                                        task.name.clone()
                                    };
                                    ui.selectable_value(
                                        &mut self.selected_task_parent_id,
                                        Some(*task.get_id()),
                                        display_name,
                                    );
                                }
                            });
                    });
                }

                if ui.button("Create").clicked() {
                    match self.create_task() {
                        Ok(()) => {
                            self.show_new_task_dialog = false;
                            self.error_message = None;
                        }
                        Err(e) => self.error_message = Some(e.to_string()),
                    }
                }
            });
        if !open {
            self.show_new_task_dialog = false;
        }
    }

    fn show_new_resource_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Добавление ресурса")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.new_resource_name);
                ui.horizontal(|ui| {
                    ui.label("Ставка");
                    ui.text_edit_singleline(&mut self.new_resource_rate);
                });
                ui.horizontal(|ui| {
                    ui.label("Тип ставки");
                    egui::ComboBox::from_id_salt("rate_measure")
                        .selected_text(format!("{:?}", self.new_resource_measure))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_resource_measure,
                                RateMeasure::Hourly,
                                "Почасовая",
                            );
                            ui.selectable_value(
                                &mut self.new_resource_measure,
                                RateMeasure::Daily,
                                "Ежедевная",
                            );
                            ui.selectable_value(
                                &mut self.new_resource_measure,
                                RateMeasure::Monthly,
                                "Помесячная",
                            );
                        });
                });
                if ui.button("Добавить").clicked() {
                    match self.create_resource() {
                        Ok(()) => {
                            self.show_new_resource_dialog = false;
                            self.error_message = None
                        }
                        Err(e) => self.error_message = Some(e.to_string()),
                    }
                }
            });
        if !open {
            self.show_new_resource_dialog = false;
        }
    }

    fn show_unavailable_period_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Добавить период недоступности")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Причина:");
                    egui::ComboBox::from_id_salt("exception_type")
                        .selected_text(format!("{:?}", self.unavailable_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.unavailable_type,
                                ExceptionType::Vacation,
                                "Отпуск",
                            );
                            ui.selectable_value(
                                &mut self.unavailable_type,
                                ExceptionType::SickLeave,
                                "Болезнь",
                            );
                            ui.selectable_value(
                                &mut self.unavailable_type,
                                ExceptionType::PersonalDay,
                                "Отгул",
                            );
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Дата начала периода:");
                    egui_extras::DatePickerButton::new(&mut self.unavailable_start)
                        .id_salt("unavail_start_picker")
                        .ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("Дата окончания периода:");
                    egui_extras::DatePickerButton::new(&mut self.unavailable_end)
                        .id_salt("unavail_end_picker")
                        .ui(ui);
                });
                if ui.button("Добавить").clicked() {
                    match self.add_unavailable_period() {
                        Ok(()) => {
                            self.show_unavailable_period_dialog = false;
                            self.error_message = None;
                            self.selected_resource_id = None;
                        }
                        Err(e) => self.error_message = Some(e.to_string()),
                    }
                }
            });
        if !open {
            self.show_unavailable_period_dialog = false;
        }
    }

    fn show_assign_resource_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Назначение ресурса на задачу")
            .open(&mut open)
            .show(ctx, |ui| {
                // Выбор ресурса (выпадающий список)
                let resource_service = ResourceService::new(&mut self.container);
                let resources = resource_service.list_resources();

                ui.horizontal(|ui| {
                    ui.label("Ресурс:");
                    egui::ComboBox::from_id_salt("resource_select")
                        .selected_text(
                            resources
                                .iter()
                                .find(|r| Some(r.id) == self.selected_resource_id)
                                .map(|r| r.name.clone())
                                .unwrap_or_else(|| "Выберите ресурс".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for r in resources {
                                ui.selectable_value(
                                    &mut self.selected_resource_id,
                                    Some(r.id),
                                    &r.name,
                                );
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Занятость (0.0-1.0):");
                    ui.text_edit_singleline(&mut self.assign_engagement);
                });

                ui.separator();

                ui.checkbox(
                    &mut self.assign_use_full_window,
                    "Назначить на все время задачи",
                );

                if !self.assign_use_full_window {
                    ui.horizontal(|ui| {
                        ui.label("Дата начала работ(измененая):");
                        egui_extras::DatePickerButton::new(&mut self.assign_custom_start)
                            .id_salt("assign_start_picker")
                            .ui(ui);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Дата окончания работ(измененная):");
                        egui_extras::DatePickerButton::new(&mut self.assign_custom_end)
                            .id_salt("assign_end_picker")
                            .ui(ui);
                    });
                }

                if ui.button("Назначить").clicked() {
                    match self.assing_resource() {
                        Ok(()) => {
                            self.show_assign_resource_dialog = false;
                            self.selected_task_id = None;
                            self.selected_resource_id = None;
                            self.error_message = None;
                            self.assign_engagement = String::from("0.5");
                            self.assign_use_full_window = true;
                        }
                        Err(e) => self.error_message = Some(e.to_string()),
                    }
                }
            });
        if !open {
            self.show_assign_resource_dialog = false;
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
        self.selected_project_id = self.container.list_projects().last().map(|p| *p.get_id());
        Ok(())
    }
    fn create_task(&mut self) -> anyhow::Result<()> {
        let project = self.container.list_projects().first().cloned();
        if let Some(project) = project {
            let project_id = *project.get_id();
            let start = self.new_task_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let end = self.new_task_end.and_hms_opt(0, 0, 0).unwrap().and_utc();

            let mut task_service = TaskService::new(&mut self.container);
            if !self.new_task_is_summary {
                task_service.create_regular_task(
                    project_id,
                    self.new_task_name.clone(),
                    start,
                    end,
                    self.selected_task_parent_id,
                )?;
            } else {
                task_service.create_summary_task(
                    project_id,
                    self.new_task_name.clone(),
                    self.selected_task_parent_id,
                )?;
            };

            // Очистить поля
            self.new_task_name.clear();
            self.new_task_start = Utc::now().date_naive();
            self.new_task_end = Utc::now().date_naive();
            self.new_task_is_summary = false;
            self.selected_task_parent_id = None;
            Ok(())
        } else {
            eprintln!("No project found");
            Err(anyhow::anyhow!("No project"))
        }
    }
    fn create_resource(&mut self) -> anyhow::Result<()> {
        let rate: f64 = self.new_resource_rate.parse()?;
        let mut resource_service = ResourceService::new(&mut self.container);
        let resource = resource_service.create_resource(
            self.new_resource_name.clone(),
            rate,
            self.new_resource_measure.clone(),
        )?;
        resource_service.add_resource(resource)?;
        self.new_resource_name.clear();
        self.new_resource_rate = String::from("1000");
        Ok(())
    }

    fn add_unavailable_period(&mut self) -> anyhow::Result<()> {
        let resource_id = self
            .selected_resource_id
            .ok_or_else(|| anyhow::anyhow!("Не выбран ресурс"))?;
        let period = TimeWindow::new(
            self.unavailable_start
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            self.unavailable_end.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )?;
        let exception_period = ExceptionPeriod {
            period,
            exception_type: self.unavailable_type.clone(),
        };
        let mut resource_service = ResourceService::new(&mut self.container);
        resource_service.add_unavailable_period(resource_id, exception_period)?;
        Ok(())
    }

    fn assing_resource(&mut self) -> anyhow::Result<()> {
        let binding = self.container.list_projects();
        let project = binding
            .first()
            .ok_or_else(|| anyhow::anyhow!("Не выбран проект"))?;
        let project_id = *project.get_id();

        let task_id = self
            .selected_task_id
            .ok_or_else(|| anyhow::anyhow!("Не выбрана задача"))?;
        let resource_id = self
            .selected_resource_id
            .ok_or_else(|| anyhow::anyhow!("Не выбран ресурс"))?;
        let engagement: f64 = self.assign_engagement.parse()?;
        if !(0.0..=1.0).contains(&engagement) {
            anyhow::bail!("Занятость должна быть между 0.0 и 1.0");
        }
        let time_window = if self.assign_use_full_window {
            None
        } else {
            let start = self
                .assign_custom_start
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = self
                .assign_custom_end
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            Some(TimeWindow::new(start, end)?)
        };
        let mut task_service = TaskService::new(&mut self.container);
        task_service.allocate_resource(
            project_id,
            task_id,
            resource_id,
            engagement,
            time_window,
        )?;
        Ok(())
    }

    fn save_project(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).save_file() {
            match serde_json::to_string_pretty(&self.container) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        self.error_message = Some(format!("Ошибка записи файла: {}", e));
                    } else {
                        self.error_message = None;
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Ошибка создания файла проекта: {}", e))
                }
            }
        }
    }

    fn load_project(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<SingleProjectContainer>(&content) {
                    Ok(container) => {
                        self.container = container;
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Ошибка парсинга файла проекта: {}", e))
                    }
                },
                Err(e) => self.error_message = Some(format!("Ошибка чтения файла проекта: {}", e)),
            }
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
                // TODO: Пока что у нас нет обработки нескольких контейнеров в одном окне - поэтому этот функционал не используем

                // if ui.button("Новый контейнер").clicked() {
                //     self.container = SingleProjectContainer::new();
                //     ui.close();
                // }
                if ui.button(" 💾 Сохранить проект").clicked() {
                    self.save_project();
                    ui.close();
                }
                if ui.button(" ⬇︎ Загрузить проект").clicked() {
                    self.load_project();
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
            ui.selectable_value(&mut self.selected_tab, Tab::Gantt, "📊 Gantt")
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.container.list_projects().is_empty() {
                // Приветственный экран, если пусто в контейнере
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("RS Project - Добро пожаловать");
                    ui.label("Нет активных проектов");
                    if ui.button("Создать проект").clicked() {
                        self.show_new_project_dialog = true;
                    }
                });
            } else {
                match self.selected_tab {
                    Tab::Project => project::show(ui, self),
                    Tab::Tasks => task::show(ui, self),
                    Tab::Resources => resources::show(ui, self),
                    Tab::Gantt => gantt::show(ui, self),
                }
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

        if self.show_new_resource_dialog {
            self.show_new_resource_dialog(ctx);
        }
        if self.show_unavailable_period_dialog {
            self.show_unavailable_period_dialog(ctx);
        }
        if self.show_assign_resource_dialog {
            self.show_assign_resource_dialog(ctx);
        }
        if self.show_task_details_dialog {
            let mut open = true;
            egui::Window::new("Детали задачи")
                .open(&mut open)
                .show(ctx, |ui| {
                    if let Some(task_id) = self.details_task_id {
                        let project_id = *self.selected_project_id.as_ref().unwrap();
                        // Получение данных (скопируйте код из правой панели, который был ранее)
                        let (task_name, task_cost, alloc_ids, task_start, task_end) = {
                            let task_service = logic::TaskService::new(&mut self.container);
                            if let Some(project) = task_service.get_project(&project_id) {
                                if let Some(task) = project.tasks.get(&task_id) {
                                    let name = task.name.clone();
                                    let alloc_ids = task.get_resource_allocations().clone();
                                    let cost = task_service
                                        .calculate_task_cost(&project_id, &task_id)
                                        .unwrap_or(0.0);
                                    (
                                        Some(name),
                                        Some(cost),
                                        alloc_ids,
                                        Some(*task.get_date_start()),
                                        Some(*task.get_date_end()),
                                    )
                                } else {
                                    (None, None, Vec::new(), None, None)
                                }
                            } else {
                                (None, None, Vec::new(), None, None)
                            }
                        };
                        if let Some(name) = task_name {
                            ui.label(format!("Имя: {}", name));
                        }
                        if let Some(cost) = task_cost {
                            ui.label(format!("Стоимость задачи: {:.2}", cost));
                        }
                        if let Some(start) = task_start {
                            ui.label(format!("Начало задачи: {}", start.format("%Y-%m-%d")));
                        }
                        if let Some(end) = task_end {
                            ui.label(format!("Окончание задачи : {}", end.format("%Y-%m-%d")));
                        }
                        ui.separator();
                        ui.strong("Назначенные ресурсы:");
                        if let Some(calendar) = self.container.calendar(&project_id) {
                            let pool = self.container.resource_pool();
                            for alloc_id in alloc_ids {
                                if let Some(allocation) = pool.get_allocation(&alloc_id)
                                    && let Some(resource) =
                                        pool.get_resource(allocation.get_resource_id())
                                {
                                    let tw = allocation.get_time_window();
                                    let hours = tw.duration_hours(calendar) as f64
                                        * allocation.get_engagement_rate();
                                    let cost = pool
                                        .calculate_allocation_cost(&alloc_id, calendar)
                                        .unwrap_or(0.0);
                                    ui.separator();
                                    ui.label(format!("Ресурс: {}", resource.name));
                                    ui.label(format!(
                                        "Период занятости ресурса: {} - {}",
                                        tw.date_start.format("%Y-%m-%d"),
                                        tw.date_end.format("%Y-%m-%d")
                                    ));
                                    ui.label(format!(
                                        "Занятость: {:.0}%",
                                        allocation.get_engagement_rate() * 100.0
                                    ));
                                    ui.label(format!("Часы: {:.1}", hours));
                                    ui.label(format!("Стоимость ресурса: {:.2}", cost));
                                }
                            }
                        }
                    } else {
                        ui.label("Задача не выбрана");
                    }
                });
            if !open {
                self.show_task_details_dialog = false;
                self.details_task_id = None;
            }
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
