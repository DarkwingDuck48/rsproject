use anyhow::bail;
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base_structures::{
    Dependency, ProjectCreationErrors, traits::BasicGettersForStructures,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskStatus {
    New,
    Wait,
    Processed,
    Complete,
    Rejected,
    Closed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

/// Описание структуры
/// id - UUID задачи
/// name - Имя задачи (публичное)
/// date_start - Дата начала
/// date_end - Дата окончания
/// duration - Продолжительность задачи
/// status - статус задачи
/// resource_allocations - назначенные ресурсы
/// dependencies - зависимые задачи (предшественники)
/// parent_id - UUID группирующей задачи
/// is_summary - признак, является ли задача группирующей
pub struct Task {
    id: Uuid,
    pub name: String,
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
    pub duration: TimeDelta,
    status: TaskStatus,
    resource_allocations: Vec<Uuid>,
    dependencies: Vec<Dependency>,
    pub parent_id: Option<Uuid>,
    pub is_summary: bool,
}

impl Task {
    pub fn new_regular(
        name: impl Into<String>,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        parent_id: Option<Uuid>,
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
            parent_id,
            is_summary: false,
        })
    }

    // Creates a summary task
    pub fn new_summary(
        name: impl Into<String>,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        parent_id: Option<Uuid>,
    ) -> Result<Self, ProjectCreationErrors> {
        Ok(Self {
            id: Uuid::new_v4(),
            name: name.into(),
            date_start,
            date_end,
            status: TaskStatus::New,
            duration: date_end - date_start,
            resource_allocations: vec![],
            dependencies: vec![],
            parent_id,
            is_summary: true,
        })
    }
    pub fn get_status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn change_status(&mut self, new_status: TaskStatus) {
        self.status = new_status
    }

    pub fn set_resource_allocation(&mut self, allocation_id: Uuid) {
        self.resource_allocations.push(allocation_id)
    }

    pub fn is_resource_assigned(&self, allocation_id: &Uuid) -> bool {
        self.resource_allocations.contains(allocation_id)
    }

    pub fn get_resource_allocations(&self) -> &Vec<Uuid> {
        &self.resource_allocations
    }

    pub fn delete_resource_allocation(&mut self, allocation_id: Uuid) {
        self.resource_allocations.retain(|a| a != &allocation_id);
    }

    pub fn add_dependency(&mut self, dependency: Dependency) -> Result<(), anyhow::Error> {
        if self
            .dependencies
            .iter()
            .any(|d| d.depends_on == dependency.depends_on)
        {
            bail!("Задача не может зависеть от одной и той же задачи");
        }
        self.dependencies.push(dependency);
        Ok(())
    }

    pub fn get_dependencies(&self) -> &Vec<Dependency> {
        &self.dependencies
    }
    pub fn delete_dependency(&mut self, depends_on: Uuid) -> Result<(), anyhow::Error> {
        self.dependencies.retain(|d| d.depends_on != depends_on);
        Ok(())
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

        let task = Task::new_regular("Test", date_start, date_end, None);
        assert!(task.is_err());
    }

    #[test]
    fn test_valid_task() {
        let date_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_end = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();

        let task = Task::new_regular("Test", date_start, date_end, None);
        assert!(task.is_ok());
    }
}
