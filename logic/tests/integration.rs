use chrono::{TimeZone, Utc};
use logic::{
    BasicGettersForStructures, DependencyType, ExceptionPeriod, ExceptionType, Project,
    ProjectContainer, RateMeasure, ResourceService, Scheduler, SingleProjectContainer, TaskService,
    TaskUpdate, TimeWindow,
};

#[test]
fn test_full_scenario() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();

    // Создаем проект внутри контейнера
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Integration test", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    // Создание задачи через отдельный выделенный namespace
    let (task_id, task_start, task_end) = {
        let mut task_service = TaskService::new(&mut container);
        let task_start = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
        let task_end = Utc.with_ymd_and_hms(2026, 2, 15, 0, 0, 0).unwrap();

        let task = task_service.create_regular_task(
            project_id,
            "Design".into(),
            task_start,
            task_end,
            None,
        )?;
        let task_id = *task.get_id();
        (task_id, task_start, task_end)
    };

    // Создаем ресурс через Resource Service
    let resource_id = {
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("Max", 1000.0, RateMeasure::Hourly)?;
        resource_service.add_resource(resource.clone())?;

        // Добавляем период недоступности
        let vacation = ExceptionPeriod {
            period: TimeWindow::new(
                Utc.with_ymd_and_hms(2025, 2, 16, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2025, 2, 20, 0, 0, 0).unwrap(),
            )?,
            exception_type: ExceptionType::Vacation,
        };
        resource_service.add_unavailable_period(resource.id, vacation)?;
        resource.id
    };

    {
        let mut task_service = TaskService::new(&mut container);
        let time_window = TimeWindow::new(task_start, task_end)?;
        task_service.allocate_resource(project_id, task_id, resource_id, 0.8, Some(time_window))?;
    }

    let utilization = {
        let resource_service = ResourceService::new(&mut container);
        resource_service.get_resource_utilization(resource_id)
    };
    assert_eq!(utilization, 0.8);

    // Проверяем стоимость задачи
    let task_cost = {
        let task_service = TaskService::new(&mut container);
        task_service.calculate_task_cost(&project_id, &task_id)?
    };
    eprintln!("Calculated task cost: {}", task_cost);
    // 80 часов (10 рабочих дней) * 0.8 engagement rate * 1000 hourly rate
    assert!(task_cost == 1000.0 * 0.8 * 80.0);

    Ok(())
}

/// Проверка на цикл при смене родителя (задача 5.20): задача не может стать
/// потомком своего собственного потомка. Дерево: A → B → C.
#[test]
fn test_cannot_make_task_child_of_own_descendant() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();

    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Cycle test", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let task_date = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
    let (task_a, task_b, task_c) = {
        let mut task_service = TaskService::new(&mut container);
        let a = task_service.create_regular_task(
            project_id,
            "A".into(),
            task_date,
            task_date + chrono::TimeDelta::days(5),
            None,
        )?;
        let a_id = *a.get_id();

        let b = task_service.create_regular_task(
            project_id,
            "B".into(),
            task_date,
            task_date + chrono::TimeDelta::days(5),
            Some(a_id),
        )?;
        let b_id = *b.get_id();

        let c = task_service.create_regular_task(
            project_id,
            "C".into(),
            task_date,
            task_date + chrono::TimeDelta::days(5),
            Some(b_id),
        )?;
        let c_id = *c.get_id();

        (a_id, b_id, c_id)
    };

    // Сделать A (корень) потомком C (её потомка) → цикл → ошибка.
    let cycle_error = {
        let mut task_service = TaskService::new(&mut container);
        task_service.update_task(
            project_id,
            task_a,
            TaskUpdate {
                name: None,
                start: None,
                end: None,
                parent_id: Some(Some(task_c)),
                status: None,
            },
        )
    };
    assert!(
        cycle_error.is_err(),
        "Ожидали ошибку о цикле, но задача A стала потомком C"
    );

    // Дерево не изменилось: A всё ещё корень, B — потомок A.
    {
        let task_service = TaskService::new(&mut container);
        let a = task_service
            .get_project(&project_id)
            .unwrap()
            .tasks
            .get(&task_a)
            .unwrap();
        assert_eq!(a.parent_id, None);
        let b = task_service
            .get_project(&project_id)
            .unwrap()
            .tasks
            .get(&task_b)
            .unwrap();
        assert_eq!(b.parent_id, Some(task_a));
    }

    // Позитивный случай: B (потомок A) → ставим родителем A — цикла нет.
    let ok = {
        let mut task_service = TaskService::new(&mut container);
        task_service.update_task(
            project_id,
            task_b,
            TaskUpdate {
                name: None,
                start: None,
                end: None,
                parent_id: Some(Some(task_a)),
                status: None,
            },
        )
    };
    assert!(
        ok.is_ok(),
        "Перенос B под A (не цикл) должен быть разрешён: {:?}",
        ok.err()
    );

    Ok(())
}

/// Сброс родителя в корень (задача 5.20): `Some(None)` очищает parent_id
/// и не должен считаться циклом.
#[test]
fn test_clear_parent_to_root() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();

    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Clear parent", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let task_date = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
    let (parent_id, child_id) = {
        let mut task_service = TaskService::new(&mut container);
        let parent = task_service.create_regular_task(
            project_id,
            "Parent".into(),
            task_date,
            task_date + chrono::TimeDelta::days(5),
            None,
        )?;
        let parent_id = *parent.get_id();

        let child = task_service.create_regular_task(
            project_id,
            "Child".into(),
            task_date,
            task_date + chrono::TimeDelta::days(5),
            Some(parent_id),
        )?;
        (parent_id, *child.get_id())
    };

    {
        let mut task_service = TaskService::new(&mut container);
        task_service.update_task(
            project_id,
            child_id,
            TaskUpdate {
                name: None,
                start: None,
                end: None,
                parent_id: Some(None), // очистить родителя
                status: None,
            },
        )?;
    }

    let child = {
        let task_service = TaskService::new(&mut container);
        task_service
            .get_project(&project_id)
            .unwrap()
            .tasks
            .get(&child_id)
            .unwrap()
            .clone()
    };
    assert_eq!(child.parent_id, None);

    // Самого родителя трогать не должны.
    let parent = {
        let task_service = TaskService::new(&mut container);
        task_service
            .get_project(&project_id)
            .unwrap()
            .tasks
            .get(&parent_id)
            .unwrap()
            .clone()
    };
    assert_eq!(parent.parent_id, None);

    Ok(())
}

/// Дата окончания проекта не влияет на критический путь (issue #17).
///
/// Планировщик считает путь как самую длинную цепочку работ от даты старта проекта
/// по длительностям и зависимостям: поздний финиш задач без последователей берётся
/// равным расчётному окончанию, а не дате окончания проекта. Поэтому широкое окно
/// проекта (дата окончания намного позже расчётного финиша) не делает путь пустым.
#[test]
fn test_critical_path_does_not_depend_on_project_end() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();

    // Окно проекта — почти год, а работы занимают около двух недель.
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Critical path vs project end", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    // A (10 дней) → B (5 дней): расчётный финиш — середина января, то есть
    // задолго до даты окончания проекта.
    let (task_a, task_b) = {
        let mut task_service = TaskService::new(&mut container);
        let a_start = Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap();

        let a = task_service.create_regular_task(
            project_id,
            "A".into(),
            a_start,
            a_start + chrono::TimeDelta::days(10),
            None,
        )?;
        let a_id = *a.get_id();

        let b = task_service.create_regular_task(
            project_id,
            "B".into(),
            a_start + chrono::TimeDelta::days(10),
            a_start + chrono::TimeDelta::days(15),
            None,
        )?;
        let b_id = *b.get_id();

        task_service.add_dependency(project_id, b_id, a_id, DependencyType::Blocking, None)?;

        (a_id, b_id)
    };

    let critical = {
        let scheduler = Scheduler::new(&container);
        scheduler.critical_path(project_id)?
    };

    assert!(
        !critical.is_empty(),
        "критический путь не должен пропадать из-за даты окончания проекта"
    );
    assert_eq!(critical, vec![task_a, task_b]);

    Ok(())
}
