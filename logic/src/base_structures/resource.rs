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
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AllocationTimeWindow {
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
}

impl AllocationTimeWindow {
    pub fn new(date_start: DateTime<Utc>, date_end: DateTime<Utc>) -> Self {
        Self {
            date_start,
            date_end,
        }
    }
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
    engagement_rate: f64,
    time_window: AllocationTimeWindow,
}

impl AllocationRequest {
    pub fn new(
        resource_id: Uuid,
        task_id: Uuid,
        project_id: Uuid,
        engagement_rate: f64,
        time_window: AllocationTimeWindow,
    ) -> Self {
        Self {
            resource_id,
            task_id,
            project_id,
            engagement_rate,
            time_window,
        }
    }
}

// Объект для описания назначения одного из ресурсов на задачу
#[derive(Default, Debug)]
pub struct ResourceAllocation {
    id: Uuid,
    resource_id: Uuid,
    task_id: Uuid,
    project_id: Uuid,
    engagement_rate: f64,
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
            engagement_rate: request.engagement_rate,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

#[derive(Default, Debug)]
pub struct LocalResourcePool {
    resources: HashMap<Uuid, Resource>,
    allocations: HashMap<Uuid, ResourceAllocation>,
}

impl LocalResourcePool {
    fn check_resource_exists(&self, resource_id: &Uuid) -> bool {
        self.resources.contains_key(resource_id)
    }

    pub fn get_resource_by_name(&self, find_name: String) -> &Resource {
        self.resources
            .values()
            .filter(|r| r.name == find_name.as_str())
            .collect::<Vec<&Resource>>()[0]
    }

    /// Функция должна проверить, что ресурс можно корректно назначить на
    pub fn get_resource_existing_allocations(
        &self,
        resource_id: &Uuid,
    ) -> Vec<&ResourceAllocation> {
        let existing_allocations: Vec<&ResourceAllocation> = self
            .allocations
            .values()
            .filter(|a| &a.resource_id == resource_id)
            .collect();
        existing_allocations
    }
}

impl ResourcePoll for LocalResourcePool {
    fn allocate(&mut self, request: AllocationRequest) -> anyhow::Result<()> {
        match self.check_allocation_correct(&request) {
            Ok(()) => {
                let allocation = ResourceAllocation::new(request);
                self.allocations.insert(allocation.get_id(), allocation);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Несколько проверок перед назначением ресурса на задачу в пуле
    /// 1. Ресурс с таким ID существует в пуле
    fn check_allocation_correct(&self, request: &AllocationRequest) -> anyhow::Result<()> {
        if !self.check_resource_exists(&request.resource_id) {
            return Err(anyhow::Error::msg("Resource not found in Resource pool"));
        };
        let existing_allocation_on_resource =
            self.get_resource_existing_allocations(&request.resource_id);

        // Ресурс есть в пуле и у него еще нет никаких аллокаций - можем смело добавлять.
        if existing_allocation_on_resource.is_empty() {
            return Ok(());
        }

        // Нашли существующие аллокации - нужно проверить, что
        // 1. У ресуса есть свободное окно, чтобы заниматься работой
        // 2. Если окна занятости пересекаются - сумма всех engagement_rate у всех пересекающихся аллокаций должна быть <= 1.0

        Ok(())
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
    use chrono::DateTime;

    use crate::base_structures::{
        resource::{AllocationRequest, AllocationTimeWindow, LocalResourcePool, Resource},
        traits::ResourcePoll,
    };

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

    #[test]
    fn test_get_allocation_by_resource() {
        let resource = Resource::new(String::from("Test"), 1000.0, super::RateMeasure::Hourly)
            .expect("Can't create resource");

        let mut lrp = LocalResourcePool::default();
        lrp.add_resource(resource).unwrap();
        let resource_from_lrp = lrp.get_resource_by_name(String::from("Test")).id;
        let zero_allocations = lrp.get_resource_existing_allocations(&resource_from_lrp);

        assert_eq!(zero_allocations.len(), 0);

        let ar1 = AllocationRequest::new(
            resource_from_lrp,
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            0.2,
            AllocationTimeWindow::new(DateTime::default(), DateTime::default()),
        );

        let ar2 = AllocationRequest::new(
            resource_from_lrp,
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            0.4,
            AllocationTimeWindow::new(DateTime::default(), DateTime::default()),
        );

        lrp.allocate(ar1).expect("Allocation not completed");

        let one_allocations = lrp.get_resource_existing_allocations(&resource_from_lrp);
        assert_eq!(one_allocations.len(), 1);

        lrp.allocate(ar2).expect("Allocation is not completed");

        let two_allocations = lrp.get_resource_existing_allocations(&resource_from_lrp);
        assert_eq!(two_allocations.len(), 2);
    }
}
