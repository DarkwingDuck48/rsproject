use chrono::{TimeZone, Utc};
use logic::{
    BasicGettersForStructures, ExceptionPeriod, ExceptionType, Project, ProjectContainer,
    RateMeasure, ResourceService, SingleProjectContainer, TaskService, TimeWindow,
};

#[test]
fn test_full_scenario() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();

    // Создаем проект внутри контейнера
    let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Integration test", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    // Создаем задачу через Task Service
    let mut task_service = TaskService::new(&mut container);
    let task_start = Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();
    let task_end = Utc.with_ymd_and_hms(2025, 2, 15, 0, 0, 0).unwrap();

    let task = task_service.create_task(project_id, "Design".into(), task_start, task_end)?;
    let _task_id = *task.get_id();

    // Создаем ресурс через Resource Service
    let mut resource_service = ResourceService::new(&mut container);
    let resource = resource_service.create_resource("Max", 1000.0, RateMeasure::Hourly)?;
    resource_service.add_resource(resource.clone())?;

    // Добавляем период недоступности
    let vacation = ExceptionPeriod {
        period: TimeWindow::new(
            Utc.with_ymd_and_hms(2025, 2, 10, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 2, 20, 0, 0, 0).unwrap(),
        )?,
        exception_type: ExceptionType::Vacation,
    };
    resource_service.add_unavailable_period(resource.id, vacation)?;

    // Проверяем, что ресурс доступен для задачи (если задача раньше отпуска)
    let calendar = resource_service.get_calendar(&project_id).unwrap();
    assert!(resource.is_available(&TimeWindow::new(task_start, task_end)?, calendar));

    // TODO: назначение ресурса (будет реализовано позже)

    // Проверяем утилизацию (пока 0)
    let utilization = resource_service.get_resource_utilization(resource.id);
    assert_eq!(utilization, 0.0);

    Ok(())
}
