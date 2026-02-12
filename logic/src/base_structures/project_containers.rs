/// Модуль для хранения известных контейнеров проектов
///
/// Будем реализовывать 2 контейнера - одиночный и мульти контейнер
/// Для контейнеров может быть реализована дополнительная логика обработки, но базово будем реализовывать
/// трейт ProjectContainer
use uuid::Uuid;

use crate::{
    Project,
    base_structures::traits::{BasicGettersForStructures, ProjectContainer},
};

pub struct SingleProjectContainer {
    project: Option<Project>,
}

impl ProjectContainer for SingleProjectContainer {
    // Если тут уже был проект, то его заменит
    fn add_project(&mut self, project: Project) -> anyhow::Result<()> {
        if self.project.is_none() {
            self.project = Some(project);
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
}
