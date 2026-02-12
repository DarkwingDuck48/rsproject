use std::collections::HashMap;

use anyhow::Error;
use chrono::{DateTime, Utc};
/// Глобальные ресурсы.
///
/// Идея в том, что ресурс - это глобальная структура, которая одинакова между разными проектами
/// и ее UUID не зависит от проекта.
/// Однако, с использованием глобальных ресурсов мы можем поймать ситуацию, когда ресурсы будут не совпадать в проектах, созданных отдельно
/// Поэтому нужно реализовать следующую логику:
/// 1. В рамках запуска программы создается глобальный реестр ресурсов
/// 2. В каждом проекте есть локальная версия ресурсов, которая отвечает за используемые в проекте ресурсы из глобальных
/// 3. Если открыто несколько проектов - то нужно выполнить мэппинг локальных ресурсов в глобальные реестр - таким образом мы сможем выполнить оптимизацию всех ресурсов.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base_structures::traits::ResourcePoll;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllocationTimeWindow {
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
}

// Для итогового расчета затрат будем пользоваться перечисление RateMeasure
// При создании ресурса пользователь будет указывать какую ставку он определяет, основываясь на элементах RateMeasure.
// Далее, при отображении данных можно будет трансформировать по формуле:
// Hourly - будет базовой ставкой
// Daily = Hourly * 8 (8 рабочих часов в одном дне)
// Monthly = Daily * 22 (в среднем столько дней в рабочем месяце) = Hourly * 8 * 22
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource {
    id: Uuid,
    name: String,
    rate: f64,
    rate_measure: RateMeasure,
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
        })
    }

    pub fn get_base_rate(&self) -> &f64 {
        &self.rate
    }
    pub fn get_converted_rate(&self, to_measure: RateMeasure) -> f64 {
        self.rate_measure.convert(to_measure, self.rate)
    }
}

pub struct AllocationRequest {
    resource_id: Uuid,
    task_id: Uuid,
    project_id: Uuid,
    time_window: AllocationTimeWindow,
}

// Объект для описания назначения одного из ресурсов на задачу
pub struct ResourceAllocation {
    id: Uuid,
    resource_id: Uuid,
    task_id: Uuid,
    project_id: Uuid,
    time_window: AllocationTimeWindow,
}

impl ResourceAllocation {
    pub fn new(request: AllocationRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource_id: request.resource_id,
            task_id: request.task_id,
            project_id: request.project_id,
            time_window: request.time_window,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

pub struct LocalResourcePool {
    resources: HashMap<Uuid, Resource>,
    allocations: HashMap<Uuid, ResourceAllocation>,
}

impl ResourcePoll for LocalResourcePool {
    fn allocate(&mut self, request: AllocationRequest) -> anyhow::Result<()> {
        if self.check_allocation_correct(&request) {
            let allocation = ResourceAllocation::new(request);
            self.allocations.insert(allocation.get_id(), allocation);
            Ok(())
        } else {
            Err(anyhow::Error::msg("Can;t create resource allocation"))
        }
    }

    fn check_allocation_correct(&self, request: &AllocationRequest) -> bool {
        true
    }
    fn deallocate(&mut self, allocation_id: Uuid) -> anyhow::Result<()> {
        todo!()
    }

    fn add_resource(&mut self, resource: Resource) -> anyhow::Result<()> {
        self.resources.insert(resource.id, resource);
        Ok(())
    }

    fn remove_resource(&mut self, id: &Uuid) -> anyhow::Result<()> {
        match self.resources.contains_key(id) {
            true => {
                self.resources.remove(id);
                Ok(())
            }
            false => Err(anyhow::Error::msg(format!(
                "No resource with id {} in LocalPool",
                id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base_structures::resource::Resource;

    #[test]
    fn test_resource_measure_converter() {
        let resource = Resource::new(String::from("Test"), 1000.0, super::RateMeasure::Hourly)
            .expect("Can't create resource");
        assert_eq!(resource.get_base_rate(), &1000.0);
        assert_eq!(
            resource.get_converted_rate(crate::base_structures::resource::RateMeasure::Daily),
            8000.0
        );
        assert_eq!(
            resource.get_converted_rate(crate::base_structures::resource::RateMeasure::Monthly),
            22000.0
        );
    }
}
