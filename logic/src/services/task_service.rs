use crate::{
    Project, TimeWindow,
    base_structures::{
        AllocationRequest, BasicGettersForStructures, DependencyType, ProjectContainer, Task,
    },
};
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
    pub fn get_project(&self, project_id: &Uuid) -> Option<&Project> {
        self.container.get_project(project_id)
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
    // Мы должны создать запрос на аллокацию ресурса и отправить его в ресурсы, чтобы мы смогли их назначить
    // Вообще предполагается, что ресурс назначается на весь промежуток задачи, однако мы можем явно указать период, на который ресурс будет зайствован
    // В этом случае надо бы проверить, что это окно входит в промежуток задачи
    pub fn allocate_resource(
        &mut self,
        project_id: Uuid,
        task_id: Uuid,
        resource_id: Uuid,
        engagement: f64,
        time_window: Option<TimeWindow>,
    ) -> anyhow::Result<()> {
        let project = self
            .container
            .get_project(&project_id)
            .ok_or_else(|| anyhow::Error::msg("Запрошенный проект не найден"))?;
        let task = project.tasks.get(&task_id).ok_or_else(|| {
            anyhow::Error::msg(format!("Задача не найдена в проекте {}", project.name))
        })?;

        let allocation_time_window = match time_window {
            Some(tw) => tw,
            None => TimeWindow::new(*task.get_date_start(), *task.get_date_end())?,
        };

        let allocation_request = AllocationRequest::new(
            resource_id,
            task_id,
            project_id,
            engagement,
            allocation_time_window,
        );
        let project_calendar = self
            .container
            .calendar(&project_id)
            .ok_or_else(|| anyhow::Error::msg("Календарь не найден в проекте"))?
            .clone();

        self.container
            .resource_pool_mut()
            .allocate(allocation_request, &project_calendar)
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
