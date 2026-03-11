use std::collections::{HashMap, VecDeque};

use crate::{BasicGettersForStructures, Project, ProjectContainer};
use chrono::{DateTime, TimeDelta, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
struct Graph {
    predecessors: HashMap<Uuid, Vec<(Uuid, TimeDelta)>>, // Предшественник, lag
    successors: HashMap<Uuid, Vec<(Uuid, TimeDelta)>>,   // (последователь, lag)
    durations: HashMap<Uuid, TimeDelta>,
}

#[derive(Copy, Clone, Debug)]
pub struct Scheduler<'a, C: ProjectContainer> {
    container: &'a C,
}

impl<'a, C: ProjectContainer> Scheduler<'a, C> {
    pub fn new(container: &'a C) -> Self {
        Self { container }
    }

    pub fn critical_path(&self, project_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
        let project = self
            .container
            .get_project(&project_id)
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
        let graph = build_graph(project);
        let order = topological_sort(&graph)?;
        let (es, ef) = forward_pass(*project.get_date_start(), &graph, &order);
        let (ls, lf) = backward_pass(*project.get_date_end(), &graph, &es, &ef, &order);
        Ok(find_critical_path(&graph, &es, &ef, &ls, &lf))
    }
}

fn build_graph(project: &Project) -> Graph {
    let tasks = project.get_project_tasks();
    let mut graph = Graph::default();
    for task in tasks {
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

/// Алгоритм Кана
/// Топологическая сортировка упорядочивает вершины ориентированного ациклического графа так, что для каждого ребра (u → v) вершина u предшествует v в порядке.
/// В нашем случае ребро направлено от предшественника к зависимой задаче (если задача B зависит от A, то ребро A → B).
/// Для сортировки используем алгоритм Кана:
/// Вычислить входящую степень (количество предшественников) для каждой вершины.
/// Инициализировать очередь всеми вершинами с нулевой входящей степенью.
/// Пока очередь не пуста:
/// Извлечь вершину u из очереди и добавить её в результат.
/// Для каждого последователя v вершины u:
/// Уменьшить входящую степень v на 1.
/// Если входящая степень v стала 0, добавить v в очередь.
/// Если после завершения количество вершин в результате меньше общего числа вершин, граф содержит цикл — ошибка.
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

fn forward_pass(
    project_start: DateTime<Utc>,
    graph: &Graph,
    order: &[Uuid],
) -> (HashMap<Uuid, DateTime<Utc>>, HashMap<Uuid, DateTime<Utc>>) {
    let mut es = HashMap::new();
    let mut ef = HashMap::new();

    for &task_id in order {
        // Обрабатываем предшественников
        if let Some(preds) = graph.predecessors.get(&task_id) {
            // Вычисляем максимум из (ef[p] + lag) по всем предшественникам
            let mut max_ef_plus_lag: Option<DateTime<Utc>> = None;
            for (pred_id, lag) in preds {
                let pred_ef = ef
                    .get(pred_id)
                    .expect("Predecessor EF not computed – graph order broken");
                let candidate = *pred_ef + *lag;
                max_ef_plus_lag = Some(match max_ef_plus_lag {
                    None => candidate,
                    Some(prev) => prev.max(candidate),
                });
            }
            es.insert(task_id, max_ef_plus_lag.unwrap());
        } else {
            // Нет предшественников – стартуем в начале проекта
            es.insert(task_id, project_start);
        }

        // Вычисляем ранний финиш
        let duration = graph
            .durations
            .get(&task_id)
            .expect("Duration missing for task");
        let finish = es[&task_id] + *duration;
        ef.insert(task_id, finish);
    }

    (es, ef)
}

fn backward_pass(
    project_end: DateTime<Utc>,
    graph: &Graph,
    es: &HashMap<Uuid, DateTime<Utc>>,
    ef: &HashMap<Uuid, DateTime<Utc>>,
    order: &[Uuid],
) -> (HashMap<Uuid, DateTime<Utc>>, HashMap<Uuid, DateTime<Utc>>) {
    let max_ef = ef.values().max().copied().expect("No tasks in graph");

    let mut ls = HashMap::new();
    let mut lf = HashMap::new();

    for &task_id in order.iter().rev() {
        let duration = graph.durations.get(&task_id).unwrap();

        if let Some(succs) = graph.successors.get(&task_id) {
            // Вычисляем минимум из (ls[succ] - lag) по всем последователям
            let mut min_ls_minus_lag: Option<DateTime<Utc>> = None;
            for (succ_id, lag) in succs {
                let succ_ls = ls
                    .get(succ_id)
                    .expect("Successor LS not computed – reverse order broken");
                let candidate = *succ_ls - *lag;
                min_ls_minus_lag = Some(match min_ls_minus_lag {
                    None => candidate,
                    Some(prev) => prev.min(candidate),
                });
            }
            let late_finish = min_ls_minus_lag.unwrap();
            lf.insert(task_id, late_finish);
            ls.insert(task_id, late_finish - *duration);
        } else {
            // Нет последователей
            lf.insert(task_id, max_ef);
            ls.insert(task_id, max_ef - *duration);
        }
    }

    (ls, lf)
}

fn find_critical_path(
    graph: &Graph,
    es: &HashMap<Uuid, DateTime<Utc>>,
    ef: &HashMap<Uuid, DateTime<Utc>>,
    ls: &HashMap<Uuid, DateTime<Utc>>,
    lf: &HashMap<Uuid, DateTime<Utc>>,
) -> Vec<Uuid> {
    // Эпсилон для сравнения дат с учётом возможных погрешностей
    const EPSILON: TimeDelta = TimeDelta::milliseconds(1);

    // Проверка, является ли задача критической (резерв <= EPSILON)
    let is_critical = |id: Uuid| -> bool {
        let slack = *lf.get(&id).unwrap() - ef.get(&id).unwrap();
        slack <= EPSILON
    };

    // Находим все критические задачи
    let critical_tasks: Vec<Uuid> = graph
        .durations
        .keys()
        .copied()
        .filter(|&id| is_critical(id))
        .collect();

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
        return Vec::new();
    }

    // Поиск пути от каждой стартовой задачи через критические последователи
    let mut best_path = Vec::new();
    let mut max_len = 0;

    for start in start_tasks {
        // Стек для DFS: (текущая вершина, путь от старта)
        let mut stack = vec![(start, vec![start])];
        while let Some((current, path)) = stack.pop() {
            // Используем if let Some вместо unwrap_or
            if let Some(successors) = graph.successors.get(&current) {
                let critical_successors: Vec<Uuid> = successors
                    .iter()
                    .map(|(id, _)| *id)
                    .filter(|&id| is_critical(id))
                    .collect();

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
            } else {
                // Нет последователей – конец пути
                if path.len() > max_len {
                    max_len = path.len();
                    best_path = path.clone();
                }
            }
        }
    }

    best_path
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

        let (es, ef) = forward_pass(start, &graph, &order);

        assert_eq!(es[&task_id], start);
        assert_eq!(ef[&task_id], start + Duration::days(5));
    }

    #[test]
    fn test_forward_pass_linear_zero_lag() {
        let (graph, t1, t2) = graph_two_tasks_linear(Duration::zero());
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];

        let (es, ef) = forward_pass(start, &graph, &order);

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

        let (es, ef) = forward_pass(start, &graph, &order);

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

        let (es, ef) = forward_pass(start, &graph, &order);

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
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let order = vec![task_id];
        let (es, ef) = forward_pass(start, &graph, &order);
        let (ls, lf) = backward_pass(end, &graph, &es, &ef, &order);

        assert_eq!(lf[&task_id], ef[&task_id]); // для одной задачи lf = ef
        assert_eq!(ls[&task_id], es[&task_id]);
    }

    #[test]
    fn test_backward_pass_linear_zero_lag() {
        let (graph, t1, t2) = graph_two_tasks_linear(Duration::zero());
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];
        let (es, ef) = forward_pass(start, &graph, &order);
        let (ls, lf) = backward_pass(end, &graph, &es, &ef, &order);

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
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];
        let (es, ef) = forward_pass(start, &graph, &order);
        let (ls, lf) = backward_pass(end, &graph, &es, &ef, &order);

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
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let order = vec![a, b, c];
        let (es, ef) = forward_pass(start, &graph, &order);
        let (ls, lf) = backward_pass(end, &graph, &es, &ef, &order);

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
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let order = vec![task_id];
        let (es, ef) = forward_pass(start, &graph, &order);
        let (ls, lf) = backward_pass(end, &graph, &es, &ef, &order);
        let path = find_critical_path(&graph, &es, &ef, &ls, &lf);
        assert_eq!(path, vec![task_id]);
    }

    #[test]
    fn test_critical_path_linear() {
        let (graph, t1, t2) = graph_two_tasks_linear(TimeDelta::zero());
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let order = vec![t1, t2];
        let (es, ef) = forward_pass(start, &graph, &order);
        let (ls, lf) = backward_pass(end, &graph, &es, &ef, &order);
        let path = find_critical_path(&graph, &es, &ef, &ls, &lf);
        assert_eq!(path, vec![t1, t2]);
    }

    #[test]
    fn test_critical_path_parallel() {
        let (graph, a, b, c) = graph_parallel();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let order = vec![a, b, c];
        let (es, ef) = forward_pass(start, &graph, &order);
        let (ls, lf) = backward_pass(end, &graph, &es, &ef, &order);
        let path = find_critical_path(&graph, &es, &ef, &ls, &lf);
        // Ожидаем, что критический путь b -> c (т.к. b длиннее a)
        assert_eq!(path, vec![b, c]);
    }
}
