use std::collections::HashMap;

/// Модуль для хранения известных контейнеров проектов
///
/// Будем реализовывать 2 контейнера - одиночный и мульти контейнер
/// Для контейнеров может быть реализована дополнительная логика обработки, но базово будем реализовывать
/// трейт ProjectContainer
use uuid::Uuid;

use crate::{
    Project,
    base_structures::{
        project_calendar::ProjectCalendar,
        resource_pool::LocalResourcePool,
        traits::{BasicGettersForStructures, ProjectContainer, ResourcePool},
    },
};

pub struct SingleProjectContainer {
    project: Option<Project>,
    resource_pool: LocalResourcePool,
    calendars: HashMap<Uuid, ProjectCalendar>,
}

impl ProjectContainer for SingleProjectContainer {
    // Если тут уже был проект, то его заменит
    fn add_project(&mut self, project: Project) -> anyhow::Result<()> {
        if self.project.is_none() {
            self.project = Some(project.clone());
            self.calendars.insert(*project.get_id(), project.calendar);
            Ok(())
        } else {
            Err(anyhow::Error::msg(
                "It could be only one project in SingleContainer",
            ))
        }
    }

    fn get_project(&self, id: &Uuid) -> Option<&Project> {
        if let Some(prj) = &self.project {
            if prj.get_id() == id { Some(prj) } else { None }
        } else {
            None
        }
    }

    fn resource_pool(&self) -> &dyn ResourcePool {
        &self.resource_pool
    }

    fn resource_pool_mut(&mut self) -> &mut dyn ResourcePool {
        &mut self.resource_pool
    }

    fn calendar(&self, project_id: &Uuid) -> Option<&ProjectCalendar> {
        self.calendars.get(project_id)
    }
}
