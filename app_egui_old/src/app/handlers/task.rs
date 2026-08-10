use chrono::{Duration, Utc};
use logic::{BasicGettersForStructures, DependencyType, ProjectContainer, TaskService};
use uuid::Uuid;

use crate::ProjectApp;

impl ProjectApp {
    pub fn open_edit_task_dialog(&mut self, task_id: Uuid) {
        if let Some(project_id) = self.selected_project_id {
            let task_service = TaskService::new(&mut self.container);
            if let Some(project) = task_service.get_project(&project_id)
                && let Some(task) = project.tasks.get(&task_id)
            {
                self.new_task_name = task.name.clone();
                self.new_task_start = task.get_date_start().date_naive();
                self.new_task_end = task.get_date_end().date_naive();
                self.new_task_is_summary = task.is_summary;
                self.selected_task_parent_id = task.parent_id;
                self.new_task_dependency_task = if task.get_dependencies().is_empty() {
                    None
                } else {
                    Some(task.get_dependencies().first().unwrap().depends_on)
                };
                self.new_task_dependency_type = if task.get_dependencies().is_empty() {
                    None
                } else {
                    Some(task.get_dependencies().first().unwrap().dependency_type)
                };
                self.edit_task_id = Some(task_id);
                self.show_new_task_dialog = true;
            }
        }
    }
    pub fn create_task(&mut self) -> anyhow::Result<()> {
        let project = self.container.list_projects().first().cloned();
        if let Some(project) = project {
            let project_id = *project.get_id();
            let start = self.new_task_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let end = self.new_task_end.and_hms_opt(0, 0, 0).unwrap().and_utc();

            let mut task_service = TaskService::new(&mut self.container);
            if let Some(task_id) = self.edit_task_id {
                // Обновление
                task_service.update_task(
                    project_id,
                    task_id,
                    Some(self.new_task_name.clone()),
                    Some(start),
                    Some(end),
                    self.selected_task_parent_id,
                )?;
                // TODO: Здесь должно быть место для удаления зависимости с задачи
                if self.new_task_dependency_task.is_some() {
                    eprintln!("Добавляю новую зависимую задачу");
                    task_service.add_dependency(
                        project_id,
                        task_id,
                        self.new_task_dependency_task.unwrap(),
                        self.new_task_dependency_type
                            .unwrap_or(DependencyType::Blocking),
                        Some(Duration::zero()),
                    )?;
                }
            } else if !self.new_task_is_summary {
                let task = task_service.create_regular_task(
                    project_id,
                    self.new_task_name.clone(),
                    start,
                    end,
                    self.selected_task_parent_id,
                )?;
                if self.new_task_dependency_task.is_some() {
                    eprintln!("Добавляю новую зависимую задачу");
                    task_service.add_dependency(
                        project_id,
                        *task.get_id(),
                        self.new_task_dependency_task.unwrap(),
                        self.new_task_dependency_type.unwrap(),
                        Some(Duration::zero()),
                    )?;
                }
            } else {
                task_service.create_summary_task(
                    project_id,
                    self.new_task_name.clone(),
                    self.selected_task_parent_id,
                )?;
            }
            // Очистить поля
            self.clear_task_fields();
            Ok(())
        } else {
            eprintln!("No project found");
            Err(anyhow::anyhow!("No project"))
        }
    }

    fn clear_task_fields(&mut self) {
        self.new_task_name.clear();
        self.new_task_start = Utc::now().date_naive();
        self.new_task_end = Utc::now().date_naive();
        self.new_task_is_summary = false;
        self.selected_task_parent_id = None;
        self.edit_task_id = None;
    }
}
