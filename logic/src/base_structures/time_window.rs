use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub fn duration_hours(&self) -> i64 {
        (self.date_end - self.date_start).num_hours()
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
