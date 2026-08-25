use logic::Task;
use uuid::Uuid;

use crate::commands::utils::parse_date;
use crate::state::AppState;

#[tauri::command]
pub fn add_task(
    state: tauri::State<AppState>,
    name: String,
    date_start: String,
    date_end: String,
    is_summary: bool,
    parent_id: Option<Uuid>,
) -> Result<(), String> {
    let start_task_date = parse_date(&date_start)?;
    let end_task_date = parse_date(&date_end)?;
    Ok(())
}

#[tauri::command]
pub fn edit_task() -> Result<(), String> {
    todo!()
}

#[tauri::command]
pub fn delete_task() -> Result<(), String> {
    todo!()
}

#[tauri::command]
pub fn get_tasks() -> Result<(), String> {
    todo!()
}

#[tauri::command]
pub fn get_task_tree() -> Result<(), String> {
    todo!()
}

#[tauri::command]
pub fn recalculate_critical_path() -> Result<(), String> {
    todo!()
}
