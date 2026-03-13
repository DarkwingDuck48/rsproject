use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::base_structures::ProjectCalendar;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct TimeWindow {
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
}

impl TimeWindow {
    pub fn new(date_start: DateTime<Utc>, date_end: DateTime<Utc>) -> anyhow::Result<Self> {
        if date_start >= date_end {
            return Err(anyhow::Error::msg("TimeWindow: start must be before end"));
        }
        Ok(Self {
            date_start,
            date_end,
        })
    }

    fn calculate_working_days(&self, calendar: &ProjectCalendar) -> i64 {
        let mut working_days = 0;
        let mut current_date = self.date_start.date_naive();

        while current_date <= self.date_end.date_naive() {
            if calendar.is_working_day(current_date) {
                working_days += 1;
            }
            current_date += chrono::Duration::days(1);
        }

        working_days
    }

    /// Проверяет, что есть пересечение с переданным объектом TimeWindow
    /// И возвращает true или fasle
    pub fn overlaps(&self, other: &Self) -> bool {
        self.date_start < other.date_end && self.date_end > other.date_start
    }

    /// Проверить, попадает ли момент времени в окно
    pub fn contains(&self, dt: &DateTime<Utc>) -> bool {
        dt >= &self.date_start && dt < &self.date_end
    }

    /// Длительность в часах
    pub fn duration_hours(&self, calendar: &ProjectCalendar) -> i64 {
        self.calculate_working_days(calendar) * calendar.working_hours_per_day as i64
    }

    pub fn split_by_days(&self) -> Vec<TimeWindow> {
        let mut result = Vec::new();
        let mut current = self.date_start;

        while current < self.date_end {
            let next_day = (current + chrono::Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let day_end = next_day.min(self.date_end);

            result.push(TimeWindow::new(current, day_end).unwrap());
            current = next_day;
        }

        result
    }
}

impl PartialEq for TimeWindow {
    fn eq(&self, other: &Self) -> bool {
        self.date_start == other.date_start && self.date_end == other.date_end
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use chrono::TimeZone;

    #[test]
    fn test_duration_hours() {
        let calendar = ProjectCalendar::default();
        let start = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 2, 0, 0, 0).unwrap();
        let tw = TimeWindow::new(start, end).unwrap();
        assert_eq!(tw.duration_hours(&calendar), 8);
        let start = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 8, 0, 0, 0).unwrap();
        let tw = TimeWindow::new(start, end).unwrap();
        assert_eq!(tw.duration_hours(&calendar), 40);
    }
}
