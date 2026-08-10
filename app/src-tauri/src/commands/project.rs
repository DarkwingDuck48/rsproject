//! Project commands.

use crate::ProjectInfo;
use crate::state::AppState;
use chrono::{DateTime, NaiveDate, Utc};
use logic::SingleProjectContainer;
use logic::{BasicGettersForStructures, Project, ProjectContainer, TaskService};
use tauri_plugin_dialog::DialogExt;
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
        id.ok_or("Не выбран проект".to_string())?
    };
    let mut container = state.container();
    {
        let task_service = TaskService::new(&mut *container);
        let tasks = task_service.get_all_tasks(project_id);
        for task in tasks {
            if *task.get_date_start() < start || *task.get_date_end() > end {
                return Err(format!(
                    "Даты задачи '{}' не входят в даты проекта",
                    task.name
                ));
            }
        }
    }
    let project = container
        .get_project_mut(&project_id)
        .ok_or("Проект не найден".to_string())?;

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
    let path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_pick_file();

    let path = match path {
        Some(p) => p,
        None => return Ok(()),
    };
    let file_path = path
        .as_path()
        .ok_or("Не удалось получить путь к файлу".to_string())?;

    let content =
        std::fs::read_to_string(file_path).map_err(|e| format!("Ошибка чтения файла: {}", e))?;

    let new_container: SingleProjectContainer =
        serde_json::from_str(&content).map_err(|e| format!("Ошибка обработки файла: {}", e))?;

    let project_id = new_container
        .list_projects()
        .first()
        .map(|p| *p.get_id())
        .ok_or("Не нашли проектов в загруженном файле")?;
    *state.container() = new_container;
    *state.selected_project_id.lock().unwrap() = Some(project_id);

    // Сброс состояний
    *state.selected_task_id.lock().unwrap() = None;
    *state.selected_resource_id.lock().unwrap() = None;
    *state.critical_path.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn close_project(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Закрываем проект - диалог сохранения показываем на фронтенде
    *state.container() = SingleProjectContainer::new();
    *state.selected_project_id.lock().unwrap() = None;
    *state.selected_task_id.lock().unwrap() = None;
    *state.selected_resource_id.lock().unwrap() = None;
    *state.critical_path.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn save_project(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 1. Сериализуем (держим блокировку только на время сериализации)
    let content = {
        let container = state.container();
        serde_json::to_string_pretty(&*container)
            .map_err(|e| format!("Не удалось сохранить файл: {}", e))?
    };

    // 2. Диалог сохранения
    let path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_save_file();

    let path = match path {
        Some(p) => p,
        None => return Ok(()),
    };

    // 3. Пишем в файл
    let file_path = path.as_path().ok_or("Не удалось получить путь к файлу")?;
    std::fs::write(file_path, &content).map_err(|e| format!("Ошибка записи файла: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_project_info(
    state: tauri::State<'_, AppState>,
    project_id: Option<Uuid>,
) -> Result<ProjectInfo, String> {
    match project_id {
        Some(pr) => ProjectInfo::from_state(state, pr),
        None => {
            let pr = {
                let id = state.selected_project_id.lock().unwrap();
                id.ok_or("Не выбран проект".to_string())?
            };
            ProjectInfo::from_state(state, pr)
        }
    }
}
