use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use uuid::Uuid;

use crate::base_structures::{
    resource::Resource,
    tasks::{ResourceOnTask, Task},
};

#[derive(Serialize, Deserialize)]
pub struct Project {
    id: Uuid,
    name: String,
    description: String,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    resources: HashMap<Uuid, Resource>,
    tasks: HashMap<Uuid, Task>,
    duration: TimeDelta,
}

impl Project {
    pub fn new(
        name: impl Into<String>,
        desc: impl Into<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: desc.into(),
            date_start: start,
            date_end: end,
            resources: HashMap::new(),
            tasks: HashMap::new(),
            duration: end - start,
        }
    }

    /// Private Validations methods
    /// Check that task start and end in project duration
    fn check_new_task(&self, task: &Task) -> bool {
        self.date_start <= task.date_start && self.date_end >= task.date_end
    }

    /// Проверяем, что задача существует в текущем проекте
    fn validate_task_exist(&self, task_id: &uuid::Uuid) -> bool {
        self.tasks.contains_key(task_id)
    }

    /// Проверка правильности зависимостей - проверяем, что все задачи, указанные в зависимости существуют в проекте
    fn validate_dependent_tasks_exists(&self, task: &Task) -> bool {
        let checked_dependency = &task.dependency;
        let result_check_prev_tasks = match checked_dependency.has_prev_tasks() {
            true => checked_dependency
                .prev_tasks()
                .unwrap()
                .iter()
                .all(|e| self.validate_task_exist(e)),
            false => true,
        };
        let result_check_next_tasks = match checked_dependency.has_next_tasks() {
            true => checked_dependency
                .next_tasks()
                .unwrap()
                .iter()
                .all(|e| self.validate_task_exist(e)),
            false => true,
        };
        result_check_prev_tasks && result_check_next_tasks
    }

    /// Проверка на циклические зависимости
    /// Если приходит параметр from_task - то мы начинаем проверять от этой таски.
    /// Если None - то проверяем все задачи, начиная от Root тасок
    fn check_circular_dependency(self, from_task: Option<&Task>) -> bool {
        todo!()
    }

    /// Base method to work with project data
    /// Resource management
    pub fn add_resource(mut self, resource: Resource) -> Self {
        self.resources.insert(resource.id, resource);
        self
    }
    pub fn delete_resource(mut self, resource_id: &Uuid) -> Self {
        match self.resources.remove(resource_id) {
            Some(x) => println!("Resource {} deleted", x.name),
            None => println!("Resource with {} not found", resource_id),
        };
        self
    }

    /// Task management
    /// Add new task to project
    pub fn add_task(mut self, task: Task) -> Self {
        if self.check_new_task(&task) && self.validate_dependent_tasks_exists(&task) {
            println!("Add new task {:?}", &task.name);
            self.tasks.insert(task.id, task);
        }
        self
    }
    /// Delete existing task from project
    pub fn delete_task(mut self, task_id: &Uuid) -> Self {
        match self.tasks.remove(task_id) {
            Some(t) => println!("Task {} deleted", t.name),
            None => println!("Task with {} not found", task_id),
        };
        self
    }
    /// add resource to task
    pub fn add_resource_on_task(
        mut self,
        added_resource: ResourceOnTask,
        task_id: Uuid,
    ) -> anyhow::Result<()> {
        let task = self
            .tasks
            .get_mut(&task_id)
            .ok_or_else(|| anyhow::Error::msg(format!("No task with id {:?}", &task_id)))?;
        task.resources.push(added_resource);
        Ok(())
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Description: {}, StartDate: {}, EndDate: {}, Duration: {} days",
            self.name,
            self.description,
            self.date_start,
            self.date_end,
            self.duration.num_days()
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::Project;

    #[test]
    fn create_empty_project() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

        let project = Project::new("TestProject", "Some test project", date_start, date_end);
        println!("{}", project.duration);
        assert_eq!(project.name, String::from("TestProject"));
        assert_eq!(project.duration, date_end - date_start)
    }
}
