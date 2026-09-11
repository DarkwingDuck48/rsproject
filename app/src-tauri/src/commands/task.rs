use chrono::TimeDelta;
use logic::{
    BasicGettersForStructures, DependencyType, ProjectContainer, Scheduler, SingleProjectContainer,
    Task, TaskService, TaskUpdate, TimeWindow,
};
use uuid::Uuid;

use crate::commands::utils::{parse_date, resolve_project_id};
use crate::dto::{TaskAllocationInfo, TaskDetailInfo, TaskInfo, TaskTreeNode, TaskUpdateDto};
use crate::state::AppState;

impl TryFrom<TaskUpdateDto> for TaskUpdate {
    type Error = String;

    fn try_from(dto: TaskUpdateDto) -> Result<Self, Self::Error> {
        let start = dto.date_start.as_deref().map(parse_date).transpose()?;
        let end = dto.date_end.as_deref().map(parse_date).transpose()?;
        Ok(TaskUpdate {
            name: dto.name,
            start,
            end,
            parent_id: dto.parent_id,
            status: dto.status,
        })
    }
}

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
pub fn get_task(state: tauri::State<AppState>, task_id: Uuid) -> Result<TaskDetailInfo, String> {
    let project_id = resolve_project_id(&state, None)?;
    let mut container = state.container();
    let task_service = TaskService::new(&mut *container);

    let task = task_service
        .get_task_by_id(&project_id, &task_id)
        .ok_or(format!("Не найдена задача с ID {} ", task_id))?;
    let cost = task_service
        .calculate_task_cost(&project_id, &task_id)
        .unwrap_or(0.0);

    // Резолвим аллокации: ID -> объект -> DTO
    let pool = task_service.container.resource_pool();
    let allocations = task
        .get_resource_allocations()
        .iter()
        .filter_map(|id| {
            pool.get_allocation(id)
                .map(TaskAllocationInfo::from_allocation)
        })
        .collect();

    Ok(TaskDetailInfo {
        task: TaskInfo::from_task(task, cost),
        allocations,
    })
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

    let update = TaskUpdate::try_from(task_update)?;

    task_service
        .update_task(project_id, task_id, update)
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

#[tauri::command]
pub fn add_dependency(
    state: tauri::State<AppState>,
    task_id: Uuid,
    depends_on: Uuid,
    dep_type: DependencyType,
    lag_days: Option<i64>,
) -> Result<(), String> {
    let project_id = resolve_project_id(&state, None)?;

    let lag = match lag_days {
        Some(days) => Some(TimeDelta::try_days(days).ok_or("Лаг слишком большой")?),
        _ => None,
    };

    let mut container = state.container();
    let mut task_service = TaskService::new(&mut *container);
    task_service
        .add_dependency(project_id, task_id, depends_on, dep_type, lag)
        .map_err(|e| e.to_string())?;
    *state.critical_path.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn remove_dependency(
    state: tauri::State<AppState>,
    task_id: Uuid,
    depends_on: Uuid,
) -> Result<(), String> {
    let project_id = resolve_project_id(&state, None)?;

    let mut container = state.container();
    let mut task_service = TaskService::new(&mut *container);
    task_service
        .remove_dependency(&project_id, &task_id, depends_on)
        .map_err(|e| e.to_string())?;
    *state.critical_path.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn assign_resource(
    state: tauri::State<AppState>,
    task_id: Uuid,
    resource_id: Uuid,
    engagement: f64,
    date_start: Option<String>,
    date_end: Option<String>,
) -> Result<(), String> {
    let project_id = resolve_project_id(&state, None)?;

    let allocation_window = match (date_start, date_end) {
        (Some(start), Some(end)) => {
            let s = parse_date(&start)?;
            let e = parse_date(&end)?;
            Some(TimeWindow::new(s, e).map_err(|x| x.to_string())?)
        }
        (None, None) => None,
        _ => return Err("Укажите обе даты окна либо ни одной".to_string()),
    };

    let mut container = state.container();
    let mut task_service = TaskService::new(&mut *container);

    task_service
        .allocate_resource(
            project_id,
            task_id,
            resource_id,
            engagement,
            allocation_window,
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_assignment(
    state: tauri::State<AppState>,
    task_id: Uuid,
    allocation_id: Uuid,
) -> Result<(), String> {
    let project_id = resolve_project_id(&state, None)?;
    let mut container = state.container();
    let mut task_service = TaskService::new(&mut *container);
    task_service
        .remove_allocation(&project_id, &task_id, allocation_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}
