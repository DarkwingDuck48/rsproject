use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::base_structures::time_window::TimeWindow;

/// Глобальный календарь проекта/компании
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectCalendar {
    /// Рабочие дни недели (по умолчанию пн-пт)
    working_days: HashSet<Weekday>,

    /// Праздничные/нерабочие дни (конкретные даты)
    holidays: HashSet<NaiveDate>,

    /// Часов в рабочем дне (для пересчета в трудозатраты)
    pub working_hours_per_day: u32,

    /// Среднее число рабочих дней в месяце (для пересчёта месячных ставок,
    /// по умолчанию 22 = 52 недели × 5 рабочих дней ÷ 12 месяцев).
    /// Значение настраивается при создании календаря проекта.
    pub working_days_per_month: u32,
}

impl Default for ProjectCalendar {
    fn default() -> Self {
        let mut working_days = HashSet::new();
        working_days.insert(Weekday::Mon);
        working_days.insert(Weekday::Tue);
        working_days.insert(Weekday::Wed);
        working_days.insert(Weekday::Thu);
        working_days.insert(Weekday::Fri);

        Self {
            working_days,
            holidays: HashSet::new(),
            working_hours_per_day: 8,
            working_days_per_month: 22,
        }
    }
}

impl ProjectCalendar {
    pub fn new(working_hours_per_day: u32) -> Self {
        Self {
            working_hours_per_day,
            ..Default::default()
        }
    }

    /// Трудозатраты «среднего рабочего месяца» в часах:
    /// рабочие часы в дне × число рабочих дней в месяце (по умолчанию 8 × 22 = 176).
    pub fn working_hours_per_month(&self) -> u32 {
        self.working_hours_per_day * self.working_days_per_month
    }

    /// Является ли дата рабочим днем?
    pub fn is_working_day(&self, date: NaiveDate) -> bool {
        let weekday = date.weekday();
        self.working_days.contains(&weekday) && !self.holidays.contains(&date)
    }

    /// Получить количество рабочих дней в периоде
    pub fn count_working_days(&self, window: &TimeWindow) -> u32 {
        let mut count = 0;
        let mut current = window.date_start.date_naive();
        let end = window.date_end.date_naive();

        while current <= end {
            if self.is_working_day(current) {
                count += 1;
            }
            current += chrono::Duration::days(1);
        }

        count
    }

    /// Добавить праздник
    pub fn add_holiday(&mut self, date: NaiveDate) {
        self.holidays.insert(date);
    }

    /// Убрать праздник
    pub fn remove_holiday(&mut self, date: NaiveDate) {
        self.holidays.remove(&date);
    }
}
