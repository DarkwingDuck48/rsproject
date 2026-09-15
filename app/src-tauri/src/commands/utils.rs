use chrono::{DateTime, NaiveDate, Utc};
use logic::{BasicGettersForStructures, Task};
use uuid::Uuid;

use crate::state::AppState;

pub fn parse_date(date: &str) -> Result<DateTime<Utc>, String> {
    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    Ok(parsed_date)
}

pub fn resolve_project_id(app_state: &AppState, project_id: Option<Uuid>) -> Result<Uuid, String> {
    match project_id {
        Some(id) => Ok(id),
        None => {
            let id = app_state.selected_project_id.lock().unwrap();
            id.ok_or("Не выбран проект".to_string())
        }
    }
}

/// Сортирует массив задач по началу и по окончанию работ
pub fn sort_by_dates(tasks: &mut [&Task]) {
    tasks.sort_by(|a, b| {
        a.get_date_start()
            .cmp(b.get_date_start())
            .then_with(|| a.get_date_end().cmp(b.get_date_end()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Тестовая задача: даты задаём строками "YYYY-MM-DD",
    /// парсинг переиспользуем из того же модуля.
    fn task(name: &str, start: &str, end: &str) -> Task {
        Task::new_regular(
            name,
            parse_date(start).unwrap(),
            parse_date(end).unwrap(),
            None,
        )
        .unwrap()
    }

    /// Имена задач по порядку — так ассерты читаются как «ожидал и получил».
    fn names<'a>(tasks: &'a [&'a Task]) -> Vec<&'a str> {
        tasks.iter().map(|t| t.name.as_str()).collect()
    }

    #[test]
    fn sorts_by_start_date() {
        let tasks = [
            task("B", "2025-02-01", "2025-02-10"),
            task("A", "2025-01-01", "2025-01-10"),
            task("C", "2025-03-01", "2025-03-10"),
        ];
        let mut refs: Vec<&Task> = tasks.iter().collect();

        sort_by_dates(&mut refs);

        assert_eq!(names(&refs), ["A", "B", "C"]);
    }

    #[test]
    fn breaks_start_ties_by_end_date() {
        let tasks = [
            task("A", "2025-01-01", "2025-01-20"),
            task("B", "2025-01-01", "2025-01-05"),
            task("C", "2025-01-01", "2025-01-10"),
        ];
        let mut refs: Vec<&Task> = tasks.iter().collect();

        sort_by_dates(&mut refs);

        assert_eq!(names(&refs), ["B", "C", "A"]);
    }

    #[test]
    fn keeps_stable_order_on_full_ties() {
        let tasks = [
            task("A", "2025-01-01", "2025-01-10"),
            task("B", "2025-01-01", "2025-01-10"),
            task("C", "2025-01-01", "2025-01-10"),
        ];
        let mut refs: Vec<&Task> = tasks.iter().collect();

        sort_by_dates(&mut refs);

        // sort_by — стабильная сортировка: при полном равенстве ключей
        // сохраняется порядок входного массива.
        assert_eq!(names(&refs), ["A", "B", "C"]);
    }

    #[test]
    fn handles_empty_and_single_input() {
        let mut empty: Vec<&Task> = vec![];
        sort_by_dates(&mut empty);
        assert!(empty.is_empty());

        let tasks = [task("Solo", "2025-01-01", "2025-01-10")];
        let mut one: Vec<&Task> = tasks.iter().collect();
        sort_by_dates(&mut one);
        assert_eq!(names(&one), ["Solo"]);
    }
}
