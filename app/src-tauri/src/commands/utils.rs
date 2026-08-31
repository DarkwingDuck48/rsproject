use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::state::AppState;

pub fn parse_date(date: &str) -> Result<DateTime<Utc>, String> {
    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    Ok(parsed_date)
}

pub fn resolve_project_id(app_state: &AppState, project_id: Option<Uuid>) -> Result<Uuid, String> {
    match project_id {
        Some(id) => Ok(id),
        None => {
            let id = app_state.selected_project_id.lock().unwrap();
            id.ok_or("Не выбран проект".to_string())
        }
    }
}
