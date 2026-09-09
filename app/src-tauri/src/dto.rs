// Data Transfer Objects - описная структур для фронтенда, для упрощения передачи информации
//
//
// Структура ProjectInfo - DTO объект проекта для фронтенда

mod project;
mod resources;
mod task;

pub use project::ProjectInfo;
pub use resources::{ResourceInfo, ResourceUpdateDto};
pub use task::{TaskInfo, TaskTreeNode, TaskUpdateDto};
