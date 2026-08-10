use app::ProjectApp;
use chrono::{Duration, TimeZone, Utc};
use eframe::egui;
use logic::{
    BasicGettersForStructures, DependencyType, ExceptionPeriod, ExceptionType, Project,
    ProjectContainer, RateMeasure, ResourceService, SingleProjectContainer, TaskService,
    TimeWindow,
};

fn build_demo_container() -> anyhow::Result<SingleProjectContainer> {
    let mut container = SingleProjectContainer::new();

    // Проект длится 2 года
    let project_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let project_end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
    let project = Project::new(
        "Крупный демо-проект (50+ задач)",
        "Проект разработки программного комплекса с группировкой задач",
        project_start,
        project_end,
    )?;
    let project_id = *project.get_id();
    container.add_project(project)?;

    let mut resource_service = ResourceService::new(&mut container);

    // Ресурсы (5 человек)
    let pm = resource_service.create_resource("Project Manager", 2500.0, RateMeasure::Daily)?;
    let analyst =
        resource_service.create_resource("Business Analyst", 2000.0, RateMeasure::Daily)?;
    let dev_lead = resource_service.create_resource("Dev Lead", 2200.0, RateMeasure::Daily)?;
    let dev = resource_service.create_resource("Developer", 1800.0, RateMeasure::Daily)?;
    let tester = resource_service.create_resource("Tester", 1600.0, RateMeasure::Daily)?;
    let devops = resource_service.create_resource("DevOps", 1900.0, RateMeasure::Daily)?;

    // Добавляем все ресурсы в пул
    resource_service.add_resource(pm.clone())?;
    resource_service.add_resource(analyst.clone())?;
    resource_service.add_resource(dev_lead.clone())?;
    resource_service.add_resource(dev.clone())?;
    resource_service.add_resource(tester.clone())?;
    resource_service.add_resource(devops.clone())?;

    // Периоды недоступности (например, отпуска)
    let pm_vacation = ExceptionPeriod {
        period: TimeWindow::new(
            Utc.with_ymd_and_hms(2025, 7, 15, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 7, 25, 0, 0, 0).unwrap(),
        )?,
        exception_type: ExceptionType::Vacation,
    };
    resource_service.add_unavailable_period(pm.id, pm_vacation)?;

    let dev_vacation = ExceptionPeriod {
        period: TimeWindow::new(
            Utc.with_ymd_and_hms(2025, 12, 20, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(),
        )?,
        exception_type: ExceptionType::Vacation,
    };
    resource_service.add_unavailable_period(dev.id, dev_vacation)?;

    let mut task_service = TaskService::new(&mut container);

    // ========== Фаза 1: Инициация ==========
    let phase1 =
        task_service.create_summary_task(project_id, "1. Инициация проекта".into(), None)?;
    let phase1_id = *phase1.get_id();

    let t1_1 = task_service.create_regular_task(
        project_id,
        "Разработка устава проекта".into(),
        Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 1, 10, 0, 0, 0).unwrap(),
        Some(phase1_id),
    )?;
    let t1_1_id = *t1_1.get_id();

    let t1_2 = task_service.create_regular_task(
        project_id,
        "Назначение команды".into(),
        Utc.with_ymd_and_hms(2025, 1, 5, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 1, 12, 0, 0, 0).unwrap(),
        Some(phase1_id),
    )?;
    let t1_2_id = *t1_2.get_id();

    let t1_3 = task_service.create_regular_task(
        project_id,
        "Определение целей и ограничений".into(),
        Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).unwrap(),
        Some(phase1_id),
    )?;
    let t1_3_id = *t1_3.get_id();

    // ========== Фаза 2: Планирование ==========
    let phase2 = task_service.create_summary_task(project_id, "2. Планирование".into(), None)?;
    let phase2_id = *phase2.get_id();

    let t2_1 = task_service.create_regular_task(
        project_id,
        "Сбор требований".into(),
        Utc.with_ymd_and_hms(2025, 1, 16, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 2, 10, 0, 0, 0).unwrap(),
        Some(phase2_id),
    )?;
    let t2_1_id = *t2_1.get_id();

    let t2_2 = task_service.create_regular_task(
        project_id,
        "Анализ требований".into(),
        Utc.with_ymd_and_hms(2025, 2, 11, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 2, 25, 0, 0, 0).unwrap(),
        Some(phase2_id),
    )?;
    let t2_2_id = *t2_2.get_id();

    let t2_3 = task_service.create_regular_task(
        project_id,
        "Проектирование архитектуры".into(),
        Utc.with_ymd_and_hms(2025, 2, 26, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 15, 0, 0, 0).unwrap(),
        Some(phase2_id),
    )?;
    let t2_3_id = *t2_3.get_id();

    let t2_4 = task_service.create_regular_task(
        project_id,
        "Создание WBS".into(),
        Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 10, 0, 0, 0).unwrap(),
        Some(phase2_id),
    )?;
    let t2_4_id = *t2_4.get_id();

    let t2_5 = task_service.create_regular_task(
        project_id,
        "Оценка сроков и стоимости".into(),
        Utc.with_ymd_and_hms(2025, 3, 11, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 20, 0, 0, 0).unwrap(),
        Some(phase2_id),
    )?;
    let t2_5_id = *t2_5.get_id();

    // ========== Фаза 3: Разработка (подфазы) ==========
    let phase3 = task_service.create_summary_task(project_id, "3. Разработка".into(), None)?;
    let phase3_id = *phase3.get_id();

    // Подфаза 3.1: Настройка окружения
    let sub31 = task_service.create_summary_task(
        project_id,
        "3.1 Настройка среды".into(),
        Some(phase3_id),
    )?;
    let sub31_id = *sub31.get_id();

    let t3_1_1 = task_service.create_regular_task(
        project_id,
        "Установка CI/CD".into(),
        Utc.with_ymd_and_hms(2025, 3, 21, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 3, 30, 0, 0, 0).unwrap(),
        Some(sub31_id),
    )?;
    let t3_1_1_id = *t3_1_1.get_id();

    let t3_1_2 = task_service.create_regular_task(
        project_id,
        "Настройка репозиториев".into(),
        Utc.with_ymd_and_hms(2025, 3, 25, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 4, 5, 0, 0, 0).unwrap(),
        Some(sub31_id),
    )?;
    let t3_1_2_id = *t3_1_2.get_id();

    // Подфаза 3.2: Разработка модуля A
    let sub32 =
        task_service.create_summary_task(project_id, "3.2 Модуль A".into(), Some(phase3_id))?;
    let sub32_id = *sub32.get_id();

    let t3_2_1 = task_service.create_regular_task(
        project_id,
        "Реализация модуля A (часть 1)".into(),
        Utc.with_ymd_and_hms(2025, 4, 6, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 4, 25, 0, 0, 0).unwrap(),
        Some(sub32_id),
    )?;
    let t3_2_1_id = *t3_2_1.get_id();

    let t3_2_2 = task_service.create_regular_task(
        project_id,
        "Реализация модуля A (часть 2)".into(),
        Utc.with_ymd_and_hms(2025, 4, 26, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 5, 15, 0, 0, 0).unwrap(),
        Some(sub32_id),
    )?;
    let t3_2_2_id = *t3_2_2.get_id();

    // Подфаза 3.3: Разработка модуля B
    let sub33 =
        task_service.create_summary_task(project_id, "3.3 Модуль B".into(), Some(phase3_id))?;
    let sub33_id = *sub33.get_id();

    let t3_3_1 = task_service.create_regular_task(
        project_id,
        "Реализация модуля B (часть 1)".into(),
        Utc.with_ymd_and_hms(2025, 4, 10, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 4, 30, 0, 0, 0).unwrap(),
        Some(sub33_id),
    )?;
    let t3_3_1_id = *t3_3_1.get_id();

    let t3_3_2 = task_service.create_regular_task(
        project_id,
        "Реализация модуля B (часть 2)".into(),
        Utc.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 5, 20, 0, 0, 0).unwrap(),
        Some(sub33_id),
    )?;
    let t3_3_2_id = *t3_3_2.get_id();

    // ========== Фаза 4: Тестирование ==========
    let phase4 = task_service.create_summary_task(project_id, "4. Тестирование".into(), None)?;
    let phase4_id = *phase4.get_id();

    let t4_1 = task_service.create_regular_task(
        project_id,
        "Модульное тестирование A".into(),
        Utc.with_ymd_and_hms(2025, 5, 16, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 5, 30, 0, 0, 0).unwrap(),
        Some(phase4_id),
    )?;
    let t4_1_id = *t4_1.get_id();

    let t4_2 = task_service.create_regular_task(
        project_id,
        "Модульное тестирование B".into(),
        Utc.with_ymd_and_hms(2025, 5, 21, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 6, 5, 0, 0, 0).unwrap(),
        Some(phase4_id),
    )?;
    let t4_2_id = *t4_2.get_id();

    let t4_3 = task_service.create_regular_task(
        project_id,
        "Интеграционное тестирование".into(),
        Utc.with_ymd_and_hms(2025, 6, 6, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 6, 25, 0, 0, 0).unwrap(),
        Some(phase4_id),
    )?;
    let t4_3_id = *t4_3.get_id();

    let t4_4 = task_service.create_regular_task(
        project_id,
        "Системное тестирование".into(),
        Utc.with_ymd_and_hms(2025, 6, 26, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 7, 10, 0, 0, 0).unwrap(),
        Some(phase4_id),
    )?;
    let t4_4_id = *t4_4.get_id();

    // ========== Фаза 5: Внедрение ==========
    let phase5 = task_service.create_summary_task(project_id, "5. Внедрение".into(), None)?;
    let phase5_id = *phase5.get_id();

    let t5_1 = task_service.create_regular_task(
        project_id,
        "Подготовка документации".into(),
        Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 7, 20, 0, 0, 0).unwrap(),
        Some(phase5_id),
    )?;
    let t5_1_id = *t5_1.get_id();

    let t5_2 = task_service.create_regular_task(
        project_id,
        "Обучение пользователей".into(),
        Utc.with_ymd_and_hms(2025, 7, 15, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 8, 5, 0, 0, 0).unwrap(),
        Some(phase5_id),
    )?;
    let t5_2_id = *t5_2.get_id();

    let t5_3 = task_service.create_regular_task(
        project_id,
        "Пилотное внедрение".into(),
        Utc.with_ymd_and_hms(2025, 8, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 8, 20, 0, 0, 0).unwrap(),
        Some(phase5_id),
    )?;
    let t5_3_id = *t5_3.get_id();

    let t5_4 = task_service.create_regular_task(
        project_id,
        "Развертывание в продуктив".into(),
        Utc.with_ymd_and_hms(2025, 8, 21, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 9, 1, 0, 0, 0).unwrap(),
        Some(phase5_id),
    )?;
    let t5_4_id = *t5_4.get_id();

    // ========== Фаза 6: Пост-релиз ==========
    let phase6 =
        task_service.create_summary_task(project_id, "6. Пост-релизная поддержка".into(), None)?;
    let phase6_id = *phase6.get_id();

    let t6_1 = task_service.create_regular_task(
        project_id,
        "Сбор обратной связи".into(),
        Utc.with_ymd_and_hms(2025, 9, 2, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 9, 15, 0, 0, 0).unwrap(),
        Some(phase6_id),
    )?;
    let t6_1_id = *t6_1.get_id();

    let t6_2 = task_service.create_regular_task(
        project_id,
        "Устранение ошибок".into(),
        Utc.with_ymd_and_hms(2025, 9, 10, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 10, 5, 0, 0, 0).unwrap(),
        Some(phase6_id),
    )?;
    let t6_2_id = *t6_2.get_id();

    let t6_3 = task_service.create_regular_task(
        project_id,
        "Плановые обновления".into(),
        Utc.with_ymd_and_hms(2025, 10, 6, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 11, 1, 0, 0, 0).unwrap(),
        Some(phase6_id),
    )?;
    let t6_3_id = *t6_3.get_id();

    // ========== Зависимости (критический путь) ==========
    // Связываем фазы последовательно
    // Инициация -> Планирование -> Разработка -> Тестирование -> Внедрение -> Пост-релиз
    task_service.add_dependency(
        project_id,
        phase2_id,
        phase1_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        phase3_id,
        phase2_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        phase4_id,
        phase3_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        phase5_id,
        phase4_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        phase6_id,
        phase5_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;

    // Внутри фаз также создаём критические цепочки
    // В планировании: t2_1 (сбор требований) -> t2_2 (анализ) -> t2_3 (архитектура) -> t2_5 (оценка)
    task_service.add_dependency(
        project_id,
        t2_2_id,
        t2_1_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t2_3_id,
        t2_2_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t2_5_id,
        t2_3_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;

    // В разработке: настройка среды (t3_1_1) -> модуль A (t3_2_1, t3_2_2) -> тестирование
    task_service.add_dependency(
        project_id,
        t3_2_1_id,
        t3_1_1_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t3_2_2_id,
        t3_2_1_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t4_1_id,
        t3_2_2_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;

    // Модуль B идёт параллельно, но не критический (если его длинна меньше)
    task_service.add_dependency(
        project_id,
        t4_2_id,
        t3_3_2_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;

    // Далее по тестированию
    task_service.add_dependency(
        project_id,
        t4_3_id,
        t4_1_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t4_4_id,
        t4_3_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t5_1_id,
        t4_4_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t5_4_id,
        t5_1_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t6_1_id,
        t5_4_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;
    task_service.add_dependency(
        project_id,
        t6_3_id,
        t6_2_id,
        DependencyType::Blocking,
        Some(Duration::zero()),
    )?;

    // ========== Назначение ресурсов ==========
    // Назначаем ресурсы на задачи (некоторые с частичной занятостью)
    task_service.allocate_resource(project_id, t1_1_id, pm.id, 0.5, None)?;
    task_service.allocate_resource(project_id, t1_2_id, pm.id, 0.3, None)?;
    task_service.allocate_resource(project_id, t1_3_id, analyst.id, 0.7, None)?;

    task_service.allocate_resource(project_id, t2_1_id, analyst.id, 0.8, None)?;
    task_service.allocate_resource(project_id, t2_2_id, analyst.id, 0.6, None)?;
    task_service.allocate_resource(project_id, t2_3_id, dev_lead.id, 0.6, None)?;
    task_service.allocate_resource(project_id, t2_4_id, dev_lead.id, 0.3, None)?;
    task_service.allocate_resource(project_id, t2_5_id, pm.id, 0.1, None)?;

    task_service.allocate_resource(project_id, t3_1_1_id, devops.id, 0.5, None)?;
    task_service.allocate_resource(project_id, t3_1_2_id, devops.id, 0.5, None)?;
    task_service.allocate_resource(project_id, t3_2_1_id, dev.id, 0.3, None)?;
    task_service.allocate_resource(project_id, t3_2_2_id, dev.id, 0.3, None)?;
    task_service.allocate_resource(project_id, t3_3_1_id, dev.id, 0.3, None)?;
    task_service.allocate_resource(project_id, t3_3_2_id, dev.id, 0.1, None)?;

    task_service.allocate_resource(project_id, t4_1_id, tester.id, 0.25, None)?;
    task_service.allocate_resource(project_id, t4_2_id, tester.id, 0.25, None)?;
    task_service.allocate_resource(project_id, t4_3_id, tester.id, 0.25, None)?;
    task_service.allocate_resource(project_id, t4_4_id, tester.id, 0.25, None)?;

    task_service.allocate_resource(project_id, t5_1_id, analyst.id, 0.4, None)?;
    task_service.allocate_resource(project_id, t5_2_id, analyst.id, 0.5, None)?;
    task_service.allocate_resource(project_id, t5_3_id, dev.id, 0.3, None)?;
    task_service.allocate_resource(project_id, t5_4_id, devops.id, 0.6, None)?;

    task_service.allocate_resource(project_id, t6_1_id, analyst.id, 0.2, None)?;
    task_service.allocate_resource(project_id, t6_2_id, dev.id, 0.4, None)?;
    task_service.allocate_resource(project_id, t6_3_id, devops.id, 0.3, None)?;

    Ok(container)
}

fn main() -> eframe::Result<()> {
    let container = build_demo_container().expect("Failed to build demo container");
    let app = ProjectApp::with_container(container);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([1024.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Project Manager (Большой демо-проект)",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "FiraCodeNerd".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/FiraCodeNerdFontPropo-Regular.ttf"
                ))
                .into(),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "FiraCodeNerd".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(24.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            cc.egui_ctx.set_style(style);

            Ok(Box::new(app))
        }),
    )
}
