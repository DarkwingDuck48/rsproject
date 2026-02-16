use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base_structures::{
    ProjectCreationErrors, dependencies::Dependency, traits::BasicGettersForStructures,
};

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
pub struct Task {
    id: Uuid,
    pub name: String,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    duration: TimeDelta,
    status: TaskStatus,
    resource_allocations: Vec<Uuid>,
    dependencies: Vec<Dependency>,
}

impl Task {
    pub fn new(
        name: impl Into<String>,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
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
            status: TaskStatus::New,
            duration: date_end - date_start,
            resource_allocations: vec![],
            dependencies: vec![],
        })
    }

    pub fn get_status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn change_status(&mut self, new_status: TaskStatus) {
        self.status = new_status
    }
}

impl BasicGettersForStructures for Task {
    fn get_id(&self) -> &Uuid {
        &self.id
    }

    fn get_date_start(&self) -> &DateTime<Utc> {
        &self.date_start
    }

    fn get_date_end(&self) -> &DateTime<Utc> {
        &self.date_end
    }

    fn get_duration(&self) -> &TimeDelta {
        &self.duration
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

        let task = Task::new("Test", date_start, date_end);
        assert!(task.is_err());
    }

    #[test]
    fn test_valid_task() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();

        let task = Task::new("Test", date_start, date_end);
        assert!(task.is_ok());
    }
}
