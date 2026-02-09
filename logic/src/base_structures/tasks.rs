use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base_structures::ProjectCreationErrors;

#[derive(Serialize, Deserialize, Debug)]
pub enum TaskStatus {
    New,
    Wait,
    Processed,
    Complete,
    Rejected,
    Closed,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DependencyType {
    Blocking,
    NonBlocking,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct EngagementRate(f64);

impl EngagementRate {
    pub fn new(rate: f64) -> anyhow::Result<Self> {
        if (0.0..=1.0).contains(&rate) {
            Ok(Self(rate))
        } else {
            Err(anyhow::Error::msg(
                "Engagement rate must be between 0 and 1",
            ))
        }
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Показывает процент, на который занят конкретный ресурс на конкретной задаче.
/// Из этого показателя сможем получить денежный эквивалент затрат ресурса на задачу, умножив ставку на занятость
#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceOnTask {
    resource: Uuid,
    engagement_rate: EngagementRate,
}

impl ResourceOnTask {
    pub fn new(id: Uuid, rate: f64) -> anyhow::Result<Self> {
        Ok(Self {
            resource: id,
            engagement_rate: EngagementRate::new(rate)?,
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum DependencyNodeType {
    Root,
    Base,
    Node,
    Isolated,
}

/// Структура для определения зависимостей
/// Определяем следующие понятия:
/// Если prev_task - None, то это начало цепочки зависимостей. А так же какая то верхнеуровневая операция, например веха проекта
/// Если next_task - None, то это конец цепочки зависимостей
///
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Dependency {
    from_task: Uuid,
    to_task: Uuid,
    dependency_type: Option<DependencyType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prev_task: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_task: Option<Vec<Uuid>>,
}

impl Dependency {
    pub fn get_dependency_type(&self) -> DependencyNodeType {
        fn is_valid_tasks(tasks: &Option<Vec<Uuid>>) -> bool {
            tasks.as_ref().is_some_and(|v| !v.is_empty())
        }

        match (
            is_valid_tasks(&self.prev_task),
            is_valid_tasks(&self.next_task),
        ) {
            (false, true) => DependencyNodeType::Root,
            (true, false) => DependencyNodeType::Base,
            (true, true) => DependencyNodeType::Node,
            (false, false) => DependencyNodeType::Isolated,
        }
    }

    pub fn add_prev_task(&mut self, task_id: Uuid) {
        if self.has_prev_tasks() {
            self.prev_task.as_mut().unwrap().push(task_id);
        } else {
            self.prev_task = Some(vec![task_id]);
        }
    }

    pub fn add_next_task(&mut self, task_id: Uuid) {
        if self.has_next_tasks() {
            self.next_task.as_mut().unwrap().push(task_id);
        } else {
            self.next_task = Some(vec![task_id]);
        }
    }

    // pub fn delete_prev_task(&mut self, task_id: Uuid) {
    //     if self.has_prev_tasks() {
    //         self.prev_task
    //             .as_mut()
    //             .unwrap()
    //             .iter()
    //             .filter(|&&e| e == task_id)
    //     }
    // }

    pub fn prev_tasks(&self) -> Option<&[Uuid]> {
        self.prev_task.as_deref().filter(|v| !v.is_empty())
    }

    pub fn next_tasks(&self) -> Option<&[Uuid]> {
        self.next_task.as_deref().filter(|v| !v.is_empty())
    }

    pub fn has_prev_tasks(&self) -> bool {
        self.prev_tasks().is_some()
    }

    pub fn has_next_tasks(&self) -> bool {
        self.next_tasks().is_some()
    }

    // Если вдруг получаем пустой вектор - то превращаем его в None
    fn normalize(&mut self) {
        if let Some(v) = &self.prev_task
            && v.is_empty()
        {
            self.prev_task = None;
        }
        if let Some(v) = &self.next_task
            && v.is_empty()
        {
            self.next_task = None;
        }
    }

    // Исправляем структуру после десериализации
    pub fn sanitaze(&mut self) {
        self.normalize();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
    pub duration: TimeDelta,
    pub status: TaskStatus,
    pub resources: Vec<ResourceOnTask>,
}

impl Task {
    pub fn new(
        name: impl Into<String>,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        dependencies: Option<Dependency>,
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
            date_start,
            date_end,
            dependency: dependencies.unwrap_or_default(),
            status: TaskStatus::New,
            resources: resources.unwrap_or_default(),
            duration: date_end - date_start,
        })
    }

    pub fn add_prev_task(&mut self, task_id: Uuid) {
        self.dependency.add_prev_task(task_id);
    }

    pub fn add_next_task(&mut self, task_id: Uuid) {
        self.dependency.add_next_task(task_id);
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::base_structures::tasks::{DependencyNodeType, Task};
    #[test]
    fn test_invalid_task() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let task = Task::new("Test", date_start, date_end, None, None);
        assert!(task.is_err());
    }

    #[test]
    fn test_valid_task() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();

        let task = Task::new("Test", date_start, date_end, None, None);
        assert!(task.is_ok());
        assert_eq!(
            task.unwrap().dependency.get_dependency_type(),
            DependencyNodeType::Isolated
        )
    }
    #[test]
    fn test_for_dependencies() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();

        let mut task = Task::new("Test", date_start, date_end, None, None).unwrap();
        assert_eq!(
            task.dependency.get_dependency_type(),
            DependencyNodeType::Isolated
        );
        task.add_prev_task(uuid::Uuid::new_v4());
        assert_eq!(
            task.dependency.get_dependency_type(),
            DependencyNodeType::Base
        );
        task.add_next_task(uuid::Uuid::new_v4());
        assert_eq!(
            task.dependency.get_dependency_type(),
            DependencyNodeType::Node
        )
    }
}
