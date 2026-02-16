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
#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
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

    /// Проверяет, что есть пересечение с переданным объектом AllocationTimeWindow
    /// И возвращает true или fasle
    pub fn include(&self, other: &Self) -> bool {
        other.date_start >= self.date_start && other.date_end <= self.date_end
    }
}

impl PartialEq for AllocationTimeWindow {
    fn eq(&self, other: &Self) -> bool {
        self.date_start == other.date_start && self.date_end == other.date_end
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

#[derive(Clone, Copy, Debug)]
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

pub struct AllocationQuerryResult<'a> {
    allocations_list: Vec<&'a ResourceAllocation>,
}

impl<'a> AllocationQuerryResult<'a> {
    pub fn check_correct_timewindow(self, allocation_request: &AllocationRequest) -> bool {
        let mut same_time_window = vec![];
        for ra in self.allocations_list {
            if ra.time_window.include(&allocation_request.time_window) {
                same_time_window.push(ra);
            }
        }
        println!("{:?}", same_time_window);
        let mut full_engagement_rate = allocation_request.engagement_rate;
        if !same_time_window.is_empty() {
            for ra in same_time_window {
                full_engagement_rate += ra.get_engagement_rate()
            }
        }
        println!("{:?} > 1.0", full_engagement_rate);
        full_engagement_rate <= 1.0
    }
    pub fn len(&self) -> usize {
        self.allocations_list.len()
    }
    pub fn is_empty(&self) -> bool {
        self.allocations_list.is_empty()
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

    pub fn get_engagement_rate(&self) -> &f64 {
        &self.engagement_rate
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
        &'_ self,
        resource_id: &Uuid,
    ) -> AllocationQuerryResult<'_> {
        let existing_allocations: Vec<&ResourceAllocation> = self
            .allocations
            .values()
            .filter(|a| &a.resource_id == resource_id)
            .collect();
        AllocationQuerryResult {
            allocations_list: existing_allocations,
        }
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
        if !existing_allocation_on_resource.check_correct_timewindow(request) {
            return Err(anyhow::Error::msg(
                "This allocation can't be created, because Resoure will be utilized more than 100%",
            ));
        }

        Ok(())
    }
    fn deallocate(&mut self, allocation_id: Uuid) -> anyhow::Result<()> {
        let alocation = self.allocations.remove(&allocation_id);
        match alocation {
            Some(_) => Ok(()),
            None => Err(anyhow::Error::msg("This allocation not found")),
        }
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
    use chrono::{DateTime, TimeZone, Utc};

    use crate::base_structures::{
        resource::{AllocationRequest, AllocationTimeWindow, LocalResourcePool, Resource},
        traits::ResourcePoll,
    };

    #[test]
    fn test_deallocate() {
        let mut lrp = LocalResourcePool::default();
        let resource = Resource::new(String::from("Test"), 1000.0, super::RateMeasure::Hourly)
            .expect("Can't create resource");
        lrp.add_resource(resource.clone()).unwrap();
        let project_id = uuid::Uuid::new_v4();

        let allocation_request = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.8,
            AllocationTimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            ),
        );
        assert!(lrp.allocate(allocation_request).is_ok());

        let al = lrp.get_resource_existing_allocations(&resource.id);
        let al_id = al.allocations_list[0];

        assert!(lrp.deallocate(al_id.get_id()).is_ok())
    }
    #[test]
    fn test_allocation_check() {
        let mut lrp = LocalResourcePool::default();
        let resource = Resource::new(String::from("Test"), 1000.0, super::RateMeasure::Hourly)
            .expect("Can't create resource");

        let project_id = uuid::Uuid::new_v4();

        let allocation_request = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.8,
            AllocationTimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            ),
        );
        assert!(!lrp.check_resource_exists(&resource.id));
        // Нельзя назначить, пока ресурс не в пуле
        assert!(lrp.allocate(allocation_request).is_err());

        lrp.add_resource(resource.clone()).unwrap();
        assert!(lrp.allocate(allocation_request).is_ok());

        let allocation_request2 = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.1,
            AllocationTimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            ),
        );
        assert!(lrp.allocate(allocation_request2).is_ok());

        let allocation_request3 = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.2,
            AllocationTimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap(),
            ),
        );
        assert!(lrp.allocate(allocation_request3).is_err());
    }

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
    fn test_timewindows() {
        let date_first_start: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_first_end: DateTime<Utc> = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

        let date_second_start: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
        let date_second_end: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap();

        let timewindow1 = AllocationTimeWindow::new(
            Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        );
        let timewindow2 = AllocationTimeWindow::new(date_second_start, date_second_end);

        assert!(timewindow1.include(&timewindow2));
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
