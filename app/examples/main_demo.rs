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

    // Период недоступности (опционально)
    let vacation = ExceptionPeriod {
        period: TimeWindow::new(
            Utc.with_ymd_and_hms(2025, 4, 10, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 4, 20, 0, 0, 0).unwrap(),
        )?,
        exception_type: ExceptionType::Vacation,
    };
    resource_service.add_unavailable_period(analyst.id, vacation)?;

    let mut task_service = TaskService::new(&mut container);

    // --- Создание суммарных задач (групп) ---
    let summary1 =
        task_service.create_summary_task(project_id, "Анализ и проектирование".into(), None)?;
    let summary1_id = *summary1.get_id();

    let summary2 = task_service.create_summary_task(project_id, "Разработка".into(), None)?;
    let summary2_id = *summary2.get_id();

    let summary3 = task_service.create_summary_task(project_id, "Тестирование".into(), None)?;
    let summary3_id = *summary3.get_id();

    // --- Подзадачи группы 1 ---
    // Критическая: Сбор требований (5 дней) – аналитик
    let task1 = task_service.create_regular_task(
        project_id,
        "Сбор требований".into(),
        Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
        Some(summary1_id),
    )?;
    let task1_id = *task1.get_id();

    // Некритическая: Анализ рынка (3 дня) – разработчик (он ещё свободен)
    let task1b = task_service.create_regular_task(
        project_id,
        "Анализ рынка".into(),
        Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 4, 0, 0, 0).unwrap(),
        Some(summary1_id),
    )?;
    let task1b_id = *task1b.get_id();

    // --- Подзадачи группы 2 ---
    // Критическая: Разработка модуля A (10 дней) – разработчик
    let task2 = task_service.create_regular_task(
        project_id,
        "Разработка модуля A".into(),
        Utc.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 16, 0, 0, 0).unwrap(),
        Some(summary2_id),
    )?;
    let task2_id = *task2.get_id();

    // Некритическая: Разработка модуля B (7 дней) – тестировщик (свободен)
    let task2b = task_service.create_regular_task(
        project_id,
        "Разработка модуля B".into(),
        Utc.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 13, 0, 0, 0).unwrap(),
        Some(summary2_id),
    )?;
    let task2b_id = *task2b.get_id();

    // --- Подзадачи группы 3 ---
    // Критическая: Модульное тестирование (4 дня) – тестировщик
    let task3 = task_service.create_regular_task(
        project_id,
        "Модульное тестирование".into(),
        Utc.with_ymd_and_hms(2025, 3, 16, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 20, 0, 0, 0).unwrap(),
        Some(summary3_id),
    )?;
    let task3_id = *task3.get_id();

    // Некритическая: Интеграционное тестирование (6 дней) – аналитик (свободен)
    let task3b = task_service.create_regular_task(
        project_id,
        "Интеграционное тестирование".into(),
        Utc.with_ymd_and_hms(2025, 3, 16, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 22, 0, 0, 0).unwrap(),
        Some(summary3_id),
    )?;
    let task3b_id = *task3b.get_id();

    // --- Зависимости для формирования критического пути ---
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

    // --- Назначение ресурсов (теперь без конфликтов) ---
    task_service.allocate_resource(project_id, task1_id, analyst.id, 0.8, None)?;
    task_service.allocate_resource(project_id, task1b_id, dev.id, 0.3, None)?; // разработчик свободен
    task_service.allocate_resource(project_id, task2_id, dev.id, 1.0, None)?;
    task_service.allocate_resource(project_id, task2b_id, tester.id, 0.6, None)?; // тестировщик свободен
    task_service.allocate_resource(project_id, task3_id, tester.id, 0.9, None)?;
    task_service.allocate_resource(project_id, task3b_id, analyst.id, 0.5, None)?; // аналитик свободен после 6 марта (его критическая закончилась)

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
