use crate::cust_exceptions::ProjectCreationErrors;
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Тип ресурса.
#[derive(Serialize, Deserialize, Debug)]
pub enum ResurceTypes {
    Human,
    Material,
}

#[derive(Serialize, Deserialize, Debug)]
enum TaskStatus {
    Wait,
    Complete,
    Processed,
    New,
}

#[derive(Serialize, Deserialize, Debug)]
enum DependencyType {
    Blocking,
    NonBlocking,
}

/// Настройка ресурса. Будет настраиваться внутри каждого проекта отдельно.
/// План такой - мы в любой момент можем создать новый ресурс и задать ему тип и ставку
#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    id: Uuid,
    name: String,
    res_type: ResurceTypes,
    rate: f64,
}

impl Resource {
    pub fn new(name: impl Into<String>, res_type: ResurceTypes, rate: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            res_type,
            rate,
        }
    }
}

/// Показывает процент, на который занят конкретный ресурс на конкретной задаче.
/// Из этого показателя сможем получить денежный эквивалент затрат ресурса на задачу, умножив ставку на занятость
#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceOnTask {
    resource: Uuid,
    engagement_rate: f64,
}

impl ResourceOnTask {
    pub fn new(id: Uuid, rate: f64) -> Self {
        Self {
            resource: id,
            engagement_rate: rate,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Dependency {
    prev_task: Option<Uuid>,
    next_task: Option<Uuid>,
    dependency_type: Option<DependencyType>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    id: Uuid,
    name: String,
    dependency: Dependency,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    duration: TimeDelta,
    status: TaskStatus,
    resources: Vec<ResourceOnTask>,
}

impl Task {
    pub fn new(
        name: impl Into<String>,
        depend: Option<Dependency>,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        resources: Option<Vec<ResourceOnTask>>,
    ) -> Result<Self, ProjectCreationErrors> {
        if date_start >= date_end {
            return Err(ProjectCreationErrors::InvalidTaskDuration {
                date_start,
                date_end,
            });
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name: name.into(),
            dependency: depend.unwrap_or_default(),
            date_start,
            date_end,
            resources: resources.unwrap_or_default(),
            duration: date_end - date_start,
            status: TaskStatus::New,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    id: Uuid,
    name: String,
    description: String,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    resources: HashMap<Uuid, Resource>,
    tasks: HashMap<Uuid, Task>,
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
        }
    }

    pub fn add_resource(mut self, resource: Resource) -> Self {
        self.resources.insert(resource.id, resource);
        self
    }

    fn check_new_task(&self, task: &Task) -> bool {
        self.date_start <= task.date_start && self.date_end >= task.date_end
    }

    pub fn add_task(mut self, task: Task) -> Self {
        if self.check_new_task(&task) {
            println!("Add new task {:?}", &task.name);
            self.tasks.insert(task.id, task);
        }
        self
    }

    pub fn add_resource_on_task(
        &self,
        added_resource: ResourceOnTask,
        task_id: Uuid,
    ) -> anyhow::Result<()> {
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::base_structures::Task;

    #[test]
    fn test_invalid_task() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let task = Task::new("Test", None, date_start, date_end, None);
        assert!(task.is_err());
    }

    #[test]
    fn test_valid_task() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();

        let task = Task::new("Test", None, date_start, date_end, None);
        assert!(task.is_ok());
    }
}
