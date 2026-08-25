// Data Transfer Objects - описная структур для фронтенда, для упрощения передачи информации
//
//
//// Структура ProjectInfo - DTO объект проекта для фронтенда

use chrono::{DateTime, Utc};
use logic::{DependencyType, ProjectContainer, TaskStatus};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Serialize)]
pub struct ProjectInfo {
    id: Uuid,
    name: String,
    description: String,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    duration_days: i64,
}

impl ProjectInfo {
    pub fn from_state(state: tauri::State<'_, AppState>, project_id: Uuid) -> Result<Self, String> {
        let container = state.container();
        let project = container
            .get_project(&project_id)
            .ok_or(format!("Проект с ID {} не найден", project_id))?;
        Ok(Self {
            id: project_id,
            name: project.name.clone(),
            description: project.description.clone(),
            date_start: project.date_start,
            date_end: project.date_end,
            duration_days: project.duration.num_days(),
        })
    }
}

#[derive(Serialize, Clone)]
pub struct TaskDependencyInfo {
    depends_on: Uuid,
    name: String,
    dependency_type: DependencyType,
    lag_days: Option<i64>,
}

#[derive(Serialize, Clone)]
pub struct TaskInfo {
    id: Uuid,
    name: String,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    duration_days: i64,
    status: TaskStatus,
    is_summary: bool,
    parent_id: Option<Uuid>,
    depth: i64,
    cost: f64,
    dependencies: Vec<TaskDependencyInfo>,
}
#[derive(Serialize, Clone)]
pub struct TaskTreeNode {
    task: TaskInfo,
    children: Vec<TaskTreeNode>,
}
