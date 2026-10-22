//! Планировщик: расчёт критического пути проекта методом CPM.
//!
//! Модуль ничего не меняет в проекте: он читает длительности задач, зависимости
//! и дату старта, а возвращает список задач, лежащих на критическом пути.
//! Даты задач при этом не пересчитываются — планировщик не перепланирует проект.
//!
//! Расчёт состоит из пяти шагов, которые собирает `Scheduler::critical_path`:
//! построение графа задач, топологическая сортировка, прямой проход (ранние даты),
//! обратный проход (поздние даты) и поиск самой длинной цепочки задач без запаса.
//!
//! Учитываются только длительности задач и лаги зависимостей, отсчитываемые от даты
//! старта проекта. Календарь проекта (рабочие дни, праздники, часы в день) в расчёте
//! не участвует — он применяется для трудозатрат и проверки доступности ресурсов.
//! Дата окончания проекта на результат тоже не влияет: см.
//! [issue #17](https://github.com/DarkwingDuck48/rsproject/issues/17).

use crate::{BasicGettersForStructures, Project, ProjectContainer};
use chrono::{DateTime, TimeDelta, Utc};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Граф задач проекта, по которому считается критический путь.
///
/// Вершины — обычные задачи: сводные (групповые) задачи в граф не попадают, так как их
/// длительность производна от дочерних задач. Рёбра направлены от предшественника
/// к зависимой задаче: если `B` зависит от `A`, ребро идёт `A → B`.
///
/// Всё время здесь — календарное ([`TimeDelta`]), без учёта календаря проекта.
#[derive(Debug, Clone, Default)]
struct Graph {
    /// Задача → её предшественники и лаг каждого ребра.
    predecessors: HashMap<Uuid, Vec<(Uuid, TimeDelta)>>,
    /// Задача → задачи, которые от неё зависят, и лаг каждого ребра.
    /// Заполняется как обратное к `predecessors`.
    successors: HashMap<Uuid, Vec<(Uuid, TimeDelta)>>,
    /// Задача → длительность. Служит и списком всех задач графа:
    /// именно по этому полю определяются вершины.
    durations: HashMap<Uuid, TimeDelta>,
}

/// Расчёт критического пути проекта.
///
/// Планировщик только читает данные контейнера: он не изменяет проект и не
/// переписывает даты задач. Зависит от трейта `ProjectContainer`,
/// поэтому работает и с одиночным, и с будущим мультипроектным контейнером.
#[derive(Copy, Clone, Debug)]
pub struct Scheduler<'a, C: ProjectContainer> {
    container: &'a C,
}

impl<'a, C: ProjectContainer> Scheduler<'a, C> {
    /// Создаёт планировщик над контейнером проектов.
    pub fn new(container: &'a C) -> Self {
        Self { container }
    }

    /// Считает критический путь проекта и возвращает ID задач, лежащих на нём.
    ///
    /// Порядок расчёта:
    ///
    /// 1. `build_graph` — проект превращается в граф задач и зависимостей;
    /// 2. `topological_sort` — задачи упорядочиваются так, что предшественники идут
    ///    раньше зависимых задач (здесь же ловятся циклы и ссылки на несуществующие задачи);
    /// 3. `forward_pass` — ранние даты от даты старта проекта;
    /// 4. `backward_pass` — поздние даты;
    /// 5. `find_critical_path` — из задач без запаса собирается самая длинная цепочка.
    ///
    /// Используются длительности задач и лаги зависимостей; календарь проекта
    /// и даты, записанные в задачах, в расчёт не входят. Дата окончания проекта
    /// на результат тоже не влияет — см.
    /// [issue #17](https://github.com/DarkwingDuck48/rsproject/issues/17).
    ///
    /// Возвращает задачи критического пути по порядку следования или пустой список,
    /// если критических задач нет.
    ///
    /// Ошибку возвращает, если проект не найден, если в зависимостях есть цикл,
    /// если задача зависит от несуществующей задачи или если в проекте нет задач
    /// (обратный проход не может определить расчётное окончание проекта).
    pub fn critical_path(&self, project_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
        let project = self
            .container
            .get_project(&project_id)
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
        let graph = build_graph(project);
        let order = topological_sort(&graph)?;
        // `es` и `ls` в текущем расчёте не нужны: критического пути достаточно ранних
        // и поздних финишей. Проходы возвращают все четыре карты — на них проверяются
        // тесты, и они понадобятся, если в интерфейсе появятся запасы и ранние/поздние даты.
        let (_, ef) = forward_pass(*project.get_date_start(), &graph, &order)?;
        let (_, lf) = backward_pass(&graph, &ef, &order)?;
        find_critical_path(&graph, &ef, &lf)
    }
}

/// Строит граф задач проекта.
///
/// Сводные (групповые) задачи пропускаются: их даты и длительность — производные
/// от дочерних задач, поэтому в критическом пути они не участвуют.
///
/// Для каждой обычной задачи сохраняются её длительность, список предшественников
/// с лагами и обратный список последователей. Зависимость без лага (`lag == None`)
/// считается нулевым лагом.
fn build_graph(project: &Project) -> Graph {
    let tasks = project.get_project_tasks();
    let mut graph = Graph::default();
    for task in tasks {
        if task.is_summary {
            // Групповые задачи исключаются из расчетного критического пути
            continue;
        }
        let task_id = *task.get_id();
        graph.durations.insert(task_id, *task.get_duration());

        let dependencies: Vec<(Uuid, TimeDelta)> = task
            .get_dependencies()
            .iter()
            .map(|dep| (dep.depends_on, dep.lag.unwrap_or_else(TimeDelta::zero)))
            .collect();

        // Сохраняем предшественников для task_id
        graph.predecessors.insert(task_id, dependencies.clone());

        // Для каждого предшественника добавляем task_id в его последователи
        for (pred_id, lag) in dependencies {
            graph
                .successors
                .entry(pred_id)
                .or_default()
                .push((task_id, lag));
        }
    }
    graph
}

/// Топологическая сортировка графа задач (алгоритм Кана).
///
/// Упорядочивает вершины так, что для каждого ребра `u → v` вершина `u` идёт раньше `v`.
/// В нашем графе ребро направлено от предшественника к зависимой задаче (если `B`
/// зависит от `A`, то ребро `A → B`), поэтому на выходе получается порядок, в котором
/// задачи можно считать слева направо.
///
/// Шаги алгоритма:
///
/// 1. Посчитать входящую степень каждой вершины — количество предшественников.
/// 2. Поместить в очередь все вершины с нулевой входящей степенью.
/// 3. Пока очередь не пуста: извлечь вершину `u`, добавить её в результат, уменьшить
///    входящую степень каждого последователя `v` и положить `v` в очередь,
///    когда его степень станет нулём.
/// 4. Если в результате оказалось меньше вершин, чем в графе, значит остались вершины
///    с ненулевой степенью — граф содержит цикл.
///
/// Список всех задач берётся из `graph.durations` (это единственный источник истины
/// о составе вершин).
///
/// Ошибку возвращает, если граф содержит цикл (в тексте перечисляются незадействованные
/// задачи) или если задача зависит от задачи, которой нет в графе.
fn topological_sort(graph: &Graph) -> anyhow::Result<Vec<Uuid>> {
    // Получаем все ID задач из durations (это единственный источник истины)
    let all_tasks: Vec<Uuid> = graph.durations.keys().copied().collect();
    let mut in_degree: HashMap<Uuid, usize> = HashMap::new();

    // Инициализируем входящие степени
    for &task_id in &all_tasks {
        let pred_count = graph
            .predecessors
            .get(&task_id)
            .map(|v| v.len())
            .unwrap_or(0);
        in_degree.insert(task_id, pred_count);
    }

    let mut queue: VecDeque<Uuid> = VecDeque::new();
    for (&task_id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(task_id);
        }
    }

    let mut order = Vec::new();

    while let Some(u) = queue.pop_front() {
        order.push(u);

        if let Some(successors) = graph.successors.get(&u) {
            for (v, _) in successors {
                // Проверяем, что v существует в графе
                let deg = in_degree.get_mut(v).ok_or_else(|| {
                    anyhow::anyhow!("Task {} depends on non-existent task {}", u, v)
                })?;
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(*v);
                }
            }
        }
    }

    if order.len() != all_tasks.len() {
        anyhow::bail!(
            "Graph contains a cycle (tasks left: {:?})",
            all_tasks
                .iter()
                .filter(|id| !order.contains(id))
                .collect::<Vec<_>>()
        );
    }

    Ok(order)
}

/// Результат прямого или обратного прохода: даты задач по их ID.
type HashUuidDateTime = HashMap<Uuid, DateTime<Utc>>;

/// Прямой проход: ранние даты задач — ES (early start) и EF (early finish).
///
/// Задачи перебираются в топологическом порядке, поэтому к моменту обработки задачи
/// ранние финиши всех её предшественников уже посчитаны.
///
/// * задача без предшественников начинается в `project_start`;
/// * для остальных `ES = max(EF(предшественника) + лаг)` по всем предшественникам;
/// * `EF = ES + длительность`.
///
/// Даты считаются в календарном времени, без учёта календаря проекта.
///
/// Ошибку возвращает, если у задачи нет длительности или если в `ef` не нашлось
/// предшественника — это означало бы неверный топологический порядок.
fn forward_pass(
    project_start: DateTime<Utc>,
    graph: &Graph,
    order: &[Uuid],
) -> anyhow::Result<(HashUuidDateTime, HashUuidDateTime)> {
    let mut es = HashMap::new();
    let mut ef = HashMap::new();

    for &task_id in order {
        if let Some(preds) = graph.predecessors.get(&task_id) {
            if preds.is_empty() {
                // Пустой список предшественников – значит, их нет
                es.insert(task_id, project_start);
            } else {
                let mut max_ef_plus_lag: Option<DateTime<Utc>> = None;
                for (pred_id, lag) in preds {
                    let pred_ef = ef.get(pred_id).ok_or_else(|| {
                        anyhow::anyhow!("Predecessor {} not found in ef", pred_id)
                    })?;
                    let candidate = *pred_ef + *lag;
                    max_ef_plus_lag = Some(match max_ef_plus_lag {
                        None => candidate,
                        Some(prev) => prev.max(candidate),
                    });
                }
                es.insert(task_id, max_ef_plus_lag.unwrap()); // здесь `unwrap` безопасен, т.к. список не пуст
            }
        } else {
            es.insert(task_id, project_start);
        }

        let duration = graph
            .durations
            .get(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Duration missing for task {}", task_id))?;
        let finish = es[&task_id] + *duration;
        ef.insert(task_id, finish);
    }

    Ok((es, ef))
}

/// Обратный проход: поздние даты задач — LS (late start) и LF (late finish).
///
/// Задачи перебираются в порядке, обратном топологическому, поэтому к моменту
/// обработки задачи поздние старты всех её последователей уже посчитаны.
/// `max_ef` — самый поздний ранний финиш среди всех задач, то есть расчётное
/// окончание проекта.
///
/// * задача без последователей: `LF = max_ef`, `LS = LF - длительность`;
/// * для остальных: `LF = min(LS(последователя) - лаг)` по всем последователям,
///   `LS = LF - длительность`.
///
/// Поздний финиш задачи без последователей берётся равным `max_ef`, а не дате окончания
/// проекта: критический путь считается как самая длинная цепочка работ, без привязки
/// к дедлайну. Поэтому дата окончания проекта на расчёт не влияет — она ограничивает
/// только даты задач. Диагностика «укладывается ли план в срок» — отдельная задача, см.
/// [issue #17](https://github.com/DarkwingDuck48/rsproject/issues/17).
fn backward_pass(
    graph: &Graph,
    ef: &HashMap<Uuid, DateTime<Utc>>,
    order: &[Uuid],
) -> anyhow::Result<(HashUuidDateTime, HashUuidDateTime)> {
    let max_ef = ef
        .values()
        .max()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("No tasks in graph"))?;

    let mut ls = HashMap::new();
    let mut lf = HashMap::new();

    for &task_id in order.iter().rev() {
        let duration = graph
            .durations
            .get(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Duration missing for task {}", task_id))?;

        if let Some(succs) = graph.successors.get(&task_id) {
            if succs.is_empty() {
                // Пустой список последователей – считаем, что их нет
                lf.insert(task_id, max_ef);
                ls.insert(task_id, max_ef - *duration);
            } else {
                let mut min_ls_minus_lag: Option<DateTime<Utc>> = None;
                for (succ_id, lag) in succs {
                    let succ_ls = ls
                        .get(succ_id)
                        .ok_or_else(|| anyhow::anyhow!("Successor LS not found for {}", succ_id))?;
                    let candidate = *succ_ls - *lag;
                    min_ls_minus_lag = Some(match min_ls_minus_lag {
                        None => candidate,
                        Some(prev) => prev.min(candidate),
                    });
                }
                let late_finish = min_ls_minus_lag.unwrap(); // безопасен, т.к. список не пуст
                lf.insert(task_id, late_finish);
                ls.insert(task_id, late_finish - *duration);
            }
        } else {
            lf.insert(task_id, max_ef);
            ls.insert(task_id, max_ef - *duration);
        }
    }

    Ok((ls, lf))
}

/// Собирает критический путь из результатов прямого и обратного проходов.
///
/// Критической считается задача без запаса времени: `LF - EF <= EPSILON`. Допуск
/// в миллисекунду нужен потому, что даты сравниваются как моменты времени, а не как дни.
///
/// Одних критических задач мало: у проекта может быть несколько параллельных веток
/// без запаса. Поэтому дополнительно ищется **самая длинная цепочка**: обход в глубину
/// начинается от критических задач без предшественников и продолжается по критическим
/// последователям. Возвращается самая длинная найденная цепочка, а при равной длине — та,
/// что найдена первой.
///
/// Внимание: на выходе именно цепочка, а не множество всех критических задач —
/// при двух равных по длине критических ветках вторая в результат не попадёт.
///
/// Пустой список возвращается, если критических задач нет или ни одна из них
/// не является начальной. Ошибку — если для задачи не нашлось её `lf` или `ef`.
fn find_critical_path(
    graph: &Graph,
    ef: &HashMap<Uuid, DateTime<Utc>>,
    lf: &HashMap<Uuid, DateTime<Utc>>,
) -> anyhow::Result<Vec<Uuid>> {
    const EPSILON: TimeDelta = TimeDelta::milliseconds(1);

    let is_critical = |id: Uuid| -> anyhow::Result<bool> {
        let slack = *lf
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("lf missing for {}", id))?
            - *ef
                .get(&id)
                .ok_or_else(|| anyhow::anyhow!("ef missing for {}", id))?;
        Ok(slack <= EPSILON)
    };

    // Находим все критические задачи
    let mut critical_tasks = Vec::new();
    for &id in graph.durations.keys() {
        if is_critical(id)? {
            critical_tasks.push(id);
        }
    }

    // Стартовые критические задачи (нет предшественников)
    let start_tasks: Vec<Uuid> = critical_tasks
        .iter()
        .copied()
        .filter(|&id| {
            graph
                .predecessors
                .get(&id)
                .map(|v| v.is_empty())
                .unwrap_or(true)
        })
        .collect();

    if start_tasks.is_empty() {
        return Ok(Vec::new());
    }

    // Поиск пути от каждой стартовой задачи через критические последователи
    let mut best_path = Vec::new();
    let mut max_len = 0;

    for start in start_tasks {
        let mut stack = vec![(start, vec![start])];
        while let Some((current, path)) = stack.pop() {
            if let Some(successors) = graph.successors.get(&current) {
                let mut critical_successors = Vec::new();
                for (succ_id, _) in successors {
                    if is_critical(*succ_id)? {
                        critical_successors.push(*succ_id);
                    }
                }

                if critical_successors.is_empty() {
                    if path.len() > max_len {
                        max_len = path.len();
                        best_path = path.clone();
                    }
                } else {
                    for succ in critical_successors {
                        let mut new_path = path.clone();
                        new_path.push(succ);
                        stack.push((succ, new_path));
                    }
                }
            } else if path.len() > max_len {
                max_len = path.len();
                best_path = path.clone();
            }
        }
    }

    Ok(best_path)
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use uuid::Uuid;

    // Хелпер для создания графа из списка рёбер (предшественник -> последователь)
    // Возвращает Graph с durations по умолчанию (например, длительность 1 день для всех)
    fn build_test_graph(edges: Vec<(Uuid, Uuid)>) -> Graph {
        let mut durations = HashMap::new();
        let mut predecessors: HashMap<Uuid, Vec<(Uuid, TimeDelta)>> = HashMap::new();
        let mut successors: HashMap<Uuid, Vec<(Uuid, TimeDelta)>> = HashMap::new();

        let mut all_ids = std::collections::HashSet::new();
        for (from, to) in &edges {
            all_ids.insert(*from);
            all_ids.insert(*to);
        }
        // если нет задач, можно оставить пустым
        for &id in &all_ids {
            durations.insert(id, Duration::days(1));
        }

        for (from, to) in edges {
            let lag = Duration::zero();
            // предшественники: для to добавляем from
            predecessors.entry(to).or_default().push((from, lag));
            // последователи: для from добавляем to
            successors.entry(from).or_default().push((to, lag));
        }

        Graph {
            durations,
            predecessors,
            successors,
        }
    }

    // Вспомогательная функция для создания графа с одной задачей
    fn graph_single_task() -> (Graph, Uuid) {
        let task_id = Uuid::new_v4();
        let mut graph = Graph::default();
        graph.durations.insert(task_id, Duration::days(5));
        (graph, task_id)
    }

    // Вспомогательная функция для создания графа с двумя последовательными задачами
    fn graph_two_tasks_linear(lag: Duration) -> (Graph, Uuid, Uuid) {
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let mut graph = Graph::default();
        graph.durations.insert(task1, Duration::days(3));
        graph.durations.insert(task2, Duration::days(4));
        // task2 зависит от task1 с lag
        graph.predecessors.insert(task2, vec![(task1, lag)]);
        graph.successors.insert(task1, vec![(task2, lag)]);
        (graph, task1, task2)
    }

    // Вспомогательная функция для создания графа с двумя параллельными задачами
    fn graph_parallel() -> (Graph, Uuid, Uuid, Uuid) {
        let task_a = Uuid::new_v4();
        let task_b = Uuid::new_v4();
        let task_c = Uuid::new_v4();
        let mut graph = Graph::default();
        graph.durations.insert(task_a, Duration::days(2));
        graph.durations.insert(task_b, Duration::days(3));
        graph.durations.insert(task_c, Duration::days(1));
        // task_c зависит от task_a и task_b
        graph.predecessors.insert(
            task_c,
            vec![(task_a, Duration::zero()), (task_b, Duration::zero())],
        );
        graph
            .successors
            .insert(task_a, vec![(task_c, Duration::zero())]);
        graph
            .successors
            .insert(task_b, vec![(task_c, Duration::zero())]);
        (graph, task_a, task_b, task_c)
    }

    // Проверка построения графа из реального проекта: сводные задачи пропускаются,
    // лаги зависимостей сохраняются, обычные задачи попадают в durations.
    #[test]
    fn build_graph_skips_summary_tasks_and_keeps_lags() {
        use crate::base_structures::{Dependency, DependencyType, Task};

        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
        let mut project = Project::new("Test", "desc", start, end).unwrap();

        // Сводная задача — не должна попасть в граф.
        let summary = Task::new_summary("Summary", start, end, None).unwrap();
        let summary_id = *summary.get_id();
        project.tasks.insert(summary_id, summary);

        // Обычная задача A длительностью 5 дней.
        let task_a = Task::new_regular("A", start, start + Duration::days(5), None).unwrap();
        let a_id = *task_a.get_id();
        project.tasks.insert(a_id, task_a);

        // Обычная задача B, зависит от A с лагом 2 дня.
        let task_b = Task::new_regular(
            "B",
            start + Duration::days(5),
            start + Duration::days(8),
            None,
        )
        .unwrap();
        let b_id = *task_b.get_id();
        project.tasks.insert(b_id, task_b);

        // Зависимость B от A с лагом.
        project
            .tasks
            .get_mut(&b_id)
            .unwrap()
            .add_dependency(Dependency::new(
                DependencyType::Blocking,
                a_id,
                Some(Duration::days(2)),
            ))
            .unwrap();

        let graph = build_graph(&project);

        // Сводная задача пропущена, обычные — в durations с правильной длительностью.
        assert!(!graph.durations.contains_key(&summary_id));
        assert_eq!(graph.durations[&a_id], Duration::days(5));
        assert_eq!(graph.durations[&b_id], Duration::days(3));

        // B зависит от A: предшественник B — A, последователь A — B, лаг сохранён.
        assert_eq!(graph.predecessors[&b_id], vec![(a_id, Duration::days(2))]);
        assert_eq!(graph.successors[&a_id], vec![(b_id, Duration::days(2))]);
    }

    #[test]
    fn test_empty_graph() {
        let graph = Graph::default(); // предполагаем, что у Graph есть impl Default
        let order = topological_sort(&graph).unwrap();
        assert!(order.is_empty());
    }

    #[test]
    fn test_single_task() {
        let mut graph = Graph::default();
        let id = Uuid::new_v4();
        graph.durations.insert(id, Duration::days(1));
        // нет зависимостей
        let order = topological_sort(&graph).unwrap();
        assert_eq!(order, vec![id]);
    }

    #[test]
    fn test_linear_chain() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let edges = vec![(a, b), (b, c)];
        let graph = build_test_graph(edges);
        let order = topological_sort(&graph).unwrap();
        // Порядок должен быть a, b, c
        assert_eq!(order, vec![a, b, c]);
    }

    #[test]
    fn test_parallel_chains() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();
        // две независимые цепочки: a->b и c->d
        let edges = vec![(a, b), (c, d)];
        let graph = build_test_graph(edges);
        let order = topological_sort(&graph).unwrap();
        // В порядке могут быть a и c раньше b и d. Проверим только, что a предшествует b, c предшествует d.
        let pos_a = order.iter().position(|&x| x == a).unwrap();
        let pos_b = order.iter().position(|&x| x == b).unwrap();
        let pos_c = order.iter().position(|&x| x == c).unwrap();
        let pos_d = order.iter().position(|&x| x == d).unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_cycle() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let edges = vec![(a, b), (b, a)];
        let graph = build_test_graph(edges);
        let result = topological_sort(&graph);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }

    #[test]
    fn test_missing_dependency() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4(); // b не добавлена в durations
        let mut graph = Graph::default();
        graph.durations.insert(a, Duration::days(1));
        graph.successors.insert(a, vec![(b, Duration::zero())]);
        graph.predecessors.insert(b, vec![(a, Duration::zero())]); // но b нет в durations
        let result = topological_sort(&graph);
        // Ожидаем ошибку, потому что при обработке a попытаемся уменьшить степень b, но её нет в in_degree
        // В зависимости от реализации, либо ошибка, либо паника. Мы рассчитываем на ошибку.
        assert!(result.is_err());
    }

    #[test]
    fn test_forward_pass_single_task() {
        let (graph, task_id) = graph_single_task();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![task_id];

        let (es, ef) = forward_pass(start, &graph, &order).unwrap();

        assert_eq!(es[&task_id], start);
        assert_eq!(ef[&task_id], start + Duration::days(5));
    }

    #[test]
    fn test_forward_pass_linear_zero_lag() {
        let (graph, t1, t2) = graph_two_tasks_linear(Duration::zero());
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];

        let (es, ef) = forward_pass(start, &graph, &order).unwrap();

        assert_eq!(es[&t1], start);
        assert_eq!(ef[&t1], start + Duration::days(3));
        assert_eq!(es[&t2], ef[&t1]); // t2 стартует сразу после t1
        assert_eq!(ef[&t2], start + Duration::days(3) + Duration::days(4));
    }

    #[test]
    fn test_forward_pass_linear_with_lag() {
        let lag = Duration::days(2);
        let (graph, t1, t2) = graph_two_tasks_linear(lag);
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];

        let (es, ef) = forward_pass(start, &graph, &order).unwrap();

        assert_eq!(es[&t1], start);
        assert_eq!(ef[&t1], start + Duration::days(3));
        assert_eq!(es[&t2], ef[&t1] + lag);
        assert_eq!(ef[&t2], start + Duration::days(3) + lag + Duration::days(4));
    }

    #[test]
    fn test_forward_pass_parallel() {
        let (graph, a, b, c) = graph_parallel();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        // Топологический порядок может быть [a, b, c] или [b, a, c] – допустим [a, b, c]
        let order = vec![a, b, c];

        let (es, ef) = forward_pass(start, &graph, &order).unwrap();

        assert_eq!(es[&a], start);
        assert_eq!(ef[&a], start + Duration::days(2));
        assert_eq!(es[&b], start);
        assert_eq!(ef[&b], start + Duration::days(3));
        // c стартует после максимума из окончаний a и b
        let expected_c_start = start + Duration::days(3); // т.к. b заканчивается позже (3 дня)
        assert_eq!(es[&c], expected_c_start);
        assert_eq!(ef[&c], expected_c_start + Duration::days(1));
    }
    #[test]
    fn test_backward_pass_single_task() {
        let (graph, task_id) = graph_single_task();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![task_id];
        let (es, ef) = forward_pass(start, &graph, &order).unwrap();
        // Дата окончания проекта в обратный проход не передаётся: LF терминальных задач
        // берётся равным max_ef, а не дате окончания проекта.
        let (ls, lf) = backward_pass(&graph, &ef, &order).unwrap();

        assert_eq!(lf[&task_id], ef[&task_id]); // для одной задачи lf = ef
        assert_eq!(ls[&task_id], es[&task_id]);
    }

    #[test]
    fn test_backward_pass_linear_zero_lag() {
        let (graph, t1, t2) = graph_two_tasks_linear(Duration::zero());
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];
        let (es, ef) = forward_pass(start, &graph, &order).unwrap();
        let (ls, lf) = backward_pass(&graph, &ef, &order).unwrap();

        // Ожидаем, что поздние сроки совпадают с ранними (критический путь)
        assert_eq!(lf[&t1], ef[&t1]);
        assert_eq!(ls[&t1], es[&t1]);
        assert_eq!(lf[&t2], ef[&t2]);
        assert_eq!(ls[&t2], es[&t2]);
    }

    #[test]
    fn test_backward_pass_linear_with_lag() {
        let lag = Duration::days(2);
        let (graph, t1, t2) = graph_two_tasks_linear(lag);
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];
        let (es, ef) = forward_pass(start, &graph, &order).unwrap();
        let (ls, lf) = backward_pass(&graph, &ef, &order).unwrap();

        // t2: поздний финиш = max_ef = ef[t2] (так как t2 без последователей)
        assert_eq!(lf[&t2], ef[&t2]);
        assert_eq!(ls[&t2], lf[&t2] - graph.durations[&t2]);

        // t1: lf = ls[t2] - lag
        assert_eq!(lf[&t1], ls[&t2] - lag);
        assert_eq!(ls[&t1], lf[&t1] - graph.durations[&t1]);

        // Поскольку t1 критическая (другого пути нет), ls[t1] должно равняться es[t1]
        assert_eq!(ls[&t1], es[&t1]);
    }

    #[test]
    fn test_backward_pass_parallel() {
        let (graph, a, b, c) = graph_parallel();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![a, b, c];
        let (_, ef) = forward_pass(start, &graph, &order).unwrap();
        let (ls, lf) = backward_pass(&graph, &ef, &order).unwrap();

        // max_ef = ef[c]
        let max_ef = ef[&c];

        // c без последователей: lf = max_ef, ls = lf - duration
        assert_eq!(lf[&c], max_ef);
        assert_eq!(ls[&c], max_ef - graph.durations[&c]);

        // Для a и b: lf = ls[c] - lag (lag=0)
        assert_eq!(lf[&a], ls[&c]);
        assert_eq!(ls[&a], lf[&a] - graph.durations[&a]);
        assert_eq!(lf[&b], ls[&c]);
        assert_eq!(ls[&b], lf[&b] - graph.durations[&b]);

        // Так как a и b параллельны, их поздние сроки могут отличаться от ранних (если есть резерв)
        // Проверим, что резерв (lf - ef) неотрицателен
        assert!(lf[&a] >= ef[&a]);
        assert!(lf[&b] >= ef[&b]);
    }
    #[test]
    fn test_critical_path_single() {
        let (graph, task_id) = graph_single_task();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![task_id];
        let (_, ef) = forward_pass(start, &graph, &order).unwrap();
        let (_, lf) = backward_pass(&graph, &ef, &order).unwrap();
        let path = find_critical_path(&graph, &ef, &lf).unwrap();
        assert_eq!(path, vec![task_id]);
    }

    #[test]
    fn test_critical_path_linear() {
        let (graph, t1, t2) = graph_two_tasks_linear(TimeDelta::zero());
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];
        let (_, ef) = forward_pass(start, &graph, &order).unwrap();
        let (_, lf) = backward_pass(&graph, &ef, &order).unwrap();
        let path = find_critical_path(&graph, &ef, &lf).unwrap();
        assert_eq!(path, vec![t1, t2]);
    }

    #[test]
    fn test_critical_path_parallel() {
        let (graph, a, b, c) = graph_parallel();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![a, b, c];
        let (_, ef) = forward_pass(start, &graph, &order).unwrap();
        let (_, lf) = backward_pass(&graph, &ef, &order).unwrap();
        let path = find_critical_path(&graph, &ef, &lf).unwrap();
        // Ожидаем, что критический путь b -> c (т.к. b длиннее a)
        assert_eq!(path, vec![b, c]);
    }
}

// Property-based тесты планировщика (задача 6.3).
//
// Проверяются инварианты CPM на случайных ациклических графах:
// * ранние даты не позже поздних (ES <= LS, EF <= LF) для каждой задачи;
// * сумма длительностей задач на критическом пути не превосходит сумму длительностей
//   всех задач проекта (длина пути — подмножество задач, а длительности неотрицательны);
// * критический путь — валидная цепочка: соседние задачи связаны рёбрами графа,
//   а запас времени каждой задачи пути равен нулю.
#[cfg(test)]
mod property_tests {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Генерирует случайный ациклический граф задач.
    ///
    /// Вершины — задачи с номерами 0..n. Ребра выбираются из полного списка пар
    /// (i, j) с i < j: такая пара по построению не образует цикл, поэтому фильтры
    /// и повторы не нужны.
    fn dag_strategy() -> impl Strategy<Value = (Graph, DateTime<Utc>)> {
        (1usize..=6).prop_flat_map(|n| {
            let node_ids: Vec<Uuid> = (0..n).map(|_| Uuid::new_v4()).collect();
            // Все возможные пары (i, j), i < j — индексы рёбер.
            let all_pairs: Vec<(usize, usize)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let pair_count = all_pairs.len();
            let durations = prop::collection::vec(1i64..=20i64, n);
            let edges = prop::collection::vec(
                (0..pair_count.max(1), 0i64..=5i64), // (индекс пары, лаг в днях)
                0..=pair_count.max(1),
            );
            (durations, edges).prop_map(move |(durations, edges)| {
                let mut graph = Graph::default();
                for (i, &d) in durations.iter().enumerate() {
                    graph.durations.insert(node_ids[i], TimeDelta::days(d));
                }

                // Дубликаты рёбер отбрасываем, чтобы не засорять граф.
                let mut seen = HashSet::new();
                for (pair_idx, lag_days) in edges {
                    if pair_idx >= pair_count || !seen.insert(pair_idx) {
                        continue;
                    }
                    let (i, j) = all_pairs[pair_idx];
                    let (from, to) = (node_ids[i], node_ids[j]);
                    let lag = TimeDelta::days(lag_days);
                    graph.predecessors.entry(to).or_default().push((from, lag));
                    graph.successors.entry(from).or_default().push((to, lag));
                }

                // Старт проекта — фиксированная дата; случайность графа достигается
                // длительностями, рёбрами и лагами.
                let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
                (graph, start)
            })
        })
    }

    proptest! {
        // Инвариант 6.3: ранние даты не позже поздних (ES <= LS, EF <= LF).
        #[test]
        fn early_dates_do_not_exceed_late_dates((graph, start) in dag_strategy()) {
            let order = topological_sort(&graph).expect("граф ациклический");
            let (es, ef) = forward_pass(start, &graph, &order).unwrap();
            let (ls, lf) = backward_pass(&graph, &ef, &order).unwrap();

            for &id in graph.durations.keys() {
                let es = es[&id];
                let ls = ls[&id];
                let ef = ef[&id];
                let lf = lf[&id];
                prop_assert!(
                    es <= ls,
                    "ES ({es:?}) должен быть <= LS ({ls:?}) для задачи {id}"
                );
                prop_assert!(
                    ef <= lf,
                    "EF ({ef:?}) должен быть <= LF ({lf:?}) для задачи {id}"
                );
            }
        }

        // Инвариант 6.3: сумма длительностей задач >= длины критического пути.
        // Длина пути считается как сумма длительностей задач на нём; путь — подмножество
        // задач проекта, поэтому его сумма не может превысить общую сумму длительностей.
        #[test]
        fn critical_path_duration_does_not_exceed_total_duration((graph, start) in dag_strategy()) {
            let order = topological_sort(&graph).expect("граф ациклический");
            let (_, ef) = forward_pass(start, &graph, &order).unwrap();
            let (_, lf) = backward_pass(&graph, &ef, &order).unwrap();
            let path = find_critical_path(&graph, &ef, &lf).unwrap();

            let total: TimeDelta = graph.durations.values().fold(TimeDelta::zero(), |acc, d| acc + *d);
            let path_duration: TimeDelta = path
                .iter()
                .fold(TimeDelta::zero(), |acc, id| acc + graph.durations[id]);
            prop_assert!(
                path_duration <= total,
                "длина пути ({path_duration:?}) не должна превышать сумму длительностей ({total:?})"
            );
        }

        // Критический путь — валидная цепочка: соседние задачи связаны рёбрами,
        // и все задачи пути имеют нулевой запас времени.
        #[test]
        fn critical_path_is_a_valid_chain((graph, start) in dag_strategy()) {
            let order = topological_sort(&graph).expect("граф ациклический");
            let (_, ef) = forward_pass(start, &graph, &order).unwrap();
            let (_, lf) = backward_pass(&graph, &ef, &order).unwrap();
            let path = find_critical_path(&graph, &ef, &lf).unwrap();

            prop_assert!(!path.is_empty(), "в непустом графе должен быть критический путь");

            // Все задачи пути — критические: запас времени (lf - ef) равен нулю.
            for &id in &path {
                let slack = lf[&id] - ef[&id];
                prop_assert!(
                    slack <= TimeDelta::milliseconds(1) && slack >= TimeDelta::milliseconds(-1),
                    "задача {id} на критическом пути, но запас = {slack:?}"
                );
            }

            // Соседние задачи пути соединены ребром в графе (последовательность задач).
            for pair in path.windows(2) {
                let has_edge = graph.successors.get(&pair[0]).is_some_and(|succs| {
                    succs.iter().any(|(s, _lag)| *s == pair[1])
                });
                prop_assert!(
                    has_edge,
                    "между задачами {p0:?} -> {p1:?} нет ребра в графе",
                    p0 = pair[0],
                    p1 = pair[1],
                );
            }
        }

        // Согласованность проходов: EF = ES + длительность и LF = LS + длительность
        // для каждой задачи (базовое свойство CPM, ловит ошибки в арифметике дат).
        #[test]
        fn durations_are_consistent_with_dates((graph, start) in dag_strategy()) {
            let order = topological_sort(&graph).expect("граф ациклический");
            let (es, ef) = forward_pass(start, &graph, &order).unwrap();
            let (ls, lf) = backward_pass(&graph, &ef, &order).unwrap();

            for &id in graph.durations.keys() {
                let duration = graph.durations[&id];
                let ef_minus_es = ef[&id] - es[&id];
                let lf_minus_ls = lf[&id] - ls[&id];
                prop_assert!(
                    ef_minus_es == duration,
                    "EF - ES ({ef_minus_es:?}) != длительность задачи {id}"
                );
                prop_assert!(
                    lf_minus_ls == duration,
                    "LF - LS ({lf_minus_ls:?}) != длительность задачи {id}"
                );
            }
        }
    }
}
