use crate::dto::ResourceInfo;
use crate::state::AppState;
use crate::{
    commands::utils::{parse_date, resolve_project_id},
    dto::ResourceUpdateDto,
};
use logic::{
    ExceptionPeriod, ExceptionType, ProjectContainer, RateMeasure, ResourceService, TimeWindow,
};
use uuid::Uuid;

#[tauri::command]
pub fn add_resource(
    state: tauri::State<AppState>,
    name: String,
    rate: f64,
    rate_measure: RateMeasure,
) -> Result<(), String> {
    let mut container = state.container();
    let mut resource_service = ResourceService::new(&mut *container);

    let resource = resource_service
        .create_resource(name, rate, rate_measure)
        .map_err(|e| e.to_string())?;
    resource_service
        .add_resource(resource)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn edit_resource(
    state: tauri::State<AppState>,
    resource_id: Uuid,
    resource_update: ResourceUpdateDto,
) -> Result<(), String> {
    let mut container = state.container();
    let mut resource_service = ResourceService::new(&mut *container);

    resource_service
        .update_resource(
            resource_id,
            resource_update.name,
            resource_update.rate,
            resource_update.rate_measure,
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn delete_resource(state: tauri::State<AppState>, resource_id: Uuid) -> Result<(), String> {
    let mut container = state.container();
    let mut resource_service = ResourceService::new(&mut *container);

    resource_service
        .delete_resource(resource_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn get_resources(state: tauri::State<AppState>) -> Result<Vec<ResourceInfo>, String> {
    let mut container = state.container();
    let resource_service = ResourceService::new(&mut *container);

    let resources = resource_service
        .list_resources()
        .into_iter()
        .map(ResourceInfo::from_resource)
        .collect();

    Ok(resources)
}

#[tauri::command]
pub fn add_unavailable_period(
    state: tauri::State<AppState>,
    resource_id: Uuid,
    date_start: Option<String>,
    date_end: Option<String>,
    exception_type: ExceptionType,
) -> Result<(), String> {
    let ex_period_start = parse_date(&date_start.ok_or("Не указано начало периода")?)?;
    let ex_period_end = parse_date(&date_end.ok_or("Не указано окончание периода")?)?;

    let period = TimeWindow::new(ex_period_start, ex_period_end).map_err(|e| e.to_string())?;
    let exception_period = ExceptionPeriod {
        period,
        exception_type,
    };
    let mut container = state.container();
    let mut resource_service = ResourceService::new(&mut *container);
    resource_service
        .add_unavailable_period(resource_id, exception_period)
        .map_err(|e| e.to_string())?;

    Ok(())
}
#[tauri::command]
pub fn check_availability(
    state: tauri::State<AppState>,

    resource_id: Uuid,
    date_start: Option<String>,
    date_end: Option<String>,
) -> Result<bool, String> {
    let project_id = resolve_project_id(&state, None)?;

    let period_start = parse_date(&date_start.ok_or("Не указано начало периода")?)?;
    let period_end = parse_date(&date_end.ok_or("Не указано окончание периода")?)?;

    let checked_time_window =
        TimeWindow::new(period_start, period_end).map_err(|e| e.to_string())?;

    let container = state.container();
    let resource = container
        .resource_pool()
        .get_resource(&resource_id)
        .ok_or_else(|| format!("Ресурс {resource_id} не найден"))?;
    let calendar = container
        .calendar(&project_id)
        .ok_or_else(|| "Календарь проекта не найден".to_string())?;

    Ok(resource.is_available(&checked_time_window, calendar))
}
