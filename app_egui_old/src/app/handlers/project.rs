use logic::{
    BasicGettersForStructures, Project, ProjectContainer, SingleProjectContainer, TaskService,
};
use rfd::FileDialog;

use crate::ProjectApp;

impl ProjectApp {
    pub fn open_edit_project_dialog(&mut self) {
        if let Some(project) = self.container.list_projects().first() {
            self.new_project_name = project.name.clone();
            self.new_project_desc = project.description.clone();
            self.new_project_start = project.get_date_start().date_naive();
            self.new_project_end = project.get_date_end().date_naive();
            self.show_edit_project_dialog = true;
        }
    }

    pub fn close_project_no_save(&mut self) {
        self.container = SingleProjectContainer::new();
        self.selected_project_id = None;
        self.critical_path = None;
        self.selected_task_id = None;
        self.selected_resource_id = None;
        self.error_message = None;
    }

    pub fn close_project_with_save(&mut self) {
        self.save_project();
        self.close_project_no_save();
    }
    pub fn clear_new_project_fields(&mut self) {}

    pub fn load_project(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<SingleProjectContainer>(&content) {
                    Ok(container) => {
                        self.selected_project_id =
                            Some(*container.list_projects().first().unwrap().get_id());
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
    pub fn save_project(&mut self) {
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
    pub fn create_project(&mut self) -> anyhow::Result<()> {
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
    pub fn update_project(&mut self) -> anyhow::Result<()> {
        let project_id = *self.selected_project_id.as_ref().unwrap();
        let new_start = self
            .new_project_start
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let new_end = self.new_project_end.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Проверяем, что все задачи помещаются в новые даты
        let task_service = TaskService::new(&mut self.container);
        let tasks = task_service.get_all_tasks(project_id);
        for task in tasks {
            if *task.get_date_start() < new_start || *task.get_date_end() > new_end {
                anyhow::bail!("Задача '{}' выходит за новые границы проекта", task.name);
            }
        }

        // Обновляем проект
        let project = self.container.get_project_mut(&project_id).unwrap();
        project.name = self.new_project_name.clone();
        project.description = self.new_project_desc.clone();
        project.date_start = new_start;
        project.date_end = new_end;
        project.duration = new_end - new_start;

        Ok(())
    }
}
