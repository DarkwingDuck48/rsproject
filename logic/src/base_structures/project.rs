use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
            resources: HashMap::new(),
            tasks: HashMap::new(),
        })
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.insert(resource.id, resource);
    }

    fn check_new_task(&self, task: &Task) -> bool {
        self.date_start <= task.date_start && self.date_end >= task.date_end
    }

    pub fn add_task(&mut self, task: Task) -> anyhow::Result<()> {
        if self.check_new_task(&task) {
            println!("Add new task {:?}", &task.name);
            self.tasks.insert(task.id, task);
            Ok(())
        } else {
            Err(anyhow::Error::msg("Task dates outside project range"))
        }
    }

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
