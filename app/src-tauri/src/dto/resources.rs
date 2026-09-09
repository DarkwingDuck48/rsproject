use logic::{ExceptionPeriod, ExceptionType, RateMeasure, Resource, TimeWindow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct ExceptionPeriodInfo {
    period: TimeWindow,
    exception_type: ExceptionType,
}

impl ExceptionPeriodInfo {
    pub fn from_exception_period(ex_period: &ExceptionPeriod) -> Self {
        Self {
            period: ex_period.period,
            exception_type: ex_period.exception_type,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ResourceInfo {
    id: Uuid,
    name: String,
    rate: f64,
    rate_measure: RateMeasure,
    unavailable_periods: Vec<ExceptionPeriodInfo>,
}

impl ResourceInfo {
    pub fn from_resource(resource: &Resource) -> Self {
        Self {
            id: resource.id,
            name: resource.name.clone(),
            rate: resource.rate,
            rate_measure: resource.rate_measure,
            unavailable_periods: resource
                .get_unavailable_periods()
                .iter()
                .map(ExceptionPeriodInfo::from_exception_period)
                .collect(),
        }
    }
}

#[derive(Deserialize, Clone)]
// Умышленно не указываем здесь добавление периодов недоступности
// Редактирование периодов будет реализовано отдельными командами
pub struct ResourceUpdateDto {
    pub name: Option<String>,
    pub rate: Option<f64>,
    pub rate_measure: Option<RateMeasure>,
}
