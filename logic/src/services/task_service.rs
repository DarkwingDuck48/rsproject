use crate::base_structures::{BasicGettersForStructures, DependencyType, ProjectContainer, Task};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct TaskService<'a, C: ProjectContainer> {
    container: &'a mut C,
}

impl<'a, C: ProjectContainer> TaskService<'a, C> {
    pub fn new(container: &'a mut C) -> Self {
        Self { container }
    }

    // Создание задачи
    pub fn create_task(
        &mut self,
        project_id: Uuid,
        name: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Task> {
        let project = self
            .container
            .get_project_mut(&project_id)
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        // Валидация дат задачи относительно проекта
        if start < *project.get_date_start() || end > *project.get_date_end() {
            anyhow::bail!("Task dates must be within project dates");
        }

        let task = Task::new(name, start, end)?;
        let task_id = *task.get_id();
        project.tasks.insert(task_id, task.clone());
        Ok(task)
    }

    pub fn get_tasks(&self, project_id: &Uuid) -> Vec<&Task> {
        self.container
            .get_project(project_id)
            .map(|p| p.get_project_tasks())
            .unwrap_or_default()
    }

    // Присвоить задаче ресурс
    pub fn assign_resource(
        &mut self,
        project_id: Uuid,
        task_id: Uuid,
        resource_id: Uuid,
        engagement: f64,
    ) -> Result<()> {
        todo!()
    }

    // Добавить зависимость задач
    pub fn add_dependency(
        &mut self,
        project_id: Uuid,
        task_id: Uuid,
        depends_on: Uuid,
        dep_type: DependencyType,
    ) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_structures::{Project, SingleProjectContainer};
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_create_task() {
        let mut container = SingleProjectContainer::new();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap();
        let project = Project::new("Test", "Desc", start, end).unwrap();
        let project_id = *project.get_id();

        container.add_project(project).unwrap();

        let mut task_service = TaskService::new(&mut container);
        let task_start = Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();
        let task_end = Utc.with_ymd_and_hms(2025, 2, 15, 0, 0, 0).unwrap();
        let task = task_service
            .create_task(project_id, "task1".into(), task_start, task_end)
            .expect("Failed to create task");

        assert_eq!(task.name, "task1");
        let tasks = task_service.get_tasks(&project_id);

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "task1")
    }
}
