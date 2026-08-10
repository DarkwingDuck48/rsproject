//! Project commands.

use crate::ProjectInfo;
use crate::state::AppState;
use chrono::{DateTime, NaiveDate, Utc};
use logic::{BasicGettersForStructures, Project, ProjectContainer, TaskService};
use uuid::Uuid;

fn parse_date(date: &str) -> Result<DateTime<Utc>, String> {
    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    Ok(parsed_date)
}

// Создание проекта
#[tauri::command]
pub fn create_project(
    state: tauri::State<AppState>,
    name: String,
    description: String,
    date_start: String,
    date_end: String,
) -> Result<(), String> {
    let start = parse_date(&date_start)?;
    let end = parse_date(&date_end)?;
    let project = Project::new(name, description, start, end).map_err(|e| e.to_string())?;

    let mut container = state.container();
    container.add_project(project).map_err(|e| e.to_string())?;

    let project_id = container
        .list_projects()
        .last()
        .map(|p| *p.get_id())
        .ok_or("Не удалось получить ID созданного проекта".to_string())?;

    *state.selected_project_id.lock().unwrap() = Some(project_id);

    Ok(())
}

#[tauri::command]
pub fn edit_project(
    state: tauri::State<'_, AppState>,
    name: String,
    description: String,
    date_start: String,
    date_end: String,
) -> Result<(), String> {
    let start = parse_date(&date_start)?;
    let end = parse_date(&date_end)?;

    let project_id = {
        let id = state.selected_project_id.lock().unwrap();
        id.ok_or("No selected project".to_string())?
    };
    let mut container = state.container();
    {
        let task_service = TaskService::new(&mut *container);
        let tasks = task_service.get_all_tasks(project_id);
        for task in tasks {
            if *task.get_date_start() < start || *task.get_date_end() > end {
                return Err(format!("Task '{}' not fit to new project dates", task.name));
            }
        }
    }
    let project = container
        .get_project_mut(&project_id)
        .ok_or("Project not found".to_string())?;

    project.name = name;
    project.description = description;
    project.date_start = start;
    project.date_end = end;
    project.duration = end - start;

    Ok(())
}

#[tauri::command]
pub fn open_project(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    !todo!()
}

#[tauri::command]
pub fn close_project(state: tauri::State<'_, AppState>) -> Result<(), String> {
    !todo!()
}
#[tauri::command]
pub fn save_project(state: tauri::State<'_, AppState>) -> Result<(), String> {
    !todo!()
}
#[tauri::command]
pub fn get_project_info(
    state: tauri::State<'_, AppState>,
    project_id: Option<Uuid>,
) -> Result<ProjectInfo, String> {
    match project_id {
        Some(pr) => Ok(ProjectInfo::from_state(state, pr)),
        None => {
            let pr = {
                let id = state.selected_project_id.lock().unwrap();
                id.ok_or("No selected project".to_string())?
            };
            Ok(ProjectInfo::from_state(state, pr))
        }
    }
}
