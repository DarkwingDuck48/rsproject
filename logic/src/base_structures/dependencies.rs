use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Структура для определения зависимостей

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum DependencyType {
    Blocking,
    #[default]
    NonBlocking,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Dependency {
    // ID связанной задачи
    pub dependency_type: DependencyType,
    pub depends_on: Uuid,
    pub lag: TimeDelta, // Лаг/запас времени
}
