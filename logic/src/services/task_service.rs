use crate::base_structures::ProjectContainer;
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
}
