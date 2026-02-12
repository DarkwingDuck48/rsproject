use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;

use crate::base_structures::traits::BasicGettersForStructures;

/// Структура Project - главная структура всего проекта
/// Она хранит в себе все задачи и зависимости между ними

#[derive(Serialize, Deserialize)]
pub struct Project {
    id: Uuid,
    pub name: String,
    pub description: String,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    duration: TimeDelta,
}

impl Project {
    pub fn new(
        name: impl Into<String>,
        desc: impl Into<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        if start > end {
            return Err(anyhow::Error::msg(format!(
                "Start date of project later than End Date: {}>{}",
                start, end
            )));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: desc.into(),
            date_start: start,
            date_end: end,
            duration: end - start,
        })
    }

    // Getters for private fields

    // Project id
}

impl BasicGettersForStructures for Project {
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

        let project = Project::new("TestProject", "Some test project", date_start, date_end)
            .expect("Project is not created");
        println!("{}", project.duration);
        assert_eq!(project.name, String::from("TestProject"));
        assert_eq!(project.duration, date_end - date_start)
    }
}
