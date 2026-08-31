use logic::{BasicGettersForStructures, Scheduler, SingleProjectContainer, Task, TaskService};
use uuid::Uuid;

use crate::commands::utils::{parse_date, resolve_project_id};
use crate::dto::{TaskInfo, TaskTreeNode, TaskUpdateDto};
use crate::state::AppState;

#[tauri::command]
pub fn add_task(
    state: tauri::State<AppState>,
    name: String,
    date_start: Option<String>,
    date_end: Option<String>,
    is_summary: bool,
    parent_id: Option<Uuid>,
) -> Result<(), String> {
    let project_id = resolve_project_id(&state, None)?;
    let mut container = state.container();
    let mut task_service = TaskService::new(&mut *container);

    if is_summary {
        task_service
            .create_summary_task(project_id, name, parent_id)
            .map_err(|e| e.to_string())?;
    } else {
        let start_task_date = parse_date(&date_start.ok_or("Не указана дата начала для задачи")?)?;
        let end_task_date = parse_date(&date_end.ok_or("Не указана дата окончания для задачи")?)?;
        task_service
            .create_regular_task(project_id, name, start_task_date, end_task_date, parent_id)
            .map_err(|e| e.to_string())?;
    }
    *state.critical_path.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn get_task(state: tauri::State<AppState>, task_id: Uuid) -> Result<TaskInfo, String> {
    let project_id = resolve_project_id(&state, None)?;
    TaskInfo::from_state(state, project_id, task_id)
}

#[tauri::command]
pub fn edit_task(
    state: tauri::State<AppState>,
    task_id: Uuid,
    task_update: TaskUpdateDto,
) -> Result<(), String> {
    let project_id = resolve_project_id(&state, None)?;

    let mut container = state.container();
    let mut task_service = TaskService::new(&mut *container);

    let updated_date_start = task_update
        .date_start
        .as_deref()
        .map(parse_date)
        .transpose()?;

    let updated_date_end = task_update
        .date_end
        .as_deref()
        .map(parse_date)
        .transpose()?;

    task_service
        .update_task(
            project_id,
            task_id,
            task_update.name,
            updated_date_start,
            updated_date_end,
            task_update.parent_id,
            task_update.status,
        )
        .map_err(|e| e.to_string())?;
    *state.critical_path.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn delete_task(state: tauri::State<AppState>, task_id: Uuid) -> Result<(), String> {
    let project_id = resolve_project_id(&state, None)?;
    let mut container = state.container();
    let mut task_service = TaskService::new(&mut *container);

    task_service
        .delete_task(project_id, task_id)
        .map_err(|e| e.to_string())?;
    *state.critical_path.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn get_tasks(
    state: tauri::State<AppState>,
    project_id: Option<Uuid>,
) -> Result<Vec<TaskInfo>, String> {
    let resolved_project_id = resolve_project_id(&state, project_id)?;
    let mut container = state.container();
    let task_service = TaskService::new(&mut *container);

    let tasks = task_service.get_all_tasks(resolved_project_id);
    let task_info: Vec<TaskInfo> = tasks
        .iter()
        .map(|t| {
            let cost = task_service
                .calculate_task_cost(&resolved_project_id, t.get_id())
                .unwrap_or(0.0);
            TaskInfo::from_task(t, cost)
        })
        .collect();

    Ok(task_info)
}

fn build_node(
    task_service: &TaskService<'_, SingleProjectContainer>,
    project_id: &Uuid,
    task: &Task,
) -> TaskTreeNode {
    let subtasks = task_service.get_subtasks(project_id, *task.get_id());
    let children = subtasks
        .iter()
        .map(|t| build_node(task_service, project_id, t))
        .collect();
    let cost = task_service
        .calculate_task_cost(project_id, task.get_id())
        .unwrap_or(0.0);
    TaskTreeNode {
        task: TaskInfo::from_task(task, cost),
        children,
    }
}
#[tauri::command]
pub fn get_task_tree(
    state: tauri::State<AppState>,
    project_id: Option<Uuid>,
) -> Result<Vec<TaskTreeNode>, String> {
    let project_id = resolve_project_id(&state, project_id)?;
    let mut container = state.container();
    let task_service = TaskService::new(&mut *container);
    let roots = task_service.get_root_tasks(project_id);
    Ok(roots
        .iter()
        .map(|t| build_node(&task_service, &project_id, t))
        .collect())
}

#[tauri::command]
pub fn recalculate_critical_path(
    state: tauri::State<AppState>,
    project_id: Option<Uuid>,
) -> Result<Vec<Uuid>, String> {
    let resolved_project_id = resolve_project_id(&state, project_id)?;
    let critical_path = {
        let container = state.container();
        Scheduler::new(&*container)
            .critical_path(resolved_project_id)
            .map_err(|e| e.to_string())?
    };
    *state.critical_path.lock().unwrap() = Some(critical_path.clone());
    Ok(critical_path)
}
