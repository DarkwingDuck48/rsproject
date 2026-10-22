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

// ===== Задача 6.1: тесты аллокации ресурсов (перегрузка, окна, календарь и исключения) =====

/// Перегрузка ресурса: два назначения с пересекающимися окнами, суммарная занятость
/// которых больше 1.0, — второе должно быть отклонено.
#[test]
fn test_allocation_rejects_overload() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Overload", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(
            project_id,
            "Task".into(),
            start + chrono::TimeDelta::days(10),
            start + chrono::TimeDelta::days(40),
            None,
        )?;
        let task_id = *task.get_id();

        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_id, resource_id)
    };

    let window = TimeWindow::new(
        start + chrono::TimeDelta::days(10),
        start + chrono::TimeDelta::days(20),
    )?;

    let mut task_service = TaskService::new(&mut container);
    // Первое назначение: 0.7 занятости.
    task_service.allocate_resource(project_id, task_id, resource_id, 0.7, Some(window))?;
    // Второе с тем же окном: 0.7 + 0.4 = 1.1 > 1.0 — перегрузка.
    let second =
        task_service.allocate_resource(project_id, task_id, resource_id, 0.4, Some(window));
    assert!(
        second.is_err(),
        "назначение с суммарной занятостью > 1.0 должно быть отклонено"
    );

    Ok(())
}

/// Граничное значение: суммарная занятость ровно 1.0 разрешена (два назначения 0.5 + 0.5).
#[test]
fn test_allocation_allows_sum_exactly_one() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Boundary 1.0", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(
            project_id,
            "Task".into(),
            start + chrono::TimeDelta::days(10),
            start + chrono::TimeDelta::days(40),
            None,
        )?;
        let task_id = *task.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_id, resource_id)
    };

    let window = TimeWindow::new(
        start + chrono::TimeDelta::days(10),
        start + chrono::TimeDelta::days(20),
    )?;

    let mut task_service = TaskService::new(&mut container);
    task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(window))?;
    let second =
        task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(window));
    assert!(
        second.is_ok(),
        "суммарная занятость ровно 1.0 должна быть разрешена: {:?}",
        second.err()
    );

    Ok(())
}

/// Непересекающиеся окна: сумма занятости может превышать 1.0, если ресурс
/// работает над задачами в разное время.
#[test]
fn test_allocation_disjoint_windows_with_sum_over_one() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Disjoint windows", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(
            project_id,
            "Task".into(),
            start + chrono::TimeDelta::days(10),
            start + chrono::TimeDelta::days(60),
            None,
        )?;
        let task_id = *task.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_id, resource_id)
    };

    let mut task_service = TaskService::new(&mut container);
    let w1 = TimeWindow::new(
        start + chrono::TimeDelta::days(10),
        start + chrono::TimeDelta::days(20),
    )?;
    let w2 = TimeWindow::new(
        start + chrono::TimeDelta::days(30),
        start + chrono::TimeDelta::days(40),
    )?;

    task_service.allocate_resource(project_id, task_id, resource_id, 0.7, Some(w1))?;
    // Окна не пересекаются — 0.7 + 0.7 = 1.4 допустимо.
    let second = task_service.allocate_resource(project_id, task_id, resource_id, 0.7, Some(w2));
    assert!(
        second.is_ok(),
        "непересекающиеся окна не должны приводить к перегрузке: {:?}",
        second.err()
    );

    Ok(())
}

/// Календарь и исключения: назначение на период отпуска ресурса отклоняется.
#[test]
fn test_allocation_rejected_during_vacation() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Vacation", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(
            project_id,
            "Task".into(),
            start + chrono::TimeDelta::days(10),
            start + chrono::TimeDelta::days(60),
            None,
        )?;
        let task_id = *task.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource.clone())?;

        // Отпуск ресурса в течение первых 15 дней окна задачи.
        let vacation = ExceptionPeriod {
            period: TimeWindow::new(
                start + chrono::TimeDelta::days(15),
                start + chrono::TimeDelta::days(30),
            )?,
            exception_type: ExceptionType::Vacation,
        };
        resource_service.add_unavailable_period(resource_id, vacation)?;
        (task_id, resource_id)
    };

    let mut task_service = TaskService::new(&mut container);
    // Окно полностью внутри отпуска — назначение невозможно.
    let in_vacation = TimeWindow::new(
        start + chrono::TimeDelta::days(18),
        start + chrono::TimeDelta::days(22),
    )?;
    let failed =
        task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(in_vacation));
    assert!(
        failed.is_err(),
        "назначение на период отпуска должно быть отклонено"
    );

    // Окно после отпуска — доступно.
    let after_vacation = TimeWindow::new(
        start + chrono::TimeDelta::days(35),
        start + chrono::TimeDelta::days(45),
    )?;
    let ok =
        task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(after_vacation));
    assert!(
        ok.is_ok(),
        "назначение вне отпуска должно проходить: {:?}",
        ok.err()
    );

    Ok(())
}

/// Календарь: выходные дни не считаются рабочими, поэтому окно, состоящее только
/// из выходных, недоступно для назначения.
#[test]
fn test_allocation_rejected_on_weekend_only_window() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Weekend", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(project_id, "Task".into(), start, end, None)?;
        let task_id = *task.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_id, resource_id)
    };

    // 2026-01-03 — суббота, 2026-01-04 — воскресенье: в окне нет рабочих дней.
    // Окно [03.01, 04.01): подсчёт идёт по датам включительно до конца — 03.01 и 04.01
    // оба выходные, рабочих дней ноль.
    let weekend = TimeWindow::new(
        Utc.with_ymd_and_hms(2026, 1, 3, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 1, 4, 0, 0, 0).unwrap(),
    )?;
    let mut task_service = TaskService::new(&mut container);
    let result =
        task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(weekend));
    assert!(
        result.is_err(),
        "окно без рабочих дней не должно принимать назначения"
    );

    Ok(())
}

// ===== Задача 6.1: тесты расчёта стоимости =====

/// Стоимость задачи с почасовой ставкой: rate * рабочие часы * engagement.
#[test]
fn test_task_cost_hourly_rate() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Hourly cost", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    // 2026-01-01 — четверг, поэтому 02.01 (пт), 03.01 (сб), 04.01 (вс),
    // 05.01 (пн) ... 09.01 (пт). Окно [05.01, 10.01) с учётом включения даты окончания
    // в подсчёт рабочих дней даёт ровно 5 рабочих дней (пн–пт).
    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(
            project_id,
            "Task".into(),
            Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 1, 10, 0, 0, 0).unwrap(),
            None,
        )?;
        let task_id = *task.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_id, resource_id)
    };

    let window = TimeWindow::new(
        Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 1, 10, 0, 0, 0).unwrap(),
    )?;

    let mut task_service = TaskService::new(&mut container);
    task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(window))?;

    let task_service = TaskService::new(&mut container);
    let cost = task_service.calculate_task_cost(&project_id, &task_id)?;
    // 5 рабочих дней * 8 часов = 40 часов * 100 ₽/ч * 0.5 engagement = 2000 ₽
    assert_eq!(cost, 40.0 * 100.0 * 0.5);

    Ok(())
}

/// Стоимость задачи с дневной ставкой: ставка делится на часы в рабочем дне.
#[test]
fn test_task_cost_daily_rate() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Daily cost", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    // 800 ₽/день при 8 часах в рабочем дне эквивалентны 100 ₽/час.
    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(
            project_id,
            "Task".into(),
            Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 1, 10, 0, 0, 0).unwrap(),
            None,
        )?;
        let task_id = *task.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 800.0, RateMeasure::Daily)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_id, resource_id)
    };

    let window = TimeWindow::new(
        Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 1, 10, 0, 0, 0).unwrap(),
    )?;

    let mut task_service = TaskService::new(&mut container);
    task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(window))?;

    let task_service = TaskService::new(&mut container);
    let cost = task_service.calculate_task_cost(&project_id, &task_id)?;
    // 40 часов * (800 / 8 часов в дне) * 0.5 = 40 * 100 * 0.5 = 2000 ₽
    assert_eq!(cost, 40.0 * 100.0 * 0.5);

    Ok(())
}

/// Стоимость задачи с месячной ставкой.
///
/// Часовая ставка получается делением месячной на количество рабочих часов
/// в «среднем месяце» календаря (22 дня × 8 ч = 176 ч). Стоимость при этом
/// пропорциональна фактическим трудозатратам в окне: 40 ч × (22 000 / 176)
/// × 0.5 = 2500, а не равна месячной ставке целиком (так было до исправления
/// конвертации в задаче 6.1/6.3).
#[test]
fn test_task_cost_monthly_rate() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Monthly cost", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (task_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task = task_service.create_regular_task(
            project_id,
            "Task".into(),
            Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 1, 10, 0, 0, 0).unwrap(),
            None,
        )?;
        let task_id = *task.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 22_000.0, RateMeasure::Monthly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_id, resource_id)
    };

    let window = TimeWindow::new(
        Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 1, 10, 0, 0, 0).unwrap(),
    )?;

    let mut task_service = TaskService::new(&mut container);
    task_service.allocate_resource(project_id, task_id, resource_id, 0.5, Some(window))?;

    let task_service = TaskService::new(&mut container);
    let cost = task_service.calculate_task_cost(&project_id, &task_id)?;
    // 5 рабочих дней × 8 ч = 40 ч; часовая ставка = 22 000 / (22 × 8) = 125.
    // cost = 125 × 40 × 0.5 = 2500.
    assert_eq!(cost, 22_000.0 / (22.0 * 8.0) * 40.0 * 0.5);

    Ok(())
}

/// Стоимость: сводная (групповая) задача агрегирует стоимость дочерних задач.
#[test]
fn test_summary_task_cost_sums_children() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Summary cost", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (summary_id, child1_id, child2_id, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let summary = task_service.create_summary_task(project_id, "Summary".into(), None)?;
        let summary_id = *summary.get_id();

        // Дети сводной задачи (даты обновятся автоматически).
        let child1 = task_service.create_regular_task(
            project_id,
            "Child 1".into(),
            start + chrono::TimeDelta::days(10),
            start + chrono::TimeDelta::days(17),
            Some(summary_id),
        )?;
        let child1_id = *child1.get_id();
        let child2 = task_service.create_regular_task(
            project_id,
            "Child 2".into(),
            start + chrono::TimeDelta::days(20),
            start + chrono::TimeDelta::days(27),
            Some(summary_id),
        )?;
        let child2_id = *child2.get_id();

        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (summary_id, child1_id, child2_id, resource_id)
    };

    let mut task_service = TaskService::new(&mut container);
    let w1 = TimeWindow::new(
        start + chrono::TimeDelta::days(10),
        start + chrono::TimeDelta::days(17),
    )?;
    let w2 = TimeWindow::new(
        start + chrono::TimeDelta::days(20),
        start + chrono::TimeDelta::days(27),
    )?;
    task_service.allocate_resource(project_id, child1_id, resource_id, 0.5, Some(w1))?;
    task_service.allocate_resource(project_id, child2_id, resource_id, 0.5, Some(w2))?;

    let task_service = TaskService::new(&mut container);
    let child1_cost = task_service.calculate_task_cost(&project_id, &child1_id)?;
    let child2_cost = task_service.calculate_task_cost(&project_id, &child2_id)?;
    let summary_cost = task_service.calculate_task_cost(&project_id, &summary_id)?;
    assert!(
        (summary_cost - (child1_cost + child2_cost)).abs() < 1e-6,
        "сводная задача должна равняться сумме детей: {child1_cost} + {child2_cost} != {summary_cost}"
    );

    Ok(())
}

/// Стоимость проекта — суммарная стоимость корневых задач.
#[test]
fn test_project_cost_sums_root_tasks() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Project cost", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (task_a, task_b, resource_id) = {
        let mut task_service = TaskService::new(&mut container);
        let task_a = task_service.create_regular_task(
            project_id,
            "A".into(),
            start + chrono::TimeDelta::days(10),
            start + chrono::TimeDelta::days(17),
            None,
        )?;
        let task_a = *task_a.get_id();
        let task_b = task_service.create_regular_task(
            project_id,
            "B".into(),
            start + chrono::TimeDelta::days(20),
            start + chrono::TimeDelta::days(27),
            None,
        )?;
        let task_b = *task_b.get_id();
        let mut resource_service = ResourceService::new(&mut container);
        let resource = resource_service.create_resource("R", 100.0, RateMeasure::Hourly)?;
        let resource_id = resource.id;
        resource_service.add_resource(resource)?;
        (task_a, task_b, resource_id)
    };

    let mut task_service = TaskService::new(&mut container);
    let w1 = TimeWindow::new(
        start + chrono::TimeDelta::days(10),
        start + chrono::TimeDelta::days(17),
    )?;
    let w2 = TimeWindow::new(
        start + chrono::TimeDelta::days(20),
        start + chrono::TimeDelta::days(27),
    )?;
    task_service.allocate_resource(project_id, task_a, resource_id, 0.5, Some(w1))?;
    task_service.allocate_resource(project_id, task_b, resource_id, 0.5, Some(w2))?;

    let task_service = TaskService::new(&mut container);
    let cost_a = task_service.calculate_task_cost(&project_id, &task_a)?;
    let cost_b = task_service.calculate_task_cost(&project_id, &task_b)?;
    let project_cost = task_service.calculate_project_cost(project_id)?;
    assert!(
        (project_cost - (cost_a + cost_b)).abs() < 1e-6,
        "стоимость проекта должна быть суммой корневых задач: {project_cost} != {cost_a} + {cost_b}"
    );

    Ok(())
}

// ===== Задача 6.1: критический путь через публичный API =====

/// Циклические зависимости через публичный API: critical_path возвращает ошибку,
/// а не зависает и не паникует.
#[test]
fn test_critical_path_detects_cycle_via_api() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Cycle", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (_task_a, _task_b) = {
        let mut task_service = TaskService::new(&mut container);
        let task_a = task_service.create_regular_task(
            project_id,
            "A".into(),
            start + chrono::TimeDelta::days(1),
            start + chrono::TimeDelta::days(5),
            None,
        )?;
        let task_a = *task_a.get_id();
        let task_b = task_service.create_regular_task(
            project_id,
            "B".into(),
            start + chrono::TimeDelta::days(5),
            start + chrono::TimeDelta::days(10),
            None,
        )?;
        let task_b = *task_b.get_id();

        // A зависит от B, B зависит от A — цикл.
        task_service.add_dependency(project_id, task_a, task_b, DependencyType::Blocking, None)?;
        task_service.add_dependency(project_id, task_b, task_a, DependencyType::Blocking, None)?;
        (task_a, task_b)
    };

    let scheduler = Scheduler::new(&container);
    let result = scheduler.critical_path(project_id);
    assert!(
        result.is_err(),
        "цикл в зависимостях должен давать ошибку, а не результат: {:?}",
        result.err()
    );

    Ok(())
}

/// Сводные задачи исключаются из критического пути: путь состоит только из обычных задач.
#[test]
fn test_critical_path_excludes_summary_tasks() -> anyhow::Result<()> {
    let mut container = SingleProjectContainer::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new("Test", "Summary in path", start, end)?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let (summary_id, task_a, task_b) = {
        let mut task_service = TaskService::new(&mut container);
        let summary = task_service.create_summary_task(project_id, "Summary".into(), None)?;
        let summary_id = *summary.get_id();

        let task_a = task_service.create_regular_task(
            project_id,
            "A".into(),
            start + chrono::TimeDelta::days(1),
            start + chrono::TimeDelta::days(3),
            None,
        )?;
        let task_a = *task_a.get_id();
        let task_b = task_service.create_regular_task(
            project_id,
            "B".into(),
            start + chrono::TimeDelta::days(3),
            start + chrono::TimeDelta::days(6),
            None,
        )?;
        let task_b = *task_b.get_id();
        task_service.add_dependency(project_id, task_b, task_a, DependencyType::Blocking, None)?;
        (summary_id, task_a, task_b)
    };

    let scheduler = Scheduler::new(&container);
    let critical = scheduler.critical_path(project_id)?;
    assert!(
        !critical.contains(&summary_id),
        "сводная задача не должна попадать в критический путь: {critical:?}"
    );
    assert_eq!(critical, vec![task_a, task_b]);

    Ok(())
}
