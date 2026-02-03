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
    pub id: Uuid,
    pub name: String,
    pub dependency: Dependency,
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
    pub duration: TimeDelta,
    pub status: TaskStatus,
    pub resources: Vec<ResourceOnTask>,
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::base_structures::tasks::Task;
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
