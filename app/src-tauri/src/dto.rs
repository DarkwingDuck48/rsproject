// Data Transfer Objects - описная структур для фронтенда, для упрощения передачи информации
//
//
//// Структура ProjectInfo - DTO объект проекта для фронтенда

use chrono::{DateTime, Utc};
use logic::ProjectContainer;
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
    pub fn from_state(state: tauri::State<'_, AppState>, project_id: Uuid) -> Self {
        let container = state.container();
        let project = container
            .get_project(&project_id)
            .expect(&format!("Can't find project with {}", project_id));
        Self {
            id: project_id,
            name: project.name.clone(),
            description: project.description.clone(),
            date_start: project.date_start,
            date_end: project.date_end,
            duration_days: project.duration.num_days(),
        }
    }
}
