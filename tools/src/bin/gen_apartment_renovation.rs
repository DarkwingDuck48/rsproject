//! Генератор демо-проекта «Ремонт квартиры» для папки `examples/`.
//!
//! Второй сценарий демонстрации: не разработка ПО, а ремонт — с другой сеткой
//! ролей (бригада, а не программисты), технологическим перерывом и
//! последовательной загрузкой исполнителей.
//!
//! ```text
//! cargo run -p tools --bin gen_apartment_renovation > examples/apartment_renovation.json
//! ```
//!
//! ## Что заложено специально
//!
//! * **Иерархия.** 5 фаз-групп; их даты приложение пересчитывает само — по
//!   минимальному началу и максимальному окончанию дочерних задач.
//! * **Последовательная бригада.** У каждого исполнителя окна назначений идут
//!   «встык», без наложений: `ResourcePool` отвергает назначение, если сумма
//!   занятости по пересекающимся окнам превысит 100%.
//! * **Частичная занятость.** Прораб заходит на объект долей 0.2–0.3, при этом
//!   электромонтаж и сантехника идут параллельно (0.3 + 0.3 = 0.6 — допустимо).
//! * **Лаг.** Гидроизоляция начинается через 7 дней после стяжки: стяжка должна
//!   высохнуть.
//! * **Периоды недоступности.** Отпуск прораба и больничный электрика.
//! * **Календарь.** Праздники 9 марта, 1 и 11 мая уменьшают трудозатраты.
//!
//! ## Почему работы расписаны «встык»
//!
//! `Scheduler` считает критический путь по длительностям и зависимостям,
//! начиная от даты старта проекта: ранние даты — это максимум из ранних
//! окончаний предшественников плюс лаг. Даты, записанные в задачах, он не
//! читает (и дату окончания проекта тоже — см. `backward_pass`, где для задач
//! без последователей поздний финиш берётся как максимальный ранний финиш).
//!
//! Значит, календарный план в файле — это то, что видит пользователь, и он
//! должен совпадать с тем, что подсветит «Рассчитать критический путь».
//! Поэтому каждая задача начинается ровно тогда, когда заканчиваются её
//! предшественники: тогда критическая цепочка выглядит на диаграмме Ганта
//! непрерывной полосой.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use logic::{
    BasicGettersForStructures, DependencyType, ExceptionPeriod, ExceptionType, Project,
    ProjectContainer, RateMeasure, ResourceService, SingleProjectContainer, TaskService,
    TimeWindow,
};

/// Дата в 2026 году (полночь UTC) — сокращает шум в описании расписания.
fn day(month: u32, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, month, day, 0, 0, 0).unwrap()
}

/// Связывает две задачи блокирующей зависимостью с указанным лагом.
fn link(
    tasks: &mut TaskService<'_, SingleProjectContainer>,
    project_id: uuid::Uuid,
    successor: uuid::Uuid,
    predecessor: uuid::Uuid,
    lag: Duration,
) -> anyhow::Result<()> {
    tasks.add_dependency(
        project_id,
        successor,
        predecessor,
        DependencyType::Blocking,
        Some(lag),
    )
}

/// Собирает контейнер с демо-проектом.
///
/// Возвращает ещё и `project_id`: у контейнера нет публичного доступа к своему
/// проекту без идентификатора, а он нужен тесту, который считает критический путь.
fn build_demo_container() -> anyhow::Result<(SingleProjectContainer, uuid::Uuid)> {
    let mut container = SingleProjectContainer::new();

    let mut project = Project::new(
        "Ремонт квартиры",
        "Комплексный ремонт двухкомнатной квартиры 62 м²: демонтаж, инженерные сети, черновая и чистовая отделка",
        day(3, 2),
        day(7, 1),
    )?;

    // Праздники: 9 марта (перенос с 8 марта), 1 мая и 11 мая (перенос с 9 мая).
    // Календарь не влияет на расписание, но учитывается в трудозатратах
    // и в проверке доступности ресурсов.
    project
        .calendar
        .add_holiday(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap());
    project
        .calendar
        .add_holiday(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
    project
        .calendar
        .add_holiday(NaiveDate::from_ymd_opt(2026, 5, 11).unwrap());

    let project_id = *project.get_id();
    container.add_project(project)?;

    // ─── Пул ресурсов: бригада + проектировщик ───
    let mut resources = ResourceService::new(&mut container);
    let foreman = resources.create_resource("Прораб", 3500.0, RateMeasure::Daily)?;
    let designer = resources.create_resource("Дизайнер", 2000.0, RateMeasure::Hourly)?;
    let finisher = resources.create_resource("Отделочник", 2800.0, RateMeasure::Daily)?;
    let electrician = resources.create_resource("Электрик", 3200.0, RateMeasure::Daily)?;
    let plumber = resources.create_resource("Сантехник", 3000.0, RateMeasure::Daily)?;
    let tiler = resources.create_resource("Плиточник", 3000.0, RateMeasure::Daily)?;

    resources.add_resource(foreman.clone())?;
    resources.add_resource(designer.clone())?;
    resources.add_resource(finisher.clone())?;
    resources.add_resource(electrician.clone())?;
    resources.add_resource(plumber.clone())?;
    resources.add_resource(tiler.clone())?;

    // Отпуск прораба и больничный электрика. Периоды выбраны так, чтобы не
    // пересекаться с назначениями: иначе ресурс нельзя назначить на задачу.
    resources.add_unavailable_period(
        foreman.id,
        ExceptionPeriod {
            period: TimeWindow::new(day(4, 6), day(4, 12))?,
            exception_type: ExceptionType::Vacation,
        },
    )?;
    resources.add_unavailable_period(
        electrician.id,
        ExceptionPeriod {
            period: TimeWindow::new(day(4, 13), day(4, 15))?,
            exception_type: ExceptionType::SickLeave,
        },
    )?;

    let mut tasks = TaskService::new(&mut container);

    // ═══ Фаза 1. Демонтаж и подготовка ═══
    let phase1 = tasks.create_summary_task(project_id, "Демонтаж и подготовка".into(), None)?;
    let phase1_id = *phase1.get_id();

    let estimate = tasks.create_regular_task(
        project_id,
        "Замеры и корректировка сметы".into(),
        day(3, 2),
        day(3, 4),
        Some(phase1_id),
    )?;
    let estimate_id = *estimate.get_id();

    let materials = tasks.create_regular_task(
        project_id,
        "Подбор отделочных материалов".into(),
        day(3, 4),
        day(3, 11),
        Some(phase1_id),
    )?;
    let materials_id = *materials.get_id();

    let demolition = tasks.create_regular_task(
        project_id,
        "Демонтаж старых покрытий".into(),
        day(3, 4),
        day(3, 11),
        Some(phase1_id),
    )?;
    let demolition_id = *demolition.get_id();

    let partitions_out = tasks.create_regular_task(
        project_id,
        "Демонтаж перегородок и сантехкабины".into(),
        day(3, 11),
        day(3, 17),
        Some(phase1_id),
    )?;
    let partitions_out_id = *partitions_out.get_id();

    let garbage = tasks.create_regular_task(
        project_id,
        "Вывоз строительного мусора".into(),
        day(3, 17),
        day(3, 19),
        Some(phase1_id),
    )?;
    let garbage_id = *garbage.get_id();

    // ═══ Фаза 2. Инженерные сети ═══
    let phase2 = tasks.create_summary_task(project_id, "Инженерные сети".into(), None)?;
    let phase2_id = *phase2.get_id();

    let wiring = tasks.create_regular_task(
        project_id,
        "Электромонтаж: разводка кабеля".into(),
        day(3, 19),
        day(3, 27),
        Some(phase2_id),
    )?;
    let wiring_id = *wiring.get_id();

    let plumbing = tasks.create_regular_task(
        project_id,
        "Сантехника: разводка труб".into(),
        day(3, 19),
        day(3, 26),
        Some(phase2_id),
    )?;
    let plumbing_id = *plumbing.get_id();

    let ventilation = tasks.create_regular_task(
        project_id,
        "Монтаж вентиляции и кондиционирования".into(),
        day(3, 26),
        day(4, 1),
        Some(phase2_id),
    )?;
    let ventilation_id = *ventilation.get_id();

    let pressure_test = tasks.create_regular_task(
        project_id,
        "Опрессовка и приёмка инженерных сетей".into(),
        day(4, 1),
        day(4, 3),
        Some(phase2_id),
    )?;
    let pressure_test_id = *pressure_test.get_id();

    // ═══ Фаза 3. Черновая отделка ═══
    let phase3 = tasks.create_summary_task(project_id, "Черновая отделка".into(), None)?;
    let phase3_id = *phase3.get_id();

    let new_partitions = tasks.create_regular_task(
        project_id,
        "Возведение перегородок (гипсокартон)".into(),
        day(4, 3),
        day(4, 14),
        Some(phase3_id),
    )?;
    let new_partitions_id = *new_partitions.get_id();

    let plaster = tasks.create_regular_task(
        project_id,
        "Штукатурка стен и потолков".into(),
        day(4, 14),
        day(4, 24),
        Some(phase3_id),
    )?;
    let plaster_id = *plaster.get_id();

    let screed = tasks.create_regular_task(
        project_id,
        "Стяжка пола".into(),
        day(4, 24),
        day(4, 30),
        Some(phase3_id),
    )?;
    let screed_id = *screed.get_id();

    let waterproofing = tasks.create_regular_task(
        project_id,
        "Гидроизоляция санузлов".into(),
        day(5, 7),
        day(5, 13),
        Some(phase3_id),
    )?;
    let waterproofing_id = *waterproofing.get_id();

    // ═══ Фаза 4. Чистовая отделка ═══
    let phase4 = tasks.create_summary_task(project_id, "Чистовая отделка".into(), None)?;
    let phase4_id = *phase4.get_id();

    let tiling = tasks.create_regular_task(
        project_id,
        "Укладка плитки (санузлы, кухня)".into(),
        day(5, 13),
        day(5, 28),
        Some(phase4_id),
    )?;
    let tiling_id = *tiling.get_id();

    let painting = tasks.create_regular_task(
        project_id,
        "Шпаклёвка и покраска стен и потолков".into(),
        day(5, 13),
        day(5, 27),
        Some(phase4_id),
    )?;
    let painting_id = *painting.get_id();

    let flooring = tasks.create_regular_task(
        project_id,
        "Напольные покрытия (ламинат)".into(),
        day(5, 27),
        day(6, 9),
        Some(phase4_id),
    )?;
    let flooring_id = *flooring.get_id();

    let doors = tasks.create_regular_task(
        project_id,
        "Установка дверей и плинтусов".into(),
        day(6, 9),
        day(6, 16),
        Some(phase4_id),
    )?;
    let doors_id = *doors.get_id();

    let sockets = tasks.create_regular_task(
        project_id,
        "Монтаж освещения и розеток".into(),
        day(5, 27),
        day(6, 4),
        Some(phase4_id),
    )?;
    let sockets_id = *sockets.get_id();

    // ═══ Фаза 5. Финиш и сдача ═══
    let phase5 = tasks.create_summary_task(project_id, "Финиш и сдача".into(), None)?;
    let phase5_id = *phase5.get_id();

    let fixtures = tasks.create_regular_task(
        project_id,
        "Установка сантехники и бытовой техники".into(),
        day(6, 16),
        day(6, 23),
        Some(phase5_id),
    )?;
    let fixtures_id = *fixtures.get_id();

    let cleanup = tasks.create_regular_task(
        project_id,
        "Финальная уборка".into(),
        day(6, 23),
        day(6, 28),
        Some(phase5_id),
    )?;
    let cleanup_id = *cleanup.get_id();

    let handover = tasks.create_regular_task(
        project_id,
        "Приёмка и передача объекта".into(),
        day(6, 28),
        day(7, 1),
        Some(phase5_id),
    )?;
    let handover_id = *handover.get_id();

    // ─── Технологическая цепочка работ ───
    let zero = Duration::zero();

    link(&mut tasks, project_id, materials_id, estimate_id, zero)?;
    link(&mut tasks, project_id, demolition_id, estimate_id, zero)?;
    link(
        &mut tasks,
        project_id,
        partitions_out_id,
        demolition_id,
        zero,
    )?;
    link(&mut tasks, project_id, garbage_id, partitions_out_id, zero)?;
    link(&mut tasks, project_id, wiring_id, garbage_id, zero)?;
    link(&mut tasks, project_id, plumbing_id, garbage_id, zero)?;
    link(&mut tasks, project_id, ventilation_id, plumbing_id, zero)?;
    link(&mut tasks, project_id, pressure_test_id, wiring_id, zero)?;
    link(
        &mut tasks,
        project_id,
        pressure_test_id,
        ventilation_id,
        zero,
    )?;
    link(
        &mut tasks,
        project_id,
        new_partitions_id,
        pressure_test_id,
        zero,
    )?;
    link(&mut tasks, project_id, plaster_id, new_partitions_id, zero)?;
    link(&mut tasks, project_id, screed_id, plaster_id, zero)?;
    // Стяжка сохнет неделю — отсюда лаг в 7 дней.
    link(
        &mut tasks,
        project_id,
        waterproofing_id,
        screed_id,
        Duration::days(7),
    )?;
    link(&mut tasks, project_id, tiling_id, waterproofing_id, zero)?;
    link(&mut tasks, project_id, painting_id, waterproofing_id, zero)?;
    link(&mut tasks, project_id, flooring_id, tiling_id, zero)?;
    link(&mut tasks, project_id, flooring_id, painting_id, zero)?;
    link(&mut tasks, project_id, doors_id, flooring_id, zero)?;
    link(&mut tasks, project_id, sockets_id, painting_id, zero)?;
    link(&mut tasks, project_id, fixtures_id, doors_id, zero)?;
    link(&mut tasks, project_id, fixtures_id, sockets_id, zero)?;
    link(&mut tasks, project_id, cleanup_id, fixtures_id, zero)?;
    link(&mut tasks, project_id, handover_id, cleanup_id, zero)?;

    // ─── Назначения ресурсов ───
    // Последний аргумент — окно назначения; `None` означает «на всю задачу».
    tasks.allocate_resource(project_id, estimate_id, designer.id, 0.4, None)?;
    tasks.allocate_resource(project_id, materials_id, designer.id, 0.5, None)?;

    tasks.allocate_resource(project_id, demolition_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, demolition_id, foreman.id, 0.2, None)?;
    tasks.allocate_resource(project_id, partitions_out_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, garbage_id, finisher.id, 0.5, None)?;

    // Прораб ведёт электромонтаж и сантехнику параллельно: 0.3 + 0.3 = 0.6.
    tasks.allocate_resource(project_id, wiring_id, electrician.id, 1.0, None)?;
    tasks.allocate_resource(project_id, wiring_id, foreman.id, 0.3, None)?;
    tasks.allocate_resource(project_id, plumbing_id, plumber.id, 1.0, None)?;
    tasks.allocate_resource(project_id, plumbing_id, foreman.id, 0.3, None)?;
    tasks.allocate_resource(project_id, ventilation_id, plumber.id, 1.0, None)?;
    tasks.allocate_resource(project_id, pressure_test_id, foreman.id, 1.0, None)?;

    tasks.allocate_resource(project_id, new_partitions_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, plaster_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, plaster_id, foreman.id, 0.2, None)?;
    tasks.allocate_resource(project_id, screed_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, waterproofing_id, tiler.id, 1.0, None)?;

    tasks.allocate_resource(project_id, tiling_id, tiler.id, 1.0, None)?;
    tasks.allocate_resource(project_id, tiling_id, foreman.id, 0.2, None)?;
    tasks.allocate_resource(project_id, painting_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, flooring_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, doors_id, finisher.id, 1.0, None)?;
    tasks.allocate_resource(project_id, sockets_id, electrician.id, 1.0, None)?;

    tasks.allocate_resource(project_id, fixtures_id, plumber.id, 1.0, None)?;
    tasks.allocate_resource(project_id, cleanup_id, finisher.id, 0.5, None)?;
    tasks.allocate_resource(project_id, handover_id, foreman.id, 1.0, None)?;

    Ok((container, project_id))
}

/// Собирает демо-контейнер и печатает его JSON в stdout.
/// Запуск из корня репозитория:
/// `cargo run -p tools --bin gen_apartment_renovation > examples/apartment_renovation.json`.
fn main() -> anyhow::Result<()> {
    let (container, _) = build_demo_container()?;
    println!("{}", serde_json::to_string_pretty(&container)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use logic::Scheduler;

    /// Демо-проект обязан собираться без ошибок, читаться обратно приложением
    /// и давать непустой критический путь.
    ///
    /// Первое проверяет `build_demo_container` (ошибку вернут сами сервисы:
    /// пересечение назначений, даты вне проекта, период недоступности),
    /// второе — те же два шага, что делает «Файл → Открыть проект».
    #[test]
    fn demo_project_reads_back_and_has_critical_path() {
        let (container, project_id) =
            build_demo_container().expect("демо-проект должен собираться");

        // Так же файл открывает приложение (см. commands/project.rs).
        let json = serde_json::to_string_pretty(&container).expect("сериализация не должна падать");
        serde_json::from_str::<SingleProjectContainer>(&json)
            .expect("сгенерированный JSON должен читаться обратно");

        let critical = Scheduler::new(&container)
            .critical_path(project_id)
            .expect("критический путь должен считаться");
        assert!(
            !critical.is_empty(),
            "у демо-проекта должен быть непустой критический путь"
        );
    }
}
