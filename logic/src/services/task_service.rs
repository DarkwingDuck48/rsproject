use crate::{
    Project, TimeWindow,
    base_structures::{
        AllocationRequest, BasicGettersForStructures, Dependency, DependencyType, ProjectContainer,
        Task,
    },
};
use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
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
    ) -> anyhow::Result<Uuid> {
        let (actual_window, task_start, task_end) = {
            let project = self
                .container
                .get_project(&project_id)
                .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
            let task = project
                .tasks
                .get(&task_id)
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

            let task_start = *task.get_date_start();
            let task_end = *task.get_date_end();

            // Определяем окно: либо переданное, либо вся задача
            let window = match time_window {
                Some(w) => {
                    // Проверка, что окно внутри задачи
                    if w.date_start < task_start || w.date_end > task_end {
                        anyhow::bail!(
                            "Time window {:?} is not within task dates [{:?}, {:?}]",
                            w,
                            task_start,
                            task_end
                        );
                    }
                    w
                }
                None => TimeWindow::new(task_start, task_end)?,
            };

            (window, task_start, task_end)
        };

        let calendar = self
            .container
            .calendar(&project_id)
            .ok_or_else(|| anyhow::anyhow!("Calendar not found"))?
            .clone();

        // Шаг 4: Создаём запрос
        let request =
            AllocationRequest::new(resource_id, task_id, project_id, engagement, actual_window);

        // Шаг 5: Выделяем ресурс (мутабельно, но контейнер свободен)
        let allocation_id = self
            .container
            .resource_pool_mut()
            .allocate(request, &calendar)?;

        // Шаг 6: Снова получаем мутабельный доступ к задаче и сохраняем ID
        {
            let project = self
                .container
                .get_project_mut(&project_id)
                .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
            let task = project
                .tasks
                .get_mut(&task_id)
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
            task.set_resource_allocation(allocation_id);
        }

        Ok(allocation_id)
    }

    // Добавить зависимость задач
    pub fn add_dependency(
        &mut self,
        project_id: Uuid,
        task_id: Uuid,
        depends_on: Uuid,
        dep_type: DependencyType,
        lag: Option<TimeDelta>,
    ) -> Result<()> {
        if task_id == depends_on {
            anyhow::bail!("Task cannot depend on itself");
        }
        let project = self
            .container
            .get_project(&project_id)
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        // Проверяем существование обеих задач
        if !project.tasks.contains_key(&task_id) {
            anyhow::bail!("Task with id {} not found", task_id);
        }
        if !project.tasks.contains_key(&depends_on) {
            anyhow::bail!("Dependency task with id {} not found", depends_on);
        }

        // Создаём объект зависимости
        let dependency = Dependency {
            dependency_type: dep_type,
            depends_on,
            lag,
        };
        let project = self
            .container
            .get_project_mut(&project_id)
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        let task = project
            .tasks
            .get_mut(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.add_dependency(dependency);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RateMeasure, ResourceService,
        base_structures::{Project, SingleProjectContainer},
    };
    use chrono::{Duration, TimeZone, Utc};

    // Вспомогательная функция: создаёт контейнер с проектом и одной задачей,
    // возвращает контейнер и идентификаторы/даты.
    fn setup_task() -> (
        SingleProjectContainer,
        Uuid,
        Uuid,
        DateTime<Utc>,
        DateTime<Utc>,
    ) {
        let mut container = SingleProjectContainer::new();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap();
        let project = Project::new("Test", "Desc", start, end).unwrap();
        let project_id = *project.get_id();
        container.add_project(project).unwrap();

        let task_start = Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();
        let task_end = Utc.with_ymd_and_hms(2025, 2, 15, 0, 0, 0).unwrap();
        let mut task_service = TaskService::new(&mut container);
        let task = task_service
            .create_task(project_id, "Task".into(), task_start, task_end)
            .unwrap();
        let task_id = *task.get_id();

        (container, project_id, task_id, task_start, task_end)
    }

    // Вспомогательная функция: добавляет ресурс в пул контейнера.
    fn setup_resource(container: &mut SingleProjectContainer) -> Uuid {
        let mut resource_service = ResourceService::new(container);
        let resource = resource_service
            .create_resource("TestRes", 1000.0, RateMeasure::Hourly)
            .unwrap();
        let resource_id = resource.id;
        resource_service.add_resource(resource).unwrap();
        resource_id
    }

    // 1. Пользователь не передал окно → окно = всей задаче.
    #[test]
    fn test_allocate_resource_without_window() -> anyhow::Result<()> {
        let (mut container, project_id, task_id, _, _) = setup_task();
        let engagement = 0.5;
        let resource_id = {
            let resource_id = setup_resource(&mut container);

            let mut task_service = TaskService::new(&mut container);

            let allocation_id = task_service.allocate_resource(
                project_id,
                task_id,
                resource_id,
                engagement,
                None,
            )?;
            let task = task_service
                .get_project(&project_id)
                .unwrap()
                .tasks
                .get(&task_id)
                .unwrap();
            assert!(task.is_resource_assigned(&allocation_id));
            resource_id
        };
        // Проверяем утилизацию ресурса
        {
            let resource_service = ResourceService::new(&mut container);
            let utilization = resource_service.get_resource_utilization(resource_id);
            assert_eq!(utilization, engagement);
        }

        Ok(())
    }

    // 2. Пользователь передал корректное окно (внутри задачи).
    #[test]
    fn test_allocate_resource_with_valid_window() -> anyhow::Result<()> {
        let (mut container, project_id, task_id, task_start, _) = setup_task();
        let engagement = 0.5;

        let resource_id = {
            let resource_id = setup_resource(&mut container);
            let mut task_service = TaskService::new(&mut container);
            let window = TimeWindow::new(task_start, task_start + Duration::days(3))?;
            let allocation_id = task_service.allocate_resource(
                project_id,
                task_id,
                resource_id,
                engagement,
                Some(window),
            )?;
            let task = task_service
                .get_project(&project_id)
                .unwrap()
                .tasks
                .get(&task_id)
                .unwrap();
            assert!(task.is_resource_assigned(&allocation_id));
            resource_id
        };
        {
            let resource_service = ResourceService::new(&mut container);
            let utilization = resource_service.get_resource_utilization(resource_id);
            assert_eq!(utilization, engagement);
        };

        Ok(())
    }

    // 3. Пользователь передал окно, выходящее за пределы задачи → ошибка.
    #[test]
    fn test_allocate_resource_with_window_outside_task() -> anyhow::Result<()> {
        let (mut container, project_id, task_id, task_start, task_end) = setup_task();
        let resource_id = setup_resource(&mut container);

        let mut task_service = TaskService::new(&mut container);
        let engagement = 0.5;

        // Окно начинается до начала задачи
        let window_before = TimeWindow::new(
            task_start - Duration::days(1),
            task_start + Duration::days(2),
        )?;
        let result = task_service.allocate_resource(
            project_id,
            task_id,
            resource_id,
            engagement,
            Some(window_before),
        );
        assert!(result.is_err());

        // Окно заканчивается после окончания задачи
        let window_after =
            TimeWindow::new(task_end - Duration::days(2), task_end + Duration::days(1))?;
        let result = task_service.allocate_resource(
            project_id,
            task_id,
            resource_id,
            engagement,
            Some(window_after),
        );
        assert!(result.is_err());

        // Окно полностью вне задачи
        let window_outside =
            TimeWindow::new(task_end + Duration::days(1), task_end + Duration::days(5))?;
        let result = task_service.allocate_resource(
            project_id,
            task_id,
            resource_id,
            engagement,
            Some(window_outside),
        );
        assert!(result.is_err());

        Ok(())
    }

    // 4. (Дополнительно) Пользователь передал окно, равное задаче — должно работать.
    #[test]
    fn test_allocate_resource_with_window_equal_task() -> anyhow::Result<()> {
        let (mut container, project_id, task_id, task_start, task_end) = setup_task();
        let resource_id = setup_resource(&mut container);

        let mut task_service = TaskService::new(&mut container);
        let engagement = 0.5;
        let window = TimeWindow::new(task_start, task_end)?;
        let allocation_id = task_service.allocate_resource(
            project_id,
            task_id,
            resource_id,
            engagement,
            Some(window),
        )?;
        let task = task_service
            .get_project(&project_id)
            .unwrap()
            .tasks
            .get(&task_id)
            .unwrap();
        assert!(task.is_resource_assigned(&allocation_id));

        let resource_service = ResourceService::new(&mut container);
        let utilization = resource_service.get_resource_utilization(resource_id);
        assert_eq!(utilization, engagement);

        Ok(())
    }
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
    fn setup_two_tasks() -> (SingleProjectContainer, Uuid, Uuid, Uuid) {
        let mut container = SingleProjectContainer::new();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap();
        let project = Project::new("Test", "Desc", start, end).unwrap();
        let project_id = *project.get_id();
        container.add_project(project).unwrap();

        let mut task_service = TaskService::new(&mut container);
        let task1 = task_service
            .create_task(
                project_id,
                "Task1".into(),
                Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2025, 2, 10, 0, 0, 0).unwrap(),
            )
            .unwrap();
        let task2 = task_service
            .create_task(
                project_id,
                "Task2".into(),
                Utc.with_ymd_and_hms(2025, 2, 11, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2025, 2, 20, 0, 0, 0).unwrap(),
            )
            .unwrap();
        (container, project_id, *task1.get_id(), *task2.get_id())
    }

    #[test]
    fn test_add_dependency_success() -> anyhow::Result<()> {
        let (mut container, project_id, task1_id, task2_id) = setup_two_tasks();
        let mut task_service = TaskService::new(&mut container);

        task_service.add_dependency(
            project_id,
            task1_id,
            task2_id,
            DependencyType::Blocking,
            Duration::zero().into(),
        )?;

        // Проверяем, что зависимость добавилась
        let task1 = task_service
            .get_project(&project_id)
            .unwrap()
            .tasks
            .get(&task1_id)
            .unwrap();
        assert_eq!(task1.get_dependencies().len(), 1);
        let dep = &task1.get_dependencies()[0];
        assert_eq!(dep.depends_on, task2_id);
        assert!(matches!(dep.dependency_type, DependencyType::Blocking));
        assert_eq!(dep.lag, Duration::zero().into());

        Ok(())
    }

    #[test]
    fn test_add_dependency_self_dependency() -> anyhow::Result<()> {
        let (mut container, project_id, task1_id, _) = setup_two_tasks();
        let mut task_service = TaskService::new(&mut container);

        let result = task_service.add_dependency(
            project_id,
            task1_id,
            task1_id,
            DependencyType::Blocking,
            Duration::zero().into(),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot depend on itself")
        );

        Ok(())
    }

    #[test]
    fn test_add_dependency_task_not_found() -> anyhow::Result<()> {
        let (mut container, project_id, task1_id, task2_id) = setup_two_tasks();
        let mut task_service = TaskService::new(&mut container);
        let fake_id = Uuid::new_v4();

        // Неверный task_id
        let result = task_service.add_dependency(
            project_id,
            fake_id,
            task2_id,
            DependencyType::Blocking,
            Duration::zero().into(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Неверный depends_on
        let result = task_service.add_dependency(
            project_id,
            task1_id,
            fake_id,
            DependencyType::Blocking,
            Duration::zero().into(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        Ok(())
    }

    #[test]
    fn test_add_dependency_project_not_found() -> anyhow::Result<()> {
        let (mut container, project_id, task1_id, task2_id) = setup_two_tasks();
        let mut task_service = TaskService::new(&mut container);
        let fake_project = Uuid::new_v4();

        let result = task_service.add_dependency(
            fake_project,
            task1_id,
            task2_id,
            DependencyType::Blocking,
            Duration::zero().into(),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Project not found")
        );

        Ok(())
    }
}
