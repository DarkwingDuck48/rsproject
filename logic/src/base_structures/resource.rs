/// Глобальные ресурсы.
///
/// Идея в том, что ресурс - это глобальная структура, которая одинакова между разными проектами
/// и ее UUID не зависит от проекта.
/// Однако, с использованием глобальных ресурсов мы можем поймать ситуацию, когда ресурсы будут не совпадать в проектах, созданных отдельно
/// Поэтому нужно реализовать следующую логику:
/// 1. В рамках запуска программы создается глобальный реестр ресурсов
/// 2. В каждом проекте есть локальная версия ресурсов, которая отвечает за используемые в проекте ресурсы из глобальных
/// 3. Если открыто несколько проектов - то нужно выполнить мэппинг локальных ресурсов в глобальные реестр - таким образом мы сможем выполнить оптимизацию всех ресурсов.
use anyhow::Error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base_structures::{project_calendar::ProjectCalendar, time_window::TimeWindow};

/// Период исключения (отпуск, отгул)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionPeriod {
    pub period: TimeWindow,
    pub exception_type: ExceptionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExceptionType {
    Vacation,    // Полностью не работает
    SickLeave,   // Не работает
    PersonalDay, // Не работает
    Overtime,    // Работает сверх нормы (можно указать часы)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EngagementRate {
    engagement_rate: f64,
}

impl EngagementRate {
    pub fn new(rate: f64) -> anyhow::Result<Self> {
        if (0.0..=1.0).contains(&rate) {
            Ok(Self {
                engagement_rate: rate,
            })
        } else {
            Err(anyhow::Error::msg(
                "EngagementRate must be set as percent, so value must be between 0.0 and 1.0",
            ))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub enum RateMeasure {
    Daily,
    #[default]
    Hourly,
    Monthly,
}
impl RateMeasure {
    pub fn convert(&self, to_measure: RateMeasure, rate: f64) -> f64 {
        match self {
            RateMeasure::Daily => match to_measure {
                RateMeasure::Daily => rate,
                RateMeasure::Hourly => rate / 8.0,
                RateMeasure::Monthly => rate * 22.0,
            },
            RateMeasure::Hourly => match to_measure {
                RateMeasure::Hourly => rate,
                RateMeasure::Daily => rate * 8.0,
                RateMeasure::Monthly => rate * 22.0,
            },
            RateMeasure::Monthly => match to_measure {
                RateMeasure::Daily => rate / 22.0,
                RateMeasure::Hourly => rate / (22.0 * 8.0),
                RateMeasure::Monthly => rate,
            },
        }
    }
}

// TODO: Реализовать возможность сравнения окон занятости между ресурсами
//

// Для итогового расчета затрат будем пользоваться перечисление RateMeasure
// При создании ресурса пользователь будет указывать какую ставку он определяет, основываясь на элементах RateMeasure.
// Далее, при отображении данных можно будет трансформировать по формуле:
// Hourly - будет базовой ставкой
// Daily = Hourly * 8 (8 рабочих часов в одном дне)
// Monthly = Daily * 22 (в среднем столько дней в рабочем месяце) = Hourly * 8 * 22
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource {
    pub id: Uuid,
    pub name: String,
    pub rate: f64,
    pub rate_measure: RateMeasure,
    unavailable_periods: Vec<ExceptionPeriod>,
}

impl Resource {
    pub fn new(name: String, rate: f64, measure: RateMeasure) -> anyhow::Result<Self> {
        if rate <= 0f64 {
            return Err(Error::msg(format!(
                "Rate for Resource must be > 0. {}",
                rate
            )));
        }
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            rate,
            rate_measure: measure,
            unavailable_periods: vec![],
        })
    }

    pub fn get_base_rate(&self) -> &f64 {
        &self.rate
    }

    // TODO: По хорошему тут должен быть расчет от TimeWindow, чтобы мы смогли сконверировать корректно
    // в базовом варианте пока принимаем неоторые константы по дням
    pub fn get_converted_rate(&self, to_measure: RateMeasure) -> f64 {
        self.rate_measure.convert(to_measure, self.rate)
    }
    pub fn get_rate_measure(&self) -> &RateMeasure {
        &self.rate_measure
    }

    pub fn add_unavailable_period(&mut self, exception_period: ExceptionPeriod) {
        self.unavailable_periods.push(exception_period);
    }

    pub fn get_unavailable_periods(&self) -> &Vec<ExceptionPeriod> {
        &self.unavailable_periods
    }

    pub fn is_available(&self, period: &TimeWindow, calendar: &ProjectCalendar) -> bool {
        if calendar.count_working_days(period) == 0 {
            return false; // Нет рабочих дней в периоде
        }

        for unavailable in &self.unavailable_periods {
            if unavailable.period.overlaps(period) {
                return false;
            }
        }

        true
    }
}
