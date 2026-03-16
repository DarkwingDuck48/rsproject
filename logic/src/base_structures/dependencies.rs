use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Структура для определения зависимостей

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, Copy)]
pub enum DependencyType {
    Blocking,
    #[default]
    NonBlocking,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Blocking => write!(f, "Блокирующая"),
            DependencyType::NonBlocking => write!(f, "Неблокирующая"),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
pub struct Dependency {
    // ID связанной задачи
    pub dependency_type: DependencyType,
    pub depends_on: Uuid,
    pub lag: Option<TimeDelta>, // Лаг/запас времени
}

impl Dependency {
    pub fn new(dependency_type: DependencyType, depends_on: Uuid, lag: Option<TimeDelta>) -> Self {
        Self {
            dependency_type,
            depends_on,
            lag,
        }
    }
}
