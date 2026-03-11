use app::ProjectApp;
use chrono::{Duration, TimeZone, Utc};
use logic::{
    BasicGettersForStructures, DependencyType, ExceptionPeriod, ExceptionType, Project,
    ProjectContainer, RateMeasure, ResourceService, SingleProjectContainer, TaskService,
    TimeWindow,
};

fn build_demo_container() -> anyhow::Result<SingleProjectContainer> {
    let mut container = SingleProjectContainer::new();

    let project_start = Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap();
    let project_end = Utc.with_ymd_and_hms(2025, 6, 30, 0, 0, 0).unwrap();
    let project = Project::new(
        "Demo Project",
        "A sample project to demonstrate the application",
        project_start,
        project_end,
    )?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let mut resource_service = ResourceService::new(&mut container);

    // Ресурсы
    let analyst = resource_service.create_resource("Analyst", 1500.0, RateMeasure::Daily)?;
    let dev = resource_service.create_resource("Developer", 2000.0, RateMeasure::Daily)?;
    let tester = resource_service.create_resource("Tester", 1200.0, RateMeasure::Daily)?;
    resource_service.add_resource(analyst.clone())?;
    resource_service.add_resource(dev.clone())?;
    resource_service.add_resource(tester.clone())?;

    // Период недоступности
    let vacation = ExceptionPeriod {
        period: TimeWindow::new(
            Utc.with_ymd_and_hms(2025, 4, 10, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 4, 20, 0, 0, 0).unwrap(),
        )?,
        exception_type: ExceptionType::Vacation,
    };
    resource_service.add_unavailable_period(analyst.id, vacation)?;

    let mut task_service = TaskService::new(&mut container);
    // Задачи
    let task1 = task_service.create_task(
        project_id,
        "Requirements Analysis".into(),
        Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(),
    )?;
    let task1_id = *task1.get_id();

    let task2 = task_service.create_task(
        project_id,
        "System Design".into(),
        Utc.with_ymd_and_hms(2025, 3, 15, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 4, 5, 0, 0, 0).unwrap(),
    )?;
    let task2_id = *task2.get_id();

    let task3 = task_service.create_task(
        project_id,
        "Implementation".into(),
        Utc.with_ymd_and_hms(2025, 4, 6, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 5, 3, 0, 0, 0).unwrap(),
    )?;
    let task3_id = *task3.get_id();

    let task4 = task_service.create_task(
        project_id,
        "Testing".into(),
        Utc.with_ymd_and_hms(2025, 5, 4, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 5, 24, 0, 0, 0).unwrap(),
    )?;
    let task4_id = *task4.get_id();

    let task5 = task_service.create_task(
        project_id,
        "Documentation".into(),
        Utc.with_ymd_and_hms(2025, 5, 4, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 5, 17, 0, 0, 0).unwrap(),
    )?;
    let task5_id = *task5.get_id();

    // Зависимости
    task_service.add_dependency(
        project_id,
        task2_id,
        task1_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        task3_id,
        task2_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        task4_id,
        task3_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;

    // Назначения
    task_service.allocate_resource(project_id, task1_id, analyst.id, 1.0, None)?;
    task_service.allocate_resource(project_id, task2_id, analyst.id, 0.5, None)?;
    task_service.allocate_resource(project_id, task2_id, dev.id, 0.5, None)?;
    task_service.allocate_resource(project_id, task3_id, dev.id, 1.0, None)?;
    task_service.allocate_resource(project_id, task4_id, tester.id, 1.0, None)?;
    task_service.allocate_resource(project_id, task5_id, analyst.id, 0.3, None)?;

    Ok(container)
}

fn main() -> eframe::Result<()> {
    let container = build_demo_container().expect("Failed to build demo container");
    let app = ProjectApp::with_container(container);
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Project Manager (Demo)",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
